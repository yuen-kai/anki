import * as THREE from "three";

const SAMPLE_SPACING = 2.0;
const GRID_CELL = 24;

/** Squared distance from a point to a segment, plus the parameter along it. */
function pointSegment(px, pz, x1, z1, x2, z2) {
    const dx = x2 - x1;
    const dz = z2 - z1;
    const lengthSq = dx * dx + dz * dz;
    let t = 0;
    if (lengthSq > 1e-9) {
        t = ((px - x1) * dx + (pz - z1) * dz) / lengthSq;
        t = Math.max(0, Math.min(1, t));
    }
    const cx = x1 + dx * t;
    const cz = z1 + dz * t;
    const ex = px - cx;
    const ez = pz - cz;
    return { distanceSq: ex * ex + ez * ez, t, x: cx, z: cz };
}

function segmentsIntersect(a, b) {
    const r1 = a.x2 - a.x1;
    const r2 = a.z2 - a.z1;
    const s1 = b.x2 - b.x1;
    const s2 = b.z2 - b.z1;
    const denominator = r1 * s2 - r2 * s1;
    if (Math.abs(denominator) < 1e-9) {
        return null;
    }
    const qpx = b.x1 - a.x1;
    const qpz = b.z1 - a.z1;
    const t = (qpx * s2 - qpz * s1) / denominator;
    const u = (qpx * r2 - qpz * r1) / denominator;
    if (t < 0 || t > 1 || u < 0 || u > 1) {
        return null;
    }
    return { x: a.x1 + r1 * t, z: a.z1 + r2 * t };
}

/** Uniform grid over segments, for nearest-road and overlap queries. */
class SegmentIndex {
    constructor(cellSize = GRID_CELL) {
        this.cellSize = cellSize;
        this.cells = new Map();
        this.segments = [];
    }

    _key(cx, cz) {
        return cx * 100003 + cz;
    }

    add(segment) {
        const index = this.segments.length;
        this.segments.push(segment);
        const minX = Math.floor(Math.min(segment.x1, segment.x2) / this.cellSize);
        const maxX = Math.floor(Math.max(segment.x1, segment.x2) / this.cellSize);
        const minZ = Math.floor(Math.min(segment.z1, segment.z2) / this.cellSize);
        const maxZ = Math.floor(Math.max(segment.z1, segment.z2) / this.cellSize);
        for (let cx = minX; cx <= maxX; cx += 1) {
            for (let cz = minZ; cz <= maxZ; cz += 1) {
                const key = this._key(cx, cz);
                let bucket = this.cells.get(key);
                if (!bucket) {
                    bucket = [];
                    this.cells.set(key, bucket);
                }
                bucket.push(index);
            }
        }
    }

    query(x, z, radius) {
        const minX = Math.floor((x - radius) / this.cellSize);
        const maxX = Math.floor((x + radius) / this.cellSize);
        const minZ = Math.floor((z - radius) / this.cellSize);
        const maxZ = Math.floor((z + radius) / this.cellSize);
        const seen = new Set();
        const found = [];
        for (let cx = minX; cx <= maxX; cx += 1) {
            for (let cz = minZ; cz <= maxZ; cz += 1) {
                const bucket = this.cells.get(this._key(cx, cz));
                if (!bucket) {
                    continue;
                }
                for (const index of bucket) {
                    if (!seen.has(index)) {
                        seen.add(index);
                        found.push(this.segments[index]);
                    }
                }
            }
        }
        return found;
    }

    /** Nearest centreline distance, and how far inside that road's width it is. */
    nearest(x, z, searchRadius = GRID_CELL * 2) {
        let best = null;
        let radius = searchRadius;
        for (let attempt = 0; attempt < 4 && best === null; attempt += 1) {
            for (const segment of this.query(x, z, radius)) {
                const hit = pointSegment(x, z, segment.x1, segment.z1, segment.x2, segment.z2);
                if (best === null || hit.distanceSq < best.distanceSq) {
                    best = { distanceSq: hit.distanceSq, segment, x: hit.x, z: hit.z };
                }
            }
            radius *= 2;
        }
        if (best === null) {
            return { distance: Infinity, clearance: Infinity, segment: null };
        }
        const distance = Math.sqrt(best.distanceSq);
        return {
            distance,
            clearance: distance - best.segment.halfWidth,
            segment: best.segment,
            x: best.x,
            z: best.z,
        };
    }
}

function samplePoints(entry) {
    const points = entry.points.map(([x, z]) => new THREE.Vector3(x, 0, z));
    if (entry.curved) {
        const curve = new THREE.CatmullRomCurve3(points, Boolean(entry.closed), "centripetal", 0.5);
        const count = Math.max(8, Math.round(curve.getLength() / SAMPLE_SPACING));
        const sampled = curve.getSpacedPoints(count);
        return sampled.map((p) => [p.x, p.z]);
    }
    const source = entry.closed ? [...entry.points, entry.points[0]] : entry.points;
    const out = [source[0]];
    for (let i = 1; i < source.length; i += 1) {
        const [x1, z1] = source[i - 1];
        const [x2, z2] = source[i];
        const length = Math.hypot(x2 - x1, z2 - z1);
        const steps = Math.max(1, Math.round(length / SAMPLE_SPACING));
        for (let s = 1; s <= steps; s += 1) {
            const t = s / steps;
            out.push([x1 + (x2 - x1) * t, z1 + (z2 - z1) * t]);
        }
    }
    return out;
}

function polylineLength(polyline) {
    let total = 0;
    for (let i = 1; i < polyline.length; i += 1) {
        total += Math.hypot(polyline[i][0] - polyline[i - 1][0], polyline[i][1] - polyline[i - 1][1]);
    }
    return total;
}

export class RoadNetwork {
    constructor(mapData) {
        this.map = mapData;
        this.roads = mapData.roads.map((entry) => {
            const polyline = samplePoints(entry);
            return {
                id: entry.id,
                width: entry.width,
                halfWidth: entry.width / 2,
                closed: Boolean(entry.closed),
                polyline,
                length: polylineLength(polyline),
            };
        });

        this.index = new SegmentIndex();
        for (const road of this.roads) {
            for (let i = 1; i < road.polyline.length; i += 1) {
                this.index.add({
                    x1: road.polyline[i - 1][0],
                    z1: road.polyline[i - 1][1],
                    x2: road.polyline[i][0],
                    z2: road.polyline[i][1],
                    roadId: road.id,
                    halfWidth: road.halfWidth,
                });
            }
        }

        this.junctions = this._findJunctions();
    }

    _findJunctions() {
        const junctions = [];
        const push = (x, z, radius) => {
            for (const existing of junctions) {
                if (Math.hypot(existing.x - x, existing.z - z) < 6) {
                    existing.radius = Math.max(existing.radius, radius);
                    return;
                }
            }
            junctions.push({ x, z, radius });
        };

        for (let a = 0; a < this.roads.length; a += 1) {
            const roadA = this.roads[a];
            for (let b = a + 1; b < this.roads.length; b += 1) {
                const roadB = this.roads[b];
                const radius = Math.max(roadA.halfWidth, roadB.halfWidth) + 2;
                for (let i = 1; i < roadA.polyline.length; i += 1) {
                    const segA = {
                        x1: roadA.polyline[i - 1][0],
                        z1: roadA.polyline[i - 1][1],
                        x2: roadA.polyline[i][0],
                        z2: roadA.polyline[i][1],
                    };
                    const minX = Math.min(segA.x1, segA.x2) - radius;
                    const maxX = Math.max(segA.x1, segA.x2) + radius;
                    const minZ = Math.min(segA.z1, segA.z2) - radius;
                    const maxZ = Math.max(segA.z1, segA.z2) + radius;
                    for (let j = 1; j < roadB.polyline.length; j += 1) {
                        const bx1 = roadB.polyline[j - 1][0];
                        const bz1 = roadB.polyline[j - 1][1];
                        const bx2 = roadB.polyline[j][0];
                        const bz2 = roadB.polyline[j][1];
                        if (Math.max(bx1, bx2) < minX || Math.min(bx1, bx2) > maxX) {
                            continue;
                        }
                        if (Math.max(bz1, bz2) < minZ || Math.min(bz1, bz2) > maxZ) {
                            continue;
                        }
                        const hit = segmentsIntersect(segA, { x1: bx1, z1: bz1, x2: bx2, z2: bz2 });
                        if (hit) {
                            push(hit.x, hit.z, radius);
                        }
                    }
                }
            }
        }

        for (const road of this.roads) {
            if (road.closed) {
                continue;
            }
            for (const end of [road.polyline[0], road.polyline[road.polyline.length - 1]]) {
                push(end[0], end[1], road.halfWidth + 2);
            }
        }
        return junctions;
    }

    /** True when the point is inside a junction disc, where markings stop. */
    nearJunction(x, z, extra = 0) {
        for (const junction of this.junctions) {
            const limit = junction.radius + extra;
            const dx = junction.x - x;
            const dz = junction.z - z;
            if (dx * dx + dz * dz < limit * limit) {
                return true;
            }
        }
        return false;
    }

    /** Distance from the point to the nearest road edge; negative means on-road. */
    clearanceFromRoads(x, z) {
        return this.index.nearest(x, z).clearance;
    }

    nearestRoad(x, z) {
        return this.index.nearest(x, z);
    }

    /**
     * Reports endpoints that do not land on another road, so an authoring
     * mistake surfaces immediately instead of appearing as a stub in the world.
     */
    validate() {
        const problems = [];
        for (const road of this.roads) {
            if (road.closed) {
                continue;
            }
            const ends = [
                { point: road.polyline[0], label: "start" },
                { point: road.polyline[road.polyline.length - 1], label: "end" },
            ];
            for (const { point, label } of ends) {
                let connected = false;
                for (const candidate of this.index.query(point[0], point[1], 40)) {
                    if (candidate.roadId === road.id) {
                        continue;
                    }
                    const hit = pointSegment(
                        point[0],
                        point[1],
                        candidate.x1,
                        candidate.z1,
                        candidate.x2,
                        candidate.z2,
                    );
                    const reach = candidate.halfWidth + road.halfWidth;
                    if (hit.distanceSq <= reach * reach) {
                        connected = true;
                        break;
                    }
                }
                if (!connected) {
                    problems.push(
                        `road "${road.id}" ${label} at (${point[0].toFixed(1)}, ${
                            point[1].toFixed(1)
                        }) does not meet another road`,
                    );
                }
            }
        }
        return problems;
    }
}

export { pointSegment };

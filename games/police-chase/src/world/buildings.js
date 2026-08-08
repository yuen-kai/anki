import * as THREE from "three";

import { FACADE_BAYS, FACADE_FLOORS, makeRandom } from "./textures.js";

const BAY = 3.4;
const FLOOR = 3.3;
const TILE_U = FACADE_BAYS * BAY;
const TILE_V = FACADE_FLOORS * FLOOR;
const PLINTH_HEIGHT = 1.3;
const PARAPET_HEIGHT = 0.8;

/** Arc-length lookup along a sampled polyline. */
function measurePolyline(polyline) {
    const cumulative = [0];
    for (let i = 1; i < polyline.length; i += 1) {
        cumulative.push(
            cumulative[i - 1] + Math.hypot(polyline[i][0] - polyline[i - 1][0], polyline[i][1] - polyline[i - 1][1]),
        );
    }
    return cumulative;
}

function sampleAt(polyline, cumulative, distance) {
    const total = cumulative[cumulative.length - 1];
    const target = Math.max(0, Math.min(total, distance));
    let low = 0;
    let high = cumulative.length - 1;
    while (high - low > 1) {
        const mid = (low + high) >> 1;
        if (cumulative[mid] <= target) {
            low = mid;
        } else {
            high = mid;
        }
    }
    const span = cumulative[high] - cumulative[low] || 1;
    const t = (target - cumulative[low]) / span;
    const ax = polyline[low][0];
    const az = polyline[low][1];
    const bx = polyline[high][0];
    const bz = polyline[high][1];
    let tx = bx - ax;
    let tz = bz - az;
    const length = Math.hypot(tx, tz) || 1;
    tx /= length;
    tz /= length;
    return { x: ax + (bx - ax) * t, z: az + (bz - az) * t, tx, tz };
}

function insideShape(shape, x, z, margin) {
    if (shape.shape === "circle") {
        return Math.hypot(x - shape.x, z - shape.z) < shape.radius + margin;
    }
    return x > shape.x0 - margin && x < shape.x1 + margin && z > shape.z0 - margin && z < shape.z1 + margin;
}

class ObbIndex {
    constructor(cellSize = 30) {
        this.cellSize = cellSize;
        this.cells = new Map();
    }

    _key(cx, cz) {
        return cx * 100003 + cz;
    }

    _range(box) {
        const reach = Math.hypot(box.halfW, box.halfD);
        return {
            minX: Math.floor((box.x - reach) / this.cellSize),
            maxX: Math.floor((box.x + reach) / this.cellSize),
            minZ: Math.floor((box.z - reach) / this.cellSize),
            maxZ: Math.floor((box.z + reach) / this.cellSize),
        };
    }

    add(box) {
        const { minX, maxX, minZ, maxZ } = this._range(box);
        for (let cx = minX; cx <= maxX; cx += 1) {
            for (let cz = minZ; cz <= maxZ; cz += 1) {
                const key = this._key(cx, cz);
                let bucket = this.cells.get(key);
                if (!bucket) {
                    bucket = [];
                    this.cells.set(key, bucket);
                }
                bucket.push(box);
            }
        }
    }

    nearby(box) {
        const { minX, maxX, minZ, maxZ } = this._range(box);
        const seen = new Set();
        const found = [];
        for (let cx = minX; cx <= maxX; cx += 1) {
            for (let cz = minZ; cz <= maxZ; cz += 1) {
                const bucket = this.cells.get(this._key(cx, cz));
                if (!bucket) {
                    continue;
                }
                for (const other of bucket) {
                    if (!seen.has(other)) {
                        seen.add(other);
                        found.push(other);
                    }
                }
            }
        }
        return found;
    }
}

function obbAxes(box) {
    const cos = Math.cos(box.rotY);
    const sin = Math.sin(box.rotY);
    return [{ x: cos, z: -sin }, { x: sin, z: cos }];
}

function obbCorners(box, expand = 0) {
    const [u, v] = obbAxes(box);
    const halfW = box.halfW + expand;
    const halfD = box.halfD + expand;
    const corners = [];
    for (const sw of [-1, 1]) {
        for (const sd of [-1, 1]) {
            corners.push({
                x: box.x + u.x * halfW * sw + v.x * halfD * sd,
                z: box.z + u.z * halfW * sw + v.z * halfD * sd,
            });
        }
    }
    return corners;
}

function obbOverlap(a, b, gap = 0) {
    const axes = [...obbAxes(a), ...obbAxes(b)];
    const cornersA = obbCorners(a, gap * 0.5);
    const cornersB = obbCorners(b, gap * 0.5);
    for (const axis of axes) {
        let minA = Infinity;
        let maxA = -Infinity;
        let minB = Infinity;
        let maxB = -Infinity;
        for (const c of cornersA) {
            const p = c.x * axis.x + c.z * axis.z;
            minA = Math.min(minA, p);
            maxA = Math.max(maxA, p);
        }
        for (const c of cornersB) {
            const p = c.x * axis.x + c.z * axis.z;
            minB = Math.min(minB, p);
            maxB = Math.max(maxB, p);
        }
        if (maxA < minB || maxB < minA) {
            return false;
        }
    }
    return true;
}

class MeshBuilder {
    constructor(withColor = true) {
        this.positions = [];
        this.normals = [];
        this.uvs = [];
        this.colors = withColor ? [] : null;
        this.indices = [];
    }

    face(corners, normal, uvs, color) {
        const base = this.positions.length / 3;
        for (let i = 0; i < 4; i += 1) {
            this.positions.push(corners[i][0], corners[i][1], corners[i][2]);
            this.normals.push(normal[0], normal[1], normal[2]);
            this.uvs.push(uvs[i][0], uvs[i][1]);
            if (this.colors) {
                this.colors.push(color.r, color.g, color.b);
            }
        }
        this.indices.push(base, base + 1, base + 2, base, base + 2, base + 3);
    }

    isEmpty() {
        return this.positions.length === 0;
    }

    build() {
        const geometry = new THREE.BufferGeometry();
        geometry.setAttribute("position", new THREE.Float32BufferAttribute(this.positions, 3));
        geometry.setAttribute("normal", new THREE.Float32BufferAttribute(this.normals, 3));
        geometry.setAttribute("uv", new THREE.Float32BufferAttribute(this.uvs, 2));
        if (this.colors) {
            geometry.setAttribute("color", new THREE.Float32BufferAttribute(this.colors, 3));
        }
        geometry.setIndex(this.indices);
        geometry.computeBoundingSphere();
        return geometry;
    }
}

/** Adds the four vertical faces of a rotated box, UV'd in metres. */
function addSides(builder, box, yBottom, yTop, color, uvScale) {
    const [u, v] = obbAxes(box);
    const halfW = box.halfW;
    const halfD = box.halfD;
    const corner = (sw, sd) => [box.x + u.x * halfW * sw + v.x * halfD * sd, box.z + u.z * halfW * sw + v.z * halfD * sd];

    const p00 = corner(-1, -1);
    const p10 = corner(1, -1);
    const p11 = corner(1, 1);
    const p01 = corner(-1, 1);
    const height = yTop - yBottom;

    const walls = [
        { a: p00, b: p10, normal: [-v.x, 0, -v.z], width: halfW * 2 },
        { a: p10, b: p11, normal: [u.x, 0, u.z], width: halfD * 2 },
        { a: p11, b: p01, normal: [v.x, 0, v.z], width: halfW * 2 },
        { a: p01, b: p00, normal: [-u.x, 0, -u.z], width: halfD * 2 },
    ];

    for (const wall of walls) {
        const uMax = (wall.width / TILE_U) * uvScale;
        const vMax = (height / TILE_V) * uvScale;
        const vBase = (yBottom / TILE_V) * uvScale;
        builder.face(
            [
                [wall.a[0], yBottom, wall.a[1]],
                [wall.b[0], yBottom, wall.b[1]],
                [wall.b[0], yTop, wall.b[1]],
                [wall.a[0], yTop, wall.a[1]],
            ],
            wall.normal,
            [[0, vBase], [uMax, vBase], [uMax, vBase + vMax], [0, vBase + vMax]],
            color,
        );
    }
}

function addTop(builder, box, y, color, uvScale = 1 / 8) {
    const [u, v] = obbAxes(box);
    const corner = (sw, sd) => [
        box.x + u.x * box.halfW * sw + v.x * box.halfD * sd,
        box.z + u.z * box.halfW * sw + v.z * box.halfD * sd,
    ];
    const p00 = corner(-1, -1);
    const p10 = corner(1, -1);
    const p11 = corner(1, 1);
    const p01 = corner(-1, 1);
    builder.face(
        [[p00[0], y, p00[1]], [p01[0], y, p01[1]], [p11[0], y, p11[1]], [p10[0], y, p10[1]]],
        [0, 1, 0],
        [
            [p00[0] * uvScale, p00[1] * uvScale],
            [p01[0] * uvScale, p01[1] * uvScale],
            [p11[0] * uvScale, p11[1] * uvScale],
            [p10[0] * uvScale, p10[1] * uvScale],
        ],
        color,
    );
}

function addRing(builder, box, yBottom, yTop, thickness, color) {
    addSides(builder, box, yBottom, yTop, color, 1);
    const inner = { ...box, halfW: Math.max(0.3, box.halfW - thickness), halfD: Math.max(0.3, box.halfD - thickness) };
    addTop(builder, box, yTop, color, 1 / 4);
    addTop(builder, inner, yTop - 0.05, color, 1 / 4);
}

function floorsForLocation(distanceFromCentre, random) {
    const t = Math.min(1, distanceFromCentre / 330);
    const core = 16 - t * 12;
    const spread = 3 + (1 - t) * 6;
    const value = core + (random() - 0.5) * spread * 2;
    const tower = random() < 0.035 ? 10 + random() * 16 : 0;
    return Math.max(2, Math.round(value + tower));
}

export function createBuildings({ map, network, surfaces, facadeTextures, maxBuildings = 1400 }) {
    const random = makeRandom(20260808);
    const placed = [];
    const index = new ObbIndex();
    const excluded = [...map.parks, ...map.lots];
    const limit = map.cityHalf - 14;

    const stats = { attempts: 0, outOfBounds: 0, tooCloseToRoad: 0, inExcludedShape: 0, overlapping: 0 };

    const isBlocked = (box) => {
        stats.attempts += 1;
        if (Math.abs(box.x) > map.builtEdge || Math.abs(box.z) > map.builtEdge) {
            stats.outOfBounds += 1;
            return true;
        }
        const probes = [...obbCorners(box), { x: box.x, z: box.z }];
        const [u, v] = obbAxes(box);
        probes.push(
            { x: box.x + u.x * box.halfW, z: box.z + u.z * box.halfW },
            { x: box.x - u.x * box.halfW, z: box.z - u.z * box.halfW },
            { x: box.x + v.x * box.halfD, z: box.z + v.z * box.halfD },
            { x: box.x - v.x * box.halfD, z: box.z - v.z * box.halfD },
        );
        for (const probe of probes) {
            if (Math.abs(probe.x) > limit || Math.abs(probe.z) > limit) {
                stats.outOfBounds += 1;
                return true;
            }
            if (network.clearanceFromRoads(probe.x, probe.z) < map.sidewalkWidth * 0.9) {
                stats.tooCloseToRoad += 1;
                return true;
            }
            for (const shape of excluded) {
                if (insideShape(shape, probe.x, probe.z, 2)) {
                    stats.inExcludedShape += 1;
                    return true;
                }
            }
        }
        for (const other of index.nearby(box)) {
            if (obbOverlap(box, other, 1.2)) {
                stats.overlapping += 1;
                return true;
            }
        }
        return false;
    };

    const candidateAt = (road, cumulative, cursor, side, rowOffset, widthBays, depthBays) => {
        const width = widthBays * BAY;
        const depth = depthBays * BAY;
        const sample = sampleAt(road.polyline, cumulative, cursor + width / 2);
        const distance = road.halfWidth + map.sidewalkWidth + depth / 2 + 0.7 + rowOffset;
        return {
            x: sample.x + sample.tz * side * distance,
            z: sample.z - sample.tx * side * distance,
            halfW: width / 2,
            halfD: depth / 2,
            rotY: Math.atan2(-sample.tz, sample.tx),
            width,
            depth,
        };
    };

    stats.perRoad = {};
    for (const road of network.roads) {
        const cumulative = measurePolyline(road.polyline);
        const total = cumulative[cumulative.length - 1];
        stats.perRoad[road.id] = { length: Math.round(total), placed: 0 };
        for (const side of [-1, 1]) {
            for (let row = 0; row < 2; row += 1) {
                const depthBays = row === 0 ? 3 + Math.floor(random() * 2) : 3;
                const rowOffset = row === 0 ? 0 : depthBays * BAY + 2 + random() * 2.5;
                let cursor = 4 + random() * 8;
                while (cursor < total - 4 && placed.length < maxBuildings) {
                    let chosen = null;
                    // Largest frontage that still fits, so short gaps between
                    // cross streets are filled instead of skipped.
                    for (let widthBays = 6; widthBays >= 2 && chosen === null; widthBays -= 1) {
                        const box = candidateAt(road, cumulative, cursor, side, rowOffset, widthBays, depthBays);
                        if (!isBlocked(box)) {
                            chosen = box;
                        }
                    }

                    if (chosen) {
                        const floors = floorsForLocation(Math.hypot(chosen.x, chosen.z), random);
                        chosen.height = floors * FLOOR;
                        chosen.style = Math.floor(random() * facadeTextures.length);
                        chosen.tint = random();
                        chosen.setback = floors > 9 && random() < 0.55;
                        placed.push(chosen);
                        index.add(chosen);
                        stats.perRoad[road.id].placed += 1;
                        cursor += chosen.width + 0.6 + random() * 1.6;
                    } else {
                        cursor += BAY;
                    }
                }
            }
        }
    }

    const facadeBuilders = facadeTextures.map(() => new MeshBuilder(true));
    const roofBuilder = new MeshBuilder(true);
    const trimBuilder = new MeshBuilder(true);
    const tintColor = new THREE.Color();
    const trimColor = new THREE.Color();

    for (const box of placed) {
        tintColor.setHSL(0.07 + box.tint * 0.06, 0.05 + box.tint * 0.08, 0.46 + box.tint * 0.22);
        trimColor.setHSL(0.09, 0.04, 0.38 + box.tint * 0.12);

        const plinth = { ...box, halfW: box.halfW + 0.35, halfD: box.halfD + 0.35 };
        addSides(trimBuilder, plinth, 0, PLINTH_HEIGHT, trimColor, 1);

        const bodyTop = box.height;
        const facade = facadeBuilders[box.style];
        if (box.setback) {
            const splitAt = PLINTH_HEIGHT + Math.round((bodyTop - PLINTH_HEIGHT) * 0.62 / FLOOR) * FLOOR;
            addSides(facade, box, PLINTH_HEIGHT, splitAt, tintColor, 1);
            const upper = { ...box, halfW: box.halfW * 0.74, halfD: box.halfD * 0.74 };
            addTop(roofBuilder, box, splitAt, trimColor);
            addRing(trimBuilder, box, splitAt, splitAt + PARAPET_HEIGHT * 0.8, 0.5, trimColor);
            addSides(facade, upper, splitAt, bodyTop, tintColor, 1);
            addTop(roofBuilder, upper, bodyTop, trimColor);
            addRing(trimBuilder, upper, bodyTop, bodyTop + PARAPET_HEIGHT, 0.45, trimColor);
        } else {
            addSides(facade, box, PLINTH_HEIGHT, bodyTop, tintColor, 1);
            addTop(roofBuilder, box, bodyTop, trimColor);
            addRing(trimBuilder, box, bodyTop, bodyTop + PARAPET_HEIGHT, 0.45, trimColor);
        }
    }

    const group = new THREE.Group();
    group.name = "buildings";

    facadeBuilders.forEach((builder, i) => {
        if (builder.isEmpty()) {
            return;
        }
        const material = new THREE.MeshStandardMaterial({
            map: facadeTextures[i],
            vertexColors: true,
            roughness: 0.72,
            metalness: 0.06,
        });
        const mesh = new THREE.Mesh(builder.build(), material);
        mesh.castShadow = true;
        mesh.receiveShadow = true;
        mesh.matrixAutoUpdate = false;
        group.add(mesh);
    });

    const roofMaterial = new THREE.MeshStandardMaterial({
        map: surfaces.roof,
        vertexColors: true,
        roughness: 0.94,
        metalness: 0,
    });
    const roofMesh = new THREE.Mesh(roofBuilder.build(), roofMaterial);
    roofMesh.castShadow = true;
    roofMesh.receiveShadow = true;
    roofMesh.matrixAutoUpdate = false;
    group.add(roofMesh);

    const trimMaterial = new THREE.MeshStandardMaterial({
        map: surfaces.concrete,
        vertexColors: true,
        roughness: 0.9,
        metalness: 0,
    });
    const trimMesh = new THREE.Mesh(trimBuilder.build(), trimMaterial);
    trimMesh.castShadow = true;
    trimMesh.receiveShadow = true;
    trimMesh.matrixAutoUpdate = false;
    group.add(trimMesh);

    return {
        group,
        stats,
        colliders: placed.map((box) => ({
            x: box.x,
            z: box.z,
            halfW: box.halfW + 0.35,
            halfD: box.halfD + 0.35,
            rotY: box.rotY,
            height: box.height,
        })),
    };
}

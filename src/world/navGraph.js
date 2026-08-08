import { pointSegment } from "./roadNetwork.js";

/**
 * Drivable graph derived from the road network: nodes at junctions, edges along
 * the road centrelines between them. Used for routing and for picking places on
 * the map relative to a car's position and heading.
 */
export class NavGraph {
    constructor(network) {
        this.nodes = network.junctions.map((junction) => ({
            x: junction.x,
            z: junction.z,
            radius: junction.radius,
            links: [],
        }));
        this.edges = [];

        for (const road of network.roads) {
            this._splitRoad(road);
        }
    }

    _splitRoad(road) {
        const polyline = road.polyline;
        const cumulative = [0];
        for (let i = 1; i < polyline.length; i += 1) {
            cumulative.push(
                cumulative[i - 1]
                    + Math.hypot(polyline[i][0] - polyline[i - 1][0], polyline[i][1] - polyline[i - 1][1]),
            );
        }

        const hits = [];
        this.nodes.forEach((node, nodeIndex) => {
            let best = null;
            for (let i = 1; i < polyline.length; i += 1) {
                const hit = pointSegment(
                    node.x,
                    node.z,
                    polyline[i - 1][0],
                    polyline[i - 1][1],
                    polyline[i][0],
                    polyline[i][1],
                );
                if (best === null || hit.distanceSq < best.distanceSq) {
                    const span = cumulative[i] - cumulative[i - 1];
                    best = { distanceSq: hit.distanceSq, distance: cumulative[i - 1] + span * hit.t };
                }
            }
            const reach = Math.max(node.radius, road.halfWidth + 3);
            if (best && best.distanceSq <= reach * reach) {
                hits.push({ nodeIndex, distance: best.distance });
            }
        });

        hits.sort((a, b) => a.distance - b.distance);
        const unique = hits.filter((hit, i) => i === 0 || hit.nodeIndex !== hits[i - 1].nodeIndex);
        if (unique.length < 2) {
            return;
        }

        const pairs = [];
        for (let i = 1; i < unique.length; i += 1) {
            pairs.push([unique[i - 1], unique[i]]);
        }
        if (road.closed) {
            pairs.push([unique[unique.length - 1], { ...unique[0], distance: cumulative[cumulative.length - 1] }]);
        }

        for (const [from, to] of pairs) {
            if (from.nodeIndex === to.nodeIndex || to.distance - from.distance < 4) {
                continue;
            }
            const points = this._slice(polyline, cumulative, from.distance, to.distance);
            points[0] = [this.nodes[from.nodeIndex].x, this.nodes[from.nodeIndex].z];
            points[points.length - 1] = [this.nodes[to.nodeIndex].x, this.nodes[to.nodeIndex].z];
            const edge = {
                index: this.edges.length,
                a: from.nodeIndex,
                b: to.nodeIndex,
                roadId: road.id,
                width: road.width,
                points,
                length: to.distance - from.distance,
            };
            this.edges.push(edge);
            this.nodes[from.nodeIndex].links.push({ edge, to: to.nodeIndex, forward: true });
            this.nodes[to.nodeIndex].links.push({ edge, to: from.nodeIndex, forward: false });
        }
    }

    _slice(polyline, cumulative, fromDistance, toDistance) {
        const points = [];
        for (let i = 0; i < polyline.length; i += 1) {
            if (cumulative[i] >= fromDistance && cumulative[i] <= toDistance) {
                points.push([polyline[i][0], polyline[i][1]]);
            }
        }
        if (points.length < 2) {
            points.length = 0;
            points.push([polyline[0][0], polyline[0][1]], [polyline[1][0], polyline[1][1]]);
        }
        return points;
    }

    nearestNode(x, z) {
        let best = -1;
        let bestDistance = Infinity;
        for (let i = 0; i < this.nodes.length; i += 1) {
            const dx = this.nodes[i].x - x;
            const dz = this.nodes[i].z - z;
            const distance = dx * dx + dz * dz;
            if (distance < bestDistance) {
                bestDistance = distance;
                best = i;
            }
        }
        return best;
    }

    /**
     * Node that best matches a direction of travel from a point: prefers nodes
     * ahead, within a distance band, and roughly on the bearing.
     */
    nodeTowards(x, z, dirX, dirZ, { minDistance = 30, maxDistance = 200, spread = 0.6 } = {}) {
        let best = -1;
        let bestScore = -Infinity;
        for (let i = 0; i < this.nodes.length; i += 1) {
            const dx = this.nodes[i].x - x;
            const dz = this.nodes[i].z - z;
            const distance = Math.hypot(dx, dz);
            if (distance < minDistance || distance > maxDistance) {
                continue;
            }
            const alignment = (dx * dirX + dz * dirZ) / distance;
            if (alignment < spread) {
                continue;
            }
            const score = alignment * 2 - distance / maxDistance;
            if (score > bestScore) {
                bestScore = score;
                best = i;
            }
        }
        return best;
    }

    /** A* over the node graph. Returns node indices from `start` to `goal`. */
    route(start, goal) {
        if (start === goal || start < 0 || goal < 0) {
            return [start];
        }
        const nodes = this.nodes;
        const heuristic = (i) => Math.hypot(nodes[i].x - nodes[goal].x, nodes[i].z - nodes[goal].z);

        const cameFrom = new Map();
        const bestCost = new Map([[start, 0]]);
        const open = [{ node: start, priority: heuristic(start) }];

        while (open.length > 0) {
            let bestIndex = 0;
            for (let i = 1; i < open.length; i += 1) {
                if (open[i].priority < open[bestIndex].priority) {
                    bestIndex = i;
                }
            }
            const current = open.splice(bestIndex, 1)[0].node;
            if (current === goal) {
                const path = [current];
                let step = current;
                while (cameFrom.has(step)) {
                    step = cameFrom.get(step);
                    path.push(step);
                }
                return path.reverse();
            }

            for (const link of nodes[current].links) {
                const tentative = bestCost.get(current) + link.edge.length;
                if (tentative < (bestCost.get(link.to) ?? Infinity)) {
                    bestCost.set(link.to, tentative);
                    cameFrom.set(link.to, current);
                    open.push({ node: link.to, priority: tentative + heuristic(link.to) });
                }
            }
        }
        return [start];
    }

    /** Turns a node route into a dense polyline that follows the road curves. */
    pathPoints(route) {
        const points = [];
        for (let i = 1; i < route.length; i += 1) {
            const from = route[i - 1];
            const to = route[i];
            const link = this.nodes[from].links.find((candidate) => candidate.to === to);
            if (!link) {
                points.push([this.nodes[to].x, this.nodes[to].z]);
                continue;
            }
            const edgePoints = link.forward ? link.edge.points : [...link.edge.points].reverse();
            for (const point of edgePoints) {
                const last = points[points.length - 1];
                if (!last || Math.hypot(last[0] - point[0], last[1] - point[1]) > 1.5) {
                    points.push([point[0], point[1]]);
                }
            }
        }
        return points;
    }
}

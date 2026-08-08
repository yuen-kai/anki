import { rotateVector } from "../engine/mathUtils.js";
import { PoliceCar } from "./police.js";
import { CHASE } from "./tuning.js";

export const ROLE = {
    PURSUE: "pursue",
    INTERCEPT: "intercept",
    FLANK: "flank",
    BLOCK: "block",
    ROADBLOCK: "roadblock",
    CONTAIN: "contain",
};

const ROLE_SLOTS = [
    ROLE.PURSUE,
    ROLE.INTERCEPT,
    ROLE.PURSUE,
    ROLE.FLANK,
    ROLE.BLOCK,
    ROLE.ROADBLOCK,
    ROLE.ROADBLOCK,
    ROLE.FLANK,
    ROLE.INTERCEPT,
    ROLE.BLOCK,
    ROLE.CONTAIN,
    ROLE.PURSUE,
    ROLE.CONTAIN,
    ROLE.FLANK,
    ROLE.CONTAIN,
    ROLE.BLOCK,
    ROLE.CONTAIN,
    ROLE.INTERCEPT,
];

function composeRoles(count) {
    const roles = ROLE_SLOTS.slice(0, count);
    while (roles.length < count) {
        roles.push(ROLE.CONTAIN);
    }
    // A roadblock needs a partner; a lone one becomes an intersection block.
    const roadblocks = roles.filter((role) => role === ROLE.ROADBLOCK).length;
    if (roadblocks === 1) {
        roles[roles.indexOf(ROLE.ROADBLOCK)] = ROLE.BLOCK;
    }
    return roles;
}

/**
 * The police as one unit. Every reassignment tick it works out where the
 * interesting places on the map are relative to the player, decides which roles
 * the current number of cars supports, and hands each role to whichever car is
 * best placed for it. Cars are interchangeable; only the assignment changes.
 */
export class PoliceUnit {
    constructor({ graph, collision, obstacles }) {
        this.graph = graph;
        this.collision = collision;
        this.obstacles = obstacles;
        this.cars = [];
        this.nextId = 0;
        this.reassignIn = 0;
        this.spawnIn = CHASE.spawnIntervalSeconds;
        this.spawnInterval = CHASE.spawnIntervalSeconds;
        this.elapsed = 0;
    }

    get count() {
        return this.cars.length;
    }

    reset() {
        this.cars = [];
        this.nextId = 0;
        this.reassignIn = 0;
        this.spawnIn = CHASE.spawnIntervalSeconds;
        this.spawnInterval = CHASE.spawnIntervalSeconds;
        this.elapsed = 0;
    }

    /** Places the opening cars on the road behind the player. */
    deployBehind(player, count = CHASE.initialCars) {
        const backX = -Math.sin(player.heading);
        const backZ = -Math.cos(player.heading);
        for (let i = 0; i < count; i += 1) {
            const distance = CHASE.initialSpawnOffset + i * CHASE.initialSpawnGap;
            const lateral = (i % 2 === 0 ? 1 : -1) * 3.1;
            const side = rotateVector(backX, backZ, Math.PI / 2);
            let x = player.x + backX * distance + side.x * lateral;
            let z = player.z + backZ * distance + side.z * lateral;
            let heading = player.heading;

            if (this.collision.blocked(x, z, 1.6)) {
                const node = this.graph.nodeTowards(player.x, player.z, backX, backZ, {
                    minDistance: 20,
                    maxDistance: 160,
                    spread: 0.1,
                });
                if (node >= 0) {
                    x = this.graph.nodes[node].x;
                    z = this.graph.nodes[node].z;
                    heading = Math.atan2(player.x - x, player.z - z);
                }
            }
            this.cars.push(new PoliceCar(this.nextId++, { x, z, heading }));
        }
    }

    /** Adds one car at a junction off to the side of the player's route. */
    spawnFromSideStreet(player) {
        const nodes = this.graph.nodes;
        const forwardX = Math.sin(player.heading);
        const forwardZ = Math.cos(player.heading);
        let best = -1;
        let bestScore = -Infinity;

        for (let i = 0; i < nodes.length; i += 1) {
            const dx = nodes[i].x - player.x;
            const dz = nodes[i].z - player.z;
            const distance = Math.hypot(dx, dz);
            if (distance < CHASE.sideSpawnMin || distance > CHASE.sideSpawnMax) {
                continue;
            }
            const alignment = Math.abs((dx * forwardX + dz * forwardZ) / distance);
            let occupied = false;
            for (const car of this.cars) {
                if (car.distanceTo(nodes[i].x, nodes[i].z) < 30) {
                    occupied = true;
                    break;
                }
            }
            if (occupied) {
                continue;
            }
            // Prefer junctions off to the side rather than straight down the
            // road the player is already on.
            const score = (1 - alignment) * 2 - Math.abs(distance - 140) / 200 + Math.random() * 0.35;
            if (score > bestScore) {
                bestScore = score;
                best = i;
            }
        }

        if (best < 0) {
            return null;
        }
        const node = nodes[best];
        const car = new PoliceCar(this.nextId++, {
            x: node.x,
            z: node.z,
            heading: Math.atan2(player.x - node.x, player.z - node.z),
        });
        this.cars.push(car);
        return car;
    }

    _targets(player) {
        const graph = this.graph;
        const forwardX = Math.sin(player.heading);
        const forwardZ = Math.cos(player.heading);
        const lead = Math.min(
            CHASE.interceptMaxLead,
            Math.max(CHASE.interceptMinLead, Math.abs(player.speed) * CHASE.interceptSeconds),
        );

        const left = rotateVector(forwardX, forwardZ, -Math.PI / 2.6);
        const right = rotateVector(forwardX, forwardZ, Math.PI / 2.6);

        let centroidX = 0;
        let centroidZ = 0;
        for (const car of this.cars) {
            centroidX += car.x;
            centroidZ += car.z;
        }
        if (this.cars.length > 0) {
            centroidX /= this.cars.length;
            centroidZ /= this.cars.length;
        }
        let awayX = player.x - centroidX;
        let awayZ = player.z - centroidZ;
        const awayLength = Math.hypot(awayX, awayZ) || 1;
        awayX /= awayLength;
        awayZ /= awayLength;

        const nodeOr = (index, fallbackX, fallbackZ) =>
            index >= 0 ? { x: graph.nodes[index].x, z: graph.nodes[index].z } : { x: fallbackX, z: fallbackZ };

        return {
            [ROLE.PURSUE]: { x: player.x, z: player.z },
            [ROLE.INTERCEPT]: { x: player.x + forwardX * lead, z: player.z + forwardZ * lead },
            [ROLE.BLOCK]: nodeOr(
                graph.nodeTowards(player.x, player.z, forwardX, forwardZ, {
                    minDistance: 45,
                    maxDistance: 150,
                    spread: 0.72,
                }),
                player.x + forwardX * 70,
                player.z + forwardZ * 70,
            ),
            [ROLE.ROADBLOCK]: nodeOr(
                graph.nodeTowards(player.x, player.z, forwardX, forwardZ, {
                    minDistance: 150,
                    maxDistance: 300,
                    spread: 0.55,
                }),
                player.x + forwardX * 190,
                player.z + forwardZ * 190,
            ),
            flankLeft: nodeOr(
                graph.nodeTowards(player.x, player.z, left.x, left.z, {
                    minDistance: 55,
                    maxDistance: 190,
                    spread: 0.3,
                }),
                player.x + left.x * 90,
                player.z + left.z * 90,
            ),
            flankRight: nodeOr(
                graph.nodeTowards(player.x, player.z, right.x, right.z, {
                    minDistance: 55,
                    maxDistance: 190,
                    spread: 0.3,
                }),
                player.x + right.x * 90,
                player.z + right.z * 90,
            ),
            [ROLE.CONTAIN]: nodeOr(
                graph.nodeTowards(player.x, player.z, awayX, awayZ, {
                    minDistance: 70,
                    maxDistance: 240,
                    spread: 0.25,
                }),
                player.x + awayX * 120,
                player.z + awayZ * 120,
            ),
        };
    }

    _assign(player) {
        const roles = composeRoles(this.cars.length);
        const targets = this._targets(player);
        const unassigned = new Set(this.cars.map((car) => car.id));
        const byId = new Map(this.cars.map((car) => [car.id, car]));
        let flankFlip = 0;

        for (const role of roles) {
            let target = targets[role];
            if (role === ROLE.FLANK) {
                target = flankFlip % 2 === 0 ? targets.flankLeft : targets.flankRight;
                flankFlip += 1;
            }

            let chosen = null;
            let chosenCost = Infinity;
            for (const id of unassigned) {
                const car = byId.get(id);
                let cost = car.distanceTo(target.x, target.z);
                if (car.role === role) {
                    cost *= 0.78;
                }
                if (cost < chosenCost) {
                    chosenCost = cost;
                    chosen = car;
                }
            }
            if (!chosen) {
                break;
            }

            unassigned.delete(chosen.id);
            const changed = chosen.role !== role
                || Math.hypot(chosen.target.x - target.x, chosen.target.z - target.z) > 25;
            chosen.role = role;
            chosen.target = { x: target.x, z: target.z };
            chosen.hold = role === ROLE.ROADBLOCK || role === ROLE.BLOCK;
            if (changed) {
                chosen.repathIn = 0;
            }
        }
    }

    _repath(car, player) {
        const graph = this.graph;
        const from = graph.nearestNode(car.x, car.z);
        const goalX = car.role === ROLE.PURSUE ? player.x : car.target.x;
        const goalZ = car.role === ROLE.PURSUE ? player.z : car.target.z;
        const to = graph.nearestNode(goalX, goalZ);
        const route = graph.route(from, to);
        const points = graph.pathPoints(route);
        points.push([goalX, goalZ]);
        car.setPath(points);
        car.repathIn = CHASE.repathIntervalSeconds * (0.75 + (car.id % 5) * 0.12);
    }

    update(dt, player) {
        this.elapsed += dt;

        this.reassignIn -= dt;
        if (this.reassignIn <= 0) {
            this._assign(player);
            this.reassignIn = CHASE.reassignIntervalSeconds;
        }

        this.spawnIn -= dt;
        if (this.spawnIn <= 0 && this.cars.length < CHASE.maxCars) {
            this.spawnFromSideStreet(player);
            this.spawnInterval = Math.max(
                CHASE.spawnIntervalFloor,
                this.spawnInterval * CHASE.spawnIntervalDecay,
            );
            this.spawnIn = this.spawnInterval;
            this.reassignIn = 0;
        }

        for (const car of this.cars) {
            car.repathIn -= dt;
            if (car.repathIn <= 0 || car.path.length === 0 || car.pathIndex >= car.path.length) {
                this._repath(car, player);
            }
            const lineOfSight = this.obstacles.firstHitAlong(car.x, car.z, player.x, player.z, 8, 0.2) === null;
            car.update(dt, { collision: this.collision, player, lineOfSight });
        }

        this._separate();
    }

    /** Cars push each other apart instead of overlapping. */
    _separate() {
        const cars = this.cars;
        for (let i = 0; i < cars.length; i += 1) {
            for (let j = i + 1; j < cars.length; j += 1) {
                const a = cars[i].vehicle;
                const b = cars[j].vehicle;
                const dx = b.x - a.x;
                const dz = b.z - a.z;
                const distance = Math.hypot(dx, dz);
                const minimum = 3.4;
                if (distance > minimum || distance < 1e-4) {
                    continue;
                }
                const push = (minimum - distance) * 0.5;
                const nx = dx / distance;
                const nz = dz / distance;
                const aMoved = this.collision.move(a.x, a.z, a.x - nx * push, a.z - nz * push, a.profile.radius);
                const bMoved = this.collision.move(b.x, b.z, b.x + nx * push, b.z + nz * push, b.profile.radius);
                a.x = aMoved.x;
                a.z = aMoved.z;
                b.x = bMoved.x;
                b.z = bMoved.z;
            }
        }
    }

    /** Distance from the player to the closest car, or Infinity when empty. */
    closestDistance(player) {
        let closest = Infinity;
        for (const car of this.cars) {
            closest = Math.min(closest, car.distanceTo(player.x, player.z));
        }
        return closest;
    }
}

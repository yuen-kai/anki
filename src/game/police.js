import { clamp, headingTo, shortestAngle } from "../engine/mathUtils.js";
import { Vehicle } from "./vehicle.js";
import { POLICE_PROFILE } from "./vehicleProfiles.js";

const ARRIVE_RADIUS = 8;
const STUCK_SPEED = 1.6;
const STUCK_SECONDS = 0.75;
const REVERSE_SECONDS = 0.9;

/**
 * A single police car: an arcade vehicle plus a path follower. The unit sets
 * `role`, `target` and `hold`; everything below is only about getting there.
 */
export class PoliceCar {
    constructor(id, spawn) {
        this.id = id;
        this.vehicle = new Vehicle(POLICE_PROFILE, spawn);
        this.role = "pursue";
        this.target = { x: spawn.x, z: spawn.z };
        this.hold = false;
        this.path = [];
        this.pathIndex = 0;
        this.repathIn = 0;
        this.stuckFor = 0;
        this.reverseFor = 0;
        this.sirenPhase = Math.random() * Math.PI * 2;
    }

    get x() {
        return this.vehicle.x;
    }

    get z() {
        return this.vehicle.z;
    }

    get speed() {
        return this.vehicle.speed;
    }

    setPath(points) {
        this.path = points;
        this.pathIndex = 0;
    }

    distanceTo(x, z) {
        return Math.hypot(this.vehicle.x - x, this.vehicle.z - z);
    }

    /** Point on the path roughly `lookahead` metres ahead of the car. */
    _lookaheadPoint(lookahead) {
        const path = this.path;
        while (this.pathIndex < path.length) {
            const point = path[this.pathIndex];
            const distance = this.distanceTo(point[0], point[1]);
            const ahead = (point[0] - this.vehicle.x) * this.vehicle.forwardX
                + (point[1] - this.vehicle.z) * this.vehicle.forwardZ;
            if (distance < ARRIVE_RADIUS || (ahead < 0 && distance < ARRIVE_RADIUS * 2.5)) {
                this.pathIndex += 1;
            } else {
                break;
            }
        }
        if (this.pathIndex >= path.length) {
            return null;
        }

        let travelled = this.distanceTo(path[this.pathIndex][0], path[this.pathIndex][1]);
        let index = this.pathIndex;
        while (travelled < lookahead && index + 1 < path.length) {
            travelled += Math.hypot(path[index + 1][0] - path[index][0], path[index + 1][1] - path[index][1]);
            index += 1;
        }
        return path[index];
    }

    update(dt, { collision, player, lineOfSight }) {
        const vehicle = this.vehicle;
        const speed = Math.abs(vehicle.speed);

        if (speed < STUCK_SPEED && vehicle.contact) {
            this.stuckFor += dt;
        } else if (speed > STUCK_SPEED) {
            this.stuckFor = 0;
        }
        if (this.stuckFor > STUCK_SECONDS && this.reverseFor <= 0) {
            this.reverseFor = REVERSE_SECONDS;
            this.stuckFor = 0;
        }

        if (this.reverseFor > 0) {
            this.reverseFor -= dt;
            vehicle.update(dt, { throttle: -1, steer: this.id % 2 === 0 ? 1 : -1, handbrake: false }, collision);
            return;
        }

        const distanceToPlayer = this.distanceTo(player.x, player.z);
        let aimX;
        let aimZ;

        // Once the player is close and in the open, chase the car itself rather
        // than the road route, so the last few metres stay tight.
        if (lineOfSight && distanceToPlayer < 75) {
            aimX = player.x;
            aimZ = player.z;
        } else {
            const lookahead = 9 + speed * 0.42;
            const point = this._lookaheadPoint(lookahead);
            if (point) {
                aimX = point[0];
                aimZ = point[1];
            } else {
                aimX = this.target.x;
                aimZ = this.target.z;
            }
        }

        const desired = headingTo(vehicle.x, vehicle.z, aimX, aimZ);
        const error = shortestAngle(vehicle.heading, desired);
        const steer = clamp(error * 1.9, -1, 1);

        let throttle = 1;
        const turnSeverity = Math.min(1, Math.abs(error) / 1.1);
        throttle -= turnSeverity * 0.75;

        if (this.hold) {
            const distanceToTarget = this.distanceTo(this.target.x, this.target.z);
            if (distanceToTarget < 7) {
                throttle = speed > 1 ? -1 : 0;
            } else if (distanceToTarget < 22) {
                throttle = Math.min(throttle, 0.35);
            }
        }

        vehicle.update(dt, { throttle, steer, handbrake: false }, collision);
    }
}

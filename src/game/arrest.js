import { CHASE } from "./tuning.js";

/**
 * Holds the arrest timer: it runs while at least one police car is inside the
 * radius and resets the moment none are, and completing it ends the run.
 */
export class ArrestTimer {
    constructor({ radius = CHASE.arrestRadius, seconds = CHASE.arrestSeconds } = {}) {
        this.radius = radius;
        this.seconds = seconds;
        this.held = 0;
        this.closest = Infinity;
        this.carsInside = 0;
    }

    reset() {
        this.held = 0;
        this.closest = Infinity;
        this.carsInside = 0;
    }

    get active() {
        return this.held > 0;
    }

    get progress() {
        return Math.min(1, this.held / this.seconds);
    }

    /** Returns true on the frame the arrest completes. */
    update(dt, player, cars) {
        let closest = Infinity;
        let inside = 0;
        for (const car of cars) {
            const distance = car.distanceTo(player.x, player.z);
            closest = Math.min(closest, distance);
            if (distance <= this.radius) {
                inside += 1;
            }
        }
        this.closest = closest;
        this.carsInside = inside;

        if (inside > 0) {
            this.held += dt;
            return this.held >= this.seconds;
        }
        this.held = 0;
        return false;
    }
}

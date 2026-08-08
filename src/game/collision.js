/**
 * Movement against the solid parts of the world: buildings and the map edge.
 * A blocked move is retried one axis at a time so a car scrapes along a wall
 * instead of stopping dead against it.
 */
export class CollisionWorld {
    constructor({ obstacles, boundsRadius }) {
        this.obstacles = obstacles;
        this.boundsRadius = boundsRadius;
    }

    blocked(x, z, radius) {
        if (Math.hypot(x, z) > this.boundsRadius) {
            return true;
        }
        return this.obstacles.contains(x, z, radius) !== null;
    }

    /**
     * Returns the resolved position and which axes were refused, so callers can
     * scrub the speed they lost.
     */
    move(fromX, fromZ, toX, toZ, radius) {
        if (!this.blocked(toX, toZ, radius)) {
            return { x: toX, z: toZ, hit: false, slid: false };
        }
        if (!this.blocked(toX, fromZ, radius)) {
            return { x: toX, z: fromZ, hit: true, slid: true };
        }
        if (!this.blocked(fromX, toZ, radius)) {
            return { x: fromX, z: toZ, hit: true, slid: true };
        }
        return { x: fromX, z: fromZ, hit: true, slid: false };
    }
}

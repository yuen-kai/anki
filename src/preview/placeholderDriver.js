/**
 * Stand-in movement used only to inspect the world.
 *
 * This is not the vehicle model. The handling questions in QUESTIONS.md are
 * unanswered, so nothing in this file is authoritative and all of it is
 * expected to be deleted once DESIGN.md covers driving.
 */

const FORWARD_KEYS = ["ArrowUp", "KeyW"];
const BACK_KEYS = ["ArrowDown", "KeyS"];
const LEFT_KEYS = ["ArrowLeft", "KeyA"];
const RIGHT_KEYS = ["ArrowRight", "KeyD"];

export class PlaceholderDriver {
    constructor({ start, obstacles, bounds }) {
        this.x = start.x;
        this.z = start.z;
        this.y = 0;
        this.heading = start.heading;
        this.speed = 0;
        this.steer = 0;
        this.travelled = 0;
        this.obstacles = obstacles;
        this.bounds = bounds;
    }

    get state() {
        return { x: this.x, y: this.y, z: this.z, heading: this.heading, speed: this.speed };
    }

    update(dt, keyboard) {
        const throttle = keyboard.anyDown(FORWARD_KEYS) ? 1 : 0;
        const brake = keyboard.anyDown(BACK_KEYS) ? 1 : 0;
        const steerInput = (keyboard.anyDown(LEFT_KEYS) ? -1 : 0) + (keyboard.anyDown(RIGHT_KEYS) ? 1 : 0);

        const drag = 1.1 + Math.abs(this.speed) * 0.045;
        this.speed += (throttle * 16 - brake * 22 - Math.sign(this.speed) * drag) * dt;
        this.speed = Math.max(-8, Math.min(38, this.speed));
        if (throttle === 0 && brake === 0 && Math.abs(this.speed) < 0.25) {
            this.speed = 0;
        }

        const grip = 1 - Math.min(0.55, Math.abs(this.speed) / 70);
        const targetSteer = steerInput * 0.52 * grip;
        this.steer += (targetSteer - this.steer) * Math.min(1, dt * 9);
        this.heading += this.steer * (this.speed / 6.5) * dt;

        const step = this.speed * dt;
        const nextX = this.x + Math.sin(this.heading) * step;
        const nextZ = this.z + Math.cos(this.heading) * step;

        const blocked = this.obstacles && this.obstacles.contains(nextX, nextZ, 1.1);
        const outside = Math.hypot(nextX, nextZ) > this.bounds;
        if (blocked || outside) {
            this.speed *= -0.15;
        } else {
            this.x = nextX;
            this.z = nextZ;
            this.travelled += Math.abs(step);
        }
    }
}

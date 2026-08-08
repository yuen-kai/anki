/**
 * Arcade vehicle: velocity is always along the heading, so there is no lateral
 * slide. Inputs are throttle (-1..1), steer (-1..1) and handbrake.
 */
export class Vehicle {
    constructor(profile, { x, z, heading }) {
        this.profile = profile;
        this.x = x;
        this.z = z;
        this.y = 0;
        this.heading = heading;
        this.speed = 0;
        this.steer = 0;
        this.handbraking = false;
        this.travelled = 0;
        this.contact = false;
    }

    get state() {
        return { x: this.x, y: this.y, z: this.z, heading: this.heading, speed: this.speed };
    }

    get forwardX() {
        return Math.sin(this.heading);
    }

    get forwardZ() {
        return Math.cos(this.heading);
    }

    /** Fraction of top speed, for camera and HUD blending. */
    get speedFraction() {
        return Math.min(1, Math.abs(this.speed) / this.profile.topSpeed);
    }

    update(dt, input, world) {
        const profile = this.profile;
        const throttle = Math.max(-1, Math.min(1, input.throttle ?? 0));
        const steerInput = Math.max(-1, Math.min(1, input.steer ?? 0));
        this.handbraking = Boolean(profile.handbrake && input.handbrake);

        if (throttle > 0) {
            if (this.speed < 0) {
                this.speed += profile.brake * throttle * dt;
            } else {
                this.speed += profile.accelerate * throttle * dt;
            }
        } else if (throttle < 0) {
            if (this.speed > 0) {
                this.speed += profile.brake * throttle * dt;
            } else {
                this.speed += profile.accelerate * throttle * dt;
            }
        } else {
            const drag = profile.coastDrag * dt;
            if (Math.abs(this.speed) <= drag) {
                this.speed = 0;
            } else {
                this.speed -= Math.sign(this.speed) * drag;
            }
        }

        if (this.handbraking) {
            const drag = profile.handbrake.drag * dt;
            if (Math.abs(this.speed) <= drag) {
                this.speed = 0;
            } else {
                this.speed -= Math.sign(this.speed) * drag;
            }
        }

        this.speed = Math.max(-profile.reverseSpeed, Math.min(profile.topSpeed, this.speed));

        this.steer += (steerInput - this.steer) * Math.min(1, dt * profile.steerResponse);

        const speedRatio = Math.abs(this.speed) / profile.topSpeed;
        const engagement = Math.min(1, Math.abs(this.speed) / profile.gripSpeed);
        const authority = engagement * (1 - profile.highSpeedTurnLoss * speedRatio);
        const handbrakeBoost = this.handbraking ? profile.handbrake.turnMultiplier : 1;
        const direction = this.speed < 0 ? -1 : 1;
        this.heading += this.steer * profile.turnRate * authority * handbrakeBoost * direction * dt;

        const step = this.speed * dt;
        const targetX = this.x + this.forwardX * step;
        const targetZ = this.z + this.forwardZ * step;
        const resolved = world.move(this.x, this.z, targetX, targetZ, profile.radius);

        const movedX = resolved.x - this.x;
        const movedZ = resolved.z - this.z;
        this.x = resolved.x;
        this.z = resolved.z;
        this.travelled += Math.hypot(movedX, movedZ);
        this.contact = resolved.hit;

        if (resolved.hit && !resolved.slid) {
            this.speed = 0;
        }
    }
}

import * as THREE from "three";

/** Frame-rate independent exponential approach. */
function damp(current, target, lambda, dt) {
    return current + (target - current) * (1 - Math.exp(-lambda * dt));
}

function shortestAngle(from, to) {
    let delta = (to - from) % (Math.PI * 2);
    if (delta > Math.PI) {
        delta -= Math.PI * 2;
    }
    if (delta < -Math.PI) {
        delta += Math.PI * 2;
    }
    return delta;
}

/**
 * Camera rig that sits behind the car, turns with it, leads the view in the
 * direction of travel and pulls back as speed rises. `referenceSpeed` is the
 * speed at which the pull-back reaches its maximum.
 */
export class ChaseCamera {
    constructor(camera, { referenceSpeed = 34, obstacles = null } = {}) {
        this.camera = camera;
        this.referenceSpeed = referenceSpeed;
        this.obstacles = obstacles;

        this.yaw = 0;
        this.speedBlend = 0;
        this.position = new THREE.Vector3();
        this.lookAt = new THREE.Vector3();
        this.initialised = false;

        this.settings = {
            distance: [7.4, 11.2],
            height: [3.1, 4.05],
            lookAhead: [4.5, 13.5],
            lookHeight: [1.15, 1.4],
            fov: [58, 72],
            yawLambda: [5.2, 8.5],
            positionLambda: 9.5,
        };
    }

    _mix(range) {
        return range[0] + (range[1] - range[0]) * this.speedBlend;
    }

    /** Places the rig without any smoothing, for the first frame or a reset. */
    snap(state) {
        this.yaw = state.heading;
        this.speedBlend = 0;
        this.initialised = false;
        this.update(state, 0.016);
    }

    update(state, dt) {
        const speed = Math.abs(state.speed ?? 0);
        const targetBlend = Math.min(1, speed / this.referenceSpeed);
        this.speedBlend = damp(this.speedBlend, targetBlend, 2.6, dt);

        const yawLambda = this._mix(this.settings.yawLambda);
        this.yaw += shortestAngle(this.yaw, state.heading) * (1 - Math.exp(-yawLambda * dt));

        const forwardX = Math.sin(this.yaw);
        const forwardZ = Math.cos(this.yaw);
        const distance = this._mix(this.settings.distance);
        const height = this._mix(this.settings.height);
        const lookAhead = this._mix(this.settings.lookAhead);
        const lookHeight = this._mix(this.settings.lookHeight);

        let desiredX = state.x - forwardX * distance;
        let desiredZ = state.z - forwardZ * distance;

        if (this.obstacles) {
            const hit = this.obstacles.firstHitAlong(state.x, state.z, desiredX, desiredZ, 10, 0.6);
            if (hit) {
                const pullback = Math.max(0.35, hit.t - 0.12);
                desiredX = state.x - forwardX * distance * pullback;
                desiredZ = state.z - forwardZ * distance * pullback;
            }
        }

        const target = new THREE.Vector3(desiredX, state.y + height, desiredZ);
        const focus = new THREE.Vector3(
            state.x + forwardX * lookAhead,
            state.y + lookHeight,
            state.z + forwardZ * lookAhead,
        );

        if (!this.initialised) {
            this.position.copy(target);
            this.lookAt.copy(focus);
            this.initialised = true;
        } else {
            const lambda = this.settings.positionLambda;
            this.position.x = damp(this.position.x, target.x, lambda, dt);
            this.position.y = damp(this.position.y, target.y, lambda * 0.8, dt);
            this.position.z = damp(this.position.z, target.z, lambda, dt);
            this.lookAt.x = damp(this.lookAt.x, focus.x, lambda * 1.15, dt);
            this.lookAt.y = damp(this.lookAt.y, focus.y, lambda, dt);
            this.lookAt.z = damp(this.lookAt.z, focus.z, lambda * 1.15, dt);
        }

        const fov = this._mix(this.settings.fov);
        if (Math.abs(this.camera.fov - fov) > 0.01) {
            this.camera.fov = fov;
            this.camera.updateProjectionMatrix();
        }

        this.camera.position.copy(this.position);
        this.camera.up.set(0, 1, 0);
        this.camera.lookAt(this.lookAt);
    }
}

export const TAU = Math.PI * 2;

/** Frame-rate independent exponential approach. */
export function damp(current, target, lambda, dt) {
    return current + (target - current) * (1 - Math.exp(-lambda * dt));
}

/** Signed smallest rotation from one angle to another. */
export function shortestAngle(from, to) {
    let delta = (to - from) % TAU;
    if (delta > Math.PI) {
        delta -= TAU;
    }
    if (delta < -Math.PI) {
        delta += TAU;
    }
    return delta;
}

export function clamp(value, low, high) {
    return Math.max(low, Math.min(high, value));
}

/** Heading whose forward vector is (sin h, cos h). */
export function headingTo(fromX, fromZ, toX, toZ) {
    return Math.atan2(toX - fromX, toZ - fromZ);
}

export function rotateVector(x, z, angle) {
    const cos = Math.cos(angle);
    const sin = Math.sin(angle);
    return { x: x * cos + z * sin, z: -x * sin + z * cos };
}

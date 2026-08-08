import { PENDING } from "../pendingAnswers.js";

const THROTTLE = "KeyW";
const BRAKE = "KeyS";
const LEFT = "KeyA";
const RIGHT = "KeyD";

export const PLAYER_KEYS = [THROTTLE, BRAKE, LEFT, RIGHT, PENDING.handbrakeKey];

export function readPlayerInput(keyboard) {
    return {
        throttle: (keyboard.isDown(THROTTLE) ? 1 : 0) - (keyboard.isDown(BRAKE) ? 1 : 0),
        steer: (keyboard.isDown(RIGHT) ? 1 : 0) - (keyboard.isDown(LEFT) ? 1 : 0),
        handbrake: keyboard.isDown(PENDING.handbrakeKey),
    };
}

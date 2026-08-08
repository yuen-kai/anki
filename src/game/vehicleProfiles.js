/**
 * Vehicle constants. The direction of every difference between these two
 * profiles is fixed by DESIGN.md; the magnitudes below are implementation
 * tuning and can be replaced the moment a number is specified.
 *
 * Speeds are metres per second, accelerations metres per second squared, turn
 * rates radians per second.
 */

export const PLAYER_PROFILE = {
    id: "player",
    topSpeed: 42,
    reverseSpeed: 15,
    accelerate: 15,
    brake: 30,
    coastDrag: 4.2,
    turnRate: 2.35,
    steerResponse: 12,
    gripSpeed: 7.5,
    highSpeedTurnLoss: 0.3,
    handbrake: {
        turnMultiplier: 1.8,
        drag: 14,
    },
    radius: 1.25,
};

export const POLICE_PROFILE = {
    id: "police",
    topSpeed: 48.3,
    reverseSpeed: 7,
    accelerate: 9.5,
    brake: 17,
    coastDrag: 3.4,
    turnRate: 1.45,
    steerResponse: 5.5,
    gripSpeed: 11,
    highSpeedTurnLoss: 0.5,
    handbrake: null,
    radius: 1.25,
};

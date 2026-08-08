/**
 * Values DESIGN.md does not specify yet.
 *
 * None of these is a decision. They are parked here, each tagged with the
 * question it is waiting on, so the game can run before every answer is in.
 * They are listed on screen so nothing is silently assumed, and they are all
 * expected to be replaced by whatever the answer turns out to be.
 */

export const PENDING = {
    /** 12.1b — which key is the handbrake. */
    handbrakeKey: "Space",
    /** 2.5c — whether street trees and lamps are solid. */
    propsAreSolid: false,
    /** 10.2b — whether the minimap turns with the car or stays north-up. */
    minimapRotates: false,
};

export const PENDING_NOTES = [
    { question: "12.1b", what: "handbrake key", parked: "Space" },
    { question: "2.5c", what: "trees and lamps solid", parked: "no" },
    { question: "10.2b", what: "minimap orientation", parked: "north-up" },
];

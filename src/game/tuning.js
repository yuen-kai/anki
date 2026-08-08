/**
 * Numbers the design doc leaves to the implementer. Counts marked "specified"
 * come straight from DESIGN.md; the rest fill in the "you decide the numbers"
 * gaps and can be replaced without touching any logic.
 */
export const CHASE = {
    /** specified */
    initialCars: 5,
    spawnIntervalSeconds: 16,
    spawnIntervalFloor: 7,
    spawnIntervalDecay: 0.94,
    maxCars: 18,

    arrestRadius: 15,
    arrestSeconds: 3,

    /** How far ahead the unit aims when predicting where the player will be. */
    interceptSeconds: 3.2,
    interceptMinLead: 25,
    interceptMaxLead: 150,

    reassignIntervalSeconds: 0.9,
    repathIntervalSeconds: 1.1,

    initialSpawnGap: 13,
    initialSpawnOffset: 55,
    sideSpawnMin: 95,
    sideSpawnMax: 230,
};

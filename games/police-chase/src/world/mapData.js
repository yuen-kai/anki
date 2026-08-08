/**
 * Hand-authored map source data. Every coordinate is in metres on the ground
 * plane: +x east, +z south, y up. Road entries are centrelines; `curved: true`
 * runs the points through a Catmull-Rom spline, otherwise they are used as a
 * polyline. `closed: true` joins the last point back to the first.
 *
 * Endpoints are placed so that each one lands on another road; `validateMap`
 * in roadNetwork.js asserts this at load time.
 */

export const ROAD_WIDTH = {
    ring: 20,
    avenue: 20,
    boulevard: 18,
    street: 14,
};

export const SIDEWALK_WIDTH = 4.5;

/** Street grid lines. Blocks are the cells between these and the ring. */
const GRID = [-250, -170, -85, 0, 85, 170, 250];
const RING_EXTENT = 330;
const RING_CORNER = 265;

/**
 * Rounded rectangle through the grid crossings. The collinear points along each
 * edge keep the spline straight between the corners, so grid streets meet the
 * ring square-on.
 */
function ringPoints() {
    const along = [-RING_CORNER, ...GRID, RING_CORNER];
    const points = [];
    points.push([-RING_EXTENT, -RING_CORNER]);
    for (const x of along) {
        points.push([x, -RING_EXTENT]);
    }
    points.push([RING_EXTENT, -RING_CORNER]);
    for (const z of along) {
        points.push([RING_EXTENT, z]);
    }
    points.push([RING_CORNER, RING_EXTENT]);
    for (const x of [...along].reverse()) {
        points.push([x, RING_EXTENT]);
    }
    points.push([-RING_EXTENT, RING_CORNER]);
    for (const z of [...along].reverse()) {
        points.push([-RING_EXTENT, z]);
    }
    return points;
}

const RING = ringPoints();

function roundabout(radius, segments = 24) {
    const points = [];
    for (let i = 0; i < segments; i += 1) {
        const angle = (i / segments) * Math.PI * 2;
        points.push([Math.cos(angle) * radius, Math.sin(angle) * radius]);
    }
    return points;
}

function northSouth(x, width = ROAD_WIDTH.street) {
    return { points: [[x, -RING_EXTENT], [x, RING_EXTENT]], width };
}

function eastWest(z, width = ROAD_WIDTH.street) {
    return { points: [[-RING_EXTENT, z], [RING_EXTENT, z]], width };
}

export const ROADS = [
    { id: "ring", points: RING, width: ROAD_WIDTH.ring, curved: true, closed: true },
    { id: "circus", points: roundabout(46), width: ROAD_WIDTH.street, closed: true },

    { id: "central-n", points: [[0, -RING_EXTENT], [0, -46]], width: ROAD_WIDTH.avenue },
    { id: "central-s", points: [[0, 46], [0, RING_EXTENT]], width: ROAD_WIDTH.avenue },
    { id: "broad-w", points: [[-RING_EXTENT, 0], [-46, 0]], width: ROAD_WIDTH.avenue },
    { id: "broad-e", points: [[46, 0], [RING_EXTENT, 0]], width: ROAD_WIDTH.avenue },

    { id: "ns-250w", ...northSouth(-250) },
    { id: "ns-85w", ...northSouth(-85) },
    { id: "ns-85e", ...northSouth(85) },
    { id: "ns-250e", ...northSouth(250) },

    { id: "ew-250n", ...eastWest(-250) },
    { id: "ew-170n", ...eastWest(-170) },
    { id: "ew-85n", ...eastWest(-85) },
    { id: "ew-170s", ...eastWest(170) },
    { id: "ew-250s", ...eastWest(250) },

    {
        id: "bend-west",
        points: [[-170, -330], [-198, -212], [-152, -74], [-186, 66], [-158, 210], [-170, 330]],
        width: ROAD_WIDTH.street,
        curved: true,
    },
    {
        id: "bend-south",
        points: [[-330, 85], [-180, 104], [0, 118], [180, 101], [330, 85]],
        width: ROAD_WIDTH.street,
        curved: true,
    },
    {
        id: "diagonal",
        points: [[-330, 60], [-232, 112], [-120, 182], [12, 242], [170, 330]],
        width: ROAD_WIDTH.boulevard,
        curved: true,
    },
    {
        id: "crescent",
        points: [[330, -85], [278, -158], [198, -214], [85, -250]],
        width: ROAD_WIDTH.street,
        curved: true,
    },

    // Split so the block between them can be a park with street frontage.
    { id: "ns-170e-n", points: [[170, -330], [170, -170]], width: ROAD_WIDTH.street },
    { id: "ns-170e-s", points: [[170, -85], [170, 330]], width: ROAD_WIDTH.street },
];

/** Grass areas. Buildings are excluded from these footprints. */
export const PARKS = [
    { id: "circus-green", shape: "circle", x: 0, z: 0, radius: 34 },
    { id: "kestrel-park", shape: "rect", x0: 92, z0: -163, x1: 163, z1: -92 },
    { id: "quarry-green", shape: "rect", x0: -243, z0: -78, x1: -177, z1: -7 },
    { id: "elm-square", shape: "rect", x0: 177, z0: 177, x1: 243, z1: 243 },
];

/** Paved open ground. Buildings are excluded from these footprints. */
export const LOTS = [
    { id: "north-yard", shape: "rect", x0: -163, z0: -243, x1: -92, z1: -177 },
    { id: "east-yard", shape: "rect", x0: 257, z0: -78, x1: 323, z1: -7 },
    { id: "south-yard", shape: "rect", x0: -78, z0: 257, x1: -7, z1: 323 },
];

export const MAP = {
    /** Half-extent of the square that the surface mask covers. */
    cityHalf: 370,
    /** Buildings are kept inside this half-extent, which is just inside the ring. */
    builtEdge: 332,
    /** Radius of the flat terrain disc the city sits on. */
    groundRadius: 620,
    mountains: {
        innerRadius: 596,
        outerRadius: 1500,
        peakHeight: 260,
    },
    roads: ROADS,
    parks: PARKS,
    lots: LOTS,
    sidewalkWidth: SIDEWALK_WIDTH,
    /** Grid lines, exported so other builders can reason about block cells. */
    grid: GRID,
    ringExtent: RING_EXTENT,
    start: { x: -85, z: 210, heading: Math.PI },
};

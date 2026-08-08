import * as THREE from "three";
import { RoundedBoxGeometry } from "three/addons/geometries/RoundedBoxGeometry.js";

const WHEEL_RADIUS = 0.35;
const WHEEL_WIDTH = 0.3;
const TRACK = 0.84;
const WHEELBASE = 1.36;

const shared = {
    wheel: null,
    hub: null,
};

function wheelGeometry() {
    if (!shared.wheel) {
        shared.wheel = new THREE.CylinderGeometry(WHEEL_RADIUS, WHEEL_RADIUS, WHEEL_WIDTH, 20);
        shared.wheel.rotateZ(Math.PI / 2);
        shared.hub = new THREE.CylinderGeometry(WHEEL_RADIUS * 0.52, WHEEL_RADIUS * 0.52, WHEEL_WIDTH + 0.02, 12);
        shared.hub.rotateZ(Math.PI / 2);
    }
    return shared;
}

function makeWheel(hubColor) {
    const { wheel, hub } = wheelGeometry();
    const group = new THREE.Group();
    const tyre = new THREE.Mesh(
        wheel,
        new THREE.MeshStandardMaterial({ color: 0x14161a, roughness: 0.92, metalness: 0.02 }),
    );
    tyre.castShadow = true;
    const rim = new THREE.Mesh(
        hub,
        new THREE.MeshStandardMaterial({ color: hubColor, roughness: 0.34, metalness: 0.85 }),
    );
    group.add(tyre, rim);
    return group;
}

/**
 * Low-poly car built from rounded boxes. `livery` selects the extra pieces:
 * "police" adds a roof bar, push bar and door panels.
 */
export function createCarModel({
    bodyColor = 0xb63a3a,
    roofColor = null,
    rimColor = 0xb9c0c8,
    livery = "plain",
} = {}) {
    const group = new THREE.Group();
    group.name = `car-${livery}`;

    const paint = new THREE.MeshPhysicalMaterial({
        color: bodyColor,
        roughness: 0.34,
        metalness: 0.32,
        clearcoat: 0.7,
        clearcoatRoughness: 0.22,
    });
    const trim = new THREE.MeshStandardMaterial({ color: 0x1b1e23, roughness: 0.62, metalness: 0.25 });
    const glass = new THREE.MeshPhysicalMaterial({
        color: 0x121b24,
        roughness: 0.12,
        metalness: 0.1,
        clearcoat: 1,
        clearcoatRoughness: 0.05,
    });

    const body = new THREE.Mesh(new RoundedBoxGeometry(1.86, 0.7, 4.32, 4, 0.24), paint);
    body.position.y = 0.68;
    body.castShadow = true;
    body.receiveShadow = true;
    group.add(body);

    const skirt = new THREE.Mesh(new RoundedBoxGeometry(1.74, 0.26, 4.06, 2, 0.1), trim);
    skirt.position.y = 0.36;
    skirt.castShadow = true;
    group.add(skirt);

    const greenhouse = new THREE.Mesh(new RoundedBoxGeometry(1.6, 0.62, 2.02, 4, 0.2), glass);
    greenhouse.position.set(0, 1.24, -0.16);
    greenhouse.castShadow = true;
    group.add(greenhouse);

    const roof = new THREE.Mesh(
        new RoundedBoxGeometry(1.5, 0.18, 1.78, 3, 0.09),
        roofColor === null ? paint : new THREE.MeshPhysicalMaterial({
            color: roofColor,
            roughness: 0.34,
            metalness: 0.32,
            clearcoat: 0.7,
        }),
    );
    roof.position.set(0, 1.5, -0.2);
    roof.castShadow = true;
    group.add(roof);

    const bonnetVent = new THREE.Mesh(new RoundedBoxGeometry(0.9, 0.08, 0.5, 2, 0.04), trim);
    bonnetVent.position.set(0, 1.03, 1.26);
    group.add(bonnetVent);

    const headlightMaterial = new THREE.MeshStandardMaterial({
        color: 0xfff6e0,
        emissive: 0xfff0cc,
        emissiveIntensity: 0.85,
        roughness: 0.2,
    });
    const taillightMaterial = new THREE.MeshStandardMaterial({
        color: 0x8c1414,
        emissive: 0xff2a2a,
        emissiveIntensity: 0.7,
        roughness: 0.3,
    });
    for (const side of [-1, 1]) {
        const headlight = new THREE.Mesh(new THREE.BoxGeometry(0.46, 0.16, 0.1), headlightMaterial);
        headlight.position.set(side * 0.6, 0.78, 2.13);
        group.add(headlight);

        const taillight = new THREE.Mesh(new THREE.BoxGeometry(0.44, 0.16, 0.08), taillightMaterial);
        taillight.position.set(side * 0.62, 0.84, -2.14);
        group.add(taillight);
    }

    const wheels = {};
    const wheelPositions = {
        frontLeft: [-TRACK, WHEEL_RADIUS, WHEELBASE],
        frontRight: [TRACK, WHEEL_RADIUS, WHEELBASE],
        rearLeft: [-TRACK, WHEEL_RADIUS, -WHEELBASE],
        rearRight: [TRACK, WHEEL_RADIUS, -WHEELBASE],
    };
    for (const [name, position] of Object.entries(wheelPositions)) {
        const pivot = new THREE.Group();
        pivot.position.set(position[0], position[1], position[2]);
        const wheel = makeWheel(rimColor);
        pivot.add(wheel);
        group.add(pivot);
        wheels[name] = { pivot, wheel };
    }

    const sirens = [];
    if (livery === "police") {
        const panel = new THREE.MeshStandardMaterial({ color: 0x14181d, roughness: 0.5, metalness: 0.2 });
        for (const side of [-1, 1]) {
            const door = new THREE.Mesh(new RoundedBoxGeometry(0.06, 0.5, 2.2, 2, 0.03), panel);
            door.position.set(side * 0.94, 0.72, -0.1);
            group.add(door);
        }

        const pushBar = new THREE.Mesh(new RoundedBoxGeometry(1.6, 0.5, 0.12, 2, 0.05), panel);
        pushBar.position.set(0, 0.62, 2.24);
        pushBar.castShadow = true;
        group.add(pushBar);

        const barBase = new THREE.Mesh(new RoundedBoxGeometry(1.18, 0.09, 0.3, 2, 0.04), panel);
        barBase.position.set(0, 1.63, -0.05);
        barBase.castShadow = true;
        group.add(barBase);

        const lensGeometry = new RoundedBoxGeometry(0.5, 0.15, 0.26, 2, 0.06);
        for (const [side, color] of [[-1, 0xff2b2b], [1, 0x2b6bff]]) {
            const material = new THREE.MeshStandardMaterial({
                color,
                emissive: color,
                emissiveIntensity: 0.4,
                roughness: 0.25,
                transparent: true,
                opacity: 0.92,
            });
            const lens = new THREE.Mesh(lensGeometry, material);
            lens.position.set(side * 0.31, 1.72, -0.05);
            group.add(lens);
            sirens.push({ mesh: lens, material, color: new THREE.Color(color), side });
        }
    }

    return {
        group,
        wheels,
        sirens,
        /** Steering angle in radians applied to the front wheels. */
        setSteer(angle) {
            wheels.frontLeft.pivot.rotation.y = angle;
            wheels.frontRight.pivot.rotation.y = angle;
        },
        /** Rolls the wheels by a travelled distance in metres. */
        roll(distance) {
            const spin = distance / WHEEL_RADIUS;
            for (const name of Object.keys(wheels)) {
                wheels[name].wheel.rotation.x -= spin;
            }
        },
    };
}

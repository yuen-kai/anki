import * as THREE from "three";
import { mergeGeometries } from "three/addons/utils/BufferGeometryUtils.js";
import { RoundedBoxGeometry } from "three/addons/geometries/RoundedBoxGeometry.js";

const WHEEL_RADIUS = 0.35;
const WHEEL_WIDTH = 0.3;
const TRACK = 0.84;
const WHEELBASE = 1.36;

/**
 * Parts that share a material are merged into a single geometry, so a car costs
 * a handful of draw calls rather than one per box. Only the wheels and the
 * siren lenses stay separate, because they animate independently.
 */
function placed(geometry, [x, y, z], rotationY = 0) {
    const clone = geometry.clone();
    if (rotationY !== 0) {
        clone.rotateY(rotationY);
    }
    clone.translate(x, y, z);
    return clone;
}

let wheelGeometry = null;

function buildWheelGeometry() {
    if (wheelGeometry) {
        return wheelGeometry;
    }
    const tyre = new THREE.CylinderGeometry(WHEEL_RADIUS, WHEEL_RADIUS, WHEEL_WIDTH, 18);
    tyre.rotateZ(Math.PI / 2);
    const hub = new THREE.CylinderGeometry(WHEEL_RADIUS * 0.54, WHEEL_RADIUS * 0.54, WHEEL_WIDTH + 0.03, 12);
    hub.rotateZ(Math.PI / 2);

    const colours = [];
    const paint = (geometry, colour) => {
        const count = geometry.attributes.position.count;
        for (let i = 0; i < count; i += 1) {
            colours.push(colour.r, colour.g, colour.b);
        }
    };
    const tyreColour = new THREE.Color(0x14161a);
    const hubColour = new THREE.Color(0xb9c0c8);
    paint(tyre, tyreColour);
    paint(hub, hubColour);

    const merged = mergeGeometries([tyre, hub], false);
    merged.setAttribute("color", new THREE.Float32BufferAttribute(colours, 3));
    wheelGeometry = merged;
    return merged;
}

export function createCarModel({
    bodyColor = 0xb63a3a,
    roofColor = null,
    rimColor = 0xb9c0c8,
    livery = "plain",
} = {}) {
    const group = new THREE.Group();
    group.name = `car-${livery}`;

    const paintParts = [];
    const trimParts = [];

    paintParts.push(placed(new RoundedBoxGeometry(1.86, 0.7, 4.32, 4, 0.24), [0, 0.68, 0]));
    paintParts.push(placed(new RoundedBoxGeometry(1.5, 0.18, 1.78, 3, 0.09), [0, 1.5, -0.2]));

    trimParts.push(placed(new RoundedBoxGeometry(1.74, 0.26, 4.06, 2, 0.1), [0, 0.36, 0]));
    trimParts.push(placed(new RoundedBoxGeometry(0.9, 0.08, 0.5, 2, 0.04), [0, 1.03, 1.26]));

    const lightParts = [];
    const tailParts = [];
    for (const side of [-1, 1]) {
        lightParts.push(placed(new THREE.BoxGeometry(0.46, 0.16, 0.1), [side * 0.6, 0.78, 2.13]));
        tailParts.push(placed(new THREE.BoxGeometry(0.44, 0.16, 0.08), [side * 0.62, 0.84, -2.14]));
    }

    if (livery === "police") {
        for (const side of [-1, 1]) {
            trimParts.push(placed(new RoundedBoxGeometry(0.06, 0.5, 2.2, 2, 0.03), [side * 0.94, 0.72, -0.1]));
        }
        trimParts.push(placed(new RoundedBoxGeometry(1.6, 0.5, 0.12, 2, 0.05), [0, 0.62, 2.24]));
        trimParts.push(placed(new RoundedBoxGeometry(1.18, 0.09, 0.3, 2, 0.04), [0, 1.63, -0.05]));
    }

    const paintMaterial = new THREE.MeshPhysicalMaterial({
        color: bodyColor,
        roughness: 0.34,
        metalness: 0.32,
        clearcoat: 0.7,
        clearcoatRoughness: 0.22,
    });
    const trimMaterial = new THREE.MeshStandardMaterial({ color: 0x1b1e23, roughness: 0.62, metalness: 0.25 });
    const glassMaterial = new THREE.MeshPhysicalMaterial({
        color: 0x121b24,
        roughness: 0.12,
        metalness: 0.1,
        clearcoat: 1,
        clearcoatRoughness: 0.05,
    });

    const body = new THREE.Mesh(mergeGeometries(paintParts, false), paintMaterial);
    body.castShadow = true;
    body.receiveShadow = true;
    group.add(body);

    if (roofColor !== null && roofColor !== bodyColor) {
        body.geometry.dispose();
        body.geometry = mergeGeometries([paintParts[0]], false);
        const roof = new THREE.Mesh(
            paintParts[1],
            new THREE.MeshPhysicalMaterial({ color: roofColor, roughness: 0.34, metalness: 0.32, clearcoat: 0.7 }),
        );
        roof.castShadow = true;
        group.add(roof);
    }

    const trim = new THREE.Mesh(mergeGeometries(trimParts, false), trimMaterial);
    trim.castShadow = true;
    group.add(trim);

    const greenhouse = new THREE.Mesh(new RoundedBoxGeometry(1.6, 0.62, 2.02, 4, 0.2), glassMaterial);
    greenhouse.position.set(0, 1.24, -0.16);
    greenhouse.castShadow = true;
    group.add(greenhouse);

    group.add(
        new THREE.Mesh(
            mergeGeometries(lightParts, false),
            new THREE.MeshStandardMaterial({
                color: 0xfff6e0,
                emissive: 0xfff0cc,
                emissiveIntensity: 0.85,
                roughness: 0.2,
            }),
        ),
    );
    group.add(
        new THREE.Mesh(
            mergeGeometries(tailParts, false),
            new THREE.MeshStandardMaterial({
                color: 0x8c1414,
                emissive: 0xff2a2a,
                emissiveIntensity: 0.7,
                roughness: 0.3,
            }),
        ),
    );

    const wheelMaterial = new THREE.MeshStandardMaterial({
        color: rimColor,
        vertexColors: true,
        roughness: 0.7,
        metalness: 0.3,
    });
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
        const wheel = new THREE.Mesh(buildWheelGeometry(), wheelMaterial);
        wheel.castShadow = true;
        pivot.add(wheel);
        group.add(pivot);
        wheels[name] = { pivot, wheel };
    }

    const sirens = [];
    if (livery === "police") {
        const lensGeometry = new RoundedBoxGeometry(0.5, 0.15, 0.26, 2, 0.06);
        for (const [side, color] of [[-1, 0xff2b2b], [1, 0x2b6bff]]) {
            const material = new THREE.MeshStandardMaterial({
                color,
                emissive: color,
                emissiveIntensity: 0.4,
                roughness: 0.25,
            });
            const lens = new THREE.Mesh(lensGeometry, material);
            lens.position.set(side * 0.31, 1.72, -0.05);
            group.add(lens);
            sirens.push({ mesh: lens, material, side });
        }
    }

    return {
        group,
        wheels,
        sirens,
        setSteer(angle) {
            wheels.frontLeft.pivot.rotation.y = angle;
            wheels.frontRight.pivot.rotation.y = angle;
        },
        roll(distance) {
            const spin = distance / WHEEL_RADIUS;
            for (const name of Object.keys(wheels)) {
                wheels[name].wheel.rotation.x -= spin;
            }
        },
    };
}

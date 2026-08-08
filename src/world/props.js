import * as THREE from "three";

import { ObbField } from "./colliders.js";
import { makeRandom } from "./textures.js";

const TREE_SPACING_PARK = 8.5;
const TREE_SPACING_STREET = 26;
const LAMP_SPACING = 42;

function insideShape(shape, x, z, inset) {
    if (shape.shape === "circle") {
        return Math.hypot(x - shape.x, z - shape.z) < shape.radius - inset;
    }
    return x > shape.x0 + inset && x < shape.x1 - inset && z > shape.z0 + inset && z < shape.z1 - inset;
}

function shapeBounds(shape) {
    if (shape.shape === "circle") {
        return { x0: shape.x - shape.radius, z0: shape.z - shape.radius, x1: shape.x + shape.radius, z1: shape.z + shape.radius };
    }
    return shape;
}

function makeInstanced(geometry, material, count) {
    const mesh = new THREE.InstancedMesh(geometry, material, count);
    mesh.instanceMatrix.setUsage(THREE.StaticDrawUsage);
    mesh.castShadow = true;
    mesh.receiveShadow = true;
    mesh.matrixAutoUpdate = false;
    mesh.frustumCulled = true;
    return mesh;
}

/**
 * Scenery: park and street trees, and street lamps. These are visual only —
 * whether any of them obstructs a car is question 2.5c and is not decided here,
 * so no collider is produced.
 */
export function createProps({ map, network, colliders }) {
    const random = makeRandom(778899);
    const buildings = new ObbField(colliders);
    const group = new THREE.Group();
    group.name = "props";

    const trees = [];
    const placedTrees = [];
    const canPlaceTree = (x, z, spacing) => {
        if (buildings.contains(x, z, 1.2)) {
            return false;
        }
        for (const other of placedTrees) {
            if (Math.hypot(other[0] - x, other[1] - z) < spacing) {
                return false;
            }
        }
        return true;
    };

    for (const park of map.parks) {
        const bounds = shapeBounds(park);
        const area = (bounds.x1 - bounds.x0) * (bounds.z1 - bounds.z0);
        const attempts = Math.min(600, Math.round(area / 18));
        for (let i = 0; i < attempts; i += 1) {
            const x = bounds.x0 + random() * (bounds.x1 - bounds.x0);
            const z = bounds.z0 + random() * (bounds.z1 - bounds.z0);
            if (!insideShape(park, x, z, 4)) {
                continue;
            }
            if (network.clearanceFromRoads(x, z) < 3) {
                continue;
            }
            if (!canPlaceTree(x, z, TREE_SPACING_PARK)) {
                continue;
            }
            placedTrees.push([x, z]);
            trees.push({ x, z, scale: 0.85 + random() * 0.8, tint: random(), spin: random() * Math.PI * 2 });
        }
    }

    const lamps = [];
    for (const road of network.roads) {
        let travelled = 0;
        let nextTree = 12 + random() * 10;
        let nextLamp = 20 + random() * 20;
        for (let i = 1; i < road.polyline.length; i += 1) {
            const [ax, az] = road.polyline[i - 1];
            const [bx, bz] = road.polyline[i];
            const segment = Math.hypot(bx - ax, bz - az);
            if (segment < 1e-5) {
                continue;
            }
            const tx = (bx - ax) / segment;
            const tz = (bz - az) / segment;
            travelled += segment;

            while (travelled > nextTree) {
                nextTree += TREE_SPACING_STREET * (0.7 + random() * 0.7);
                if (random() < 0.4) {
                    continue;
                }
                const side = random() < 0.5 ? -1 : 1;
                const offset = road.halfWidth + 2.3;
                const x = bx + tz * offset * side;
                const z = bz - tx * offset * side;
                if (network.nearJunction(x, z, 4) || !canPlaceTree(x, z, TREE_SPACING_STREET * 0.5)) {
                    continue;
                }
                if (Math.abs(x) > map.cityHalf - 6 || Math.abs(z) > map.cityHalf - 6) {
                    continue;
                }
                placedTrees.push([x, z]);
                trees.push({ x, z, scale: 0.7 + random() * 0.45, tint: random(), spin: random() * Math.PI * 2 });
            }

            while (travelled > nextLamp) {
                nextLamp += LAMP_SPACING;
                const side = lamps.length % 2 === 0 ? -1 : 1;
                const offset = road.halfWidth + 1.3;
                const x = bx + tz * offset * side;
                const z = bz - tx * offset * side;
                if (network.nearJunction(x, z, 6)) {
                    continue;
                }
                if (buildings.contains(x, z, 0.8)) {
                    continue;
                }
                if (Math.abs(x) > map.cityHalf - 4 || Math.abs(z) > map.cityHalf - 4) {
                    continue;
                }
                lamps.push({ x, z, angle: Math.atan2(tz * side, -tx * side) });
            }
        }
    }

    if (trees.length > 0) {
        const trunkGeometry = new THREE.CylinderGeometry(0.17, 0.26, 2.4, 6);
        trunkGeometry.translate(0, 1.2, 0);
        const canopyGeometry = new THREE.IcosahedronGeometry(1, 0);

        const trunkMesh = makeInstanced(
            trunkGeometry,
            new THREE.MeshStandardMaterial({ color: 0x5c4632, roughness: 0.95, flatShading: true }),
            trees.length,
        );
        const canopyMesh = makeInstanced(
            canopyGeometry,
            new THREE.MeshStandardMaterial({ roughness: 0.88, flatShading: true, vertexColors: false }),
            trees.length,
        );
        canopyMesh.instanceColor = new THREE.InstancedBufferAttribute(new Float32Array(trees.length * 3), 3);

        const matrix = new THREE.Matrix4();
        const quaternion = new THREE.Quaternion();
        const positionVector = new THREE.Vector3();
        const scaleVector = new THREE.Vector3();
        const color = new THREE.Color();

        trees.forEach((tree, i) => {
            positionVector.set(tree.x, 0, tree.z);
            quaternion.setFromAxisAngle(new THREE.Vector3(0, 1, 0), tree.spin);
            scaleVector.setScalar(tree.scale);
            matrix.compose(positionVector, quaternion, scaleVector);
            trunkMesh.setMatrixAt(i, matrix);

            positionVector.set(tree.x, 2.4 * tree.scale + 1.35 * tree.scale, tree.z);
            scaleVector.set(1.85 * tree.scale, 1.55 * tree.scale, 1.85 * tree.scale);
            matrix.compose(positionVector, quaternion, scaleVector);
            canopyMesh.setMatrixAt(i, matrix);

            color.setHSL(0.24 + tree.tint * 0.06, 0.42 + tree.tint * 0.18, 0.24 + tree.tint * 0.12);
            canopyMesh.setColorAt(i, color);
        });
        trunkMesh.instanceMatrix.needsUpdate = true;
        canopyMesh.instanceMatrix.needsUpdate = true;
        canopyMesh.instanceColor.needsUpdate = true;
        canopyMesh.updateMatrix();
        trunkMesh.updateMatrix();
        group.add(trunkMesh, canopyMesh);
    }

    if (lamps.length > 0) {
        const poleGeometry = new THREE.CylinderGeometry(0.09, 0.13, 7, 6);
        poleGeometry.translate(0, 3.5, 0);
        const headGeometry = new THREE.BoxGeometry(0.36, 0.16, 1.1);
        headGeometry.translate(0, 6.95, 0.6);

        const metal = new THREE.MeshStandardMaterial({ color: 0x3d444c, roughness: 0.55, metalness: 0.6 });
        const poleMesh = makeInstanced(poleGeometry, metal, lamps.length);
        const headMesh = makeInstanced(headGeometry, metal, lamps.length);

        const matrix = new THREE.Matrix4();
        const quaternion = new THREE.Quaternion();
        const axis = new THREE.Vector3(0, 1, 0);
        const positionVector = new THREE.Vector3();
        const scaleVector = new THREE.Vector3(1, 1, 1);

        lamps.forEach((lamp, i) => {
            positionVector.set(lamp.x, 0, lamp.z);
            quaternion.setFromAxisAngle(axis, lamp.angle);
            matrix.compose(positionVector, quaternion, scaleVector);
            poleMesh.setMatrixAt(i, matrix);
            headMesh.setMatrixAt(i, matrix);
        });
        poleMesh.instanceMatrix.needsUpdate = true;
        headMesh.instanceMatrix.needsUpdate = true;
        poleMesh.updateMatrix();
        headMesh.updateMatrix();
        group.add(poleMesh, headMesh);
    }

    return { group, treeCount: trees.length, lampCount: lamps.length };
}

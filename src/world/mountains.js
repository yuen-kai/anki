import * as THREE from "three";

import { makeRandom } from "./textures.js";

/** Tileable-in-angle value noise on a polar grid. */
function polarNoise(angleCells, radialCells, seed) {
    const random = makeRandom(seed);
    const grid = new Float32Array(angleCells * radialCells);
    for (let i = 0; i < grid.length; i += 1) {
        grid[i] = random();
    }
    const smooth = (t) => t * t * (3 - 2 * t);
    return (angleT, radialT) => {
        const fa = angleT * angleCells;
        const fr = Math.max(0, Math.min(radialCells - 1.001, radialT * (radialCells - 1)));
        const a0 = Math.floor(fa);
        const r0 = Math.floor(fr);
        const ta = smooth(fa - a0);
        const tr = smooth(fr - r0);
        const at = (a, r) => grid[(((a % angleCells) + angleCells) % angleCells) * radialCells + Math.min(radialCells - 1, r)];
        const top = at(a0, r0) + (at(a0 + 1, r0) - at(a0, r0)) * ta;
        const bottom = at(a0, r0 + 1) + (at(a0 + 1, r0 + 1) - at(a0, r0 + 1)) * ta;
        return top + (bottom - top) * tr;
    };
}

export function createMountains(map) {
    const { innerRadius, outerRadius, peakHeight } = map.mountains;
    const angleSegments = 320;
    const radialSegments = 26;

    const ridge = polarNoise(48, 8, 8123);
    const detail = polarNoise(160, 14, 5501);
    const coarse = polarNoise(11, 5, 991);

    const positions = [];
    const colors = [];
    const indices = [];

    const rock = new THREE.Color(0x6b6459);
    const rockDark = new THREE.Color(0x4c4740);
    const grass = new THREE.Color(0x55703c);
    const snow = new THREE.Color(0xe8eef2);
    const scratch = new THREE.Color();

    for (let a = 0; a <= angleSegments; a += 1) {
        const angleT = a / angleSegments;
        const angle = angleT * Math.PI * 2;
        const cos = Math.cos(angle);
        const sin = Math.sin(angle);
        for (let r = 0; r <= radialSegments; r += 1) {
            const radialT = r / radialSegments;
            const radius = innerRadius + (outerRadius - innerRadius) * Math.pow(radialT, 1.35);

            const rise = Math.pow(Math.min(1, radialT / 0.42), 1.6);
            const massif = 0.35 + coarse(angleT, radialT * 0.6) * 0.95;
            const crest = Math.pow(ridge(angleT, radialT), 1.25);
            const grain = detail(angleT, radialT) - 0.5;
            const height = Math.max(
                0,
                peakHeight * rise * massif * (0.35 + crest * 0.9) + grain * 46 * rise,
            );

            positions.push(cos * radius, height, sin * radius);

            const elevation = height / peakHeight;
            if (elevation < 0.12) {
                scratch.copy(grass).lerp(rock, elevation / 0.12);
            } else if (elevation < 0.62) {
                scratch.copy(rock).lerp(rockDark, (elevation - 0.12) / 0.5);
            } else {
                scratch.copy(rockDark).lerp(snow, Math.min(1, (elevation - 0.62) / 0.28));
            }
            colors.push(scratch.r, scratch.g, scratch.b);
        }
    }

    const stride = radialSegments + 1;
    for (let a = 0; a < angleSegments; a += 1) {
        for (let r = 0; r < radialSegments; r += 1) {
            const i0 = a * stride + r;
            const i1 = (a + 1) * stride + r;
            indices.push(i0, i1, i0 + 1, i1, i1 + 1, i0 + 1);
        }
    }

    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute("position", new THREE.Float32BufferAttribute(positions, 3));
    geometry.setAttribute("color", new THREE.Float32BufferAttribute(colors, 3));
    geometry.setIndex(indices);
    geometry.computeVertexNormals();
    geometry.computeBoundingSphere();

    const material = new THREE.MeshStandardMaterial({
        vertexColors: true,
        roughness: 0.96,
        metalness: 0,
        flatShading: true,
    });

    const mesh = new THREE.Mesh(geometry, material);
    mesh.name = "mountains";
    mesh.receiveShadow = true;
    mesh.castShadow = false;
    mesh.matrixAutoUpdate = false;
    mesh.updateMatrix();
    return mesh;
}

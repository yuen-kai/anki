import * as THREE from "three";

const MARKING_Y = 0.022;
const GROUND_Y = 0;

/** Offsets a polyline sideways by `offset` metres using per-point normals. */
function offsetPolyline(polyline, offset) {
    const out = [];
    for (let i = 0; i < polyline.length; i += 1) {
        const previous = polyline[Math.max(0, i - 1)];
        const next = polyline[Math.min(polyline.length - 1, i + 1)];
        let tx = next[0] - previous[0];
        let tz = next[1] - previous[1];
        const length = Math.hypot(tx, tz) || 1;
        tx /= length;
        tz /= length;
        out.push([polyline[i][0] + tz * offset, polyline[i][1] - tx * offset]);
    }
    return out;
}

class StripBuilder {
    constructor() {
        this.positions = [];
        this.normals = [];
        this.uvs = [];
        this.indices = [];
    }

    quad(ax, az, bx, bz, width) {
        let tx = bx - ax;
        let tz = bz - az;
        const length = Math.hypot(tx, tz);
        if (length < 1e-4) {
            return;
        }
        tx /= length;
        tz /= length;
        const nx = tz * width * 0.5;
        const nz = -tx * width * 0.5;
        const base = this.positions.length / 3;
        this.positions.push(
            ax - nx, MARKING_Y, az - nz,
            ax + nx, MARKING_Y, az + nz,
            bx + nx, MARKING_Y, bz + nz,
            bx - nx, MARKING_Y, bz - nz,
        );
        for (let i = 0; i < 4; i += 1) {
            this.normals.push(0, 1, 0);
        }
        this.uvs.push(0, 0, 1, 0, 1, 1, 0, 1);
        this.indices.push(base, base + 1, base + 2, base, base + 2, base + 3);
    }

    build() {
        const geometry = new THREE.BufferGeometry();
        geometry.setAttribute("position", new THREE.Float32BufferAttribute(this.positions, 3));
        geometry.setAttribute("normal", new THREE.Float32BufferAttribute(this.normals, 3));
        geometry.setAttribute("uv", new THREE.Float32BufferAttribute(this.uvs, 2));
        geometry.setIndex(this.indices);
        geometry.computeBoundingSphere();
        return geometry;
    }
}

function emitDashed(builder, network, polyline, width, dashLength, gapLength) {
    let travelled = 0;
    let drawing = true;
    let remaining = dashLength;
    for (let i = 1; i < polyline.length; i += 1) {
        const [ax, az] = polyline[i - 1];
        const [bx, bz] = polyline[i];
        const segmentLength = Math.hypot(bx - ax, bz - az);
        if (segmentLength < 1e-5) {
            continue;
        }
        let consumed = 0;
        while (consumed < segmentLength) {
            const take = Math.min(remaining, segmentLength - consumed);
            const t0 = consumed / segmentLength;
            const t1 = (consumed + take) / segmentLength;
            if (drawing) {
                const sx = ax + (bx - ax) * t0;
                const sz = az + (bz - az) * t0;
                const ex = ax + (bx - ax) * t1;
                const ez = az + (bz - az) * t1;
                if (!network.nearJunction((sx + ex) / 2, (sz + ez) / 2, 3)) {
                    builder.quad(sx, sz, ex, ez, width);
                }
            }
            consumed += take;
            travelled += take;
            remaining -= take;
            if (remaining <= 1e-6) {
                drawing = !drawing;
                remaining = drawing ? dashLength : gapLength;
            }
        }
    }
    return travelled;
}

function emitSolid(builder, network, polyline, width, stepSkipRadius = 2) {
    for (let i = 1; i < polyline.length; i += 1) {
        const [ax, az] = polyline[i - 1];
        const [bx, bz] = polyline[i];
        if (network.nearJunction((ax + bx) / 2, (az + bz) / 2, stepSkipRadius)) {
            continue;
        }
        builder.quad(ax, az, bx, bz, width);
    }
}

function createMarkings(network) {
    const white = new StripBuilder();
    const yellow = new StripBuilder();

    for (const road of network.roads) {
        const edgeOffset = road.halfWidth - 0.55;
        emitSolid(white, network, offsetPolyline(road.polyline, edgeOffset), 0.16);
        emitSolid(white, network, offsetPolyline(road.polyline, -edgeOffset), 0.16);

        if (road.width >= 18) {
            emitSolid(yellow, network, offsetPolyline(road.polyline, 0.22), 0.14);
            emitSolid(yellow, network, offsetPolyline(road.polyline, -0.22), 0.14);
            emitDashed(white, network, offsetPolyline(road.polyline, road.halfWidth * 0.52), 0.14, 3, 4.5);
            emitDashed(white, network, offsetPolyline(road.polyline, -road.halfWidth * 0.52), 0.14, 3, 4.5);
        } else {
            emitDashed(yellow, network, road.polyline, 0.16, 3, 4.5);
        }
    }

    const group = new THREE.Group();
    group.name = "markings";

    const whiteMaterial = new THREE.MeshStandardMaterial({
        color: 0xe9ecef,
        roughness: 0.78,
        metalness: 0,
        polygonOffset: true,
        polygonOffsetFactor: -2,
        polygonOffsetUnits: -2,
    });
    const yellowMaterial = new THREE.MeshStandardMaterial({
        color: 0xd8b13a,
        roughness: 0.8,
        metalness: 0,
        polygonOffset: true,
        polygonOffsetFactor: -2,
        polygonOffsetUnits: -2,
    });

    const whiteMesh = new THREE.Mesh(white.build(), whiteMaterial);
    const yellowMesh = new THREE.Mesh(yellow.build(), yellowMaterial);
    for (const mesh of [whiteMesh, yellowMesh]) {
        mesh.receiveShadow = true;
        mesh.castShadow = false;
        mesh.matrixAutoUpdate = false;
        group.add(mesh);
    }
    return group;
}

function createSplatMaterial({ mask, asphalt, concrete, paving, grass, planeSize }) {
    const material = new THREE.MeshStandardMaterial({
        color: 0xffffff,
        map: mask,
        roughness: 1,
        metalness: 0,
    });

    material.onBeforeCompile = (shader) => {
        shader.uniforms.uAsphalt = { value: asphalt };
        shader.uniforms.uConcrete = { value: concrete };
        shader.uniforms.uPaving = { value: paving };
        shader.uniforms.uGrass = { value: grass };
        shader.uniforms.uPlaneSize = { value: planeSize };
        shader.uniforms.uTiling = { value: new THREE.Vector4(1 / 9, 1 / 4.5, 1 / 6, 1 / 13) };
        shader.uniforms.uSurfaceRoughness = { value: new THREE.Vector4(0.93, 0.95, 0.9, 1.0) };

        shader.fragmentShader = shader.fragmentShader
            .replace(
                "#include <common>",
                /* glsl */ `#include <common>
                uniform sampler2D uAsphalt;
                uniform sampler2D uConcrete;
                uniform sampler2D uPaving;
                uniform sampler2D uGrass;
                uniform float uPlaneSize;
                uniform vec4 uTiling;
                uniform vec4 uSurfaceRoughness;`,
            )
            .replace(
                "#include <map_fragment>",
                /* glsl */ `
                vec4 maskSample = texture2D( map, vMapUv );
                vec4 surfaceWeights = vec4(
                    maskSample.r,
                    maskSample.g,
                    maskSample.b,
                    max( 0.0, 1.0 - maskSample.r - maskSample.g - maskSample.b )
                );
                surfaceWeights /= max( 1e-4, dot( surfaceWeights, vec4( 1.0 ) ) );
                vec2 worldXZ = ( vMapUv - 0.5 ) * uPlaneSize;
                vec3 blendedSurface =
                    texture2D( uAsphalt, worldXZ * uTiling.x ).rgb * surfaceWeights.x +
                    texture2D( uConcrete, worldXZ * uTiling.y ).rgb * surfaceWeights.y +
                    texture2D( uPaving, worldXZ * uTiling.z ).rgb * surfaceWeights.z +
                    texture2D( uGrass, worldXZ * uTiling.w ).rgb * surfaceWeights.w;
                diffuseColor.rgb *= blendedSurface;`,
            )
            .replace(
                "#include <roughnessmap_fragment>",
                /* glsl */ `float roughnessFactor = dot( surfaceWeights, uSurfaceRoughness );`,
            );
    };

    return material;
}

function createOuterTerrain(map, grassTexture) {
    const shape = new THREE.Shape();
    shape.absarc(0, 0, map.groundRadius, 0, Math.PI * 2, false);
    const hole = new THREE.Path();
    const half = map.cityHalf;
    hole.moveTo(-half, -half);
    hole.lineTo(-half, half);
    hole.lineTo(half, half);
    hole.lineTo(half, -half);
    hole.closePath();
    shape.holes.push(hole);

    const geometry = new THREE.ShapeGeometry(shape, 96);
    geometry.rotateX(-Math.PI / 2);

    const uv = geometry.attributes.uv;
    const position = geometry.attributes.position;
    for (let i = 0; i < uv.count; i += 1) {
        uv.setXY(i, position.getX(i) / 13, position.getZ(i) / 13);
    }
    uv.needsUpdate = true;

    const material = new THREE.MeshStandardMaterial({
        map: grassTexture,
        roughness: 1,
        metalness: 0,
    });
    const mesh = new THREE.Mesh(geometry, material);
    mesh.position.y = GROUND_Y;
    mesh.receiveShadow = true;
    mesh.matrixAutoUpdate = false;
    mesh.updateMatrix();
    return mesh;
}

export function createGround({ map, network, surfaces }) {
    const group = new THREE.Group();
    group.name = "ground";

    const planeSize = map.cityHalf * 2;
    const plate = new THREE.Mesh(
        new THREE.PlaneGeometry(planeSize, planeSize, 1, 1).rotateX(-Math.PI / 2),
        createSplatMaterial({
            mask: surfaces.mask,
            asphalt: surfaces.asphalt,
            concrete: surfaces.concrete,
            paving: surfaces.paving,
            grass: surfaces.grass,
            planeSize,
        }),
    );
    plate.position.y = GROUND_Y;
    plate.receiveShadow = true;
    plate.matrixAutoUpdate = false;
    plate.updateMatrix();
    group.add(plate);

    group.add(createOuterTerrain(map, surfaces.grass));
    group.add(createMarkings(network));

    return group;
}

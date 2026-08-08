import * as THREE from "three";

const SKY_VERTEX = /* glsl */ `
varying vec3 vDirection;
void main() {
    vDirection = ( modelMatrix * vec4( position, 1.0 ) ).xyz - cameraPosition;
    gl_Position = projectionMatrix * modelViewMatrix * vec4( position, 1.0 );
    gl_Position.z = gl_Position.w;
}
`;

const SKY_FRAGMENT = /* glsl */ `
uniform vec3 uZenith;
uniform vec3 uHorizon;
uniform vec3 uGround;
uniform vec3 uSunDirection;
uniform vec3 uSunColor;
uniform float uCloudAmount;
varying vec3 vDirection;

float hash( vec2 p ) {
    return fract( sin( dot( p, vec2( 127.1, 311.7 ) ) ) * 43758.5453123 );
}

float noise( vec2 p ) {
    vec2 i = floor( p );
    vec2 f = fract( p );
    vec2 u = f * f * ( 3.0 - 2.0 * f );
    return mix(
        mix( hash( i ), hash( i + vec2( 1.0, 0.0 ) ), u.x ),
        mix( hash( i + vec2( 0.0, 1.0 ) ), hash( i + vec2( 1.0, 1.0 ) ), u.x ),
        u.y
    );
}

float fbm( vec2 p ) {
    float total = 0.0;
    float amplitude = 0.5;
    for ( int i = 0; i < 5; i ++ ) {
        total += noise( p ) * amplitude;
        p *= 2.03;
        amplitude *= 0.5;
    }
    return total;
}

void main() {
    vec3 direction = normalize( vDirection );
    float height = direction.y;

    vec3 sky = mix( uHorizon, uZenith, pow( clamp( height, 0.0, 1.0 ), 0.62 ) );
    sky = mix( sky, uGround, smoothstep( 0.0, -0.28, height ) );

    float sunDot = max( dot( direction, uSunDirection ), 0.0 );
    sky += uSunColor * ( pow( sunDot, 900.0 ) * 8.0 + pow( sunDot, 14.0 ) * 0.28 );

    if ( height > 0.02 ) {
        vec2 cloudUv = direction.xz / max( 0.08, height ) * 0.55;
        float cover = fbm( cloudUv * 0.7 + 12.4 );
        float mask = smoothstep( 0.52, 0.86, cover ) * uCloudAmount;
        mask *= smoothstep( 0.02, 0.24, height );
        float shading = smoothstep( 0.42, 0.95, fbm( cloudUv * 1.7 - 4.0 ) );
        vec3 cloud = mix( vec3( 0.62, 0.66, 0.72 ), vec3( 1.05, 1.04, 1.02 ), shading );
        sky = mix( sky, cloud, mask );
    }

    gl_FragColor = vec4( sky, 1.0 );

    #include <tonemapping_fragment>
    #include <colorspace_fragment>
}
`;

/**
 * Daylight rig: gradient sky shell, a shadow-casting sun, sky fill and a warm
 * bounce term. The shadow camera is sized to the whole city so a single pass
 * covers the map.
 */
export function createDaylight({ map, quality }) {
    const sunDirection = new THREE.Vector3(0.42, 0.78, 0.46).normalize();

    const skyMaterial = new THREE.ShaderMaterial({
        uniforms: {
            uZenith: { value: new THREE.Color(0x2d6fbd) },
            uHorizon: { value: new THREE.Color(0xbcd6ea) },
            uGround: { value: new THREE.Color(0x8a9aa6) },
            uSunDirection: { value: sunDirection.clone() },
            uSunColor: { value: new THREE.Color(0xfff2d6) },
            uCloudAmount: { value: 0.55 },
        },
        vertexShader: SKY_VERTEX,
        fragmentShader: SKY_FRAGMENT,
        side: THREE.BackSide,
        depthWrite: false,
        depthTest: false,
        fog: false,
    });

    const sky = new THREE.Mesh(new THREE.SphereGeometry(1, 32, 16), skyMaterial);
    sky.name = "sky";
    sky.frustumCulled = false;
    sky.renderOrder = -1000;
    sky.onBeforeRender = (renderer, scene, camera) => {
        sky.position.copy(camera.position);
        sky.scale.setScalar(camera.far * 0.85);
        sky.updateMatrixWorld(true);
    };

    const sun = new THREE.DirectionalLight(0xfff0d8, 2.5);
    sun.position.copy(sunDirection).multiplyScalar(600);
    sun.castShadow = quality.shadowsEnabled;
    sun.shadow.mapSize.set(quality.shadowMapSize, quality.shadowMapSize);
    sun.shadow.bias = -0.0006;
    sun.shadow.normalBias = 0.9;
    sun.shadow.camera.near = 80;
    sun.shadow.camera.far = 1400;

    const hemisphere = new THREE.HemisphereLight(0xcfe2f2, 0x6d6c60, 1.35);
    const bounce = new THREE.DirectionalLight(0xd8e4ef, 0.5);
    bounce.position.set(-0.5, 0.35, -0.7).multiplyScalar(400);

    const fog = new THREE.Fog(0xbcd6ea, map.groundRadius * 0.55, map.mountains.outerRadius * 1.15);

    return {
        sky,
        sun,
        hemisphere,
        bounce,
        fog,
        sunDirection,
        /**
         * Keeps the shadow frustum tight around the viewer so shadow texels
         * stay small on a map this size.
         */
        followShadow(target, radius = 190) {
            sun.target.position.set(target.x, 0, target.z);
            sun.target.updateMatrixWorld();
            sun.position.set(
                target.x + sunDirection.x * 600,
                sunDirection.y * 600,
                target.z + sunDirection.z * 600,
            );
            const camera = sun.shadow.camera;
            if (camera.right !== radius) {
                camera.left = -radius;
                camera.right = radius;
                camera.top = radius;
                camera.bottom = -radius;
                camera.updateProjectionMatrix();
            }
        },
    };
}

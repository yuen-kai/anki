import * as THREE from "three";

/**
 * Quality tiers scale the cost of the expensive resources (shadow map, splat
 * mask, anisotropy) so the same scene can run on a weak GPU or a software
 * rasteriser without changing any of the builders.
 */
export const QUALITY_TIERS = {
    low: {
        name: "low",
        pixelRatioCap: 1,
        shadowMapSize: 1024,
        shadowsEnabled: true,
        splatMaskSize: 2048,
        textureSize: 256,
        anisotropy: 4,
        antialias: false,
    },
    medium: {
        name: "medium",
        pixelRatioCap: 1.5,
        shadowMapSize: 2048,
        shadowsEnabled: true,
        splatMaskSize: 4096,
        textureSize: 512,
        anisotropy: 8,
        antialias: true,
    },
    high: {
        name: "high",
        pixelRatioCap: 2,
        shadowMapSize: 4096,
        shadowsEnabled: true,
        splatMaskSize: 4096,
        textureSize: 512,
        anisotropy: 16,
        antialias: true,
    },
};

/** Picks a tier from the URL (?quality=low) or from coarse device hints. */
export function detectQuality() {
    const requested = new URLSearchParams(window.location.search).get("quality");
    if (requested && QUALITY_TIERS[requested]) {
        return QUALITY_TIERS[requested];
    }
    const memory = navigator.deviceMemory ?? 8;
    const cores = navigator.hardwareConcurrency ?? 8;
    if (memory <= 4 || cores <= 4) {
        return QUALITY_TIERS.low;
    }
    if (memory <= 8) {
        return QUALITY_TIERS.medium;
    }
    return QUALITY_TIERS.high;
}

export function createRenderer(canvas, quality) {
    const renderer = new THREE.WebGLRenderer({
        canvas,
        antialias: quality.antialias,
        powerPreference: "high-performance",
        stencil: false,
    });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, quality.pixelRatioCap));
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
    renderer.toneMappingExposure = 1.05;
    renderer.shadowMap.enabled = quality.shadowsEnabled;
    renderer.shadowMap.type = THREE.PCFShadowMap;
    renderer.info.autoReset = true;
    return renderer;
}

/**
 * Keeps renderer and camera in step with the canvas' CSS size. Returns a
 * dispose function so the listener can be removed.
 */
export function attachResize(renderer, camera) {
    const apply = () => {
        const width = window.innerWidth;
        const height = window.innerHeight;
        renderer.setSize(width, height, false);
        camera.aspect = width / Math.max(1, height);
        camera.updateProjectionMatrix();
    };
    apply();
    window.addEventListener("resize", apply);
    return () => window.removeEventListener("resize", apply);
}

export function maxAnisotropy(renderer, quality) {
    return Math.min(renderer.capabilities.getMaxAnisotropy(), quality.anisotropy);
}

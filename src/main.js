import * as THREE from "three";

import { Keyboard } from "./engine/input.js";
import { FixedLoop } from "./engine/loop.js";
import { attachResize, createRenderer, detectQuality, maxAnisotropy } from "./engine/renderer.js";
import { CollisionWorld } from "./game/collision.js";
import { PLAYER_KEYS, readPlayerInput } from "./game/playerControls.js";
import { Vehicle } from "./game/vehicle.js";
import { PLAYER_PROFILE } from "./game/vehicleProfiles.js";
import { PENDING_NOTES } from "./pendingAnswers.js";
import { createCarModel } from "./render/carModel.js";
import { ChaseCamera } from "./render/chaseCamera.js";
import { createDaylight } from "./render/sky.js";
import { createBuildings } from "./world/buildings.js";
import { ObbField } from "./world/colliders.js";
import { createGround } from "./world/ground.js";
import { MAP } from "./world/mapData.js";
import { createMountains } from "./world/mountains.js";
import { createProps } from "./world/props.js";
import { RoadNetwork } from "./world/roadNetwork.js";
import {
    createAsphaltTexture,
    createConcreteTexture,
    createFacadeTextures,
    createGrassTexture,
    createPavingTexture,
    createRoofTexture,
    createSurfaceMask,
} from "./world/textures.js";

const CAMERA_MODES = ["chase", "orbit", "aerial", "map"];

const ui = {
    loading: document.getElementById("loading"),
    loadingFill: document.getElementById("loading-fill"),
    loadingStep: document.getElementById("loading-step"),
    error: document.getElementById("error"),
    errorDetail: document.getElementById("error-detail"),
    notice: document.getElementById("preview-notice"),
    pendingNotes: document.getElementById("pending-notes"),
};

function listPendingAnswers() {
    if (PENDING_NOTES.length === 0) {
        return;
    }
    const parts = PENDING_NOTES.map((note) => `${note.what} (${note.question}) parked at ${note.parked}`);
    ui.pendingNotes.textContent = `Awaiting answers: ${parts.join("; ")}`;
}

function nextFrame() {
    return new Promise((resolve) => requestAnimationFrame(() => resolve()));
}

async function step(fraction, label, work) {
    ui.loadingFill.style.width = `${Math.round(fraction * 100)}%`;
    ui.loadingStep.textContent = label;
    await nextFrame();
    const result = work();
    await nextFrame();
    return result;
}

function showError(error) {
    ui.loading.classList.add("hidden");
    ui.error.classList.remove("hidden");
    ui.errorDetail.textContent = `${error && error.message ? error.message : error}\n\n${
        (error && error.stack) || ""
    }`.trim();
    console.error(error);
}

async function boot() {
    const canvas = document.getElementById("viewport");
    const quality = detectQuality();
    const renderer = createRenderer(canvas, quality);
    const anisotropy = maxAnisotropy(renderer, quality);

    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(60, 1, 0.4, 4200);
    attachResize(renderer, camera);

    const network = await step(0.08, "laying out streets", () => new RoadNetwork(MAP));
    const problems = network.validate();
    if (problems.length > 0) {
        console.warn(`map validation: ${problems.length} problem(s)`);
        for (const problem of problems) {
            console.warn(`  ${problem}`);
        }
    }

    const surfaces = await step(0.2, "mixing surfaces", () => ({
        asphalt: createAsphaltTexture(quality.textureSize, anisotropy),
        concrete: createConcreteTexture(quality.textureSize, anisotropy),
        grass: createGrassTexture(quality.textureSize, anisotropy),
        paving: createPavingTexture(quality.textureSize, anisotropy),
        roof: createRoofTexture(quality.textureSize, anisotropy),
    }));

    surfaces.mask = await step(
        0.34,
        "painting the city",
        () => createSurfaceMask(MAP, network, quality.splatMaskSize),
    );

    const facadeTextures = await step(
        0.46,
        "printing facades",
        () => createFacadeTextures(quality.textureSize, anisotropy),
    );

    const ground = await step(0.56, "pouring roads", () => createGround({ map: MAP, network, surfaces }));
    scene.add(ground);

    const buildings = await step(
        0.72,
        "raising buildings",
        () => createBuildings({ map: MAP, network, surfaces, facadeTextures }),
    );
    scene.add(buildings.group);

    const props = await step(
        0.82,
        "planting parks",
        () => createProps({ map: MAP, network, colliders: buildings.colliders }),
    );
    scene.add(props.group);

    const mountains = await step(0.9, "raising mountains", () => createMountains(MAP));
    scene.add(mountains);

    const daylight = await step(0.96, "lighting the sky", () => createDaylight({ map: MAP, quality }));
    scene.add(daylight.sky, daylight.sun, daylight.sun.target, daylight.hemisphere, daylight.bounce);
    scene.fog = daylight.fog;

    const obstacles = new ObbField(buildings.colliders);
    const collision = new CollisionWorld({ obstacles, boundsRadius: MAP.groundRadius - 30 });

    const playerCar = createCarModel({ bodyColor: 0x1f2b3a, roofColor: 0x151d28, rimColor: 0xc7cdd4 });
    scene.add(playerCar.group);

    const player = new Vehicle(PLAYER_PROFILE, MAP.start);

    const chase = new ChaseCamera(camera, { obstacles, referenceSpeed: PLAYER_PROFILE.topSpeed * 0.8 });
    chase.snap(player.state);

    const keyboard = new Keyboard();
    keyboard.swallow([...PLAYER_KEYS, "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"]);

    let cameraMode = 0;
    let orbitAngle = 0;

    const loop = new FixedLoop({
        step: 1 / 120,
        update(dt) {
            player.update(dt, readPlayerInput(keyboard), collision);
        },
        render(alpha, frameSeconds) {
            const dt = Math.min(0.05, frameSeconds);
            const state = player.state;

            playerCar.group.position.set(state.x, state.y, state.z);
            playerCar.group.rotation.y = state.heading;
            playerCar.setSteer(player.steer * 0.55);
            playerCar.roll(state.speed * dt);

            if (keyboard.wasPressed("KeyC")) {
                cameraMode = (cameraMode + 1) % CAMERA_MODES.length;
            }
            if (keyboard.wasPressed("KeyP")) {
                ui.notice.classList.toggle("hidden");
            }

            if (CAMERA_MODES[cameraMode] === "chase") {
                chase.update(state, dt);
            } else if (CAMERA_MODES[cameraMode] === "orbit") {
                orbitAngle += dt * 0.24;
                camera.position.set(
                    state.x + Math.sin(orbitAngle) * 26,
                    state.y + 11,
                    state.z + Math.cos(orbitAngle) * 26,
                );
                camera.lookAt(state.x, state.y + 1.2, state.z);
                chase.initialised = false;
            } else if (CAMERA_MODES[cameraMode] === "aerial") {
                camera.position.set(state.x + 60, 150, state.z + 190);
                camera.lookAt(state.x, 0, state.z);
                chase.initialised = false;
            } else {
                camera.position.set(0, 900, 0.001);
                camera.lookAt(0, 0, 0);
                chase.initialised = false;
            }

            daylight.followShadow(state);
            renderer.render(scene, camera);
            keyboard.endFrame();
        },
    });

    ui.loadingFill.style.width = "100%";
    ui.loadingStep.textContent = "ready";
    await nextFrame();
    renderer.render(scene, camera);
    ui.loading.classList.add("hidden");
    listPendingAnswers();
    ui.notice.classList.remove("hidden");
    loop.start();

    window.policeChase = { scene, renderer, camera, player, chase, network, buildings, quality };
}

boot().catch(showError);
window.addEventListener("error", (event) => showError(event.error ?? event.message));
window.addEventListener("unhandledrejection", (event) => showError(event.reason));

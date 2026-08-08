import * as THREE from "three";

import { clamp } from "../engine/mathUtils.js";
import { createCarModel } from "../render/carModel.js";
import { ArrestTimer } from "./arrest.js";
import { formatTime, Hud } from "./hud.js";
import { readPlayerInput } from "./playerControls.js";
import { PoliceUnit } from "./policeUnit.js";
import { Vehicle } from "./vehicle.js";
import { PLAYER_PROFILE } from "./vehicleProfiles.js";

const STATE = { RUNNING: "running", ARRESTED: "arrested", SCORE: "score" };
const CINEMATIC_SECONDS = 2.6;
const FADE_START = 1.2;
const SIREN_LIGHTS = 3;
const SIREN_HZ = 2.4;

/** One run: the player, the police unit, the arrest timer and the end screen. */
export class Session {
    constructor({ scene, camera, chase, map, network, graph, collision, obstacles, keyboard }) {
        this.scene = scene;
        this.camera = camera;
        this.chase = chase;
        this.map = map;
        this.collision = collision;
        this.obstacles = obstacles;
        this.keyboard = keyboard;

        this.player = new Vehicle(PLAYER_PROFILE, map.start);
        this.unit = new PoliceUnit({ graph, collision, obstacles });
        this.arrestTimer = new ArrestTimer();
        this.hud = new Hud({ network, map });

        this.playerModel = createCarModel({ bodyColor: 0x1f2b3a, roofColor: 0x151d28, rimColor: 0xc7cdd4 });
        scene.add(this.playerModel.group);

        this.policeModels = [];
        this.sirenLights = [];
        for (let i = 0; i < SIREN_LIGHTS; i += 1) {
            const light = new THREE.PointLight(0xff3333, 0, 46, 1.7);
            light.visible = false;
            scene.add(light);
            this.sirenLights.push(light);
        }

        this.fade = document.getElementById("fade");
        this.scoreOverlay = document.getElementById("score");
        this.scoreTime = document.getElementById("score-time");
        this.scoreCars = document.getElementById("score-cars");
        this.restartButton = document.getElementById("restart");
        this.restartButton.addEventListener("click", () => this.restart());

        this.state = STATE.RUNNING;
        this.elapsed = 0;
        this.cinematic = 0;
        this.cinematicAngle = 0;
    }

    start() {
        this.restart();
        this.hud.show();
    }

    restart() {
        this.player = new Vehicle(PLAYER_PROFILE, this.map.start);
        this.unit.reset();
        this.unit.deployBehind(this.player);
        this.arrestTimer.reset();
        this.elapsed = 0;
        this.cinematic = 0;
        this.state = STATE.RUNNING;
        this.scoreOverlay.classList.add("hidden");
        this.fade.style.opacity = "0";
        this.chase.snap(this.player.state);
        this.syncModels(0);
    }

    get running() {
        return this.state === STATE.RUNNING;
    }

    update(dt) {
        if (this.state === STATE.RUNNING) {
            this.elapsed += dt;
            this.player.update(dt, readPlayerInput(this.keyboard), this.collision);
            this.unit.update(dt, this.player);
            this.pushApartFromPolice();
            if (this.arrestTimer.update(dt, this.player, this.unit.cars)) {
                this.beginArrest();
            }
            return;
        }

        if (this.state === STATE.ARRESTED) {
            this.cinematic += dt;
            this.player.update(dt, { throttle: 0, steer: 0, handbrake: false }, this.collision);
            this.unit.update(dt, this.player);
            this.pushApartFromPolice();
            if (this.cinematic >= CINEMATIC_SECONDS) {
                this.showScore();
            }
        }
    }

    /** Police shove the player around; the player is not pinned by contact. */
    pushApartFromPolice() {
        const player = this.player;
        for (const car of this.unit.cars) {
            const other = car.vehicle;
            const dx = other.x - player.x;
            const dz = other.z - player.z;
            const distance = Math.hypot(dx, dz);
            const minimum = 3.4;
            if (distance > minimum || distance < 1e-4) {
                continue;
            }
            const push = (minimum - distance) * 0.5;
            const nx = dx / distance;
            const nz = dz / distance;
            const moved = this.collision.move(
                player.x,
                player.z,
                player.x - nx * push,
                player.z - nz * push,
                player.profile.radius,
            );
            player.x = moved.x;
            player.z = moved.z;
            const pushed = this.collision.move(
                other.x,
                other.z,
                other.x + nx * push,
                other.z + nz * push,
                other.profile.radius,
            );
            other.x = pushed.x;
            other.z = pushed.z;
        }
    }

    beginArrest() {
        this.state = STATE.ARRESTED;
        this.cinematic = 0;
        this.cinematicAngle = this.player.heading;
        this.hud.hide();
    }

    showScore() {
        this.state = STATE.SCORE;
        this.scoreTime.textContent = formatTime(this.elapsed);
        this.scoreCars.textContent = String(this.unit.count);
        this.scoreOverlay.classList.remove("hidden");
        this.fade.style.opacity = "0";
    }

    /** Grows a police model pool to match the number of cars in the unit. */
    syncModels(dt) {
        const cars = this.unit.cars;
        while (this.policeModels.length < cars.length) {
            const model = createCarModel({
                bodyColor: 0xf0f2f5,
                roofColor: 0xf0f2f5,
                rimColor: 0x8b929a,
                livery: "police",
            });
            this.scene.add(model.group);
            this.policeModels.push(model);
        }
        for (let i = cars.length; i < this.policeModels.length; i += 1) {
            this.policeModels[i].group.visible = false;
        }

        const player = this.player;
        this.playerModel.group.position.set(player.x, player.y, player.z);
        this.playerModel.group.rotation.y = player.heading;
        this.playerModel.setSteer(player.steer * 0.55);
        this.playerModel.roll(player.speed * dt);

        const ranked = cars
            .map((car, index) => ({ index, distance: car.distanceTo(player.x, player.z) }))
            .sort((a, b) => a.distance - b.distance);

        cars.forEach((car, i) => {
            const model = this.policeModels[i];
            model.group.visible = true;
            model.group.position.set(car.vehicle.x, car.vehicle.y, car.vehicle.z);
            model.group.rotation.y = car.vehicle.heading;
            model.setSteer(car.vehicle.steer * 0.5);
            model.roll(car.vehicle.speed * dt);

            car.sirenPhase += dt * SIREN_HZ * Math.PI * 2;
            const wave = Math.sin(car.sirenPhase);
            for (const siren of model.sirens) {
                const lit = siren.side < 0 ? Math.max(0, wave) : Math.max(0, -wave);
                siren.material.emissiveIntensity = 0.25 + lit * 3.4;
            }
        });

        for (let slot = 0; slot < this.sirenLights.length; slot += 1) {
            const light = this.sirenLights[slot];
            const entry = ranked[slot];
            if (!entry || entry.distance > 90) {
                light.visible = false;
                continue;
            }
            const car = cars[entry.index];
            const wave = Math.sin(car.sirenPhase);
            light.visible = true;
            light.position.set(car.vehicle.x, 2.1, car.vehicle.z);
            light.color.setHex(wave >= 0 ? 0xff2b2b : 0x2b6bff);
            const falloff = 1 - clamp(entry.distance / 90, 0, 1);
            light.intensity = (30 + Math.abs(wave) * 90) * falloff;
        }
    }

    updateCamera(dt) {
        if (this.state === STATE.RUNNING) {
            this.chase.update(this.player.state, dt);
            return;
        }

        // Arrest cinematic: rise and swing around the surrounded car.
        const progress = clamp(this.cinematic / CINEMATIC_SECONDS, 0, 1);
        this.cinematicAngle += dt * 0.5;
        const radius = 13 + progress * 7;
        const height = 4.5 + progress * 9;
        this.camera.position.set(
            this.player.x + Math.sin(this.cinematicAngle) * radius,
            height,
            this.player.z + Math.cos(this.cinematicAngle) * radius,
        );
        this.camera.lookAt(this.player.x, 1, this.player.z);
        this.chase.initialised = false;

        if (this.state === STATE.ARRESTED && this.cinematic > FADE_START) {
            const fade = clamp((this.cinematic - FADE_START) / (CINEMATIC_SECONDS - FADE_START), 0, 1);
            this.fade.style.opacity = String(fade);
        }
    }

    render(dt) {
        this.syncModels(dt);
        this.updateCamera(dt);
        if (this.state === STATE.RUNNING) {
            this.hud.update({
                elapsed: this.elapsed,
                player: this.player,
                cars: this.unit.cars,
                arrestTimer: this.arrestTimer,
            });
        }
        if (this.state === STATE.SCORE && this.keyboard.wasPressed("Enter")) {
            this.restart();
            this.hud.show();
        }
    }
}

/**
 * Fixed-timestep loop with an interpolating render step.
 *
 * `update(step, elapsed)` runs a whole number of times per frame at a constant
 * step so simulation results do not depend on frame rate. `render(alpha, dt)`
 * runs once per frame; `alpha` is the fraction of a step that has accumulated
 * beyond the last update, for interpolating visuals.
 */
export class FixedLoop {
    constructor({ update, render, step = 1 / 120, maxCatchUp = 8 }) {
        this.update = update;
        this.render = render;
        this.step = step;
        this.maxCatchUp = maxCatchUp;
        this.accumulator = 0;
        this.elapsed = 0;
        this.lastTime = 0;
        this.running = false;
        this.frameId = 0;
        this._tick = this._tick.bind(this);
    }

    start() {
        if (this.running) {
            return;
        }
        this.running = true;
        this.lastTime = performance.now();
        this.frameId = requestAnimationFrame(this._tick);
    }

    stop() {
        this.running = false;
        cancelAnimationFrame(this.frameId);
    }

    _tick(now) {
        if (!this.running) {
            return;
        }
        this.frameId = requestAnimationFrame(this._tick);

        // A large gap means the tab was hidden; drop it rather than catching up.
        const frameSeconds = Math.min((now - this.lastTime) / 1000, this.step * this.maxCatchUp);
        this.lastTime = now;
        this.accumulator += frameSeconds;

        let steps = 0;
        while (this.accumulator >= this.step && steps < this.maxCatchUp) {
            this.update(this.step, this.elapsed);
            this.elapsed += this.step;
            this.accumulator -= this.step;
            steps += 1;
        }

        this.render(this.accumulator / this.step, frameSeconds);
    }
}

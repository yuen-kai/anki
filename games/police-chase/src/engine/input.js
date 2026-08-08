/**
 * Raw keyboard state. This layer deliberately has no notion of what any key
 * means; callers ask about physical `KeyboardEvent.code` values and decide for
 * themselves. Edge state is latched so a press is never missed between frames.
 */
export class Keyboard {
    constructor(target = window) {
        this.target = target;
        this.down = new Set();
        this.pressedThisFrame = new Set();
        this.releasedThisFrame = new Set();
        this._swallow = new Set();

        this._onKeyDown = (event) => {
            if (event.repeat) {
                return;
            }
            if (this._swallow.has(event.code)) {
                event.preventDefault();
            }
            this.down.add(event.code);
            this.pressedThisFrame.add(event.code);
        };
        this._onKeyUp = (event) => {
            if (this._swallow.has(event.code)) {
                event.preventDefault();
            }
            this.down.delete(event.code);
            this.releasedThisFrame.add(event.code);
        };
        this._onBlur = () => {
            for (const code of this.down) {
                this.releasedThisFrame.add(code);
            }
            this.down.clear();
        };

        target.addEventListener("keydown", this._onKeyDown, { passive: false });
        target.addEventListener("keyup", this._onKeyUp, { passive: false });
        target.addEventListener("blur", this._onBlur);
    }

    /** Stops the browser acting on these codes (scrolling, quick find, ...). */
    swallow(codes) {
        for (const code of codes) {
            this._swallow.add(code);
        }
    }

    isDown(code) {
        return this.down.has(code);
    }

    anyDown(codes) {
        return codes.some((code) => this.down.has(code));
    }

    wasPressed(code) {
        return this.pressedThisFrame.has(code);
    }

    /** Call once per frame, after all consumers have read edge state. */
    endFrame() {
        this.pressedThisFrame.clear();
        this.releasedThisFrame.clear();
    }

    dispose() {
        this.target.removeEventListener("keydown", this._onKeyDown);
        this.target.removeEventListener("keyup", this._onKeyUp);
        this.target.removeEventListener("blur", this._onBlur);
    }
}

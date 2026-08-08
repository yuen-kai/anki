const MINIMAP_SIZE = 256;

export function formatTime(seconds) {
    const whole = Math.floor(seconds);
    const minutes = Math.floor(whole / 60);
    const rest = whole % 60;
    const tenths = Math.floor((seconds - whole) * 10);
    return `${minutes}:${String(rest).padStart(2, "0")}.${tenths}`;
}

/** Pre-renders the street layout once; only the markers are redrawn per frame. */
function renderStreets(network, map) {
    const canvas = document.createElement("canvas");
    canvas.width = MINIMAP_SIZE;
    canvas.height = MINIMAP_SIZE;
    const ctx = canvas.getContext("2d");
    const extent = map.ringExtent + 30;
    const scale = MINIMAP_SIZE / (extent * 2);
    const toPx = (world) => (world + extent) * scale;

    ctx.fillStyle = "rgba(10, 14, 20, 0.55)";
    ctx.fillRect(0, 0, MINIMAP_SIZE, MINIMAP_SIZE);

    ctx.lineCap = "round";
    ctx.lineJoin = "round";
    ctx.strokeStyle = "rgba(196, 214, 232, 0.5)";
    for (const road of network.roads) {
        ctx.lineWidth = Math.max(1.2, road.width * scale * 0.9);
        ctx.beginPath();
        road.polyline.forEach(([x, z], i) => {
            if (i === 0) {
                ctx.moveTo(toPx(x), toPx(z));
            } else {
                ctx.lineTo(toPx(x), toPx(z));
            }
        });
        ctx.stroke();
    }

    ctx.fillStyle = "rgba(104, 166, 112, 0.4)";
    for (const park of map.parks) {
        ctx.beginPath();
        if (park.shape === "circle") {
            ctx.arc(toPx(park.x), toPx(park.z), park.radius * scale, 0, Math.PI * 2);
        } else {
            ctx.rect(toPx(park.x0), toPx(park.z0), (park.x1 - park.x0) * scale, (park.z1 - park.z0) * scale);
        }
        ctx.fill();
    }

    return { canvas, toPx, scale };
}

export class Hud {
    constructor({ network, map }) {
        this.root = document.getElementById("hud");
        this.timeLabel = document.getElementById("hud-time");
        this.arrest = document.getElementById("arrest");
        this.arrestFill = document.getElementById("arrest-fill");
        this.canvas = document.getElementById("minimap");
        this.ctx = this.canvas.getContext("2d");

        const streets = renderStreets(network, map);
        this.streets = streets.canvas;
        this.toPx = streets.toPx;
        this.arrestVisible = false;
    }

    show() {
        this.root.classList.remove("hidden");
    }

    hide() {
        this.root.classList.add("hidden");
    }

    update({ elapsed, player, cars, arrestTimer }) {
        this.timeLabel.textContent = formatTime(elapsed);

        const shouldShow = arrestTimer.active;
        if (shouldShow !== this.arrestVisible) {
            this.arrest.classList.toggle("hidden", !shouldShow);
            this.arrestVisible = shouldShow;
        }
        if (shouldShow) {
            this.arrestFill.style.width = `${Math.round(arrestTimer.progress * 100)}%`;
        }

        const ctx = this.ctx;
        ctx.clearRect(0, 0, MINIMAP_SIZE, MINIMAP_SIZE);
        ctx.drawImage(this.streets, 0, 0);

        for (const car of cars) {
            const x = this.toPx(car.x);
            const z = this.toPx(car.z);
            ctx.beginPath();
            ctx.arc(x, z, 3.4, 0, Math.PI * 2);
            ctx.fillStyle = "#4d8dff";
            ctx.fill();
            ctx.beginPath();
            ctx.arc(x, z, 1.6, 0, Math.PI * 2);
            ctx.fillStyle = "#ff4d4d";
            ctx.fill();
        }

        const px = this.toPx(player.x);
        const pz = this.toPx(player.z);
        ctx.save();
        ctx.translate(px, pz);
        ctx.rotate(Math.PI - player.heading);
        ctx.beginPath();
        ctx.moveTo(0, -6.5);
        ctx.lineTo(4.4, 5);
        ctx.lineTo(0, 2.6);
        ctx.lineTo(-4.4, 5);
        ctx.closePath();
        ctx.fillStyle = "#ffd36f";
        ctx.fill();
        ctx.restore();
    }
}

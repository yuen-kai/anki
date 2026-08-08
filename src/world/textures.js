import * as THREE from "three";

/** Deterministic PRNG so the world is identical on every load. */
export function makeRandom(seed) {
    let state = seed >>> 0;
    return function random() {
        state |= 0;
        state = (state + 0x6d2b79f5) | 0;
        let t = Math.imul(state ^ (state >>> 15), 1 | state);
        t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
        return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
}

function createCanvas(size, height = size) {
    const canvas = document.createElement("canvas");
    canvas.width = size;
    canvas.height = height;
    return canvas;
}

function toTexture(canvas, { repeat = true, srgb = true, anisotropy = 8 } = {}) {
    const texture = new THREE.CanvasTexture(canvas);
    texture.wrapS = repeat ? THREE.RepeatWrapping : THREE.ClampToEdgeWrapping;
    texture.wrapT = repeat ? THREE.RepeatWrapping : THREE.ClampToEdgeWrapping;
    texture.colorSpace = srgb ? THREE.SRGBColorSpace : THREE.NoColorSpace;
    texture.anisotropy = anisotropy;
    texture.generateMipmaps = true;
    texture.minFilter = THREE.LinearMipmapLinearFilter;
    texture.magFilter = THREE.LinearFilter;
    texture.needsUpdate = true;
    return texture;
}

/** Tileable value noise sampled with wrap-around so edges match. */
function valueNoiseField(size, cells, random) {
    const grid = new Float32Array(cells * cells);
    for (let i = 0; i < grid.length; i += 1) {
        grid[i] = random();
    }
    const at = (cx, cz) => grid[((cz % cells) + cells) % cells * cells + (((cx % cells) + cells) % cells)];
    const smooth = (t) => t * t * (3 - 2 * t);
    const field = new Float32Array(size * size);
    const scale = cells / size;
    for (let y = 0; y < size; y += 1) {
        const fy = y * scale;
        const y0 = Math.floor(fy);
        const ty = smooth(fy - y0);
        for (let x = 0; x < size; x += 1) {
            const fx = x * scale;
            const x0 = Math.floor(fx);
            const tx = smooth(fx - x0);
            const v00 = at(x0, y0);
            const v10 = at(x0 + 1, y0);
            const v01 = at(x0, y0 + 1);
            const v11 = at(x0 + 1, y0 + 1);
            const top = v00 + (v10 - v00) * tx;
            const bottom = v01 + (v11 - v01) * tx;
            field[y * size + x] = top + (bottom - top) * ty;
        }
    }
    return field;
}

function fbmField(size, octaves, baseCells, random) {
    const field = new Float32Array(size * size);
    let amplitude = 1;
    let total = 0;
    let cells = baseCells;
    for (let o = 0; o < octaves; o += 1) {
        const layer = valueNoiseField(size, cells, random);
        for (let i = 0; i < field.length; i += 1) {
            field[i] += layer[i] * amplitude;
        }
        total += amplitude;
        amplitude *= 0.5;
        cells *= 2;
    }
    for (let i = 0; i < field.length; i += 1) {
        field[i] /= total;
    }
    return field;
}

function paintField(ctx, size, field, colorFor) {
    const image = ctx.createImageData(size, size);
    const data = image.data;
    for (let i = 0; i < field.length; i += 1) {
        const [r, g, b] = colorFor(field[i], i % size, Math.floor(i / size));
        data[i * 4] = r;
        data[i * 4 + 1] = g;
        data[i * 4 + 2] = b;
        data[i * 4 + 3] = 255;
    }
    ctx.putImageData(image, 0, 0);
}

export function createAsphaltTexture(size, anisotropy) {
    const canvas = createCanvas(size);
    const ctx = canvas.getContext("2d");
    const random = makeRandom(1337);
    const grain = fbmField(size, 4, 32, random);
    const patches = fbmField(size, 3, 4, makeRandom(991));
    paintField(ctx, size, grain, (v, x, y) => {
        const patch = patches[y * size + x];
        const base = 44 + patch * 16;
        const speck = (v - 0.5) * 34;
        const level = Math.max(18, Math.min(96, base + speck));
        return [level, level + 1, level + 3];
    });

    ctx.globalAlpha = 0.16;
    ctx.strokeStyle = "#1b1d20";
    ctx.lineWidth = 1.2;
    for (let i = 0; i < 14; i += 1) {
        ctx.beginPath();
        let x = random() * size;
        let y = random() * size;
        ctx.moveTo(x, y);
        for (let s = 0; s < 6; s += 1) {
            x += (random() - 0.5) * size * 0.24;
            y += (random() - 0.5) * size * 0.24;
            ctx.lineTo(x, y);
        }
        ctx.stroke();
    }
    ctx.globalAlpha = 1;
    return toTexture(canvas, { anisotropy });
}

export function createConcreteTexture(size, anisotropy) {
    const canvas = createCanvas(size);
    const ctx = canvas.getContext("2d");
    const random = makeRandom(4242);
    const grain = fbmField(size, 4, 24, random);
    const patches = fbmField(size, 3, 5, makeRandom(77));
    paintField(ctx, size, grain, (v, x, y) => {
        const patch = patches[y * size + x];
        const base = 138 + patch * 22;
        const speck = (v - 0.5) * 26;
        const level = Math.max(90, Math.min(205, base + speck));
        return [level, level - 1, level - 5];
    });

    // Slab joints: the tile spans four slabs each way.
    ctx.strokeStyle = "rgba(70, 70, 72, 0.5)";
    ctx.lineWidth = Math.max(1, size / 220);
    for (let i = 0; i <= 4; i += 1) {
        const p = (i * size) / 4;
        ctx.beginPath();
        ctx.moveTo(p, 0);
        ctx.lineTo(p, size);
        ctx.moveTo(0, p);
        ctx.lineTo(size, p);
        ctx.stroke();
    }
    return toTexture(canvas, { anisotropy });
}

export function createGrassTexture(size, anisotropy) {
    const canvas = createCanvas(size);
    const ctx = canvas.getContext("2d");
    const random = makeRandom(20250808);
    const blades = fbmField(size, 4, 48, random);
    const patches = fbmField(size, 3, 6, makeRandom(515));
    paintField(ctx, size, blades, (v, x, y) => {
        const patch = patches[y * size + x];
        const r = 84 + patch * 26 + (v - 0.5) * 22;
        const g = 100 + patch * 30 + (v - 0.5) * 24;
        const b = 62 + patch * 18 + (v - 0.5) * 16;
        return [Math.max(58, Math.min(150, r)), Math.max(74, Math.min(165, g)), Math.max(40, Math.min(115, b))];
    });
    return toTexture(canvas, { anisotropy });
}

export function createPavingTexture(size, anisotropy) {
    const canvas = createCanvas(size);
    const ctx = canvas.getContext("2d");
    const random = makeRandom(60606);
    const grain = fbmField(size, 4, 28, random);
    paintField(ctx, size, grain, (v) => {
        const level = Math.max(60, Math.min(140, 88 + (v - 0.5) * 30));
        return [level, level, level + 2];
    });
    ctx.strokeStyle = "rgba(40, 40, 44, 0.45)";
    ctx.lineWidth = Math.max(1, size / 256);
    for (let i = 0; i <= 2; i += 1) {
        const p = (i * size) / 2;
        ctx.beginPath();
        ctx.moveTo(p, 0);
        ctx.lineTo(p, size);
        ctx.moveTo(0, p);
        ctx.lineTo(size, p);
        ctx.stroke();
    }
    return toTexture(canvas, { anisotropy });
}

/**
 * Facade tiles are drawn to cover FLOORS x BAYS of a building so UVs can be
 * expressed directly in metres by the geometry builder.
 */
export const FACADE_FLOORS = 4;
export const FACADE_BAYS = 4;

const FACADE_STYLES = [
    {
        id: "glass-tower",
        wall: "#8d99a6",
        trim: "#aab4bf",
        glass: ["#26506e", "#2d5f83", "#1d4460", "#356e93"],
        spandrel: "#5f6b78",
        windowInset: 0.06,
        bandHeight: 0.06,
    },
    {
        id: "brick-walkup",
        wall: "#9b5f4a",
        trim: "#c69a83",
        glass: ["#2b2f36", "#343941", "#22262c", "#3d434c"],
        spandrel: "#8a5241",
        windowInset: 0.2,
        bandHeight: 0.04,
    },
    {
        id: "concrete-slab",
        wall: "#a8a49c",
        trim: "#bfbbb2",
        glass: ["#2f3740", "#39424c", "#272e36", "#424b56"],
        spandrel: "#918d85",
        windowInset: 0.15,
        bandHeight: 0.08,
    },
    {
        id: "stone-mid",
        wall: "#b8ad96",
        trim: "#d2c7ae",
        glass: ["#33383f", "#3d434b", "#2b3037", "#464d56"],
        spandrel: "#a1977f",
        windowInset: 0.18,
        bandHeight: 0.05,
    },
    {
        id: "warehouse",
        wall: "#7d8288",
        trim: "#949aa1",
        glass: ["#3a4148", "#434a52", "#31373d", "#4c545c"],
        spandrel: "#6c7176",
        windowInset: 0.24,
        bandHeight: 0.03,
    },
    {
        id: "civic",
        wall: "#c3c8cc",
        trim: "#dfe3e6",
        glass: ["#2c4a63", "#345771", "#254057", "#3d6280"],
        spandrel: "#adb2b7",
        windowInset: 0.12,
        bandHeight: 0.07,
    },
];

function drawFacade(ctx, size, style, seed) {
    const random = makeRandom(seed);
    ctx.fillStyle = style.wall;
    ctx.fillRect(0, 0, size, size);

    // Subtle vertical soiling so large flat walls are not perfectly even.
    for (let i = 0; i < 60; i += 1) {
        ctx.fillStyle = `rgba(0, 0, 0, ${0.012 + random() * 0.02})`;
        const w = size * (0.01 + random() * 0.05);
        ctx.fillRect(random() * size, 0, w, size);
    }

    const floorHeight = size / FACADE_FLOORS;
    const bayWidth = size / FACADE_BAYS;
    const inset = style.windowInset;

    for (let floor = 0; floor < FACADE_FLOORS; floor += 1) {
        const top = floor * floorHeight;

        ctx.fillStyle = style.spandrel;
        ctx.fillRect(0, top, size, floorHeight * style.bandHeight);

        for (let bay = 0; bay < FACADE_BAYS; bay += 1) {
            const left = bay * bayWidth;
            const wx = left + bayWidth * inset;
            const wy = top + floorHeight * (0.18 + style.bandHeight);
            const ww = bayWidth * (1 - inset * 2);
            const wh = floorHeight * (0.62 - style.bandHeight);

            ctx.fillStyle = style.trim;
            ctx.fillRect(wx - bayWidth * 0.03, wy - floorHeight * 0.035, ww + bayWidth * 0.06, wh + floorHeight * 0.07);

            const tint = style.glass[Math.floor(random() * style.glass.length)];
            const gradient = ctx.createLinearGradient(wx, wy, wx + ww * 0.4, wy + wh);
            gradient.addColorStop(0, tint);
            gradient.addColorStop(0.55, tint);
            gradient.addColorStop(1, "#101418");
            ctx.fillStyle = gradient;
            ctx.fillRect(wx, wy, ww, wh);

            ctx.fillStyle = "rgba(255, 255, 255, 0.10)";
            ctx.fillRect(wx, wy, ww, wh * 0.16);

            ctx.strokeStyle = "rgba(20, 22, 26, 0.35)";
            ctx.lineWidth = Math.max(1, size / 340);
            ctx.beginPath();
            ctx.moveTo(wx + ww / 2, wy);
            ctx.lineTo(wx + ww / 2, wy + wh);
            ctx.stroke();
        }
    }

    ctx.strokeStyle = "rgba(0, 0, 0, 0.18)";
    ctx.lineWidth = Math.max(1, size / 300);
    for (let bay = 1; bay < FACADE_BAYS; bay += 1) {
        const x = bay * bayWidth;
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, size);
        ctx.stroke();
    }
}

export function createFacadeTextures(size, anisotropy) {
    return FACADE_STYLES.map((style, index) => {
        const canvas = createCanvas(size);
        const ctx = canvas.getContext("2d");
        drawFacade(ctx, size, style, 900 + index * 137);
        const texture = toTexture(canvas, { anisotropy });
        texture.name = style.id;
        return texture;
    });
}

export function createRoofTexture(size, anisotropy) {
    const canvas = createCanvas(size);
    const ctx = canvas.getContext("2d");
    const random = makeRandom(31415);
    const grain = fbmField(size, 4, 30, random);
    paintField(ctx, size, grain, (v) => {
        const level = Math.max(48, Math.min(120, 72 + (v - 0.5) * 40));
        return [level, level - 2, level - 6];
    });
    for (let i = 0; i < 26; i += 1) {
        ctx.fillStyle = `rgba(${140 + random() * 60}, ${140 + random() * 60}, ${145 + random() * 60}, 0.22)`;
        const w = size * (0.04 + random() * 0.12);
        const h = size * (0.04 + random() * 0.12);
        ctx.fillRect(random() * size, random() * size, w, h);
    }
    return toTexture(canvas, { anisotropy });
}

/**
 * Paints the surface mask that the ground shader blends with. Channels are
 * R = asphalt, G = concrete, B = paving; anything unpainted stays grass.
 */
export function createSurfaceMask(map, network, size) {
    const canvas = createCanvas(size);
    const ctx = canvas.getContext("2d");
    const half = map.cityHalf;
    const scale = size / (half * 2);
    const toPx = (world) => (world + half) * scale;

    ctx.fillStyle = "#000000";
    ctx.fillRect(0, 0, size, size);

    // The paved plate the city sits on, so gaps between buildings read as
    // pavement rather than grass.
    ctx.fillStyle = "#00ff00";
    roundedRect(ctx, toPx(-345), toPx(-345), 690 * scale, 690 * scale, 96 * scale);
    ctx.fill();

    for (const park of map.parks) {
        ctx.fillStyle = "#000000";
        fillShape(ctx, park, toPx, scale);
    }
    for (const lot of map.lots) {
        ctx.fillStyle = "#0000ff";
        fillShape(ctx, lot, toPx, scale);
    }

    ctx.lineCap = "round";
    ctx.lineJoin = "round";
    for (const pass of ["sidewalk", "asphalt"]) {
        for (const road of network.roads) {
            ctx.strokeStyle = pass === "sidewalk" ? "#00ff00" : "#ff0000";
            ctx.lineWidth = (pass === "sidewalk" ? road.width + map.sidewalkWidth * 2 : road.width) * scale;
            ctx.beginPath();
            road.polyline.forEach(([x, z], i) => {
                const px = toPx(x);
                const pz = toPx(z);
                if (i === 0) {
                    ctx.moveTo(px, pz);
                } else {
                    ctx.lineTo(px, pz);
                }
            });
            ctx.stroke();
        }
    }

    return toTexture(canvas, { repeat: false, srgb: false, anisotropy: 4 });
}

function fillShape(ctx, shape, toPx, scale) {
    ctx.beginPath();
    if (shape.shape === "circle") {
        ctx.arc(toPx(shape.x), toPx(shape.z), shape.radius * scale, 0, Math.PI * 2);
    } else {
        ctx.rect(toPx(shape.x0), toPx(shape.z0), (shape.x1 - shape.x0) * scale, (shape.z1 - shape.z0) * scale);
    }
    ctx.fill();
}

function roundedRect(ctx, x, y, width, height, radius) {
    ctx.beginPath();
    ctx.moveTo(x + radius, y);
    ctx.arcTo(x + width, y, x + width, y + height, radius);
    ctx.arcTo(x + width, y + height, x, y + height, radius);
    ctx.arcTo(x, y + height, x, y, radius);
    ctx.arcTo(x, y, x + width, y, radius);
    ctx.closePath();
}

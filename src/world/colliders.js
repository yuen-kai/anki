/** Oriented-box field with a uniform grid, for point and segment queries. */
export class ObbField {
    constructor(boxes, cellSize = 32) {
        this.cellSize = cellSize;
        this.boxes = boxes;
        this.cells = new Map();
        boxes.forEach((box, index) => {
            const reach = Math.hypot(box.halfW, box.halfD);
            const minX = Math.floor((box.x - reach) / cellSize);
            const maxX = Math.floor((box.x + reach) / cellSize);
            const minZ = Math.floor((box.z - reach) / cellSize);
            const maxZ = Math.floor((box.z + reach) / cellSize);
            for (let cx = minX; cx <= maxX; cx += 1) {
                for (let cz = minZ; cz <= maxZ; cz += 1) {
                    const key = cx * 100003 + cz;
                    let bucket = this.cells.get(key);
                    if (!bucket) {
                        bucket = [];
                        this.cells.set(key, bucket);
                    }
                    bucket.push(index);
                }
            }
        });
    }

    nearby(x, z, radius = 0) {
        const minX = Math.floor((x - radius) / this.cellSize);
        const maxX = Math.floor((x + radius) / this.cellSize);
        const minZ = Math.floor((z - radius) / this.cellSize);
        const maxZ = Math.floor((z + radius) / this.cellSize);
        const seen = new Set();
        const found = [];
        for (let cx = minX; cx <= maxX; cx += 1) {
            for (let cz = minZ; cz <= maxZ; cz += 1) {
                const bucket = this.cells.get(cx * 100003 + cz);
                if (!bucket) {
                    continue;
                }
                for (const index of bucket) {
                    if (!seen.has(index)) {
                        seen.add(index);
                        found.push(this.boxes[index]);
                    }
                }
            }
        }
        return found;
    }

    /** Local coordinates of a world point inside a box's frame. */
    static toLocal(box, x, z) {
        const cos = Math.cos(box.rotY);
        const sin = Math.sin(box.rotY);
        const dx = x - box.x;
        const dz = z - box.z;
        return { u: dx * cos - dz * sin, v: dx * sin + dz * cos };
    }

    static containsPoint(box, x, z, margin = 0) {
        const { u, v } = ObbField.toLocal(box, x, z);
        return Math.abs(u) <= box.halfW + margin && Math.abs(v) <= box.halfD + margin;
    }

    contains(x, z, margin = 0) {
        for (const box of this.nearby(x, z, margin + 2)) {
            if (ObbField.containsPoint(box, x, z, margin)) {
                return box;
            }
        }
        return null;
    }

    /** First blocked sample along a segment, or null when the line is clear. */
    firstHitAlong(x0, z0, x1, z1, samples = 12, margin = 0.4) {
        for (let i = 1; i <= samples; i += 1) {
            const t = i / samples;
            const x = x0 + (x1 - x0) * t;
            const z = z0 + (z1 - z0) * t;
            const box = this.contains(x, z, margin);
            if (box) {
                return { t, x, z, box };
            }
        }
        return null;
    }
}

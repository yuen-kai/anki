import { createReadStream, statSync } from "node:fs";
import { createServer } from "node:http";
import { extname, join, normalize, resolve } from "node:path";

const root = resolve(new URL(".", import.meta.url).pathname);
const port = Number(process.env.PORT ?? 8123);

const MIME = {
    ".html": "text/html; charset=utf-8",
    ".js": "text/javascript; charset=utf-8",
    ".mjs": "text/javascript; charset=utf-8",
    ".css": "text/css; charset=utf-8",
    ".json": "application/json; charset=utf-8",
    ".png": "image/png",
    ".jpg": "image/jpeg",
    ".svg": "image/svg+xml",
    ".ico": "image/x-icon",
};

createServer((request, response) => {
    const requested = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
    const relative = normalize(requested === "/" ? "/index.html" : requested).replace(/^(\.\.[/\\])+/, "");
    const path = join(root, relative);

    if (!path.startsWith(root)) {
        response.writeHead(403).end("Forbidden");
        return;
    }

    let stats;
    try {
        stats = statSync(path);
    } catch {
        response.writeHead(404, { "content-type": "text/plain" }).end("Not found");
        return;
    }
    if (stats.isDirectory()) {
        response.writeHead(404, { "content-type": "text/plain" }).end("Not found");
        return;
    }

    response.writeHead(200, {
        "content-type": MIME[extname(path)] ?? "application/octet-stream",
        "content-length": stats.size,
        "cache-control": "no-cache",
    });
    createReadStream(path).pipe(response);
}).listen(port, () => {
    console.log(`Police Chase running at http://localhost:${port}/`);
});

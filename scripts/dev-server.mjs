// Local stand-in for `vercel dev`: serves www/ and dispatches /api/* to the
// same handler modules Vercel would invoke. No Vercel account needed.
import { createServer } from "node:http";
import { createReadStream, existsSync, statSync } from "node:fs";
import { readdir } from "node:fs/promises";
import { extname, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const ROOT = resolve(import.meta.dirname, "..");
const WWW = join(ROOT, "www");
const API = join(ROOT, "api");
const PORT = Number(process.env.PORT || 3000);

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".wasm": "application/wasm",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".gif": "image/gif",
  ".svg": "image/svg+xml",
  ".ico": "image/x-icon",
  ".ttf": "font/ttf",
  ".mp3": "audio/mpeg",
  ".ogg": "audio/ogg",
  ".wav": "audio/wav",
  ".turbo": "application/octet-stream",
};

// Mirrors Vercel's filesystem routing: exact file, then [param], then [...rest].
// Dynamic segments are collected into `params` so handlers can read them from
// req.query exactly like they do on Vercel.
async function resolveRoute(dir, segments, params = {}) {
  if (!segments.length) {
    const index = join(dir, "index.js");
    return existsSync(index) ? { file: index, params } : null;
  }
  const [head, ...tail] = segments;
  const entries = await readdir(dir, { withFileTypes: true });

  if (!tail.length) {
    const exact = entries.find((e) => e.isFile() && e.name === `${head}.js`);
    if (exact) return { file: join(dir, exact.name), params };
  }
  const subdir = entries.find((e) => e.isDirectory() && e.name === head);
  if (subdir) {
    const nested = await resolveRoute(join(dir, head), tail, params);
    if (nested) return nested;
  }
  if (!tail.length) {
    const param = entries.find((e) => e.isFile() && /^\[[^.].*\]\.js$/.test(e.name));
    if (param) {
      const name = param.name.slice(1, -3); // [code].js -> code
      return { file: join(dir, param.name), params: { ...params, [name]: head } };
    }
  }
  const rest = entries.find((e) => e.isFile() && /^\[\.\.\..+\]\.js$/.test(e.name));
  if (rest) {
    const name = rest.name.slice(4, -3); // [...path].js -> path
    return { file: join(dir, rest.name), params: { ...params, [name]: [head, ...tail] } };
  }
  return null;
}

function serveStatic(req, res, pathname) {
  let target = join(WWW, decodeURIComponent(pathname).replace(/^\/+/, ""));
  if (!target.startsWith(WWW)) {
    res.writeHead(403).end("Forbidden");
    return;
  }
  if (existsSync(target) && statSync(target).isDirectory()) target = join(target, "index.html");
  if (!existsSync(target)) {
    res.writeHead(404, { "Content-Type": "text/plain" }).end("Not found");
    return;
  }
  res.writeHead(200, {
    "Content-Type": MIME[extname(target).toLowerCase()] || "application/octet-stream",
    "Cache-Control": "no-store",
  });
  createReadStream(target).pipe(res);
}

const server = createServer(async (req, res) => {
  const url = new URL(req.url, "http://localhost");
  const { pathname } = url;

  if (!pathname.startsWith("/api/")) {
    const page = pathname === "/" ? "/index.html" : pathname === "/admin" ? "/admin.html" : pathname;
    serveStatic(req, res, page);
    return;
  }

  const segments = pathname.slice(5).split("/").filter(Boolean);
  const route = await resolveRoute(API, segments);
  if (!route) {
    res.writeHead(404, { "Content-Type": "application/json" });
    res.end(JSON.stringify({ ok: false, error: `No API route for ${pathname}` }));
    return;
  }
  // Vercel populates req.query with route params merged over search params.
  req.query = { ...Object.fromEntries(url.searchParams), ...route.params };
  try {
    const mod = await import(`${pathToFileURL(route.file).href}?t=${Date.now()}`);
    await mod.default(req, res);
  } catch (err) {
    console.error(`${req.method} ${pathname}`, err);
    if (!res.headersSent) res.writeHead(500, { "Content-Type": "application/json" });
    res.end(JSON.stringify({ ok: false, error: "Server error" }));
  }
});

server.listen(PORT, () => {
  console.log(`Game   http://localhost:${PORT}/`);
  console.log(`Admin  http://localhost:${PORT}/admin`);
});

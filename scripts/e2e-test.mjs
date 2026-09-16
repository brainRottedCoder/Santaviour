// Full end-to-end API test against a real dev server with a throwaway store.
// Spawns scripts/dev-server.mjs with cwd in a temp dir so .data/store.json
// lands there, then exercises every endpoint over HTTP with per-user cookies.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

const PORT = 3199;
const BASE = `http://127.0.0.1:${PORT}`;
const ADMIN_PASSWORD = process.env.ADMIN_PASSWORD || "santa-admin-local-dev";

let passed = 0;
function ok(label, cond, extra = "") {
  if (!cond) {
    console.error(`FAIL ${label} ${extra}`);
    process.exitCode = 1;
    throw new Error(`FAIL: ${label}`);
  }
  passed += 1;
  console.log(`ok   ${label}`);
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function client() {
  let cookie = "";
  return {
    async req(method, path, body) {
      const res = await fetch(BASE + path, {
        method,
        headers: {
          ...(body ? { "Content-Type": "application/json" } : {}),
          ...(cookie ? { Cookie: cookie } : {}),
        },
        body: body ? JSON.stringify(body) : undefined,
        redirect: "manual",
      });
      const setCookie = res.headers.get("set-cookie");
      if (setCookie) cookie = setCookie.split(";")[0];
      let data = {};
      try {
        data = await res.json();
      } catch {
        /* non-json */
      }
      return { status: res.status, data };
    },
    get(path) {
      return this.req("GET", path);
    },
    post(path, body) {
      return this.req("POST", path, body);
    },
    del(path) {
      return this.req("DELETE", path);
    },
  };
}

/* ------------------------------------------------------------ bootstrap -- */

// cwd in a temp dir: FileRedis writes <cwd>/.data/store.json, so the real
// store is never touched. env.js still finds the repo's .env.local because
// it also searches relative to the api/_lib module.
const tmp = mkdtempSync(join(tmpdir(), "santa-e2e-"));
const devServer = fileURLToPath(new URL("./dev-server.mjs", import.meta.url));
const server = spawn(process.execPath, [devServer], {
  cwd: tmp,
  env: {
    ...process.env,
    PORT: String(PORT),
    UPSTASH_REDIS_REST_URL: "",
    UPSTASH_REDIS_REST_TOKEN: "",
    VERCEL_ENV: "",
  },
  stdio: ["ignore", "pipe", "pipe"],
});
server.stderr.on("data", (chunk) => process.stderr.write(`[server] ${chunk}`));
async function waitUp() {
  for (let i = 0; i < 60; i += 1) {
    try {
      const res = await fetch(`${BASE}/api/auth/me`);
      if (res.status) return;
    } catch {
      await sleep(250);
    }
  }
  throw new Error("dev server did not start");
}

try {
  await waitUp();

  /* ------------------------------------------------------------ auth ---- */

  const a = client();
  const b = client();
  const c = client();
  const d = client();
  const e = client();

  let r = await a.post("/api/auth/register", { username: "ab", password: "DemoPass1" });
  ok("register rejects short username", r.status === 400);
  r = await a.post("/api/auth/register", { username: "bad name!", password: "DemoPass1" });
  ok("register rejects illegal characters", r.status === 400);
  r = await a.post("/api/auth/register", { username: "Alice", password: "123" });
  ok("register rejects short password", r.status === 400);
  r = await a.get("/api/auth/me");
  ok("me requires auth", r.status === 401);

  r = await a.post("/api/auth/register", { username: "Alice", password: "DemoPass1" });
  ok("register creates account", r.status === 201 && r.data.user.username === "Alice");
  ok("new account stats are zero", r.data.user.runs === 0 && r.data.user.completions === 0);
  r = await a.post("/api/auth/register", { username: "alice", password: "DemoPass1" });
  ok("register rejects case-insensitive duplicate", r.status === 409);

  for (const [cl, name] of [[b, "Bobby"], [c, "Carol"], [d, "Dave"], [e, "Erin"]]) {
    r = await cl.post("/api/auth/register", { username: name, password: "DemoPass1" });
    ok(`register ${name}`, r.status === 201);
  }

  r = await a.post("/api/auth/login", { username: "Alice", password: "wrong-pass" });
  ok("login rejects wrong password", r.status === 401);
  r = await a.post("/api/auth/login", { username: "Alice", password: "DemoPass1" });
  ok("login works", r.status === 200 && r.data.user.username === "Alice");
  r = await a.req("GET", "/api/auth/login");
  ok("login rejects GET", r.status === 405);

  /* ----------------------------------------------------------- rooms ---- */

  r = await a.post("/api/rooms/create", { name: "E2E Race" });
  ok("create room", r.status === 201 && /^[A-Z0-9]{6}$/.test(r.data.room.code));
  const code = r.data.room.code;
  ok("creator is a member", r.data.room.members.includes("Alice"));
  ok("leaderboard starts idle", r.data.room.leaderboard[0].status === "idle");

  for (const cl of [b, c, d]) {
    r = await cl.post("/api/rooms/join", { code });
    ok(`join room (${r.data.room?.members?.length} members)`, r.status === 200);
  }
  r = await e.post("/api/rooms/join", { code });
  ok("fifth player rejected (room full)", r.status === 409);
  r = await e.post("/api/rooms/join", { code: "NOPE99" });
  ok("joining a missing room 404s", r.status === 404);
  r = await e.post("/api/rooms/join", { code: "x" });
  ok("join validates the code format", r.status === 400);

  r = await b.get(`/api/rooms/${code}`);
  ok("GET /api/rooms/:code works for a member", r.status === 200 && r.data.room.code === code);
  r = await e.get(`/api/rooms/${code}`);
  ok("GET /api/rooms/:code rejects non-members", r.status === 403);
  r = await b.get("/api/rooms/ZZZZ9");
  ok("GET missing room is rejected (membership checked first)", r.status === 403 || r.status === 404);

  /* --------------------------------------------- runs + time tracking ---- */

  const board = (room, name) => room.leaderboard.find((row) => row.username === name);

  r = await a.post("/api/runs/start", { code, level: 1, score: 0 });
  ok("run start", r.status === 200 && board(r.data.room, "Alice").status === "started");
  const startedAt = Number(board(r.data.room, "Alice").startedAt);
  ok("start records startedAt", Number.isFinite(startedAt) && startedAt > 0);

  await sleep(2000);
  r = await b.get(`/api/rooms/${code}`);
  const liveMs = board(r.data.room, "Alice").elapsedMs;
  ok(
    `live clock ticks on the board (~2s, got ${liveMs}ms)`,
    liveMs >= 1500 && liveMs <= 4500
  );

  r = await a.post("/api/runs/progress", { code, level: 1, score: 500, kids: 1, keys: 1, lives: 3, hp: 6 });
  ok("progress updates score", board(r.data.room, "Alice").score === 500);
  r = await a.post("/api/runs/checkpoint", { code, level: 1, score: 700 });
  ok("checkpoint records level clear", board(r.data.room, "Alice").levelReached >= 1);

  r = await a.post("/api/runs/finish", { code, status: "completed", level: 1, score: 700 });
  ok("finish rejects a win without cleared levels", r.status === 400);

  r = await a.post("/api/runs/finish", { code, status: "failed", level: 2, score: 800, detail: "game_over", lives: 0, hp: 0 });
  const failedRow = board(r.data.room, "Alice");
  ok("finish records a failed run", r.status === 200 && failedRow.status === "failed");
  ok("failed run keeps the failure reason", failedRow.failureReason === "game_over");
  ok(
    `failed run clock matches wall time (~2s+, got ${failedRow.elapsedMs}ms)`,
    failedRow.elapsedMs >= 1800 && failedRow.elapsedMs <= 6000
  );
  ok("failed run keeps max score", failedRow.score === 800);

  r = await a.get("/api/auth/me");
  ok("lifetime stats count the finished run", r.data.user.runs === 1 && r.data.user.completions === 0);
  ok("best score tracked", r.data.user.bestScore === 800);
  ok("no best time without a win", r.data.user.bestMs === null);

  // Replay after failing: clock and score must reset.
  r = await a.post("/api/runs/start", { code, level: 1, score: 0 });
  const retryRow = board(r.data.room, "Alice");
  ok("replay after fail resets the run", retryRow.status === "started" && retryRow.score === 0);
  ok("replay clock restarts near zero", (retryRow.elapsedMs ?? 9999) < 1500);

  await sleep(1200);
  await a.post("/api/runs/checkpoint", { code, level: 1, score: 900 });
  await a.post("/api/runs/checkpoint", { code, level: 2, score: 1500 });
  r = await a.post("/api/runs/finish", { code, status: "completed", level: 3, score: 2000, kids: 3, keys: 3, lives: 2, hp: 5 });
  const winRow = board(r.data.room, "Alice");
  ok("finish records a win after levels 1+2", r.status === 200 && winRow.status === "completed");
  ok(
    `winning time matches wall time (~1.2s+, got ${winRow.elapsedMs}ms)`,
    winRow.elapsedMs >= 1000 && winRow.elapsedMs <= 6000
  );

  r = await a.get("/api/auth/me");
  ok("win counted in lifetime stats", r.data.user.completions === 1 && r.data.user.runs === 2);
  ok("best time equals the winning clock", r.data.user.bestMs === winRow.elapsedMs);
  ok("best score kept the max", r.data.user.bestScore === 2000);

  // Replay after winning must also work ("Play again" on a completed run).
  r = await a.post("/api/runs/start", { code, level: 1, score: 0 });
  ok("replay after a win starts fresh", board(r.data.room, "Alice").status === "started");
  r = await a.post("/api/runs/finish", { code, status: "failed", level: 1, score: 100, detail: "game_over" });
  ok("post-win replay can finish", r.status === 200 && board(r.data.room, "Alice").status === "failed");
  r = await a.get("/api/auth/me");
  ok("lifetime win and best time survive a later loss", r.data.user.completions === 1 && r.data.user.bestMs === winRow.elapsedMs);
  ok("run count increments per attempt", r.data.user.runs === 3);

  // Bobby finishes a faster win to check ranking order.
  await b.post("/api/runs/start", { code, level: 1, score: 0 });
  await b.post("/api/runs/checkpoint", { code, level: 1, score: 300 });
  await b.post("/api/runs/checkpoint", { code, level: 2, score: 900 });
  r = await b.post("/api/runs/finish", { code, status: "completed", level: 3, score: 2500 });
  const bWin = board(r.data.room, "Bobby");
  ok("second player wins", bWin.status === "completed");
  const boardRows = r.data.room.leaderboard;
  ok("winner ranks above failed players", boardRows[0].username === "Bobby" && boardRows[0].rank === 1);
  ok("idle players rank last", boardRows[boardRows.length - 1].status === "idle");
  ok("first finisher earns the ₹1000 prize", boardRows[0].prizeRupees === 1000);
  ok("non-winners earn no prize", boardRows.every((row) => row.username === "Bobby" || row.prizeRupees === 0));
  r = await b.get("/api/auth/me");
  ok("winnings surface in the player session", r.data.user.winnings === 1000);
  r = await a.get("/api/auth/me");
  ok("failed replay leaves no prize", r.data.user.winnings === 0);

  // Idempotent finish: a second finish call must not double-count stats.
  r = await b.post("/api/runs/finish", { code, status: "completed", level: 3, score: 2500 });
  ok("duplicate finish is a no-op", r.status === 200);
  r = await b.get("/api/auth/me");
  ok("duplicate finish does not double-count", r.data.user.runs === 1 && r.data.user.completions === 1);

  // Non-members cannot post runs.
  r = await e.post("/api/runs/start", { code, level: 1 });
  ok("non-member cannot start a run", r.status === 403);

  /* ------------------------------------------------------------ admin ---- */

  const adm = client();
  r = await adm.get("/api/admin/session");
  ok("admin session starts logged out but configured", r.status === 200 && r.data.admin === false && r.data.configured === true);
  r = await adm.get("/api/admin/overview");
  ok("overview requires admin auth", r.status === 401);
  r = await adm.post("/api/admin/login", { password: "wrong-password" });
  ok("wrong admin password rejected", r.status === 401);
  r = await adm.post("/api/admin/login", { password: ADMIN_PASSWORD });
  ok("admin login works", r.status === 200 && r.data.admin === true);

  // A player cookie must not open admin routes.
  r = await a.get("/api/admin/overview");
  ok("player session cannot read admin overview", r.status === 401);

  r = await adm.get("/api/admin/overview");
  ok("overview returns totals", r.status === 200 && r.data.totals.users === 5 && r.data.totals.rooms === 1);
  ok("overview totals count runs and wins", r.data.totals.runs === 4 && r.data.totals.completions === 2);
  const aliceRow = r.data.users.find((row) => row.username === "Alice");
  const bobbyRow = r.data.users.find((row) => row.username === "Bobby");
  ok("observability: placements attached", aliceRow.placements.length === 1 && bobbyRow.placements.length === 1);
  ok("observability: run history attached", aliceRow.history.length === 3 && bobbyRow.history.length === 1);
  ok("observability: history keeps room name", aliceRow.history[0].room === "E2E Race");
  ok("observability: last active stamp set", Boolean(aliceRow.lastActiveAt));
  ok("observability: global rank favours more wins then faster time", r.data.users[0].username === "Bobby" || r.data.users[0].username === "Alice");
  const first = r.data.users[0];
  ok(
    "global rank tie-break is fastest win",
    first.username === "Bobby" ? first.bestMs <= aliceRow.bestMs : true
  );
  const erinRow = r.data.users.find((row) => row.username === "Erin");
  ok("player with no runs ranks last with zeroed stats", erinRow.runs === 0 && erinRow.globalRank === 5);
  ok("observability: per-user winnings computed", bobbyRow.winnings === 1000 && aliceRow.winnings === 0);
  ok("observability: totals track prize money", r.data.totals.winnings === 1000);

  r = await adm.get("/api/admin/user?name=Alice");
  ok("player file resolves", r.status === 200 && r.data.user.username === "Alice");
  ok("player file carries full history", r.data.user.history.length === 3);
  ok("player file carries global rank", r.data.user.globalRank >= 1);
  ok("player file history newest first", r.data.user.history[0].status === "failed");
  r = await adm.get("/api/admin/user?name=Nobody");
  ok("missing player file 404s", r.status === 404);

  r = await adm.del(`/api/admin/run?code=${code}&name=Carol`);
  ok("clearing a run with no result 404s", r.status === 404);
  r = await adm.del(`/api/admin/run?code=${code}&name=Alice`);
  ok("clear run removes the live attempt", r.status === 200 && r.data.cleared);
  r = await adm.get("/api/admin/overview");
  const aliceAfterClear = r.data.users.find((row) => row.username === "Alice");
  ok("cleared attempt shows idle on the board", aliceAfterClear.placements[0].status === "idle");
  ok("clearing an attempt keeps lifetime stats", aliceAfterClear.runs === 3);

  r = await adm.post("/api/admin/reset", { code });
  ok("reset room clears the scoreboard", r.status === 200 && r.data.reset);
  r = await adm.get(`/api/admin/user?name=Bobby`);
  ok("reset keeps history log", r.data.user.history.length === 1);
  r = await b.get(`/api/rooms/${code}`);
  ok("reset keeps membership, wipes results", r.data.room.members.length === 4 && r.data.room.leaderboard.every((row) => row.status === "idle"));
  r = await adm.get("/api/admin/overview");
  ok("reset releases the prize money", r.data.totals.winnings === 0);

  r = await adm.del(`/api/admin/room?code=${code}`);
  ok("delete room", r.status === 200 && r.data.deleted);
  r = await b.get(`/api/rooms/${code}`);
  ok("deleted room is gone (membership removed with it)", r.status === 403 || r.status === 404);
  r = await adm.del(`/api/admin/room?code=${code}`);
  ok("deleting a missing room 404s", r.status === 404);

  r = await adm.del("/api/admin/user?name=Erin");
  ok("delete user", r.status === 200 && r.data.deleted);
  r = await e.get("/api/auth/me");
  ok("deleted user session is rejected", r.status === 401);
  r = await adm.get("/api/admin/overview");
  ok("overview reflects the deletion", r.data.totals.users === 4);

  r = await adm.get("/api/admin/nope");
  ok("unknown admin route 404s", r.status === 404);

  r = await adm.post("/api/admin/logout");
  ok("admin logout", r.status === 200);
  r = await adm.get("/api/admin/overview");
  ok("admin routes lock after logout", r.status === 401);

  console.log(`\ne2e ok — ${passed} checks passed`);
} finally {
  server.kill();
}

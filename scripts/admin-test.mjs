// Runs the admin data + moderation layer against a throwaway store so the real
// .data/store.json (or Upstash) is never touched.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

process.chdir(mkdtempSync(join(tmpdir(), "santa-admin-")));

await import("../api/_lib/env.js");
process.env.UPSTASH_REDIS_REST_URL = "";
process.env.UPSTASH_REDIS_REST_TOKEN = "";
process.env.VERCEL_ENV = "";
process.env.SESSION_SECRET = "admin-test-secret-value";
process.env.ADMIN_PASSWORD = "hunter2hunter2";

const { redis } = await import("../api/_lib/redis.js");
const { buildOverview, buildUserDetail, listUserKeys } = await import("../api/_lib/directory.js");
const { clearRun, deleteRoom, deleteUser, resetRoom } = await import("../api/_lib/moderation.js");
const { recordRun, readHistory } = await import("../api/_lib/history.js");
const { attachAdminSession, passwordMatches, readAdminSession } = await import(
  "../api/_lib/admin.js"
);

const db = redis();
const ok = (label, cond) => {
  if (!cond) throw new Error(`FAIL: ${label}`);
  console.log(`ok  ${label}`);
};

/* -- store primitives ---------------------------------------------------- */

await db.lpush("list:demo", "a");
await db.lpush("list:demo", "b");
await db.lpush("list:demo", "c");
ok("lpush prepends", (await db.lrange("list:demo", 0, -1)).join(",") === "c,b,a");
await db.ltrim("list:demo", 0, 1);
ok("ltrim keeps head", (await db.lrange("list:demo", 0, -1)).join(",") === "c,b");
await db.sadd("set:demo", "x");
ok("srem removes", (await db.srem("set:demo", "x")) === 1 && (await db.srem("set:demo", "x")) === 0);
ok("keys globs", (await db.keys("list:*")).includes("list:demo"));
await db.del("list:demo", "set:demo");
ok("del clears", (await db.keys("list:*")).length === 0);

/* -- fixtures: two players, one room, written WITHOUT the index sets ------ */

const now = Date.now();
const players = [
  { key: "ace", username: "Ace", runs: 3, completions: 2, bestMs: 400000, bestScore: 3000 },
  { key: "bolt", username: "Bolt", runs: 4, completions: 2, bestMs: 300000, bestScore: 2000 },
  { key: "cyn", username: "Cyn", runs: 1, completions: 0, bestMs: "", bestScore: 900 },
];
for (const player of players) {
  await db.hset(`user:${player.key}`, {
    username: player.username,
    passwordHash: "x",
    createdAt: new Date(now - 86400000).toISOString(),
    runs: player.runs,
    completions: player.completions,
    bestMs: player.bestMs === "" ? "" : String(player.bestMs),
    bestScore: player.bestScore,
  });
  await db.sadd(`user:${player.key}:rooms`, "RACE01");
  await db.sadd("room:RACE01:members", player.key);
}
await db.hset("room:RACE01", {
  code: "RACE01",
  name: "Race <one>",
  owner: "Ace",
  status: "waiting",
  createdAt: new Date(now - 3600000).toISOString(),
});
await db.hset("room:RACE01:result:ace", {
  username: "Ace",
  status: "completed",
  startedAt: String(now - 400000),
  finishedAt: new Date(now - 10000).toISOString(),
  elapsedMs: "400000",
  score: 3000,
  levelReached: 3,
  levelsCleared: "1,2",
});
await db.hset("room:RACE01:result:bolt", {
  username: "Bolt",
  status: "completed",
  startedAt: String(now - 300000),
  finishedAt: new Date(now - 5000).toISOString(),
  elapsedMs: "300000",
  score: 2000,
  levelReached: 3,
  levelsCleared: "1,2",
});
await db.hset("room:RACE01:result:cyn", {
  username: "Cyn",
  status: "started",
  startedAt: String(now - 20000),
  elapsedMs: "",
  score: 400,
  levelReached: 1,
});

/* -- discovery + aggregation --------------------------------------------- */

const keys = await listUserKeys();
ok("finds accounts with no index entry", keys.join(",") === "ace,bolt,cyn");
ok("heals the user index", (await db.smembers("index:users")).length === 3);
ok("skips user:<x>:rooms sub-keys", !keys.some((key) => key.includes(":")));

await recordRun("ace", "RACE01", {
  status: "failed",
  finishedAt: new Date(now - 500000).toISOString(),
  elapsedMs: "120000",
  score: 800,
  levelReached: 1,
  kids: 1,
  keys: 2,
  lives: 0,
  hp: 0,
  failureReason: "game_over",
  levelsCleared: "",
});
ok("history records a run", (await readHistory("ace")).length === 1);
ok("history keeps the room name", (await readHistory("ace"))[0].room === "Race <one>");

const overview = await buildOverview();
ok("overview totals users", overview.totals.users === 3);
ok("overview totals rooms", overview.totals.rooms === 1);
ok("overview counts live runs", overview.totals.activeRuns === 1);
ok(
  "global rank favours the faster winner",
  overview.users[0].username === "Bolt" && overview.users[0].globalRank === 1
);
ok("winless player ranks last", overview.users[2].username === "Cyn");
ok("every user carries placements", overview.users.every((row) => row.placements.length === 1));
ok(
  "placement rank matches the room board",
  overview.users.find((row) => row.username === "Ace").placements[0].rank === 2
);
ok("last active is populated", Boolean(overview.users[0].lastActiveAt));

const detail = await buildUserDetail("ace");
ok("detail resolves global rank", detail.globalRank === 2);
ok("detail carries full history", detail.history.length === 1);
ok("detail lists rooms", detail.placements[0].code === "RACE01");

/* -- admin session ------------------------------------------------------- */

ok("right password matches", passwordMatches("hunter2hunter2"));
ok("wrong password rejected", !passwordMatches("hunter2hunter3"));
ok("short password rejected", !passwordMatches("nope"));

const res = {
  headers: {},
  setHeader(k, v) {
    this.headers[k.toLowerCase()] = v;
  },
  getHeader(k) {
    return this.headers[k.toLowerCase()];
  },
};
await attachAdminSession({ headers: {} }, res);
const cookie = String(res.getHeader("Set-Cookie")).split(";")[0];
ok("admin cookie round-trips", (await readAdminSession({ headers: { cookie } }))?.role === "admin");
ok("no cookie means no session", (await readAdminSession({ headers: {} })) === null);

const { signSession } = await import("../api/_lib/session.js");
const playerCookie = `santa_admin=${await signSession("Ace")}`;
ok(
  "player token cannot pose as admin",
  (await readAdminSession({ headers: { cookie: playerCookie } })) === null
);

/* -- moderation ---------------------------------------------------------- */

await clearRun("RACE01", "cyn");
ok("clearRun drops one result", !(await db.hgetall("room:RACE01:result:cyn")).status);

await resetRoom("RACE01");
ok("resetRoom clears results", !(await db.hgetall("room:RACE01:result:ace")).status);
ok("resetRoom keeps members", (await db.smembers("room:RACE01:members")).length === 3);
ok("resetRoom keeps lifetime stats", (await db.hgetall("user:ace")).runs === "3");

await deleteUser("bolt");
ok("deleteUser removes the account", !(await db.hgetall("user:bolt")).username);
ok("deleteUser leaves the room", (await db.smembers("room:RACE01:members")).length === 2);
ok("deleteUser drops the index entry", !(await db.smembers("index:users")).includes("bolt"));
ok("deleteUser wipes history", (await readHistory("bolt")).length === 0);
ok("deleteUser is reflected in the overview", (await buildOverview()).totals.users === 2);

await deleteRoom("RACE01");
ok("deleteRoom removes the room", !(await db.hgetall("room:RACE01")).code);
ok("deleteRoom unlinks players", (await db.smembers("user:ace:rooms")).length === 0);
ok("deleteRoom drops the index entry", (await db.smembers("index:rooms")).length === 0);

let missing = null;
await deleteRoom("RACE01").catch((err) => {
  missing = err;
});
ok("missing room reports 404", missing?.status === 404);

console.log("\nadmin test ok");

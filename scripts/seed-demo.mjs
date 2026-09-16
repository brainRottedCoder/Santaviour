import bcrypt from "bcryptjs";
import { readFileSync } from "node:fs";
import { Redis } from "@upstash/redis";

function loadEnvFile(path) {
  try {
    const text = readFileSync(path, "utf8");
    for (const line of text.split(/\r?\n/)) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith("#")) continue;
      const eq = trimmed.indexOf("=");
      if (eq === -1) continue;
      const key = trimmed.slice(0, eq).trim();
      const value = trimmed.slice(eq + 1).trim();
      if (!process.env[key]) process.env[key] = value;
    }
  } catch {
    /* optional */
  }
}

loadEnvFile(".env.local");
loadEnvFile(".env");

const url = process.env.UPSTASH_REDIS_REST_URL;
const token = process.env.UPSTASH_REDIS_REST_TOKEN;
const db = url && token
  ? new Redis({ url, token })
  : new (await import("../api/_lib/file-redis.js")).FileRedis();

if (!url || !token) {
  console.log("Upstash env not set; seeding local .data/store.json for vercel dev.");
}

const PASSWORD = "DemoPass1";
const now = Date.now();
const hash = await bcrypt.hash(PASSWORD, 10);

const MIN = 60_000;
const HOUR = 60 * MIN;
const DAY = 24 * HOUR;

function ago(ms) {
  return now - ms;
}

function iso(ms) {
  return new Date(ms).toISOString();
}

function clock(min, sec = 0) {
  return (min * 60 + sec) * 1000;
}

function cleared(level) {
  if (level >= 3) return "1,2";
  if (level >= 2) return "1";
  return "";
}

function resultOf(entry) {
  const finished = entry.status !== "started";
  const finishedAt = finished ? iso(entry.at) : "";
  const startedAt = finished ? entry.at - entry.elapsedMs : entry.at;
  return {
    username: entry.username,
    status: entry.status,
    startedAt: String(startedAt),
    finishedAt,
    elapsedMs: entry.status === "started" ? "" : String(entry.elapsedMs),
    score: entry.score,
    levelReached: entry.level,
    kids: entry.kids ?? 0,
    keys: entry.keys ?? 0,
    lives: entry.lives ?? 0,
    hp: entry.hp ?? 0,
    failureReason: entry.reason || "",
    levelsCleared: entry.cleared ?? cleared(entry.level),
  };
}

function historyOf(entry, roomName) {
  return {
    at: iso(entry.at),
    code: entry.code,
    room: roomName,
    status: entry.status,
    score: entry.score,
    levelReached: entry.level,
    elapsedMs: entry.elapsedMs,
    kids: entry.kids ?? 0,
    keys: entry.keys ?? 0,
    lives: entry.lives ?? 0,
    hp: entry.hp ?? 0,
    failureReason: entry.reason || "",
    levelsCleared: entry.cleared ?? cleared(entry.level),
  };
}

function statsFrom(history) {
  const done = history.filter((row) => row.status === "completed" || row.status === "failed");
  const wins = done.filter((row) => row.status === "completed");
  const bestWin = wins.reduce((best, row) => Math.min(best, row.elapsedMs), Infinity);
  return {
    runs: done.length,
    completions: wins.length,
    bestMs: wins.length ? bestWin : "",
    bestScore: done.reduce((best, row) => Math.max(best, row.score), 0),
  };
}

async function upsertUser(username, createdAt, rooms, history) {
  const key = username.toLowerCase();
  const stats = statsFrom(history);
  await db.hset(`user:${key}`, {
    username,
    passwordHash: hash,
    createdAt: iso(createdAt),
    runs: stats.runs,
    completions: stats.completions,
    bestMs: stats.bestMs === "" ? "" : String(stats.bestMs),
    bestScore: stats.bestScore,
  });
  await db.del(`user:${key}:rooms`, `user:${key}:history`);
  for (const code of rooms) await db.sadd(`user:${key}:rooms`, code);
  await db.sadd("index:users", key);
  for (const entry of history) {
    await db.lpush(`user:${key}:history`, JSON.stringify(entry));
  }
}

async function upsertRoom(code, name, owner, createdAt, members, latestByUser) {
  await db.hset(`room:${code}`, {
    code,
    name,
    owner,
    status: "waiting",
    createdAt: iso(createdAt),
  });
  await db.del(`room:${code}:members`);
  for (const member of members) {
    const key = member.toLowerCase();
    await db.sadd(`room:${code}:members`, key);
    await db.del(`room:${code}:result:${key}`);
    const latest = latestByUser[member];
    if (latest) await db.hset(`room:${code}:result:${key}`, resultOf(latest));
  }
  await db.sadd("index:rooms", code);
}

const rooms = {
  DEMO01: "Christmas Challenge",
  WARM4: "Fireplace Cup",
  NIGHT1: "Night Shift",
  GIFT2: "Gift Run",
  COAL3: "Coal League",
};

// Oldest first so lpush leaves newest at the head of the list.
const demoHistory = {
  SnowMaster: [
    { at: ago(6 * DAY + 3 * HOUR), code: "DEMO01", status: "failed", elapsedMs: clock(5, 40), score: 900, level: 1, kids: 1, keys: 1, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(4 * DAY + 2 * HOUR), code: "DEMO01", status: "failed", elapsedMs: clock(7, 10), score: 2100, level: 2, kids: 2, keys: 2, lives: 0, hp: 0, reason: "time_up" },
    { at: ago(2 * DAY + 5 * HOUR), code: "DEMO01", status: "completed", elapsedMs: clock(9, 2), score: 3100, level: 3, kids: 3, keys: 3, lives: 2, hp: 4 },
    { at: ago(8 * HOUR), code: "DEMO01", status: "completed", elapsedMs: clock(8, 24), score: 3400, level: 3, kids: 3, keys: 3, lives: 3, hp: 5 },
  ],
  Rudolph: [
    { at: ago(5 * DAY), code: "DEMO01", status: "failed", elapsedMs: clock(4, 12), score: 1100, level: 1, kids: 1, keys: 2, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(3 * DAY + HOUR), code: "DEMO01", status: "failed", elapsedMs: clock(8, 50), score: 2400, level: 2, kids: 2, keys: 3, lives: 0, hp: 0, reason: "time_up" },
    { at: ago(8 * HOUR + 2 * MIN), code: "DEMO01", status: "completed", elapsedMs: clock(9, 51), score: 2950, level: 3, kids: 3, keys: 3, lives: 1, hp: 3 },
  ],
  ElfRunner: [
    { at: ago(6 * DAY + HOUR), code: "DEMO01", status: "failed", elapsedMs: clock(2, 15), score: 400, level: 1, kids: 0, keys: 1, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(5 * DAY + 4 * HOUR), code: "DEMO01", status: "failed", elapsedMs: clock(3, 40), score: 720, level: 1, kids: 1, keys: 1, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(3 * DAY), code: "DEMO01", status: "failed", elapsedMs: clock(6, 5), score: 1400, level: 2, kids: 2, keys: 2, lives: 0, hp: 0, reason: "time_up" },
    { at: ago(1 * DAY + 6 * HOUR), code: "DEMO01", status: "failed", elapsedMs: clock(5, 22), score: 1550, level: 2, kids: 2, keys: 1, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(8 * HOUR + 4 * MIN), code: "DEMO01", status: "failed", elapsedMs: clock(7, 0), score: 1800, level: 2, kids: 2, keys: 2, lives: 0, hp: 0, reason: "game_over" },
  ],
};

const extraHistory = {
  HollyDash: [
    { at: ago(10 * DAY + 2 * HOUR), code: "WARM4", status: "failed", elapsedMs: clock(1, 48), score: 420, level: 1, kids: 0, keys: 1, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(10 * DAY + HOUR), code: "WARM4", status: "failed", elapsedMs: clock(6, 12), score: 1680, level: 2, kids: 2, keys: 2, lives: 0, hp: 0, reason: "time_up" },
    { at: ago(9 * DAY + 22 * HOUR), code: "WARM4", status: "completed", elapsedMs: clock(8, 4), score: 3600, level: 3, kids: 3, keys: 3, lives: 2, hp: 4 },
    { at: ago(9 * DAY + 20 * HOUR), code: "WARM4", status: "completed", elapsedMs: clock(7, 12), score: 4250, level: 3, kids: 3, keys: 3, lives: 4, hp: 6 },
    { at: ago(4 * DAY + 3 * HOUR), code: "NIGHT1", status: "failed", elapsedMs: clock(4, 33), score: 1650, level: 2, kids: 2, keys: 2, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(2 * DAY + 14 * HOUR), code: "NIGHT1", status: "failed", elapsedMs: clock(8, 40), score: 2480, level: 2, kids: 3, keys: 3, lives: 0, hp: 0, reason: "time_up" },
    { at: ago(26 * HOUR), code: "NIGHT1", status: "completed", elapsedMs: clock(7, 48), score: 4100, level: 3, kids: 3, keys: 3, lives: 3, hp: 5 },
  ],
  CocoaKing: [
    { at: ago(9 * DAY + 6 * HOUR), code: "WARM4", status: "failed", elapsedMs: clock(3, 5), score: 880, level: 1, kids: 1, keys: 1, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(9 * DAY + 5 * HOUR), code: "WARM4", status: "completed", elapsedMs: clock(8, 51), score: 3600, level: 3, kids: 3, keys: 3, lives: 2, hp: 3 },
    { at: ago(3 * DAY + 11 * HOUR), code: "NIGHT1", status: "failed", elapsedMs: clock(7, 20), score: 2100, level: 2, kids: 2, keys: 3, lives: 0, hp: 0, reason: "time_up" },
    { at: ago(3 * DAY + 9 * HOUR), code: "NIGHT1", status: "failed", elapsedMs: clock(5, 18), score: 1920, level: 2, kids: 2, keys: 2, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(25 * HOUR), code: "NIGHT1", status: "completed", elapsedMs: clock(8, 5), score: 3880, level: 3, kids: 3, keys: 3, lives: 2, hp: 4 },
  ],
  FrostByte: [
    { at: ago(8 * DAY), code: "NIGHT1", status: "failed", elapsedMs: clock(2, 40), score: 510, level: 1, kids: 0, keys: 1, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(7 * DAY + 4 * HOUR), code: "NIGHT1", status: "failed", elapsedMs: clock(4, 10), score: 940, level: 1, kids: 1, keys: 2, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(6 * DAY + 2 * HOUR), code: "NIGHT1", status: "failed", elapsedMs: clock(8, 2), score: 1760, level: 2, kids: 2, keys: 2, lives: 0, hp: 0, reason: "time_up" },
    { at: ago(4 * DAY + 16 * HOUR), code: "NIGHT1", status: "failed", elapsedMs: clock(6, 55), score: 2010, level: 2, kids: 2, keys: 3, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(2 * DAY + 3 * HOUR), code: "NIGHT1", status: "completed", elapsedMs: clock(10, 40), score: 2800, level: 3, kids: 3, keys: 3, lives: 1, hp: 2 },
    { at: ago(20 * HOUR), code: "NIGHT1", status: "failed", elapsedMs: clock(6, 40), score: 2100, level: 2, kids: 2, keys: 2, lives: 0, hp: 0, reason: "time_up" },
  ],
  NorthStar: [
    { at: ago(5 * DAY + 8 * HOUR), code: "NIGHT1", status: "failed", elapsedMs: clock(3, 22), score: 760, level: 1, kids: 1, keys: 1, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(4 * DAY + HOUR), code: "NIGHT1", status: "failed", elapsedMs: clock(7, 45), score: 1880, level: 2, kids: 2, keys: 2, lives: 0, hp: 0, reason: "time_up" },
    { at: ago(1 * DAY + 2 * HOUR), code: "NIGHT1", status: "completed", elapsedMs: clock(9, 28), score: 3120, level: 3, kids: 3, keys: 3, lives: 2, hp: 3 },
  ],
  SleighGirl: [
    { at: ago(8 * DAY + 3 * HOUR), code: "WARM4", status: "failed", elapsedMs: clock(5, 50), score: 1320, level: 2, kids: 1, keys: 2, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(8 * DAY + HOUR), code: "WARM4", status: "completed", elapsedMs: clock(10, 6), score: 2980, level: 3, kids: 3, keys: 2, lives: 1, hp: 3 },
    { at: ago(2 * DAY + 7 * HOUR), code: "GIFT2", status: "failed", elapsedMs: clock(4, 8), score: 1010, level: 1, kids: 1, keys: 1, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(2 * DAY + 6 * HOUR), code: "GIFT2", status: "failed", elapsedMs: clock(8, 14), score: 2210, level: 2, kids: 2, keys: 3, lives: 0, hp: 0, reason: "time_up" },
    { at: ago(18 * HOUR), code: "GIFT2", status: "completed", elapsedMs: clock(9, 18), score: 3200, level: 3, kids: 3, keys: 3, lives: 2, hp: 4 },
  ],
  GiftNinja: [
    { at: ago(11 * DAY), code: "WARM4", status: "failed", elapsedMs: clock(1, 12), score: 180, level: 1, kids: 0, keys: 0, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(10 * DAY + 20 * HOUR), code: "WARM4", status: "failed", elapsedMs: clock(2, 55), score: 640, level: 1, kids: 1, keys: 1, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(10 * DAY + 12 * HOUR), code: "WARM4", status: "failed", elapsedMs: clock(6, 30), score: 1540, level: 2, kids: 2, keys: 1, lives: 0, hp: 0, reason: "time_up" },
    { at: ago(9 * DAY + 18 * HOUR), code: "WARM4", status: "failed", elapsedMs: clock(7, 8), score: 1720, level: 2, kids: 2, keys: 2, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(3 * DAY + 4 * HOUR), code: "GIFT2", status: "failed", elapsedMs: clock(3, 44), score: 890, level: 1, kids: 1, keys: 2, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(2 * DAY + 15 * HOUR), code: "GIFT2", status: "failed", elapsedMs: clock(9, 40), score: 2340, level: 2, kids: 3, keys: 3, lives: 0, hp: 0, reason: "time_up" },
    { at: ago(2 * DAY + 13 * HOUR), code: "GIFT2", status: "failed", elapsedMs: clock(5, 2), score: 1470, level: 2, kids: 2, keys: 2, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(16 * HOUR), code: "GIFT2", status: "completed", elapsedMs: clock(11, 2), score: 3050, level: 3, kids: 3, keys: 3, lives: 1, hp: 2 },
  ],
  IvyLeap: [
    { at: ago(3 * DAY + 2 * HOUR), code: "GIFT2", status: "failed", elapsedMs: clock(1, 55), score: 310, level: 1, kids: 0, keys: 1, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(2 * DAY + 20 * HOUR), code: "GIFT2", status: "failed", elapsedMs: clock(4, 40), score: 860, level: 1, kids: 1, keys: 1, lives: 0, hp: 0, reason: "time_up" },
    { at: ago(15 * HOUR), code: "GIFT2", status: "failed", elapsedMs: clock(2, 11), score: 650, level: 1, kids: 1, keys: 1, lives: 0, hp: 0, reason: "game_over" },
  ],
  JingleBot: [
    { at: ago(20 * HOUR), code: "GIFT2", status: "failed", elapsedMs: clock(3, 18), score: 540, level: 1, kids: 0, keys: 1, lives: 0, hp: 0, reason: "game_over" },
  ],
  CoalMiner: [
    { at: ago(6 * DAY + 8 * HOUR), code: "COAL3", status: "failed", elapsedMs: clock(2, 6), score: 220, level: 1, kids: 0, keys: 0, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(5 * DAY + 12 * HOUR), code: "COAL3", status: "failed", elapsedMs: clock(3, 50), score: 610, level: 1, kids: 1, keys: 1, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(4 * DAY + 9 * HOUR), code: "COAL3", status: "failed", elapsedMs: clock(7, 15), score: 1490, level: 2, kids: 2, keys: 2, lives: 0, hp: 0, reason: "time_up" },
    { at: ago(3 * DAY + 1 * HOUR), code: "COAL3", status: "failed", elapsedMs: clock(4, 28), score: 980, level: 1, kids: 1, keys: 2, lives: 0, hp: 0, reason: "game_over" },
    { at: ago(1 * DAY + 10 * HOUR), code: "COAL3", status: "failed", elapsedMs: clock(8, 33), score: 1710, level: 2, kids: 2, keys: 2, lives: 0, hp: 0, reason: "time_up" },
    { at: ago(6 * HOUR), code: "COAL3", status: "failed", elapsedMs: clock(5, 47), score: 1240, level: 2, kids: 2, keys: 1, lives: 0, hp: 0, reason: "game_over" },
  ],
  QuietMouse: [],
};

const live = {
  NorthStar: {
    username: "NorthStar",
    code: "NIGHT1",
    status: "started",
    at: ago(4 * MIN + 20_000),
    elapsedMs: clock(4, 20),
    score: 1240,
    level: 2,
    kids: 2,
    keys: 2,
    lives: 2,
    hp: 4,
  },
  JingleBot: {
    username: "JingleBot",
    code: "GIFT2",
    status: "started",
    at: ago(82_000),
    elapsedMs: 82_000,
    score: 180,
    level: 1,
    kids: 0,
    keys: 1,
    lives: 3,
    hp: 6,
  },
};

const roster = [
  { username: "SnowMaster", createdAt: ago(8 * DAY), rooms: ["DEMO01"], history: demoHistory.SnowMaster },
  { username: "Rudolph", createdAt: ago(8 * DAY), rooms: ["DEMO01"], history: demoHistory.Rudolph },
  { username: "ElfRunner", createdAt: ago(8 * DAY), rooms: ["DEMO01"], history: demoHistory.ElfRunner },
  { username: "HollyDash", createdAt: ago(12 * DAY), rooms: ["WARM4", "NIGHT1"], history: extraHistory.HollyDash },
  { username: "CocoaKing", createdAt: ago(11 * DAY), rooms: ["WARM4", "NIGHT1"], history: extraHistory.CocoaKing },
  { username: "FrostByte", createdAt: ago(9 * DAY), rooms: ["NIGHT1"], history: extraHistory.FrostByte },
  { username: "NorthStar", createdAt: ago(6 * DAY), rooms: ["NIGHT1"], history: extraHistory.NorthStar },
  { username: "SleighGirl", createdAt: ago(8 * DAY + 6 * HOUR), rooms: ["WARM4", "GIFT2"], history: extraHistory.SleighGirl },
  { username: "GiftNinja", createdAt: ago(12 * DAY + 3 * HOUR), rooms: ["WARM4", "GIFT2"], history: extraHistory.GiftNinja },
  { username: "IvyLeap", createdAt: ago(4 * DAY), rooms: ["GIFT2"], history: extraHistory.IvyLeap },
  { username: "JingleBot", createdAt: ago(22 * HOUR), rooms: ["GIFT2"], history: extraHistory.JingleBot },
  { username: "CoalMiner", createdAt: ago(7 * DAY), rooms: ["COAL3"], history: extraHistory.CoalMiner },
  { username: "QuietMouse", createdAt: ago(2 * DAY + 4 * HOUR), rooms: ["COAL3"], history: extraHistory.QuietMouse },
];

function tagged(username, rows) {
  return rows.map((row) => ({ ...row, username }));
}

function latestIn(username, code, rows) {
  const matches = tagged(username, rows).filter((row) => row.code === code);
  return matches[matches.length - 1];
}

for (const player of roster) {
  const history = tagged(player.username, player.history).map((row) => historyOf(row, rooms[row.code]));
  await upsertUser(player.username, player.createdAt, player.rooms, history);
}

await upsertRoom("DEMO01", rooms.DEMO01, "SnowMaster", ago(8 * DAY - HOUR), ["SnowMaster", "Rudolph", "ElfRunner"], {
  SnowMaster: latestIn("SnowMaster", "DEMO01", demoHistory.SnowMaster),
  Rudolph: latestIn("Rudolph", "DEMO01", demoHistory.Rudolph),
  ElfRunner: latestIn("ElfRunner", "DEMO01", demoHistory.ElfRunner),
});

await upsertRoom("WARM4", rooms.WARM4, "HollyDash", ago(11 * DAY), ["HollyDash", "CocoaKing", "SleighGirl", "GiftNinja"], {
  HollyDash: latestIn("HollyDash", "WARM4", extraHistory.HollyDash),
  CocoaKing: latestIn("CocoaKing", "WARM4", extraHistory.CocoaKing),
  SleighGirl: latestIn("SleighGirl", "WARM4", extraHistory.SleighGirl),
  GiftNinja: latestIn("GiftNinja", "WARM4", extraHistory.GiftNinja),
});

await upsertRoom("NIGHT1", rooms.NIGHT1, "HollyDash", ago(4 * DAY + 6 * HOUR), ["HollyDash", "CocoaKing", "FrostByte", "NorthStar"], {
  HollyDash: latestIn("HollyDash", "NIGHT1", extraHistory.HollyDash),
  CocoaKing: latestIn("CocoaKing", "NIGHT1", extraHistory.CocoaKing),
  FrostByte: latestIn("FrostByte", "NIGHT1", extraHistory.FrostByte),
  NorthStar: live.NorthStar,
});

await upsertRoom("GIFT2", rooms.GIFT2, "SleighGirl", ago(3 * DAY), ["SleighGirl", "GiftNinja", "IvyLeap", "JingleBot"], {
  SleighGirl: latestIn("SleighGirl", "GIFT2", extraHistory.SleighGirl),
  GiftNinja: latestIn("GiftNinja", "GIFT2", extraHistory.GiftNinja),
  IvyLeap: latestIn("IvyLeap", "GIFT2", extraHistory.IvyLeap),
  JingleBot: live.JingleBot,
});

await upsertRoom("COAL3", rooms.COAL3, "CoalMiner", ago(6 * DAY + 10 * HOUR), ["CoalMiner", "QuietMouse"], {
  CoalMiner: latestIn("CoalMiner", "COAL3", extraHistory.CoalMiner),
});

console.log("Seeded rooms DEMO01, WARM4, NIGHT1, GIFT2, COAL3");
console.log("Password for every seeded account: DemoPass1");
console.log("Original: SnowMaster, Rudolph, ElfRunner");
console.log("Added:    HollyDash, CocoaKing, FrostByte, NorthStar, SleighGirl, GiftNinja, IvyLeap, JingleBot, CoalMiner, QuietMouse");
console.log("Live runs: NorthStar in NIGHT1, JingleBot in GIFT2");

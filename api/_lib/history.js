import { redis, roomKey, userHistoryKey } from "./redis.js";

const MAX_ENTRIES = 200;

// runs/start.js resets the room result on a replay, so this append-only log is
// the only place a player's earlier attempts survive.
export async function recordRun(usernameKey, code, fields) {
  const db = redis();
  const entry = {
    at: fields.finishedAt || new Date().toISOString(),
    code,
    room: (await db.hget(roomKey(code), "name")) || "",
    status: fields.status,
    score: Number(fields.score || 0),
    levelReached: Number(fields.levelReached || 0),
    elapsedMs: Number(fields.elapsedMs || 0),
    kids: Number(fields.kids || 0),
    keys: Number(fields.keys || 0),
    lives: Number(fields.lives || 0),
    hp: Number(fields.hp || 0),
    failureReason: fields.failureReason || "",
    levelsCleared: fields.levelsCleared || "",
  };
  await db.lpush(userHistoryKey(usernameKey), JSON.stringify(entry));
  await db.ltrim(userHistoryKey(usernameKey), 0, MAX_ENTRIES - 1);
  return entry;
}

export async function readHistory(usernameKey, limit = MAX_ENTRIES) {
  const raw = (await redis().lrange(userHistoryKey(usernameKey), 0, limit - 1)) || [];
  return raw.map(decode).filter(Boolean);
}

// Upstash deserializes stored JSON for us; FileRedis hands back the raw string.
function decode(value) {
  if (value && typeof value === "object") return value;
  try {
    return JSON.parse(String(value));
  } catch {
    return null;
  }
}

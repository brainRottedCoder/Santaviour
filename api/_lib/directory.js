import {
  redis,
  ROOMS_INDEX,
  USERS_INDEX,
  userKey,
  userRoomsKey,
} from "./redis.js";
import { rankAccounts, sumWinnings } from "./rank.js";
import { loadRoom } from "./rooms.js";
import { readHistory } from "./history.js";

const USER_PREFIX = "user:";
const ROOM_PREFIX = "room:";
const OVERVIEW_HISTORY = 50;

// The index sets only exist for records created after they were introduced, so
// fall back to a key scan and heal the index with anything it was missing.
async function listIds(indexKey, prefix) {
  const db = redis();
  const known = (await db.smembers(indexKey)) || [];
  const found = new Set(known);
  try {
    for (const raw of (await db.keys(`${prefix}*`)) || []) {
      const id = String(raw).slice(prefix.length);
      if (id && !id.includes(":")) found.add(id);
    }
  } catch {
    /* store has no keys(): the index is all we have */
  }
  for (const id of found) {
    if (!known.includes(id)) await db.sadd(indexKey, id);
  }
  return [...found].sort();
}

export function listUserKeys() {
  return listIds(USERS_INDEX, USER_PREFIX);
}

export function listRoomCodes() {
  return listIds(ROOMS_INDEX, ROOM_PREFIX);
}

export async function loadAccount(usernameKey) {
  const db = redis();
  const user = await db.hgetall(userKey(usernameKey));
  if (!user || !user.username) return null;
  return {
    usernameKey,
    username: user.username,
    createdAt: user.createdAt || "",
    runs: Number(user.runs || 0),
    completions: Number(user.completions || 0),
    bestMs: user.bestMs ? Number(user.bestMs) : null,
    bestScore: Number(user.bestScore || 0),
    rooms: (await db.smembers(userRoomsKey(usernameKey))) || [],
  };
}

async function loadRooms(codes) {
  const rooms = [];
  for (const code of codes) {
    try {
      rooms.push(await loadRoom(code));
    } catch {
      /* room hash was removed but the code lingers somewhere */
    }
  }
  return rooms;
}

function placementsByUser(rooms) {
  const map = new Map();
  for (const room of rooms) {
    for (const row of room.leaderboard || []) {
      const key = String(row.username).toLowerCase();
      if (!map.has(key)) map.set(key, []);
      map.get(key).push({
        code: room.code,
        room: room.name,
        owner: room.owner,
        players: (room.members || []).length,
        ...row,
      });
    }
  }
  return map;
}

function timeOf(value) {
  if (value == null || value === "") return 0;
  if (typeof value === "number") return value;
  const parsed = Date.parse(value);
  return Number.isFinite(parsed) ? parsed : 0;
}

function lastActive(placements, history) {
  const stamps = [
    ...placements.map((row) => Math.max(timeOf(row.finishedAt), timeOf(row.startedAt))),
    ...history.map((row) => timeOf(row.at)),
  ];
  const latest = Math.max(0, ...stamps);
  return latest ? new Date(latest).toISOString() : "";
}

export async function buildOverview() {
  const [userKeys, roomCodes] = await Promise.all([listUserKeys(), listRoomCodes()]);
  const rooms = await loadRooms(roomCodes);
  const placements = placementsByUser(rooms);

  const accounts = [];
  for (const key of userKeys) {
    const account = await loadAccount(key);
    if (!account) continue;
    const mine = placements.get(key) || [];
    const history = await readHistory(key, OVERVIEW_HISTORY);
    accounts.push({
      ...account,
      placements: mine,
      history,
      historyCount: history.length,
      roomsPlayed: mine.length,
      activeRuns: mine.filter((row) => row.status === "started").length,
      lastActiveAt: lastActive(mine, history),
      winnings: sumWinnings(mine),
    });
  }

  const users = rankAccounts(accounts);
  return {
    generatedAt: new Date().toISOString(),
    serverNow: Date.now(),
    totals: {
      users: users.length,
      rooms: rooms.length,
      runs: users.reduce((sum, row) => sum + row.runs, 0),
      completions: users.reduce((sum, row) => sum + row.completions, 0),
      activeRuns: users.reduce((sum, row) => sum + row.activeRuns, 0),
      winnings: users.reduce((sum, row) => sum + Number(row.winnings || 0), 0),
    },
    users,
    rooms,
  };
}

export async function buildUserDetail(usernameKey) {
  const account = await loadAccount(usernameKey);
  if (!account) {
    throw Object.assign(new Error("Player not found"), { status: 404 });
  }
  const rooms = await loadRooms(await listRoomCodes());
  const mine = placementsByUser(rooms).get(usernameKey) || [];
  const history = await readHistory(usernameKey);
  const ranked = rankAccounts(
    (await Promise.all((await listUserKeys()).map(loadAccount))).filter(Boolean)
  );
  return {
    ...account,
    globalRank: ranked.find((row) => row.usernameKey === usernameKey)?.globalRank ?? null,
    placements: mine,
    history,
    roomsPlayed: mine.length,
    lastActiveAt: lastActive(mine, history),
    winnings: sumWinnings(mine),
    serverNow: Date.now(),
  };
}

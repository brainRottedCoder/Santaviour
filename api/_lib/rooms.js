import { redis, roomKey, roomMembersKey, roomResultKey, userKey } from "./redis.js";
import { rankPlayers } from "./rank.js";

export async function assertMember(code, usernameKey) {
  const ok = await redis().sismember(roomMembersKey(code), usernameKey);
  if (!ok) {
    throw Object.assign(new Error("You are not in this room"), { status: 403 });
  }
}

export async function loadRoom(code) {
  const db = redis();
  const room = await db.hgetall(roomKey(code));
  if (!room || !room.code) {
    throw Object.assign(new Error("Room not found"), { status: 404 });
  }
  const memberKeys = await db.smembers(roomMembersKey(code));
  const members = [];
  const resultsByKey = {};
  for (const key of memberKeys) {
    const account = await db.hgetall(userKey(key));
    const result = await db.hgetall(roomResultKey(code, key));
    const username = account?.username || result?.username || key;
    members.push(username);
    if (result && result.status) {
      resultsByKey[String(key).toLowerCase()] = result;
    }
  }
  const serverNow = Date.now();
  return {
    code: room.code,
    name: room.name,
    owner: room.owner,
    status: room.status || "waiting",
    createdAt: room.createdAt,
    members,
    serverNow,
    leaderboard: rankPlayers(members, resultsByKey, serverNow),
  };
}

export async function winningsForMember(username, codes) {
  let total = 0;
  for (const code of codes || []) {
    try {
      const room = await loadRoom(code);
      const mine = (room.leaderboard || []).find((row) => row.username === username);
      total += Number(mine?.prizeRupees || 0);
    } catch {
      /* room gone */
    }
  }
  return total;
}

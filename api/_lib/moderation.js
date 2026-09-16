import {
  redis,
  roomKey,
  roomMembersKey,
  roomResultKey,
  ROOMS_INDEX,
  USERS_INDEX,
  userHistoryKey,
  userKey,
  userRoomsKey,
} from "./redis.js";
import { listRoomCodes } from "./directory.js";

function notFound(message) {
  return Object.assign(new Error(message), { status: 404 });
}

export async function deleteUser(usernameKey) {
  const db = redis();
  const account = await db.hgetall(userKey(usernameKey));
  if (!account || !account.username) throw notFound("Player not found");

  // Sweep every room, not just the player's own room set, so a stale
  // membership can never leave an orphaned result behind.
  const cleared = [];
  for (const code of await listRoomCodes()) {
    const member = await db.sismember(roomMembersKey(code), usernameKey);
    const result = await db.hget(roomResultKey(code, usernameKey), "status");
    if (!member && !result) continue;
    await db.srem(roomMembersKey(code), usernameKey);
    await db.del(roomResultKey(code, usernameKey));
    cleared.push(code);
  }
  await db.del(userKey(usernameKey), userRoomsKey(usernameKey), userHistoryKey(usernameKey));
  await db.srem(USERS_INDEX, usernameKey);
  return { username: account.username, roomsCleared: cleared };
}

export async function deleteRoom(code) {
  const db = redis();
  const room = await db.hgetall(roomKey(code));
  if (!room || !room.code) throw notFound("Room not found");

  const members = (await db.smembers(roomMembersKey(code))) || [];
  for (const member of members) {
    await db.del(roomResultKey(code, member));
    await db.srem(userRoomsKey(member), code);
  }
  await db.del(roomKey(code), roomMembersKey(code));
  await db.srem(ROOMS_INDEX, code);
  return { code, membersRemoved: members.length };
}

// Wipes the scoreboard but keeps the room and its players, so everyone can
// race again on the same code. Lifetime account stats are left untouched.
export async function resetRoom(code) {
  const db = redis();
  const room = await db.hgetall(roomKey(code));
  if (!room || !room.code) throw notFound("Room not found");

  const members = (await db.smembers(roomMembersKey(code))) || [];
  for (const member of members) {
    await db.del(roomResultKey(code, member));
  }
  await db.hset(roomKey(code), { status: "waiting" });
  return { code, runsCleared: members.length };
}

export async function clearRun(code, usernameKey) {
  const db = redis();
  const result = await db.hgetall(roomResultKey(code, usernameKey));
  if (!result || !result.status) throw notFound("No run recorded for that player");
  await db.del(roomResultKey(code, usernameKey));
  return { code, username: result.username || usernameKey };
}

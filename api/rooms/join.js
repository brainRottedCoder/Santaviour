import { allowMethods, readJson, send, sendError } from "../_lib/http.js";
import { redis, roomKey, roomMembersKey, userRoomsKey, withHandler } from "../_lib/redis.js";
import { requireUser } from "../_lib/session.js";
import { assertRoomCode } from "../_lib/validate.js";
import { loadRoom } from "../_lib/rooms.js";

export default async function handler(req, res) {
  await withHandler(res, async () => {
    if (!allowMethods(req, res, ["POST"])) return;
    const user = await requireUser(req, res);
    if (!user) return;
    const body = await readJson(req);
    const code = assertRoomCode(body.code);
    const db = redis();
    const room = await db.hgetall(roomKey(code));
    if (!room || !room.code) {
      sendError(res, 404, "Room not found");
      return;
    }
    const already = await db.sismember(roomMembersKey(code), user.key);
    if (!already) {
      const count = await db.scard(roomMembersKey(code));
      if (count >= 4) {
        sendError(res, 409, "Room is full (4 players)");
        return;
      }
      await db.sadd(roomMembersKey(code), user.key);
    }
    await db.sadd(userRoomsKey(user.key), code);
    send(res, 200, { ok: true, room: await loadRoom(code) });
  });
}

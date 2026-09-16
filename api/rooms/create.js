import { randomBytes } from "node:crypto";
import { allowMethods, readJson, send } from "../_lib/http.js";
import { redis, roomKey, roomMembersKey, ROOMS_INDEX, userRoomsKey, withHandler } from "../_lib/redis.js";
import { requireUser } from "../_lib/session.js";
import { assertRoomName } from "../_lib/validate.js";
import { loadRoom } from "../_lib/rooms.js";

function makeCode() {
  const alphabet = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
  const bytes = randomBytes(6);
  let code = "";
  for (const b of bytes) code += alphabet[b % alphabet.length];
  return code;
}

export default async function handler(req, res) {
  await withHandler(res, async () => {
    if (!allowMethods(req, res, ["POST"])) return;
    const user = await requireUser(req, res);
    if (!user) return;
    const body = await readJson(req);
    const name = assertRoomName(body.name);
    const db = redis();
    let code = makeCode();
    for (let i = 0; i < 5; i += 1) {
      const exists = await db.hget(roomKey(code), "code");
      if (!exists) break;
      code = makeCode();
    }
    await db.hset(roomKey(code), {
      code,
      name,
      owner: user.username,
      status: "waiting",
      createdAt: new Date().toISOString(),
    });
    await db.sadd(ROOMS_INDEX, code);
    await db.sadd(roomMembersKey(code), user.key);
    await db.sadd(userRoomsKey(user.key), code);
    const room = await loadRoom(code);
    send(res, 201, { ok: true, room });
  });
}

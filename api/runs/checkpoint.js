import { allowMethods, readJson, send } from "../_lib/http.js";
import { redis, roomResultKey, withHandler } from "../_lib/redis.js";
import { requireUser } from "../_lib/session.js";
import { assertRoomCode } from "../_lib/validate.js";
import { runFields } from "../_lib/results.js";
import { assertMember, loadRoom } from "../_lib/rooms.js";

export default async function handler(req, res) {
  await withHandler(res, async () => {
    if (!allowMethods(req, res, ["POST"])) return;
    const user = await requireUser(req, res);
    if (!user) return;
    const body = await readJson(req);
    const code = assertRoomCode(body.code);
    await assertMember(code, user.key);
    const db = redis();
    const current = (await db.hgetall(roomResultKey(code, user.key))) || {};
    if (current.status === "completed" || current.status === "failed") {
      send(res, 200, { ok: true, room: await loadRoom(code) });
      return;
    }
    await db.hset(
      roomResultKey(code, user.key),
      runFields(user, current, { ...body, clearLevel: body.level }, { status: current.status || "started" })
    );
    send(res, 200, { ok: true, room: await loadRoom(code) });
  });
}

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
    const current = (await redis().hgetall(roomResultKey(code, user.key))) || {};
    // Any status other than "started" begins a fresh attempt: earlier results
    // stay in the player's history log and lifetime stats, so replaying after
    // a win or a loss always works (the overlay's "Play again" button).
    const reset = current.status !== "started";
    await redis().hset(
      roomResultKey(code, user.key),
      runFields(user, current, { ...body, level: body.level || 1, score: reset ? 0 : body.score }, {
        status: "started",
        reset,
      })
    );
    send(res, 200, { ok: true, room: await loadRoom(code) });
  });
}

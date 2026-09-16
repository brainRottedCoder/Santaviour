import { allowMethods, readJson, send, sendError } from "../_lib/http.js";
import { redis, roomResultKey, userKey, withHandler } from "../_lib/redis.js";
import { requireUser } from "../_lib/session.js";
import { assertRoomCode } from "../_lib/validate.js";
import { parseCleared, runFields } from "../_lib/results.js";
import { assertMember, loadRoom } from "../_lib/rooms.js";
import { recordRun } from "../_lib/history.js";

export default async function handler(req, res) {
  await withHandler(res, async () => {
    if (!allowMethods(req, res, ["POST"])) return;
    const user = await requireUser(req, res);
    if (!user) return;
    const body = await readJson(req);
    const code = assertRoomCode(body.code);
    await assertMember(code, user.key);
    const outcome = body.status === "failed" ? "failed" : body.status === "completed" ? "completed" : null;
    if (!outcome) {
      sendError(res, 400, "status must be completed or failed");
      return;
    }
    const db = redis();
    const current = (await db.hgetall(roomResultKey(code, user.key))) || {};
    if (current.status === "completed" || current.status === "failed") {
      send(res, 200, { ok: true, room: await loadRoom(code) });
      return;
    }
    const fields = runFields(user, current, body, {
      status: outcome,
      finished: true,
      detail: body.detail,
    });
    const cleared = parseCleared(fields.levelsCleared);
    const beatGame = Number(fields.levelReached) >= 3 || (cleared.has(1) && cleared.has(2));
    if (outcome === "completed" && !beatGame) {
      sendError(res, 400, "Complete levels 1 and 2 before recording a win");
      return;
    }
    await db.hset(roomResultKey(code, user.key), fields);
    await recordRun(user.key, code, fields);

    const stats = await db.hgetall(userKey(user.key));
    const elapsedMs = Number(fields.elapsedMs || 0);
    const score = Number(fields.score || 0);
    const runs = Number(stats?.runs || 0) + (current.status === "started" || !current.status ? 1 : 0);
    const completions = Number(stats?.completions || 0) + (outcome === "completed" ? 1 : 0);
    const prevBest = stats?.bestMs ? Number(stats.bestMs) : null;
    const bestMs =
      outcome === "completed" && (prevBest == null || elapsedMs < prevBest) ? elapsedMs : prevBest;
    const bestScore = Math.max(Number(stats?.bestScore || 0), score);
    await db.hset(userKey(user.key), {
      runs,
      completions,
      bestMs: bestMs == null ? "" : String(bestMs),
      bestScore,
    });

    send(res, 200, { ok: true, room: await loadRoom(code) });
  });
}

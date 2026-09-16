import { allowMethods, send } from "../_lib/http.js";
import { withHandler } from "../_lib/redis.js";
import { requireUser } from "../_lib/session.js";
import { assertRoomCode } from "../_lib/validate.js";
import { assertMember, loadRoom } from "../_lib/rooms.js";

export default async function handler(req, res) {
  await withHandler(res, async () => {
    if (!allowMethods(req, res, ["GET"])) return;
    const user = await requireUser(req, res);
    if (!user) return;
    // req.query.code on Vercel; fall back to the URL path for the local dev server.
    const fromQuery = req.query?.code;
    const fromPath = String(req.url || "").split("?")[0].split("/").filter(Boolean).pop();
    const code = assertRoomCode(fromQuery || fromPath);
    await assertMember(code, user.key);
    send(res, 200, { ok: true, room: await loadRoom(code) });
  });
}

import { allowMethods, send } from "../_lib/http.js";
import { withHandler } from "../_lib/redis.js";
import { dropSession } from "../_lib/session.js";

export default async function handler(req, res) {
  await withHandler(res, async () => {
    if (!allowMethods(req, res, ["POST"])) return;
    dropSession(req, res);
    send(res, 200, { ok: true });
  });
}

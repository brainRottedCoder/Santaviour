import { allowMethods, send } from "../_lib/http.js";
import { redis, userRoomsKey, withHandler } from "../_lib/redis.js";
import { requireUser } from "../_lib/session.js";
import { winningsForMember } from "../_lib/rooms.js";

export default async function handler(req, res) {
  await withHandler(res, async () => {
    if (!allowMethods(req, res, ["GET"])) return;
    const user = await requireUser(req, res);
    if (!user) return;
    const rooms = await redis().smembers(userRoomsKey(user.key));
    const winnings = await winningsForMember(user.username, rooms);
    send(res, 200, { ok: true, user: { ...user.stats, winnings }, rooms: rooms || [] });
  });
}

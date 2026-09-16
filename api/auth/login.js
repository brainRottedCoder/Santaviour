import bcrypt from "bcryptjs";
import { allowMethods, readJson, send, sendError } from "../_lib/http.js";
import { redis, userKey, userRoomsKey, withHandler } from "../_lib/redis.js";
import { attachSession, publicStats } from "../_lib/session.js";
import { assertPassword, assertUsername, usernameKey } from "../_lib/validate.js";
import { winningsForMember } from "../_lib/rooms.js";

export default async function handler(req, res) {
  await withHandler(res, async () => {
    if (!allowMethods(req, res, ["POST"])) return;
    const body = await readJson(req);
    const username = assertUsername(body.username);
    const password = assertPassword(body.password);
    const user = await redis().hgetall(userKey(usernameKey(username)));
    if (!user || !user.passwordHash) {
      sendError(res, 401, "Invalid username or password");
      return;
    }
    const match = await bcrypt.compare(password, user.passwordHash);
    if (!match) {
      sendError(res, 401, "Invalid username or password");
      return;
    }
    await attachSession(req, res, user.username);
    const rooms = await redis().smembers(userRoomsKey(usernameKey(user.username)));
    const winnings = await winningsForMember(user.username, rooms);
    send(res, 200, { ok: true, user: { ...publicStats(user), winnings } });
  });
}

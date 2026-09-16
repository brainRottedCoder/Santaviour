import bcrypt from "bcryptjs";
import { allowMethods, readJson, send, sendError } from "../_lib/http.js";
import { redis, USERS_INDEX, userKey, withHandler } from "../_lib/redis.js";
import { attachSession, publicStats } from "../_lib/session.js";
import { assertPassword, assertUsername, usernameKey } from "../_lib/validate.js";

export default async function handler(req, res) {
  await withHandler(res, async () => {
    if (!allowMethods(req, res, ["POST"])) return;
    const body = await readJson(req);
    const username = assertUsername(body.username);
    const password = assertPassword(body.password);
    const key = usernameKey(username);
    const db = redis();
    const exists = await db.hget(userKey(key), "username");
    if (exists) {
      sendError(res, 409, "That username is taken");
      return;
    }
    const passwordHash = await bcrypt.hash(password, 10);
    await db.hset(userKey(key), {
      username,
      passwordHash,
      createdAt: new Date().toISOString(),
      runs: 0,
      completions: 0,
      bestMs: "",
      bestScore: 0,
    });
    await db.sadd(USERS_INDEX, key);
    await attachSession(req, res, username);
    send(res, 201, { ok: true, user: publicStats({ username, runs: 0, completions: 0, bestScore: 0, winnings: 0 }) });
  });
}

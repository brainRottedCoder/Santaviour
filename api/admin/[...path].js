import { allowMethods, method, readJson, send, sendError } from "../_lib/http.js";
import { withHandler } from "../_lib/redis.js";
import {
  adminConfigured,
  assertLoginAllowed,
  attachAdminSession,
  clearFailedLogins,
  dropAdminSession,
  passwordMatches,
  readAdminSession,
  recordFailedLogin,
  requireAdmin,
} from "../_lib/admin.js";
import { buildOverview, buildUserDetail } from "../_lib/directory.js";
import { clearRun, deleteRoom, deleteUser, resetRoom } from "../_lib/moderation.js";
import { assertRoomCode, assertUsername, usernameKey } from "../_lib/validate.js";

// Every admin route lives in this one catch-all so the deployment stays inside
// Vercel's 12-function limit. Nested /users/:name paths 404 on `vercel dev
// --local`, so player/room actions also accept a single segment + query.
function requestUrl(req) {
  try {
    return new URL(req.url || "/", "http://localhost");
  } catch {
    return new URL("http://localhost/");
  }
}

function segments(req) {
  const parts = requestUrl(req).pathname.split("/").filter(Boolean);
  const at = parts.indexOf("admin");
  return at === -1 ? [] : parts.slice(at + 1).map((part) => decodeURIComponent(part));
}

function param(req, name) {
  return requestUrl(req).searchParams.get(name) || "";
}

export default async function handler(req, res) {
  await withHandler(res, async () => {
    const path = segments(req);
    const route = path.join("/");
    const body = ["POST", "DELETE"].includes(method(req)) ? await readJson(req).catch(() => ({})) : {};

    if (route === "session") {
      if (!allowMethods(req, res, ["GET"])) return;
      const session = await readAdminSession(req);
      send(res, 200, { ok: true, admin: Boolean(session), configured: adminConfigured() });
      return;
    }

    if (route === "login") {
      if (!allowMethods(req, res, ["POST"])) return;
      await assertLoginAllowed(req);
      if (!passwordMatches(body.password)) {
        await recordFailedLogin(req);
        sendError(res, 401, "Wrong admin password");
        return;
      }
      await clearFailedLogins(req);
      await attachAdminSession(req, res);
      send(res, 200, { ok: true, admin: true });
      return;
    }

    if (route === "logout") {
      if (!allowMethods(req, res, ["POST"])) return;
      dropAdminSession(req, res);
      send(res, 200, { ok: true });
      return;
    }

    if (!(await requireAdmin(req, res))) return;

    if (route === "overview") {
      if (!allowMethods(req, res, ["GET"])) return;
      send(res, 200, { ok: true, ...(await buildOverview()) });
      return;
    }

    const userName = path[0] === "users" && path[1] ? path[1] : param(req, "name") || body.name;
    const roomCode = path[0] === "rooms" && path[1] ? path[1] : param(req, "code") || body.code;

    if (route === "user" || (path[0] === "users" && path.length === 2)) {
      if (!allowMethods(req, res, ["GET", "DELETE"])) return;
      const key = usernameKey(assertUsername(userName));
      if (method(req) === "DELETE") {
        send(res, 200, { ok: true, deleted: await deleteUser(key) });
        return;
      }
      send(res, 200, { ok: true, user: await buildUserDetail(key) });
      return;
    }

    if (route === "room" || (path[0] === "rooms" && path.length === 2)) {
      if (!allowMethods(req, res, ["DELETE"])) return;
      send(res, 200, { ok: true, deleted: await deleteRoom(assertRoomCode(roomCode)) });
      return;
    }

    if (route === "reset" || (path[0] === "rooms" && path[2] === "reset")) {
      if (!allowMethods(req, res, ["POST"])) return;
      send(res, 200, { ok: true, reset: await resetRoom(assertRoomCode(roomCode)) });
      return;
    }

    if (route === "run" || (path[0] === "rooms" && path[2] === "results")) {
      if (!allowMethods(req, res, ["DELETE"])) return;
      send(res, 200, {
        ok: true,
        cleared: await clearRun(assertRoomCode(roomCode), usernameKey(assertUsername(userName))),
      });
      return;
    }

    sendError(res, 404, "Unknown admin route");
  });
}

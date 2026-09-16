import "./env.js";
import { SignJWT, jwtVerify } from "jose";
import { getCookie, setCookie, clearCookie, isSecureRequest, sendError } from "./http.js";
import { redis, userKey } from "./redis.js";
import { usernameKey } from "./validate.js";

export const COOKIE = "santa_session";

function secretKey() {
  const secret = process.env.SESSION_SECRET;
  if (!secret || secret.length < 16) {
    throw Object.assign(new Error("SESSION_SECRET is not configured"), { status: 503 });
  }
  return new TextEncoder().encode(secret);
}

export async function signSession(username) {
  return new SignJWT({ u: username })
    .setProtectedHeader({ alg: "HS256" })
    .setIssuedAt()
    .setExpirationTime("7d")
    .sign(secretKey());
}

export async function readSession(req) {
  const token = getCookie(req, COOKIE);
  if (!token) return null;
  try {
    const { payload } = await jwtVerify(token, secretKey());
    const username = String(payload.u || "");
    if (!username) return null;
    return { username, key: usernameKey(username) };
  } catch {
    return null;
  }
}

export async function requireUser(req, res) {
  const session = await readSession(req);
  if (!session) {
    sendError(res, 401, "Sign in required");
    return null;
  }
  const user = await redis().hgetall(userKey(session.key));
  if (!user || !user.username) {
    sendError(res, 401, "Account not found");
    return null;
  }
  return { ...session, username: user.username, stats: publicStats(user) };
}

export function publicStats(user) {
  return {
    username: user.username,
    runs: Number(user.runs || 0),
    completions: Number(user.completions || 0),
    bestMs: user.bestMs ? Number(user.bestMs) : null,
    bestScore: Number(user.bestScore || 0),
    winnings: Number(user.winnings || 0),
  };
}

export async function attachSession(req, res, username) {
  const token = await signSession(username);
  setCookie(res, COOKIE, token, { secure: isSecureRequest(req) });
}

export function dropSession(req, res) {
  clearCookie(res, COOKIE, { secure: isSecureRequest(req) });
}

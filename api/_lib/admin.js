import "./env.js";
import { createHash, timingSafeEqual } from "node:crypto";
import { SignJWT, jwtVerify } from "jose";
import { clearCookie, getCookie, isSecureRequest, sendError, setCookie } from "./http.js";
import { redis } from "./redis.js";

export const ADMIN_COOKIE = "santa_admin";

const TTL_SECONDS = 60 * 60 * 8;
const WINDOW_MS = 10 * 60 * 1000;
const MAX_ATTEMPTS = 8;

function secretKey() {
  const secret = process.env.SESSION_SECRET;
  if (!secret || secret.length < 16) {
    throw Object.assign(new Error("SESSION_SECRET is not configured"), { status: 503 });
  }
  return new TextEncoder().encode(secret);
}

export function adminConfigured() {
  return String(process.env.ADMIN_PASSWORD || "").length >= 8;
}

function adminPassword() {
  if (!adminConfigured()) {
    throw Object.assign(
      new Error("Admin panel is not configured. Set ADMIN_PASSWORD to at least 8 characters."),
      { status: 503 }
    );
  }
  return String(process.env.ADMIN_PASSWORD);
}

// Hash both sides so the comparison is fixed-length and does not leak the
// expected password length through an early return.
export function passwordMatches(candidate) {
  const digest = (value) => createHash("sha256").update(String(value ?? ""), "utf8").digest();
  return timingSafeEqual(digest(adminPassword()), digest(candidate));
}

export async function attachAdminSession(req, res) {
  const token = await new SignJWT({ r: "admin" })
    .setProtectedHeader({ alg: "HS256" })
    .setIssuedAt()
    .setExpirationTime(`${TTL_SECONDS}s`)
    .sign(secretKey());
  setCookie(res, ADMIN_COOKIE, token, {
    maxAge: TTL_SECONDS,
    secure: isSecureRequest(req),
  });
}

export function dropAdminSession(req, res) {
  clearCookie(res, ADMIN_COOKIE, { secure: isSecureRequest(req) });
}

export async function readAdminSession(req) {
  const token = getCookie(req, ADMIN_COOKIE);
  if (!token) return null;
  try {
    const { payload } = await jwtVerify(token, secretKey());
    // A player session token is signed with the same secret but carries `u`,
    // never `r`, so it can never be replayed as an admin token.
    return payload.r === "admin" ? { role: "admin" } : null;
  } catch {
    return null;
  }
}

export async function requireAdmin(req, res) {
  const session = await readAdminSession(req);
  if (!session) {
    sendError(res, 401, "Admin sign-in required");
    return null;
  }
  return session;
}

function throttleKey(req) {
  const forwarded = req.headers["x-forwarded-for"];
  const ip =
    (typeof forwarded === "string" ? forwarded.split(",")[0].trim() : "") ||
    req.socket?.remoteAddress ||
    "unknown";
  return `admin:login:${ip}`;
}

export async function assertLoginAllowed(req) {
  const record = await redis().hgetall(throttleKey(req));
  const startedAt = Number(record?.startedAt || 0);
  const count = Number(record?.count || 0);
  if (Date.now() - startedAt < WINDOW_MS && count >= MAX_ATTEMPTS) {
    const minutes = Math.max(1, Math.ceil((WINDOW_MS - (Date.now() - startedAt)) / 60000));
    throw Object.assign(new Error(`Too many attempts. Try again in ${minutes} min.`), {
      status: 429,
    });
  }
}

export async function recordFailedLogin(req) {
  const key = throttleKey(req);
  const record = await redis().hgetall(key);
  const startedAt = Number(record?.startedAt || 0);
  const fresh = Date.now() - startedAt >= WINDOW_MS;
  await redis().hset(key, {
    startedAt: fresh ? Date.now() : startedAt,
    count: fresh ? 1 : Number(record?.count || 0) + 1,
  });
}

export async function clearFailedLogins(req) {
  await redis().del(throttleKey(req));
}

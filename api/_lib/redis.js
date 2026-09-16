import "./env.js";
import { Redis } from "@upstash/redis";
import { FileRedis } from "./file-redis.js";

let client;

export function redis() {
  if (client) return client;
  const url = process.env.UPSTASH_REDIS_REST_URL;
  const token = process.env.UPSTASH_REDIS_REST_TOKEN;
  if (url && token) {
    client = new Redis({ url, token });
    return client;
  }
  const hosted = process.env.VERCEL_ENV === "production" || process.env.VERCEL_ENV === "preview";
  if (hosted) {
    throw Object.assign(new Error("Leaderboard storage is not configured"), { status: 503 });
  }
  client = new FileRedis();
  return client;
}

export const USERS_INDEX = "index:users";
export const ROOMS_INDEX = "index:rooms";

export function userKey(usernameKey) {
  return `user:${usernameKey}`;
}

export function userRoomsKey(usernameKey) {
  return `user:${usernameKey}:rooms`;
}

export function userHistoryKey(usernameKey) {
  return `user:${usernameKey}:history`;
}

export function roomKey(code) {
  return `room:${code}`;
}

export function roomMembersKey(code) {
  return `room:${code}:members`;
}

export function roomResultKey(code, usernameKey) {
  return `room:${code}:result:${usernameKey}`;
}

export async function withHandler(res, fn) {
  try {
    await fn();
  } catch (err) {
    const status = err.status || (err.name === "SyntaxError" ? 400 : 500);
    const error = err.status ? err.message : "Server error";
    res.statusCode = status;
    res.setHeader("Content-Type", "application/json; charset=utf-8");
    res.end(JSON.stringify({ ok: false, error }));
  }
}

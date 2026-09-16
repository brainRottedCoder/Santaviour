import { clampLevel, clampScore } from "./validate.js";

export function num(value, fallback = null) {
  if (value == null || value === "") return fallback;
  const n = Number(value);
  return Number.isFinite(n) ? n : fallback;
}

export function parseCleared(raw) {
  return new Set(
    String(raw || "")
      .split(",")
      .map((part) => Number(part))
      .filter((n) => Number.isInteger(n) && n >= 1 && n <= 3)
  );
}

export function clearedString(set) {
  return [...set].sort((a, b) => a - b).join(",");
}

export function inferCleared(level, existing) {
  const set = parseCleared(existing);
  const lv = clampLevel(level || 1);
  if (lv >= 2) set.add(1);
  if (lv >= 3) {
    set.add(1);
    set.add(2);
  }
  return set;
}

export function liveElapsedMs(result, now = Date.now()) {
  if (!result) return null;
  const startedAt = num(result.startedAt);
  const stored = num(result.elapsedMs);
  const status = result.status;
  if (status === "started" && startedAt != null) {
    return Math.max(0, now - startedAt);
  }
  if (stored != null && stored >= 0) return stored;
  if ((status === "completed" || status === "failed") && startedAt != null) {
    const finished = result.finishedAt ? Date.parse(result.finishedAt) : now;
    if (Number.isFinite(finished)) return Math.max(0, finished - startedAt);
  }
  return null;
}

export function presentResult(result, now = Date.now()) {
  if (!result || !result.status) return null;
  return {
    status: result.status,
    elapsedMs: liveElapsedMs(result, now),
    score: num(result.score, 0) || 0,
    levelReached: num(result.levelReached, 0) || 0,
    kids: num(result.kids, 0) || 0,
    keys: num(result.keys, 0) || 0,
    lives: num(result.lives, 0) || 0,
    hp: num(result.hp, 0) || 0,
    failureReason: result.failureReason || "",
    finishedAt: result.finishedAt || "",
    startedAt: num(result.startedAt),
    levelsCleared: result.levelsCleared || "",
  };
}

export function mergeScore(current, incoming) {
  return clampScore(Math.max(num(current, 0) || 0, num(incoming, 0) || 0));
}

export function mergeCount(current, incoming, max) {
  const next = Math.max(num(current, 0) || 0, num(incoming, 0) || 0);
  return Math.min(next, max);
}

export function runFields(user, current, body, { status, now = Date.now(), finished = false, detail = "", reset = false } = {}) {
  const prev = reset ? {} : current || {};
  const level = clampLevel(body.level ?? prev.levelReached ?? 1);
  const score = mergeScore(prev.score, body.score);
  const prevStarted = num(prev.startedAt);
  const startedAt =
    !reset && prev.status === "started" && prevStarted != null ? String(prevStarted) : String(now);
  const elapsed = finished
    ? Math.max(0, now - Number(startedAt))
    : status === "started"
      ? Math.max(0, now - Number(startedAt))
      : num(prev.elapsedMs);
  const cleared = inferCleared(level, prev.levelsCleared);
  if (body.clearLevel) cleared.add(clampLevel(body.clearLevel));
  const pick = (incoming, existing, max) =>
    incoming != null && incoming !== "" ? mergeCount(0, incoming, max) : mergeCount(existing, 0, max);
  return {
    username: user.username,
    status,
    startedAt,
    finishedAt: finished ? new Date(now).toISOString() : prev.finishedAt || "",
    elapsedMs: elapsed == null ? "" : String(elapsed),
    score,
    levelReached: Math.max(clampLevel(prev.levelReached || 1), level),
    kids: pick(body.kids, prev.kids, 9),
    keys: pick(body.keys, prev.keys, 9),
    lives: pick(body.lives, prev.lives, 99),
    hp: pick(body.hp, prev.hp, 99),
    failureReason: status === "failed" ? String(detail || body.detail || prev.failureReason || "game_over") : "",
    levelsCleared: clearedString(cleared),
  };
}

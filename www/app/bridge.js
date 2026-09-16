import { state } from "./state.js";
import { startRun, checkpointRun, progressRun, finishRun } from "./rooms.js";

const LOG_EVENTS = [
  { match: "Game started from controls page!", name: "run_start", detail: "ok" },
  { match: "Game restarted from level 1!", name: "run_start", detail: "retry" },
  { match: "Game restarted from level 1 after time up!", name: "run_start", detail: "retry" },
  { match: "Level Complete! Transitioning to next level...", name: "level_clear", detail: "ok" },
  { match: "Time's up! Level failed.", name: "run_fail", detail: "time_up" },
  { match: "Game Complete! Victory!", name: "run_complete", detail: "victory" },
];

const SCORE_BUMPS = [
  { match: "SFX: kid_rescued", add: 500 },
  { match: "SFX: key_pickup", add: 100 },
  { match: "PowerUp1 collected", add: 750 },
  { match: "Life powerup collected", add: 1000 },
];

let draining = false;
let runClosed = false;
const queue = [];
let lastProgressAt = 0;
let pendingProgress = null;
let progressTimer = null;

// The game reports every event twice: once as a `RUNSTAT ...` console line and
// once as a `turboGameEvent` CustomEvent. Drop the second copy of each.
let lastEventKey = "";
let lastEventAt = 0;

function isDuplicate(name, data) {
  const key = `${name}:${JSON.stringify(data)}`;
  const now = Date.now();
  const dup = key === lastEventKey && now - lastEventAt < 500;
  lastEventKey = key;
  lastEventAt = now;
  return dup;
}

function resetRunAccumulators() {
  state.lastScore = 0;
  state.lastKids = 0;
  state.lastKeys = 0;
  state.lastLives = 0;
  state.lastHp = 0;
  state.runLevel = 1;
}

function parseDetail(raw) {
  if (!raw) return {};
  if (typeof raw === "object") return raw;
  try {
    return JSON.parse(raw);
  } catch {
    return { detail: String(raw) };
  }
}

function rememberStats(data = {}) {
  const score = Number(data.score);
  if (Number.isFinite(score)) state.lastScore = Math.max(state.lastScore || 0, score);
  if (data.level != null && Number.isFinite(Number(data.level))) {
    state.runLevel = Math.min(3, Math.max(1, Number(data.level)));
  }
  if (data.kids != null && Number.isFinite(Number(data.kids))) state.lastKids = Number(data.kids);
  if (data.keys != null && Number.isFinite(Number(data.keys))) state.lastKeys = Number(data.keys);
  if (data.lives != null && Number.isFinite(Number(data.lives))) state.lastLives = Number(data.lives);
  if (data.hp != null && Number.isFinite(Number(data.hp))) state.lastHp = Number(data.hp);
}

function snapshot(data = {}) {
  rememberStats(data);
  return {
    level: Number(data.level || state.runLevel || 1),
    score: Math.max(Number(data.score || 0), state.lastScore || 0),
    kids: data.kids != null ? Number(data.kids) : state.lastKids,
    keys: data.keys != null ? Number(data.keys) : state.lastKeys,
    lives: data.lives != null ? Number(data.lives) : state.lastLives,
    hp: data.hp != null ? Number(data.hp) : state.lastHp,
    detail: data.detail,
  };
}

function scheduleProgress(data) {
  pendingProgress = snapshot(data);
  const wait = Math.max(0, 1200 - (Date.now() - lastProgressAt));
  if (progressTimer) return;
  progressTimer = setTimeout(async () => {
    progressTimer = null;
    const payload = pendingProgress;
    pendingProgress = null;
    if (!payload || !state.room || !state.runActive) return;
    lastProgressAt = Date.now();
    try {
      await progressRun(payload);
      window.dispatchEvent(new CustomEvent("santa-room-updated"));
    } catch (err) {
      console.warn("Santa run sync failed:", err.message);
    }
  }, wait);
}

async function handleEvent(name, data) {
  if (isDuplicate(name, data)) return;
  // A new run zeroes the accumulators, otherwise snapshot() would floor this
  // run's score/kids/keys at the previous run's values.
  if (name === "run_start") resetRunAccumulators();
  queue.push({ name, data: snapshot(data) });
  if (draining) return;
  draining = true;
  while (queue.length) {
    const item = queue.shift();
    if (!state.room) continue;
    const ended = item.name === "run_fail" || item.name === "run_complete";
    if (ended && runClosed) continue;
    if (item.name === "run_start") runClosed = false;
    if (ended) runClosed = true;
    try {
      if (item.name === "run_start") {
        state.runLevel = Number(item.data.level || 1);
        state.lastScore = Number(item.data.score || 0);
        state.lastRun = null;
        state.localStartedAt = Date.now();
        state.runActive = true;
        await startRun();
      } else if (item.name === "run_progress") {
        scheduleProgress(item.data);
        continue;
      } else if (item.name === "level_clear") {
        const level = Number(item.data.level || state.runLevel || 1);
        state.runLevel = Math.min(3, level + 1);
        await checkpointRun({ ...item.data, level, clearLevel: level });
      } else if (item.name === "run_fail") {
        await finishRun({
          status: "failed",
          ...item.data,
          detail: item.data.detail || "game_over",
        });
      } else if (item.name === "run_complete") {
        await finishRun({
          status: "completed",
          ...item.data,
          level: Math.max(3, Number(item.data.level || 3)),
          detail: item.data.detail || "victory",
        });
      }
      window.dispatchEvent(new CustomEvent("santa-room-updated"));
      if (ended) {
        state.runActive = false;
        state.lastRun = {
          status: item.name === "run_complete" ? "completed" : "failed",
          ...item.data,
        };
        window.dispatchEvent(new CustomEvent("santa-run-ended", { detail: state.lastRun }));
      }
    } catch (err) {
      console.warn("Santa run sync failed:", err.message);
      if (ended) {
        state.runActive = false;
        state.lastRun = {
          status: item.name === "run_complete" ? "completed" : "failed",
          ...item.data,
        };
        window.dispatchEvent(new CustomEvent("santa-run-ended", { detail: state.lastRun }));
      }
    }
  }
  draining = false;
}

function bumpScoreFromLog(text) {
  for (const rule of SCORE_BUMPS) {
    if (text.includes(rule.match)) {
      state.lastScore = (state.lastScore || 0) + rule.add;
      if (rule.match.includes("kid_rescued")) state.lastKids = (state.lastKids || 0) + 1;
      if (rule.match.includes("key_pickup")) state.lastKeys = (state.lastKeys || 0) + 1;
      scheduleProgress({});
      break;
    }
  }
}

function patchConsole() {
  const original = console.log.bind(console);
  console.log = (...args) => {
    original(...args);
    const text = args.map((a) => (typeof a === "string" ? a : "")).join(" ");
    const runstat = text.match(/^RUNSTAT\s+(\S+)\s+(\{.*\})$/);
    if (runstat) {
      state.absoluteStats = true;
      handleEvent(runstat[1], parseDetail(runstat[2]));
      return;
    }
    if (!state.absoluteStats) bumpScoreFromLog(text);
    queueMicrotask(() => {
      if (text.startsWith("Game Over!")) {
        handleEvent("run_fail", { detail: "game_over", level: state.runLevel, score: state.lastScore });
        return;
      }
      for (const rule of LOG_EVENTS) {
        if (text.includes(rule.match)) {
          handleEvent(rule.name, { detail: rule.detail, level: state.runLevel, score: state.lastScore });
          break;
        }
      }
    });
  };
}

export function initBridge() {
  patchConsole();
  window.addEventListener("turboGameEvent", (event) => {
    const detail = event.detail || {};
    const name = detail.name;
    if (!name) return;
    handleEvent(name, parseDetail(detail.data));
    state.absoluteStats = true;
  });
}

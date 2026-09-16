import { presentResult } from "./results.js";

export const PRIZES = [1000, 500];

export function formatRupees(n) {
  const value = Number(n);
  const amount = Number.isFinite(value) && value > 0 ? Math.floor(value) : 0;
  return `₹${amount}`;
}

export function sumWinnings(rows) {
  return (rows || []).reduce((sum, row) => sum + Number(row.prizeRupees || 0), 0);
}

export function rankPlayers(members, resultsByKey, now = Date.now()) {
  const rows = members.map((username) => {
    const key = String(username).toLowerCase();
    const presented = presentResult(resultsByKey[key], now);
    return {
      username,
      status: presented?.status || "idle",
      elapsedMs: presented?.elapsedMs ?? null,
      score: presented?.score || 0,
      levelReached: presented?.levelReached || 0,
      kids: presented?.kids || 0,
      keys: presented?.keys || 0,
      lives: presented?.lives || 0,
      hp: presented?.hp || 0,
      failureReason: presented?.failureReason || null,
      finishedAt: presented?.finishedAt || null,
      startedAt: presented?.startedAt ?? null,
    };
  });

  rows.sort((a, b) => {
    const order = (row) => {
      if (row.status === "completed") return 0;
      if (row.status === "failed") return 1;
      if (row.status === "started") return 2;
      return 3;
    };
    const ao = order(a);
    const bo = order(b);
    if (ao !== bo) return ao - bo;
    if (a.status === "completed") {
      const dt = (a.elapsedMs ?? Infinity) - (b.elapsedMs ?? Infinity);
      if (dt !== 0) return dt;
      if (b.score !== a.score) return b.score - a.score;
    }
    if (a.status === "failed") {
      if (b.levelReached !== a.levelReached) return b.levelReached - a.levelReached;
      if (b.score !== a.score) return b.score - a.score;
      const dt = (a.elapsedMs ?? Infinity) - (b.elapsedMs ?? Infinity);
      if (dt !== 0) return dt;
    }
    if (a.status === "started") {
      if (b.score !== a.score) return b.score - a.score;
      if (b.levelReached !== a.levelReached) return b.levelReached - a.levelReached;
    }
    if (b.score !== a.score) return b.score - a.score;
    return a.username.localeCompare(b.username);
  });

  let finishIndex = 0;
  return rows.map((row, i) => {
    let prizeRupees = 0;
    if (row.status === "completed") {
      prizeRupees = PRIZES[finishIndex] || 0;
      finishIndex += 1;
    }
    return { ...row, rank: i + 1, prizeRupees };
  });
}

// Lifetime standing across every room: wins first, then fastest winning run,
// then best score, then who played more.
export function rankAccounts(accounts) {
  const rows = [...accounts].sort((a, b) => {
    if (b.completions !== a.completions) return b.completions - a.completions;
    const at = a.bestMs ?? Infinity;
    const bt = b.bestMs ?? Infinity;
    if (at !== bt) return at - bt;
    if (b.bestScore !== a.bestScore) return b.bestScore - a.bestScore;
    if (b.runs !== a.runs) return b.runs - a.runs;
    return a.username.localeCompare(b.username);
  });
  return rows.map((row, i) => ({ ...row, globalRank: i + 1 }));
}

export function formatTime(ms) {
  if (ms == null || ms === "" || !Number.isFinite(Number(ms))) return "--";
  const total = Math.max(0, Math.floor(Number(ms) / 1000));
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

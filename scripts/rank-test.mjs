import { rankPlayers, formatTime, formatRupees } from "../api/_lib/rank.js";
import { liveElapsedMs } from "../api/_lib/results.js";

const now = 1_700_000_000_000;

const ranked = rankPlayers(
  ["ElfRunner", "SnowMaster", "Rudolph", "Shubh"],
  {
    snowmaster: { status: "completed", elapsedMs: 504000, score: 3400, levelReached: 3 },
    rudolph: { status: "completed", elapsedMs: 591000, score: 2950, levelReached: 3 },
    elfrunner: { status: "failed", elapsedMs: 420000, score: 1800, levelReached: 2 },
  },
  now
);

const names = ranked.map((row) => row.username);
if (names[0] !== "SnowMaster" || names[1] !== "Rudolph" || names[2] !== "ElfRunner" || names[3] !== "Shubh") {
  console.error(ranked);
  throw new Error("Unexpected rank order");
}
if (formatTime(504000) !== "08:24") throw new Error("formatTime failed");
if (ranked[0].prizeRupees !== 1000) throw new Error("first complete should win ₹1000");
if (ranked[1].prizeRupees !== 500) throw new Error("second complete should win ₹500");
if (ranked[2].prizeRupees !== 0) throw new Error("failed player should not win a prize");
if (ranked[3].prizeRupees !== 0) throw new Error("idle player should not win a prize");
if (formatRupees(1000) !== "₹1000" || formatRupees(0) !== "₹0") throw new Error("formatRupees failed");

const live = rankPlayers(
  ["first_account", "second_account"],
  {
    first_account: { status: "started", startedAt: now - 125000, elapsedMs: "", score: 900, levelReached: 2 },
    second_account: { status: "started", startedAt: now - 4000, elapsedMs: "", score: 0, levelReached: 1 },
  },
  now
);
if (live[0].username !== "first_account") throw new Error("in-run score should rank first");
if (live[0].elapsedMs !== 125000) throw new Error(`live elapsed expected 125000, got ${live[0].elapsedMs}`);
if (live[1].elapsedMs !== 4000) throw new Error(`live elapsed expected 4000, got ${live[1].elapsedMs}`);
if (formatTime(live[0].elapsedMs) !== "02:05") throw new Error("live formatTime failed");
if (live[0].prizeRupees !== 0 || live[1].prizeRupees !== 0) {
  throw new Error("in-progress runs should not receive prizes");
}
if (liveElapsedMs({ status: "started", startedAt: now - 1500, elapsedMs: "" }, now) !== 1500) {
  throw new Error("empty elapsedMs should use wall clock");
}

console.log("rank test ok");
console.log(
  ranked
    .map((row) => `${row.rank} ${row.username} ${row.status} ${formatTime(row.elapsedMs)} ${formatRupees(row.prizeRupees)}`)
    .join("\n")
);

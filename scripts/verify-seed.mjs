import { FileRedis } from "../api/_lib/file-redis.js";
import { formatTime, rankPlayers } from "../api/_lib/rank.js";

const db = new FileRedis();
const members = await db.smembers("room:DEMO01:members");
const resultsByKey = {};
const names = [];
for (const key of members) {
  const account = await db.hgetall(`user:${key}`);
  const result = await db.hgetall(`room:DEMO01:result:${key}`);
  names.push(account.username || key);
  if (result.status) resultsByKey[key] = result;
}
const ranked = rankPlayers(names, resultsByKey);
console.log(ranked.map((row) => `${row.rank}. ${row.username} ${row.status} ${formatTime(row.elapsedMs)} ${row.score}`).join("\n"));
if (ranked[0].username !== "SnowMaster" || ranked[2].username !== "ElfRunner") {
  throw new Error("Seeded leaderboard order is wrong");
}
if (ranked[0].prizeRupees !== 1000 || ranked[1].prizeRupees !== 500 || ranked[2].prizeRupees !== 0) {
  throw new Error("Seeded prizes should be ₹1000, ₹500, ₹0");
}
console.log("seed verify ok");

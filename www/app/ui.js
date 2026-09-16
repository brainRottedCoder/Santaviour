import { state } from "./state.js";
import { formatTime, formatRupees } from "./api.js";
import { initBridge } from "./bridge.js";
import { login, logout, refreshSession, register } from "./auth.js";
import { createRoom, joinRoom, loadRoom } from "./rooms.js";

const CSS_HREF = new URL("./overlay.css", import.meta.url).href;

function el(html) {
  const wrap = document.createElement("div");
  wrap.innerHTML = html.trim();
  return wrap.firstElementChild;
}

function ensureChrome() {
  if (document.getElementById("santa-overlay")) return;
  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = CSS_HREF;
  document.head.appendChild(link);

  document.body.appendChild(
    el(`
      <div id="santa-overlay" class="santa-overlay" tabindex="-1">
        <div class="santa-panel">
          <div id="santa-auth" class="santa-screen">
            <h1>SANTA'S RESCUE</h1>
            <p class="santa-muted">Create a username to save your runs. Join a 4-player room and race the clock.</p>
            <div class="santa-row">
              <input id="santa-username" placeholder="Username" maxlength="16" autocomplete="username" />
            </div>
            <div class="santa-row">
              <input id="santa-password" type="password" placeholder="Password" maxlength="72" autocomplete="current-password" />
            </div>
            <p id="santa-auth-error" class="santa-error"></p>
            <div class="santa-actions">
              <button class="santa-btn" id="santa-login">Sign in</button>
              <button class="santa-btn secondary" id="santa-register">Create account</button>
            </div>
          </div>
          <div id="santa-lobby" class="santa-screen is-hidden">
            <h1>LOBBY</h1>
            <p id="santa-hello" class="santa-muted"></p>
            <div id="santa-user-stats" class="santa-stats"></div>
            <div class="santa-row">
              <input id="santa-room-name" placeholder="Room name" maxlength="32" />
              <button class="santa-btn" id="santa-create">Create room</button>
            </div>
            <div class="santa-row">
              <input id="santa-join-code" placeholder="Room code" maxlength="8" />
              <button class="santa-btn secondary" id="santa-join">Join room</button>
            </div>
            <p id="santa-lobby-error" class="santa-error"></p>
            <div class="santa-actions">
              <button class="santa-btn ghost" id="santa-logout">Sign out</button>
            </div>
          </div>
          <div id="santa-room" class="santa-screen is-hidden">
            <h2 id="santa-room-title">ROOM</h2>
            <div class="santa-code" id="santa-room-code"></div>
            <div id="santa-result" class="santa-result is-hidden"></div>
            <p class="santa-muted">Share the code. First finish wins ₹1000, second wins ₹500. Time starts when the run begins.</p>
            <div class="santa-slots" id="santa-slots"></div>
            <table class="santa-table">
              <thead>
                <tr><th>#</th><th>Player</th><th>Result</th><th>Lvl</th><th>Time</th><th>Score</th><th>Prize</th></tr>
              </thead>
              <tbody id="santa-board"></tbody>
            </table>
            <p id="santa-room-error" class="santa-error"></p>
            <div class="santa-actions">
              <button class="santa-btn" id="santa-play">Play</button>
              <button class="santa-btn ghost" id="santa-leave">Back to lobby</button>
            </div>
          </div>
        </div>
      </div>
    `)
  );

  document.body.appendChild(
    el(`
      <div id="santa-chip" class="santa-chip is-hidden">
        <span id="santa-chip-text">ROOM</span>
        <button class="santa-btn secondary" id="santa-open-room">Board</button>
      </div>
    `)
  );
}

function setError(id, message) {
  const node = document.getElementById(id);
  if (node) node.textContent = message || "";
}

function showScreen(name) {
  state.screen = name;
  const overlay = document.getElementById("santa-overlay");
  const chip = document.getElementById("santa-chip");
  for (const id of ["santa-auth", "santa-lobby", "santa-room"]) {
    document.getElementById(id)?.classList.toggle("is-hidden", id !== `santa-${name}`);
  }
  const blocking = name !== "playing";
  overlay?.classList.toggle("is-hidden", !blocking);
  chip?.classList.toggle("is-hidden", name !== "playing");
  if (blocking) {
    overlay?.focus();
    pauseGame();
  } else {
    resumeGame();
  }
}

async function waitForTurbo() {
  if (state.turbo) return state.turbo;
  try {
    const turbo = await import("../pkg/turbo_genesis_impl_wasm_bindgen.js");
    for (let i = 0; i < 80; i += 1) {
      try {
        turbo.paused();
        state.turbo = turbo;
        return turbo;
      } catch {
        await new Promise((r) => setTimeout(r, 120));
      }
    }
    state.turbo = turbo;
    return turbo;
  } catch {
    return null;
  }
}

async function pauseGame() {
  const turbo = await waitForTurbo();
  try {
    turbo?.pause();
  } catch {
    /* runtime not ready */
  }
}

async function resumeGame() {
  const turbo = await waitForTurbo();
  try {
    turbo?.resume();
  } catch {
    /* runtime not ready */
  }
}

function renderLobby() {
  const user = state.user;
  document.getElementById("santa-hello").textContent = user
    ? `Signed in as ${user.username}`
    : "";
  document.getElementById("santa-user-stats").innerHTML = user
    ? `<span>Runs ${user.runs || 0}</span><span>Wins ${user.completions || 0}</span><span>Best ${formatTime(user.bestMs)}</span><span>Cash ${formatRupees(user.winnings)}</span>`
    : "";
}

function rowElapsed(row) {
  if (row.status === "started") {
    const base = Number(row.elapsedMs || 0);
    const serverNow = Number(state.room?.serverNow || Date.now());
    return Math.max(0, base + (Date.now() - serverNow));
  }
  return row.elapsedMs;
}

function resultLabel(row) {
  if (row.status === "completed") return "Won";
  if (row.status === "failed") {
    const why = row.failureReason === "time_up" ? "Time up" : "Failed";
    return row.levelReached ? `${why} L${row.levelReached}` : why;
  }
  if (row.status === "started") {
    return row.levelReached ? `Playing L${row.levelReached}` : "Playing";
  }
  return "Waiting";
}

function resultDetail(detail) {
  if (detail === "victory") return "Victory";
  if (detail === "time_up") return "Time's up";
  if (detail === "game_over") return "Out of lives";
  return detail || "";
}

function renderResultCard() {
  const box = document.getElementById("santa-result");
  const play = document.getElementById("santa-play");
  if (!box) return;
  const mine = (state.room?.leaderboard || []).find((row) => row.username === state.user?.username);
  const last = state.lastRun;
  const status = mine?.status || last?.status;
  if (play) play.textContent = status === "completed" || status === "failed" ? "Play again" : "Play";
  if (status !== "completed" && status !== "failed") {
    box.className = "santa-result is-hidden";
    box.innerHTML = "";
    return;
  }
  const score = mine?.score ?? last?.score ?? 0;
  const time = formatTime(rowElapsed(mine || { elapsedMs: last?.elapsedMs, status }));
  const rank = mine?.rank ? `#${mine.rank}` : "--";
  const level = mine?.levelReached || last?.level || 1;
  const kids = mine?.kids ?? last?.kids ?? 0;
  const keys = mine?.keys ?? last?.keys ?? 0;
  const prize = formatRupees(mine?.prizeRupees ?? 0);
  const reason = resultDetail(last?.detail || mine?.failureReason);
  box.className = `santa-result${status === "completed" ? " is-win" : ""}`;
  box.innerHTML = `
    <div class="santa-result-title">${status === "completed" ? "You finished!" : "Run over"}</div>
    <div class="santa-result-reason">${reason}</div>
    <div class="santa-result-grid">
      <div><span>Rank</span><strong>${rank}</strong></div>
      <div><span>Score</span><strong>${score}</strong></div>
      <div><span>Time</span><strong>${time}</strong></div>
      <div><span>Prize</span><strong>${prize}</strong></div>
      <div><span>Level</span><strong>${level}</strong></div>
      <div><span>Kids</span><strong>${kids}</strong></div>
      <div><span>Keys</span><strong>${keys}</strong></div>
    </div>
  `;
}

function renderRoom() {
  const room = state.room;
  if (!room) return;
  document.getElementById("santa-room-title").textContent = room.name || "ROOM";
  document.getElementById("santa-room-code").textContent = room.code;
  renderResultCard();
  const slots = document.getElementById("santa-slots");
  const members = room.members || [];
  slots.innerHTML = Array.from({ length: 4 }, (_, i) => {
    const name = members[i];
    return name
      ? `<div class="santa-slot">${i + 1}. ${name}</div>`
      : `<div class="santa-slot empty">${i + 1}. Open slot</div>`;
  }).join("");
  const board = document.getElementById("santa-board");
  const rows = room.leaderboard || [];
  board.innerHTML = rows
    .map((row) => {
      const elapsed = rowElapsed(row);
      return `<tr class="${row.username === state.user?.username ? "is-you" : ""}">
        <td>${row.rank}</td>
        <td>${row.username}</td>
        <td>${resultLabel(row)}</td>
        <td>${row.levelReached || (row.status === "idle" ? "-" : 1)}</td>
        <td>${formatTime(elapsed)}</td>
        <td>${row.score || 0}</td>
        <td>${formatRupees(row.prizeRupees)}</td>
      </tr>`;
    })
    .join("");
  const mine = rows.find((row) => row.username === state.user?.username);
  document.getElementById("santa-chip-text").textContent = mine
    ? `${room.code}  #${mine.rank}  ${formatTime(rowElapsed(mine))}  ${mine.score || 0}${
        mine.prizeRupees > 0 ? `  ${formatRupees(mine.prizeRupees)}` : ""
      }`
    : `${room.code}`;
}

async function openRoom(room) {
  state.room = room;
  renderRoom();
  showScreen("room");
}

async function boot() {
  ensureChrome();
  initBridge();
  bind();
  showScreen("auth");
  pauseGame();
  const user = await refreshSession();
  if (user) {
    renderLobby();
    showScreen("lobby");
  }
}

function isAdminShortcut(event) {
  return (event.ctrlKey || event.metaKey) && event.shiftKey && !event.altKey && event.key.toLowerCase() === "a";
}

function openAdminPanel() {
  window.location.assign("/admin");
}

function bind() {
  document.getElementById("santa-login").onclick = async () => {
    setError("santa-auth-error");
    try {
      await login(
        document.getElementById("santa-username").value,
        document.getElementById("santa-password").value
      );
      await refreshSession();
      renderLobby();
      showScreen("lobby");
    } catch (err) {
      setError("santa-auth-error", err.message);
    }
  };
  document.getElementById("santa-register").onclick = async () => {
    setError("santa-auth-error");
    try {
      await register(
        document.getElementById("santa-username").value,
        document.getElementById("santa-password").value
      );
      await refreshSession();
      renderLobby();
      showScreen("lobby");
    } catch (err) {
      setError("santa-auth-error", err.message);
    }
  };
  document.getElementById("santa-logout").onclick = async () => {
    await logout();
    showScreen("auth");
  };
  document.getElementById("santa-create").onclick = async () => {
    setError("santa-lobby-error");
    try {
      const room = await createRoom(document.getElementById("santa-room-name").value);
      await openRoom(room);
    } catch (err) {
      setError("santa-lobby-error", err.message);
    }
  };
  document.getElementById("santa-join").onclick = async () => {
    setError("santa-lobby-error");
    try {
      const room = await joinRoom(document.getElementById("santa-join-code").value);
      await openRoom(room);
    } catch (err) {
      setError("santa-lobby-error", err.message);
    }
  };
  document.getElementById("santa-play").onclick = () => showScreen("playing");
  document.getElementById("santa-leave").onclick = async () => {
    state.room = null;
    state.runActive = false;
    state.localStartedAt = null;
    state.lastScore = 0;
    state.lastRun = null;
    await refreshSession();
    renderLobby();
    showScreen("lobby");
  };
  document.getElementById("santa-open-room").onclick = async () => {
    if (!state.room) return;
    try {
      await loadRoom(state.room.code);
    } catch {
      /* keep last snapshot */
    }
    renderRoom();
    showScreen("room");
  };
  document.getElementById("santa-overlay").addEventListener("keydown", (event) => {
    if (isAdminShortcut(event)) {
      event.preventDefault();
      openAdminPanel();
      return;
    }
    event.stopPropagation();
  });

  window.addEventListener(
    "keydown",
    (event) => {
      if (!isAdminShortcut(event)) return;
      event.preventDefault();
      openAdminPanel();
    },
    true
  );

  let roomEpoch = 0;
  async function refreshRoom() {
    if (!state.room) return;
    const epoch = ++roomEpoch;
    const code = state.room.code;
    try {
      await loadRoom(code);
      if (epoch !== roomEpoch) return;
      renderRoom();
    } catch {
      /* keep last snapshot */
    }
  }

  window.addEventListener("santa-run-ended", async () => {
    if (!state.room) return;
    await refreshRoom();
    showScreen("room");
  });

  window.addEventListener("santa-room-updated", () => {
    renderRoom();
    if (state.screen === "playing") {
      document.getElementById("santa-chip")?.classList.remove("is-hidden");
    }
  });

  setInterval(() => {
    if (!state.room || (state.screen !== "room" && state.screen !== "playing")) return;
    renderRoom();
  }, 250);

  setInterval(() => {
    if (!state.room || (state.screen !== "room" && state.screen !== "playing")) return;
    refreshRoom();
  }, 1500);
}

boot();

import { api, formatTime, formatRupees } from "./api.js";

const state = {
  admin: false,
  configured: true,
  data: null,
  detail: null,
  tab: "ranking",
  filter: "",
  error: "",
  note: "",
  autoRefresh: true,
};

const root = document.getElementById("admin-root");

function esc(value) {
  return String(value ?? "").replace(
    /[&<>"']/g,
    (ch) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[ch]
  );
}

function date(value) {
  if (!value) return "—";
  const ms = typeof value === "number" ? value : Date.parse(value);
  if (!Number.isFinite(ms) || ms <= 0) return "—";
  const when = new Date(ms);
  const day = when.toLocaleDateString(undefined, { day: "2-digit", month: "short" });
  const time = when.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
  return `${day} ${time}`;
}

function liveElapsed(row) {
  if (row?.status !== "started") return row?.elapsedMs;
  const serverNow = Number(state.data?.serverNow || Date.now());
  return Math.max(0, Number(row.elapsedMs || 0) + (Date.now() - serverNow));
}

function statusLabel(status) {
  if (status === "completed") return "Won";
  if (status === "failed") return "Did not finish";
  if (status === "started") return "In a run";
  return "Not started";
}

function pill(status) {
  return `<span class="adm-pill is-${esc(status || "idle")}">${statusLabel(status)}</span>`;
}

function reasonLabel(reason) {
  if (reason === "time_up") return "the timer ran out";
  if (reason === "game_over") return "they ran out of lives";
  if (!reason) return "the run ended";
  return String(reason).replaceAll("_", " ");
}

function resultSummary(row) {
  if (!row || row.status === "idle") return "Joined the room but has not started a run.";
  if (row.status === "completed") {
    return `Finished the game in ${formatTime(row.elapsedMs)} with ${row.score || 0} points.`;
  }
  if (row.status === "failed") {
    return `Stopped on level ${row.levelReached || 1} because ${reasonLabel(row.failureReason)}.`;
  }
  if (row.status === "started") {
    return `Playing level ${row.levelReached || 1} · ${formatTime(liveElapsed(row))} on the clock.`;
  }
  return statusLabel(row.status);
}

function winRate(user) {
  if (!user.runs) return "—";
  return `${Math.round((user.completions / user.runs) * 100)}%`;
}

function playerBlurb(user) {
  const name = user.username;
  if (!user.runs) {
    return `${name} created an account on ${date(user.createdAt)} and has not finished a run yet.`;
  }
  const wins = user.completions
    ? `won ${user.completions} of ${user.runs} finished attempts`
    : `has ${user.runs} finished attempts and no wins yet`;
  const best = user.bestMs ? ` Fastest win: ${formatTime(user.bestMs)}.` : "";
  return `${name} is #${user.globalRank} across every room — ${wins}.${best}`;
}

function headers(cols) {
  return cols
    .map((col) => {
      if (typeof col === "string") return `<th>${col}</th>`;
      return `<th title="${esc(col.hint || "")}">
        <span class="adm-th">${esc(col.label)}</span>
        ${col.hint ? `<span class="adm-th-hint">${esc(col.hint)}</span>` : ""}
      </th>`;
    })
    .join("");
}

function table(cols, rows, { pinEnd = false, empty = "Nothing to show yet." } = {}) {
  if (!rows.length) return `<p class="adm-empty">${empty}</p>`;
  return `
    <div class="adm-table-wrap">
      <table class="adm-table${pinEnd ? " adm-pin-end" : ""}">
        <thead><tr>${headers(cols)}</tr></thead>
        <tbody>${rows.join("")}</tbody>
      </table>
    </div>`;
}

function legend() {
  return `
    <p class="adm-legend" aria-label="Result colours">
      <span>${pill("completed")} reached the end of the game</span>
      <span>${pill("failed")} lives or time ran out</span>
      <span>${pill("started")} run in progress</span>
      <span>${pill("idle")} in the room, not playing</span>
    </p>`;
}

function metric(label, value, hint) {
  return `<article class="adm-card"><b>${esc(label)}</b><strong>${value}</strong><p>${esc(hint)}</p></article>`;
}

/* ---------------------------------------------------------------- gate ---- */

function renderGate() {
  root.innerHTML = `
    <div class="adm-gate-wrap">
      <form class="adm-gate" id="adm-gate">
        <p class="adm-kicker">Staff only</p>
        <h1>Operations console</h1>
        <p class="adm-lede">${
          state.configured
            ? "This page is not part of the game. It shows every registered player, how they rank across all races, and the history of each attempt. Player usernames cannot open it."
            : "ADMIN_PASSWORD is not set on the server. Add it to .env.local or the Vercel project, then reload."
        }</p>
        <label class="adm-field">
          <span>Admin password</span>
          <input class="adm-input" id="adm-password" type="password"
            autocomplete="current-password" ${state.configured ? "" : "disabled"} />
        </label>
        ${state.error ? `<p class="adm-banner adm-error">${esc(state.error)}</p>` : ""}
        <button class="adm-btn" type="submit" ${state.configured ? "" : "disabled"}>Open console</button>
      </form>
    </div>`;

  document.getElementById("adm-gate").onsubmit = async (event) => {
    event.preventDefault();
    state.error = "";
    try {
      await api("/api/admin/login", {
        method: "POST",
        body: { password: document.getElementById("adm-password").value },
      });
      state.admin = true;
      await refresh();
    } catch (err) {
      state.error = err.message;
      renderGate();
    }
  };
}

/* ----------------------------------------------------------- dashboard ---- */

function rankingRows() {
  return (state.data?.users || []).map(
    (user) => `
      <tr>
        <td class="adm-rank">${user.globalRank}</td>
        <td class="adm-name">${esc(user.username)}</td>
        <td class="adm-num">${user.completions}</td>
        <td class="adm-num">${user.runs}</td>
        <td class="adm-num">${winRate(user)}</td>
        <td class="adm-num">${formatTime(user.bestMs)}</td>
        <td class="adm-num">${user.bestScore}</td>
        <td class="adm-num">${formatRupees(user.winnings)}</td>
        <td class="adm-num">${user.roomsPlayed}</td>
        <td>${user.activeRuns ? pill("started") : "—"}</td>
      </tr>`
  );
}

function playerRows() {
  const needle = state.filter.trim().toLowerCase();
  const rows = (state.data?.users || []).filter(
    (user) => !needle || user.username.toLowerCase().includes(needle)
  );
  return rows.map(
    (user) => `
      <tr>
        <td class="adm-rank">${user.globalRank}</td>
        <td class="adm-name">${esc(user.username)}</td>
        <td>${date(user.createdAt)}</td>
        <td class="adm-num">${user.runs}</td>
        <td class="adm-num">${user.completions}</td>
        <td class="adm-num">${formatRupees(user.winnings)}</td>
        <td class="adm-num">${formatTime(user.bestMs)}</td>
        <td class="adm-num">${user.historyCount}</td>
        <td>${date(user.lastActiveAt)}</td>
        <td>
          <div class="adm-actions-cell">
            <button class="adm-btn tiny secondary" data-view="${esc(user.username)}">Read history</button>
            <button class="adm-btn tiny danger" data-del-user="${esc(user.username)}">Remove</button>
          </div>
        </td>
      </tr>`
  );
}

function boardColumns() {
  return [
    { label: "Place", hint: "In this room only" },
    { label: "Player", hint: "Username" },
    { label: "What happened", hint: "Plain reading of the result" },
    { label: "Level", hint: "Furthest stage reached" },
    { label: "Clock", hint: "Time in this attempt" },
    { label: "Score", hint: "Points this attempt" },
    { label: "Prize", hint: "₹1000 first finish, ₹500 second" },
    { label: "Kids / keys", hint: "Rescues · keys" },
    { label: "Health", hint: "Lives left · HP" },
    { label: "", hint: "" },
  ];
}

function boardRow(room, row) {
  return `
    <tr>
      <td class="adm-rank">${row.rank}</td>
      <td class="adm-name">${esc(row.username)}</td>
      <td class="adm-result">${pill(row.status)}<div>${esc(resultSummary(row))}</div></td>
      <td class="adm-num">${row.levelReached || "—"}</td>
      <td class="adm-num">${formatTime(liveElapsed(row))}</td>
      <td class="adm-num">${row.score || 0}</td>
      <td class="adm-num">${formatRupees(row.prizeRupees)}</td>
      <td class="adm-num">${row.kids || 0} / ${row.keys || 0}</td>
      <td class="adm-num">${row.lives || 0} · ${row.hp || 0}</td>
      <td>
        <button class="adm-btn tiny ghost" data-clear-run="${esc(room.code)}" data-clear-user="${esc(row.username)}"
          title="Remove this player's current attempt from the room. Their account stays.">Clear attempt</button>
      </td>
    </tr>`;
}

function renderRaces() {
  const rooms = state.data?.rooms || [];
  if (!rooms.length) return `<p class="adm-empty">No rooms have been created yet.</p>`;
  return `<div class="adm-races">${rooms
    .map((room) => {
      const board = room.leaderboard || [];
      const live = board.filter((row) => row.status === "started").length;
      const won = board.filter((row) => row.status === "completed").length;
      const seats = (room.members || []).length;
      return `
        <article class="adm-race">
          <header class="adm-race-head">
            <div>
              <h3>${esc(room.name)}</h3>
              <p class="adm-meta">
                <span>Code <span class="adm-code">${esc(room.code)}</span></span>
                <span>Hosted by ${esc(room.owner)}</span>
                <span>Opened ${date(room.createdAt)}</span>
                <span>${seats} of 4 seats filled</span>
                <span>${live} playing now</span>
                <span>${won} already finished</span>
              </p>
            </div>
            <div class="adm-actions-cell">
              <button class="adm-btn tiny ghost" data-reset-room="${esc(room.code)}"
                title="Wipe the current race. Players stay in the room; lifetime stats are kept.">Reset race</button>
              <button class="adm-btn tiny danger" data-del-room="${esc(room.code)}"
                title="Delete the room and its current scoreboard. Player accounts remain.">Delete room</button>
            </div>
          </header>
          ${table(boardColumns(), board.map((row) => boardRow(room, row)), {
            empty: "Nobody in this room has started a run.",
          })}
        </article>`;
    })
    .join("")}</div>`;
}

function fact(label, value, hint) {
  return `<div><span>${esc(label)}</span><strong>${value}</strong>${hint ? `<small>${esc(hint)}</small>` : ""}</div>`;
}

function renderDetail() {
  const user = state.detail;
  if (!user) return `<section class="adm-detail is-hidden"></section>`;
  return `
    <section class="adm-detail" id="adm-player-file">
      <p class="adm-kicker">Player file</p>
      <h2>${esc(user.username)}</h2>
      <p class="adm-lede">${esc(playerBlurb(user))}</p>
      <div class="adm-row" style="margin-top:16px">
        <button class="adm-btn tiny ghost" id="adm-close-detail">Close file</button>
        <button class="adm-btn tiny danger" data-del-user="${esc(user.username)}">Remove this player</button>
      </div>
      <div class="adm-facts">
        ${fact("Overall rank", user.globalRank ?? "—", "Across every room")}
        ${fact("Signed up", date(user.createdAt), "Account created")}
        ${fact("Finished attempts", user.runs, "Wins plus did-not-finish")}
        ${fact("Wins", user.completions, "Reached the end of the game")}
        ${fact("Win rate", winRate(user), "Wins ÷ finished attempts")}
        ${fact("Fastest win", formatTime(user.bestMs), "Only counts completed games")}
        ${fact("Highest score", user.bestScore, "Best points in any attempt")}
        ${fact("Total cash", formatRupees(user.winnings), "Prize for 1st/2nd finish in each room")}
        ${fact("Rooms joined", user.roomsPlayed, "Distinct races entered")}
        ${fact("Last activity", date(user.lastActiveAt), "Most recent run or join")}
      </div>

      <h3>How they stand in each room</h3>
      <p class="adm-lede">A player can sit in several rooms. This is their current result in each one — not the full history.</p>
      ${table(
        [
          { label: "Place", hint: "In that room" },
          { label: "Room", hint: "Race name" },
          { label: "Code", hint: "Share code" },
          { label: "What happened", hint: "Current attempt" },
          { label: "Clock", hint: "This attempt" },
          { label: "Score", hint: "This attempt" },
          { label: "Prize", hint: "₹1000 / ₹500 for 1st / 2nd finish" },
        ],
        (user.placements || []).map(
          (row) => `
          <tr>
            <td class="adm-rank">${row.rank}</td>
            <td>${esc(row.room)}</td>
            <td class="adm-code">${esc(row.code)}</td>
            <td class="adm-result">${pill(row.status)}<div>${esc(resultSummary(row))}</div></td>
            <td class="adm-num">${formatTime(liveElapsed(row))}</td>
            <td class="adm-num">${row.score || 0}</td>
            <td class="adm-num">${formatRupees(row.prizeRupees)}</td>
          </tr>`
        ),
        { empty: "This player has not joined a room." }
      )}

      <h3>Every finished attempt (${(user.history || []).length})</h3>
      <p class="adm-lede">Replaying a room overwrites the live scoreboard. This log keeps the older attempts so you can still see them.</p>
      ${table(
        [
          { label: "When", hint: "Finish time" },
          { label: "Room", hint: "Where they played" },
          { label: "What happened", hint: "Outcome of that attempt" },
          { label: "Level", hint: "Furthest stage" },
          { label: "Clock", hint: "Duration" },
          { label: "Score", hint: "Points" },
          { label: "Kids / keys", hint: "Rescues and keys" },
        ],
        (user.history || []).map(
          (row) => `
          <tr>
            <td>${date(row.at)}</td>
            <td>${esc(row.room)} <span class="adm-code">${esc(row.code)}</span></td>
            <td class="adm-result">${pill(row.status)}<div>${esc(resultSummary(row))}</div></td>
            <td class="adm-num">${row.levelReached || "—"}</td>
            <td class="adm-num">${formatTime(row.elapsedMs)}</td>
            <td class="adm-num">${row.score || 0}</td>
            <td class="adm-num">${row.kids || 0} / ${row.keys || 0}</td>
          </tr>`
        ),
        { empty: "No finished attempts are stored yet. History starts from the moment this console was added." }
      )}
    </section>`;
}

function renderDashboard() {
  const totals = state.data?.totals || {};
  const live = totals.activeRuns ?? 0;
  root.innerHTML = `
    <header class="adm-head">
      <div>
        <p class="adm-kicker">Santa's Rescue</p>
        <h1>Operations console</h1>
        <p class="adm-lede">Use this page to see who registered, who is winning across every race, and what happened in each attempt.</p>
      </div>
      <div class="adm-head-actions">
        <span class="adm-stamp">Updated ${date(state.data?.generatedAt)}</span>
        <button class="adm-btn tiny secondary" id="adm-refresh">Refresh now</button>
        <button class="adm-btn tiny ghost" id="adm-auto" aria-pressed="${state.autoRefresh}">
          Auto-refresh ${state.autoRefresh ? "on" : "off"}
        </button>
        <button class="adm-btn tiny ghost" id="adm-logout">Sign out</button>
      </div>
    </header>

    <div class="adm-cards">
      ${metric("Players", totals.users ?? 0, "Everyone who created an account, including people who have not played yet.")}
      ${metric("Rooms", totals.rooms ?? 0, "A room is one race, with up to four players sharing a code.")}
      ${metric("Finished attempts", totals.runs ?? 0, "Wins plus did-not-finish. A replay in the same room still counts here.")}
      ${metric("Wins", totals.completions ?? 0, "Runs that reached the end of the game (levels 1 and 2 cleared).")}
      ${metric("Playing now", live, "Clocks that are still ticking. Open Races to watch them live.")}
      ${metric("Prize paid", formatRupees(totals.winnings), "Sum of every player's 1st (₹1000) and 2nd (₹500) finishes across all rooms.")}
    </div>

    ${live ? `<p class="adm-banner adm-live-callout">${live} player${live === 1 ? " is" : "s are"} in a run right now. Their times update on this page.</p>` : ""}
    ${state.error ? `<p class="adm-banner adm-error">${esc(state.error)}</p>` : ""}
    ${state.note ? `<p class="adm-banner adm-note">${esc(state.note)}</p>` : ""}

    <nav class="adm-tabs" aria-label="Console sections">
      <button class="adm-tab ${state.tab === "ranking" ? "is-active" : ""}" data-tab="ranking">Overall standings</button>
      <button class="adm-tab ${state.tab === "players" ? "is-active" : ""}" data-tab="players">Player directory</button>
      <button class="adm-tab ${state.tab === "rooms" ? "is-active" : ""}" data-tab="rooms">Races</button>
    </nav>

    <section class="adm-view ${state.tab === "ranking" ? "" : "is-hidden"}">
      <h2 class="adm-section-title">Who is ahead overall</h2>
      <p class="adm-lede">Rank is not “highest score”. Players with more wins come first. Tied winners are ordered by fastest winning time, then by high score.</p>
      ${legend()}
      ${table(
        [
          { label: "#", hint: "Overall place" },
          { label: "Player", hint: "Account name" },
          { label: "Wins", hint: "Finished the game" },
          { label: "Attempts", hint: "Finished runs" },
          { label: "Win rate", hint: "Wins ÷ attempts" },
          { label: "Fastest win", hint: "Best completed time" },
          { label: "High score", hint: "Best points ever" },
          { label: "Winnings", hint: "Total ₹ from 1st and 2nd finishes" },
          { label: "Rooms", hint: "Races joined" },
          { label: "Right now", hint: "In a live run?" },
        ],
        rankingRows()
      )}
    </section>

    <section class="adm-view ${state.tab === "players" ? "" : "is-hidden"}">
      <h2 class="adm-section-title">Every registered account</h2>
      <p class="adm-lede">Open a player file to see their standing in each room and every finished attempt. Remove deletes the account, not just one race.</p>
      <label class="adm-field search-field">
        <span>Find a username</span>
        <input class="adm-input" id="adm-filter" value="${esc(state.filter)}" autocomplete="off" />
      </label>
      ${table(
        [
          { label: "#", hint: "Overall place" },
          { label: "Username", hint: "How they signed in" },
          { label: "Signed up", hint: "Account created" },
          { label: "Attempts", hint: "Finished runs" },
          { label: "Wins", hint: "Completed games" },
          { label: "Winnings", hint: "Total ₹ from room prizes" },
          { label: "Fastest win", hint: "Best completed time" },
          { label: "Logged", hint: "Attempts kept in history" },
          { label: "Last seen", hint: "Most recent activity" },
          { label: "Actions", hint: "File or remove" },
        ],
        playerRows(),
        { pinEnd: true, empty: "No usernames match that filter." }
      )}
    </section>

    <section class="adm-view ${state.tab === "rooms" ? "" : "is-hidden"}">
      <h2 class="adm-section-title">Each race, in full</h2>
      <p class="adm-lede">Reset race clears the current scoreboard so the same four players can play again. Delete room removes the race entirely. Neither action changes a player's lifetime wins.</p>
      ${legend()}
      ${renderRaces()}
    </section>

    ${renderDetail()}`;

  bindDashboard();
}

/* -------------------------------------------------------------- actions --- */

async function guard(label, fn) {
  state.error = "";
  state.note = "";
  try {
    const result = await fn();
    state.note = label;
    return result;
  } catch (err) {
    if (/sign-in required/i.test(err.message)) {
      state.admin = false;
      renderGate();
      return null;
    }
    state.error = err.message;
    return null;
  }
}

async function refresh() {
  const data = await guard("", () => api("/api/admin/overview"));
  if (!data) {
    if (state.admin) renderDashboard();
    return;
  }
  state.data = data;
  if (state.detail) {
    state.detail = await api(`/api/admin/user?name=${encodeURIComponent(state.detail.username)}`).then(
      (res) => res.user,
      () => null
    );
  }
  renderDashboard();
}

async function openDetail(username) {
  const res = await guard("", () => api(`/api/admin/user?name=${encodeURIComponent(username)}`));
  if (!res) return;
  state.detail = res.user;
  state.tab = "players";
  renderDashboard();
  document.getElementById("adm-player-file")?.scrollIntoView({
    behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth",
    block: "start",
  });
}

function bindDashboard() {
  document.getElementById("adm-refresh").onclick = () => refresh();
  document.getElementById("adm-auto").onclick = () => {
    state.autoRefresh = !state.autoRefresh;
    renderDashboard();
  };
  document.getElementById("adm-logout").onclick = async () => {
    await api("/api/admin/logout", { method: "POST" });
    state.admin = false;
    state.data = null;
    state.detail = null;
    state.error = "";
    renderGate();
  };

  for (const tab of document.querySelectorAll("[data-tab]")) {
    tab.onclick = () => {
      state.tab = tab.dataset.tab;
      if (tab.dataset.tab !== "players") state.detail = null;
      renderDashboard();
    };
  }

  const filter = document.getElementById("adm-filter");
  if (filter) {
    filter.oninput = () => {
      state.filter = filter.value;
      renderDashboard();
      document.getElementById("adm-filter")?.focus();
    };
  }

  document.getElementById("adm-close-detail")?.addEventListener("click", () => {
    state.detail = null;
    renderDashboard();
  });

  for (const node of document.querySelectorAll("[data-view]")) {
    node.onclick = () => openDetail(node.dataset.view);
  }

  for (const node of document.querySelectorAll("[data-del-user]")) {
    node.onclick = async () => {
      const username = node.dataset.delUser;
      if (!confirm(`Remove ${username}? Their account, race results and history are deleted. This cannot be undone.`)) {
        return;
      }
      await guard(`Removed ${username} from the game.`, () =>
        api(`/api/admin/user?name=${encodeURIComponent(username)}`, { method: "DELETE" })
      );
      if (state.detail?.username === username) state.detail = null;
      await refreshKeepingNote();
    };
  }

  for (const node of document.querySelectorAll("[data-reset-room]")) {
    node.onclick = async () => {
      const code = node.dataset.resetRoom;
      if (!confirm(`Reset room ${code}? Current attempts are cleared. Players stay, and lifetime wins do not change.`)) {
        return;
      }
      await guard(`Reset race ${code}. Players can start again.`, () =>
        api("/api/admin/reset", { method: "POST", body: { code } })
      );
      await refreshKeepingNote();
    };
  }

  for (const node of document.querySelectorAll("[data-del-room]")) {
    node.onclick = async () => {
      const code = node.dataset.delRoom;
      if (!confirm(`Delete room ${code}? The race and its current scoreboard go away. Player accounts remain.`)) return;
      await guard(`Deleted room ${code}.`, () =>
        api(`/api/admin/room?code=${encodeURIComponent(code)}`, { method: "DELETE" })
      );
      await refreshKeepingNote();
    };
  }

  for (const node of document.querySelectorAll("[data-clear-run]")) {
    node.onclick = async () => {
      const code = node.dataset.clearRun;
      const username = node.dataset.clearUser;
      if (!confirm(`Clear ${username}'s current attempt in ${code}? Their account and history stay.`)) return;
      await guard(`Cleared ${username}'s attempt in ${code}.`, () =>
        api(
          `/api/admin/run?code=${encodeURIComponent(code)}&name=${encodeURIComponent(username)}`,
          { method: "DELETE" }
        )
      );
      await refreshKeepingNote();
    };
  }
}

async function refreshKeepingNote() {
  const note = state.note;
  const error = state.error;
  await refresh();
  state.note = note;
  state.error = error;
  renderDashboard();
}

async function boot() {
  try {
    const session = await api("/api/admin/session");
    state.admin = Boolean(session.admin);
    state.configured = session.configured !== false;
  } catch (err) {
    state.error = err.message;
  }
  if (state.admin) await refresh();
  else renderGate();

  setInterval(() => {
    if (!state.admin || !state.data) return;
    if (state.tab !== "rooms" && !state.detail) return;
    if (!state.data.totals?.activeRuns) return;
    renderDashboard();
  }, 1000);

  setInterval(() => {
    if (!state.admin || !state.autoRefresh) return;
    if (document.activeElement?.id === "adm-filter") return;
    refresh();
  }, 10000);
}

boot();

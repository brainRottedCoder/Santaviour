import { api } from "./api.js";
import { state } from "./state.js";

export async function createRoom(name) {
  const data = await api("/api/rooms/create", {
    method: "POST",
    body: { name },
  });
  state.room = data.room;
  return data.room;
}

export async function joinRoom(code) {
  const data = await api("/api/rooms/join", {
    method: "POST",
    body: { code },
  });
  state.room = data.room;
  return data.room;
}

export async function loadRoom(code) {
  const data = await api(`/api/rooms/${encodeURIComponent(code)}`);
  state.room = data.room;
  return data.room;
}

function roomCode() {
  return state.room?.code;
}

export async function startRun() {
  if (!roomCode()) return;
  const data = await api("/api/runs/start", {
    method: "POST",
    body: {
      code: roomCode(),
      level: state.runLevel || 1,
      score: state.lastScore || 0,
    },
  });
  state.room = data.room;
  state.runActive = true;
  if (!state.localStartedAt) state.localStartedAt = Date.now();
  return data.room;
}

export async function checkpointRun(payload) {
  if (!roomCode()) return;
  const data = await api("/api/runs/checkpoint", {
    method: "POST",
    body: { code: roomCode(), ...payload },
  });
  state.room = data.room;
  return data.room;
}

export async function progressRun(payload) {
  if (!roomCode() || !state.runActive) return;
  const data = await api("/api/runs/progress", {
    method: "POST",
    body: { code: roomCode(), ...payload },
  });
  state.room = data.room;
  return data.room;
}

export async function finishRun(payload) {
  if (!roomCode()) return;
  const data = await api("/api/runs/finish", {
    method: "POST",
    body: {
      code: roomCode(),
      ...payload,
      score: Math.max(Number(payload.score || 0), state.lastScore || 0),
      level: payload.level || state.runLevel,
      kids: payload.kids ?? state.lastKids,
      keys: payload.keys ?? state.lastKeys,
      lives: payload.lives ?? state.lastLives,
      hp: payload.hp ?? state.lastHp,
    },
  });
  state.room = data.room;
  state.runActive = false;
  return data.room;
}

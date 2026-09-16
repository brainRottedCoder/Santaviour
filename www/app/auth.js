import { api } from "./api.js";
import { state } from "./state.js";

export async function refreshSession() {
  try {
    const data = await api("/api/auth/me");
    state.user = data.user;
    state.rooms = data.rooms || [];
    return data.user;
  } catch {
    state.user = null;
    state.rooms = [];
    return null;
  }
}

export async function register(username, password) {
  const data = await api("/api/auth/register", {
    method: "POST",
    body: { username, password },
  });
  state.user = data.user;
  return data.user;
}

export async function login(username, password) {
  const data = await api("/api/auth/login", {
    method: "POST",
    body: { username, password },
  });
  state.user = data.user;
  return data.user;
}

export async function logout() {
  await api("/api/auth/logout", { method: "POST" });
  state.user = null;
  state.rooms = [];
  state.room = null;
}

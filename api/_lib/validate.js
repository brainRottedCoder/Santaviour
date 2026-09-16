const USERNAME_RE = /^[A-Za-z0-9_]{3,16}$/;

export function normalizeUsername(username) {
  return String(username || "").trim();
}

export function usernameKey(username) {
  return normalizeUsername(username).toLowerCase();
}

export function assertUsername(username) {
  const value = normalizeUsername(username);
  if (!USERNAME_RE.test(value)) {
    throw Object.assign(new Error("Username must be 3-16 letters, numbers, or underscores"), {
      status: 400,
    });
  }
  return value;
}

export function assertPassword(password) {
  const value = String(password || "");
  if (value.length < 6 || value.length > 72) {
    throw Object.assign(new Error("Password must be 6-72 characters"), { status: 400 });
  }
  return value;
}

export function assertRoomCode(code) {
  const value = String(code || "")
    .trim()
    .toUpperCase();
  if (!/^[A-Z0-9]{4,8}$/.test(value)) {
    throw Object.assign(new Error("Invalid room code"), { status: 400 });
  }
  return value;
}

export function assertRoomName(name) {
  const value = String(name || "Christmas Challenge").trim().slice(0, 32);
  return value || "Christmas Challenge";
}

export function clampScore(score) {
  const n = Number(score);
  if (!Number.isFinite(n) || n < 0) return 0;
  return Math.min(Math.floor(n), 1_000_000);
}

export function clampLevel(level) {
  const n = Number(level);
  if (!Number.isFinite(n) || n < 1) return 1;
  return Math.min(Math.floor(n), 3);
}

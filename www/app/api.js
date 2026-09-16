export async function api(path, { method = "GET", body } = {}) {
  let res;
  try {
    res = await fetch(path, {
      method,
      credentials: "include",
      headers: body ? { "Content-Type": "application/json" } : undefined,
      body: body ? JSON.stringify(body) : undefined,
    });
  } catch {
    throw new Error("API unreachable. Run npx vercel dev so /api is on the same origin.");
  }
  let data = {};
  try {
    data = await res.json();
  } catch {
    data = {};
  }
  if (!res.ok) {
    throw new Error(data.error || `Request failed (${res.status})`);
  }
  return data;
}

export function formatTime(ms) {
  if (ms == null || ms === "" || !Number.isFinite(Number(ms))) return "--";
  const total = Math.max(0, Math.floor(Number(ms) / 1000));
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

export function formatRupees(n) {
  const value = Number(n);
  const amount = Number.isFinite(value) && value > 0 ? Math.floor(value) : 0;
  return `₹${amount}`;
}

export function method(req) {
  return (req.method || "GET").toUpperCase();
}

export function send(res, status, body) {
  res.statusCode = status;
  res.setHeader("Content-Type", "application/json; charset=utf-8");
  res.setHeader("Cache-Control", "no-store");
  res.end(JSON.stringify(body));
}

export function sendError(res, status, error) {
  send(res, status, { ok: false, error });
}

export async function readJson(req) {
  if (req.body) {
    if (typeof req.body === "string") {
      const raw = req.body.trim();
      return raw ? JSON.parse(raw) : {};
    }
    if (typeof req.body === "object" && !Buffer.isBuffer(req.body)) {
      return req.body;
    }
  }
  const chunks = [];
  for await (const chunk of req) chunks.push(chunk);
  if (chunks.length === 0) return {};
  const raw = Buffer.concat(chunks).toString("utf8").trim();
  if (!raw) return {};
  return JSON.parse(raw);
}

export function setCookie(res, name, value, { maxAge = 60 * 60 * 24 * 7, secure = false } = {}) {
  const parts = [
    `${name}=${value}`,
    "HttpOnly",
    "Path=/",
    "SameSite=Lax",
    `Max-Age=${maxAge}`,
  ];
  if (secure) parts.push("Secure");
  const prev = res.getHeader("Set-Cookie");
  const next = prev ? [].concat(prev, parts.join("; ")) : parts.join("; ");
  res.setHeader("Set-Cookie", next);
}

export function clearCookie(res, name, { secure = false } = {}) {
  setCookie(res, name, "", { maxAge: 0, secure });
}

export function isSecureRequest(req) {
  const proto = req.headers["x-forwarded-proto"];
  if (typeof proto === "string") return proto.split(",")[0].trim() === "https";
  return Boolean(req.socket && req.socket.encrypted);
}

export function getCookie(req, name) {
  const header = req.headers.cookie;
  if (!header) return null;
  const parts = header.split(";");
  for (const part of parts) {
    const [k, ...rest] = part.trim().split("=");
    if (k === name) return rest.join("=");
  }
  return null;
}

export function allowMethods(req, res, verbs) {
  if (!verbs.includes(method(req))) {
    res.setHeader("Allow", verbs.join(", "));
    sendError(res, 405, "Method not allowed");
    return false;
  }
  return true;
}

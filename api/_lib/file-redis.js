import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

const STORE_PATH = join(process.cwd(), ".data", "store.json");

function empty() {
  return { hashes: {}, sets: {}, lists: {} };
}

export class FileRedis {
  constructor(path = STORE_PATH) {
    this.path = path;
    this.db = this.read();
  }

  read() {
    try {
      const parsed = JSON.parse(readFileSync(this.path, "utf8"));
      return {
        hashes: parsed.hashes || {},
        sets: parsed.sets || {},
        lists: parsed.lists || {},
      };
    } catch {
      return empty();
    }
  }

  write() {
    mkdirSync(dirname(this.path), { recursive: true });
    writeFileSync(this.path, JSON.stringify(this.db, null, 2));
  }

  hash(key) {
    if (!this.db.hashes[key]) this.db.hashes[key] = {};
    return this.db.hashes[key];
  }

  set(key) {
    if (!this.db.sets[key]) this.db.sets[key] = [];
    return this.db.sets[key];
  }

  list(key) {
    if (!this.db.lists[key]) this.db.lists[key] = [];
    return this.db.lists[key];
  }

  async hset(key, fields) {
    this.db = this.read();
    Object.assign(this.hash(key), stringifyFields(fields));
    this.write();
    return "OK";
  }

  async hgetall(key) {
    this.db = this.read();
    return { ...(this.db.hashes[key] || {}) };
  }

  async hget(key, field) {
    this.db = this.read();
    const hash = this.db.hashes[key];
    return hash && field in hash ? hash[field] : null;
  }

  async sadd(key, member) {
    this.db = this.read();
    const set = this.set(key);
    if (!set.includes(member)) {
      set.push(member);
      this.write();
      return 1;
    }
    return 0;
  }

  async smembers(key) {
    this.db = this.read();
    return [...(this.db.sets[key] || [])];
  }

  async scard(key) {
    this.db = this.read();
    return (this.db.sets[key] || []).length;
  }

  async sismember(key, member) {
    this.db = this.read();
    return (this.db.sets[key] || []).includes(member) ? 1 : 0;
  }

  async srem(key, member) {
    this.db = this.read();
    const set = this.db.sets[key];
    const at = set ? set.indexOf(member) : -1;
    if (at === -1) return 0;
    set.splice(at, 1);
    this.write();
    return 1;
  }

  async lpush(key, value) {
    this.db = this.read();
    const list = this.list(key);
    list.unshift(value == null ? "" : String(value));
    this.write();
    return list.length;
  }

  async lrange(key, start, stop) {
    this.db = this.read();
    return slice(this.db.lists[key] || [], start, stop);
  }

  async ltrim(key, start, stop) {
    this.db = this.read();
    if (!this.db.lists[key]) return "OK";
    this.db.lists[key] = slice(this.db.lists[key], start, stop);
    this.write();
    return "OK";
  }

  async del(...keys) {
    this.db = this.read();
    let removed = 0;
    for (const key of keys.flat()) {
      for (const bucket of [this.db.hashes, this.db.sets, this.db.lists]) {
        if (key in bucket) {
          delete bucket[key];
          removed += 1;
        }
      }
    }
    if (removed) this.write();
    return removed;
  }

  async keys(pattern) {
    this.db = this.read();
    const re = globToRegExp(pattern);
    const all = new Set([
      ...Object.keys(this.db.hashes),
      ...Object.keys(this.db.sets),
      ...Object.keys(this.db.lists),
    ]);
    return [...all].filter((key) => re.test(key));
  }
}

function slice(list, start, stop) {
  const from = start < 0 ? Math.max(list.length + start, 0) : start;
  const to = stop < 0 ? list.length + stop : Math.min(stop, list.length - 1);
  return to < from ? [] : list.slice(from, to + 1);
}

function globToRegExp(pattern) {
  const escaped = String(pattern ?? "*").replace(/[.*+?^${}()|[\]\\]/g, (ch) =>
    ch === "*" ? "\u0000" : `\\${ch}`
  );
  return new RegExp(`^${escaped.replace(/\u0000/g, ".*")}$`);
}

function stringifyFields(fields) {
  const out = {};
  for (const [k, v] of Object.entries(fields || {})) {
    out[k] = v == null ? "" : String(v);
  }
  return out;
}

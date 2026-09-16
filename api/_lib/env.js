import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

let loaded = false;

function applyEnvFile(path) {
  try {
    const text = readFileSync(path, "utf8");
    for (const line of text.split(/\r?\n/)) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith("#")) continue;
      const eq = trimmed.indexOf("=");
      if (eq === -1) continue;
      const key = trimmed.slice(0, eq).trim();
      let value = trimmed.slice(eq + 1).trim();
      if (
        (value.startsWith('"') && value.endsWith('"')) ||
        (value.startsWith("'") && value.endsWith("'"))
      ) {
        value = value.slice(1, -1);
      }
      if (process.env[key] == null || process.env[key] === "") {
        process.env[key] = value;
      }
    }
  } catch {
    /* optional */
  }
}

export function loadLocalEnv() {
  if (loaded) return;
  loaded = true;
  const roots = [
    process.cwd(),
    join(dirname(fileURLToPath(import.meta.url)), "..", ".."),
  ];
  for (const root of roots) {
    applyEnvFile(join(root, ".env.local"));
    applyEnvFile(join(root, ".env"));
  }
}

loadLocalEnv();

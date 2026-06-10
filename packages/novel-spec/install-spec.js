#!/usr/bin/env node
const { execFileSync, spawnSync } = require("child_process");
const fs = require("fs");
const path = require("path");

const target = process.argv[2] || process.cwd();
const specDir = path.resolve(__dirname);
const cliPkg = path.resolve(specDir, "..", "novel-lifecycle-cli");

// ── helpers ──────────────────────────────────────────────────────────────────

function cpDir(src, dst) {
  if (!fs.existsSync(src)) return;
  fs.mkdirSync(dst, { recursive: true });
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    if (entry.name === ".gitkeep") continue;
    const s = path.join(src, entry.name);
    const d = path.join(dst, entry.name);
    entry.isDirectory() ? cpDir(s, d) : fs.copyFileSync(s, d);
  }
}

// Deep-merge src into dst object (only plain objects recurse; arrays replace)
function deepMerge(dst, src) {
  for (const [k, v] of Object.entries(src)) {
    if (v && typeof v === "object" && !Array.isArray(v) &&
        dst[k] && typeof dst[k] === "object" && !Array.isArray(dst[k])) {
      deepMerge(dst[k], v);
    } else {
      dst[k] = v;
    }
  }
}

// ── 5.7 python3 check ────────────────────────────────────────────────────────

function checkPython() {
  const r = spawnSync("python3", ["-c", "import sys; print(sys.version_info[:2])"], { encoding: "utf8" });
  if (r.status !== 0) throw new Error("python3 not found. Please install python3 >= 3.8 before running install.");
  const m = r.stdout.match(/\((\d+),\s*(\d+)\)/);
  if (!m || parseInt(m[1]) < 3 || (parseInt(m[1]) === 3 && parseInt(m[2]) < 8))
    throw new Error(`python3 >= 3.8 required, found: ${r.stdout.trim()}`);
}

// ── 5.3 copy soft-layer primitives ───────────────────────────────────────────

function copySoftLayer() {
  for (const dir of ["skills", "agents", "hooks", "rules"]) {
    cpDir(path.join(specDir, dir), path.join(target, ".claude", dir));
  }
  // chmod hook scripts
  const hooksDir = path.join(target, ".claude", "hooks");
  if (fs.existsSync(hooksDir)) {
    for (const f of fs.readdirSync(hooksDir)) {
      if (f.endsWith(".sh")) fs.chmodSync(path.join(hooksDir, f), 0o755);
    }
  }
}

// ── 5.4 register hooks in settings.json (idempotent deep-merge) ──────────────

function registerHooks() {
  const settingsPath = path.join(target, ".claude", "settings.json");
  let existing = {};
  if (fs.existsSync(settingsPath)) {
    try { existing = JSON.parse(fs.readFileSync(settingsPath, "utf8")); } catch {}
  }

  const hookConfig = {
    hooks: {
      PreToolUse: [
        {
          matcher: "Write",
          hooks: [{ type: "command", command: ".claude/hooks/pre-write-chapter.sh" }]
        },
        {
          matcher: "Bash",
          hooks: [{ type: "command", command: ".claude/hooks/guard-db-write.sh" }]
        }
      ],
      PostToolUse: [
        {
          matcher: "Write",
          hooks: [
            { type: "command", command: ".claude/hooks/post-prose-converge-signal.sh" },
            { type: "command", command: ".claude/hooks/post-prose-sensitive-scan.sh" }
          ]
        }
      ]
    }
  };

  deepMerge(existing, hookConfig);
  fs.mkdirSync(path.join(target, ".claude"), { recursive: true });
  fs.writeFileSync(settingsPath, JSON.stringify(existing, null, 2) + "\n");
}

// ── 5.5 copy templates (no overwrite) ────────────────────────────────────────

function copyTemplates() {
  for (const f of ["CLAUDE.md", "introduce.md"]) {
    const src = path.join(specDir, "templates", f);
    const dst = path.join(target, f);
    if (!fs.existsSync(dst) && fs.existsSync(src)) fs.copyFileSync(src, dst);
  }
}

// ── 5.6 delegate to hard-layer installer ─────────────────────────────────────

function installHardLayer() {
  execFileSync("node", [path.join(cliPkg, "install-cli.js"), target], { stdio: "inherit" });
}

// ── main ──────────────────────────────────────────────────────────────────────

checkPython();
copySoftLayer();
registerHooks();
copyTemplates();
installHardLayer();

console.log(`\nnovel-spec installed to ${target}`);
console.log("  .claude/{skills,agents,hooks,rules,settings.json}");
console.log("  scripts/novel-cli  +  index/novel.db");
console.log("  CLAUDE.md  +  introduce.md (if not already present)");

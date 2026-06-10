#!/usr/bin/env node
const { execFileSync } = require("child_process");
const fs = require("fs");
const path = require("path");

const target = process.argv[2] || process.cwd();
const pkgDir = path.resolve(__dirname);

function cpDir(src, dst) {
  fs.mkdirSync(dst, { recursive: true });
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const s = path.join(src, entry.name);
    const d = path.join(dst, entry.name);
    if (entry.isDirectory()) cpDir(s, d);
    else fs.copyFileSync(s, d);
  }
}

// 1. Copy bin/ and src/ to <target>/scripts/
cpDir(path.join(pkgDir, "bin"), path.join(target, "scripts", "bin"));
cpDir(path.join(pkgDir, "src"), path.join(target, "scripts", "src"));

// 2. Create top-level scripts/novel-cli symlink / copy
const cliSrc = path.join(pkgDir, "bin", "novel-cli");
const cliDst = path.join(target, "scripts", "novel-cli");
if (fs.existsSync(cliDst)) fs.unlinkSync(cliDst);
fs.copyFileSync(cliSrc, cliDst);
fs.chmodSync(cliDst, 0o755);

// 3. Initialize DB
const dbPath = path.join(target, "index", "novel.db");
execFileSync("python3", [cliDst, "init", "--db-path", dbPath], { stdio: "inherit" });

console.log(`novel-lifecycle-cli installed to ${target}`);

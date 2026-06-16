#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

const root = path.resolve(__dirname, '..');
const softRoot = path.join(root, 'packages/novel-spec');
const hardManual = path.join(root, 'packages/novel-lifecycle-cli/docs/CLI使用手册.md');
const softManual = path.join(root, 'packages/novel-spec/docs/CLI使用手册.md');

const allowedCodeFiles = new Set([
  path.join(root, 'packages/novel-spec/hooks/guard-db-write.sh'),
]);

const codeExts = new Set(['.js', '.sh', '.md']);
const violations = [];

function rel(p) {
  return path.relative(root, p).replaceAll(path.sep, '/');
}

function walk(dir, out = []) {
  for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
    if (['node_modules', '.git', '.pytest_cache'].includes(ent.name)) continue;
    const p = path.join(dir, ent.name);
    if (ent.isDirectory()) walk(p, out);
    else out.push(p);
  }
  return out;
}

function add(file, lineNo, rule, line) {
  violations.push({ file: rel(file), line: lineNo, rule, content: line.trim() });
}

function checkCode(file) {
  if (!codeExts.has(path.extname(file))) return;
  if (allowedCodeFiles.has(file)) return;
  if (file.startsWith(path.join(softRoot, 'docs') + path.sep)) return;
  const lines = fs.readFileSync(file, 'utf8').split(/\r?\n/);
  lines.forEach((line, idx) => {
    if (/\b(import|from)\s+sqlite3\b|require\(['"]sqlite3['"]\)|\bsqlite3\s+/.test(line)) {
      add(file, idx + 1, 'direct-sqlite-access', line);
    }
    if (/sys\.path|PYTHONPATH/.test(line) && /novel-lifecycle-cli|lifecycle|packages/.test(line)) {
      add(file, idx + 1, 'hard-layer-path-injection', line);
    }
    if (/from\s+.*novel[-_]lifecycle|import\s+.*novel[-_]lifecycle|src\/novel|src\/elements|src\/core/.test(line)) {
      add(file, idx + 1, 'hard-layer-internal-import', line);
    }
  });
}

function checkDocs(file) {
  if (path.extname(file) !== '.md') return;
  const lines = fs.readFileSync(file, 'utf8').split(/\r?\n/);
  const mdLink = /\[[^\]]+\]\(([^)]+)\)/g;
  lines.forEach((line, idx) => {
    let m;
    while ((m = mdLink.exec(line)) !== null) {
      const target = decodeURI(m[1].split('#')[0]);
      if (!target) continue;
      if (/novel-lifecycle-cli\/docs\//.test(target) && !/CLI使用手册\.md$/.test(target)) {
        add(file, idx + 1, 'soft-doc-links-hard-internal-doc', line);
      }
    }
  });
}

function hash(file) {
  return crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
}

for (const file of walk(softRoot)) {
  checkCode(file);
  checkDocs(file);
}

if (!fs.existsSync(hardManual) || !fs.existsSync(softManual)) {
  violations.push({ file: 'packages', line: 0, rule: 'missing-cli-manual-copy', content: 'CLI 使用手册主副本或软层副本不存在' });
} else if (hash(hardManual) !== hash(softManual)) {
  violations.push({
    file: rel(softManual),
    line: 0,
    rule: 'cli-manual-out-of-sync',
    content: '软层 CLI 使用手册副本与硬层主副本不一致；请运行 npm run sync-cli-manual',
  });
}

if (violations.length) {
  console.error('Layer isolation violations found:');
  for (const v of violations) {
    const loc = v.line ? `${v.file}:${v.line}` : v.file;
    console.error(`- ${loc} [${v.rule}] ${v.content}`);
  }
  process.exit(1);
}

console.log('Layer isolation OK: soft layer uses CLI contract only, and CLI manuals are in sync.');

#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');

const root = path.resolve(__dirname, '..');
const source = path.join(root, 'packages/novel-lifecycle-cli/docs/CLI使用手册.md');
const target = path.join(root, 'packages/novel-spec/docs/CLI使用手册.md');

fs.copyFileSync(source, target);
console.log(`Synced ${path.relative(root, source)} -> ${path.relative(root, target)}`);

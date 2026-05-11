#!/usr/bin/env node
/**
 * Build SpoofDPI 1.2.1 (xzual-proxy) and copy to spoofdpi/ and src-tauri/binaries/.
 * Requires Go in PATH.
 */
const { spawnSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const root = path.resolve(__dirname, '..');
const spoofDpiDir = path.join(root, 'SpoofDPI-1.2.1', 'SpoofDPI-1.2.1');
// Use relative path for output to avoid issues with Turkish characters in absolute path
const outExeRelative = path.join('..', '..', 'spoofdpi', 'xzual-proxy.exe');
const outExeAbsolute = path.join(root, 'spoofdpi', 'xzual-proxy.exe');

if (!fs.existsSync(path.join(spoofDpiDir, 'go.mod'))) {
  console.error('SpoofDPI-1.2.1 source not found at', spoofDpiDir);
  process.exit(1);
}

const spoofdpiDir = path.join(root, 'spoofdpi');
if (!fs.existsSync(spoofdpiDir)) {
  fs.mkdirSync(spoofdpiDir, { recursive: true });
}

console.log('Building SpoofDPI (xzual-proxy)...');
const go = spawnSync('go', ['build', '-trimpath', '-ldflags="-s -w"', '-o', outExeRelative, './cmd/spoofdpi'], {
  cwd: spoofDpiDir,
  stdio: 'inherit',
  shell: true,
});

if (go.status !== 0) {
  console.error('go build failed');
  process.exit(go.status || 1);
}

console.log('Build OK:', outExeAbsolute);
console.log('Copying to src-tauri/binaries/...');
const copy = spawnSync('node', [path.join(__dirname, 'copy-proxy.cjs')], {
  cwd: root,
  stdio: 'inherit',
});
process.exit(copy.status || 0);

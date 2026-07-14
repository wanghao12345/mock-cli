#!/usr/bin/env node
// Dynamically load the platform-specific prebuilt binary package.
const path = require('path');
const fs = require('fs');
const { spawn } = require('child_process');

const platformPkg = `@mock-cli/${process.platform}-${process.arch}`;

// Resolve the installed platform package directory.
let pkgDir;
try {
  const pkgJsonPath = require.resolve(`${platformPkg}/package.json`);
  pkgDir = path.dirname(pkgJsonPath);
} catch (e) {
  console.error(
    `mock-cli: No prebuilt binary for ${process.platform}-${process.arch}.\n` +
    `Please check if this platform is supported, or build from source with: cargo install --path crates/cli`
  );
  process.exit(1);
}

// Windows binaries have an .exe suffix.
const binaryName = process.platform === 'win32' ? 'mock-cli.exe' : 'mock-cli';
const binaryPath = path.join(pkgDir, binaryName);

if (!fs.existsSync(binaryPath)) {
  console.error(`mock-cli: Binary not found: ${binaryPath}`);
  process.exit(1);
}

// Ensure the binary is executable. npm tarballs may not preserve the execute
// permission bit across publish/install, so we force it here on Unix.
// On Windows, execute permission is governed by the .exe extension, not the bit.
if (process.platform !== 'win32') {
  try {
    fs.chmodSync(binaryPath, 0o755);
  } catch (e) {
    // If chmod fails (e.g. read-only location), fall through and let spawn
    // surface the error — there's nothing else we can do here.
  }
}

// Forward all command-line arguments to the binary.
const child = spawn(binaryPath, process.argv.slice(2), { stdio: 'inherit' });
child.on('exit', (code) => process.exit(code ?? 1));

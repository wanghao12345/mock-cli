#!/bin/bash
# Publish script for npm platform sub-packages.
#
# Usage:
#   ./npm/publish.sh <version>
#   ./npm/publish.sh 0.1.1
#
# Prerequisites:
#   1. Logged in to npm via `pnpm login`
#   2. The GitHub Release for the target version is published (with platform tarballs)
#   3. curl, tar, jq, and pnpm are installed in the environment

set -e

# ===== Argument validation =====
VERSION="${1:-}"
if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 0.1.1"
  exit 1
fi

# ===== Configuration =====
REPO="wanghao12345/mock-cli"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PLATFORMS_DIR="$SCRIPT_DIR/platforms"
MAIN_PKG_DIR="$SCRIPT_DIR/mock-cli"

# Mapping: npm platform name -> Rust target triple
declare -A TARGETS=(
  ["darwin-arm64"]="aarch64-apple-darwin"
  ["darwin-x64"]="x86_64-apple-darwin"
  ["linux-arm64"]="aarch64-unknown-linux-gnu"
  ["linux-x64"]="x86_64-unknown-linux-gnu"
  ["win32-x64"]="x86_64-pc-windows-msvc"
)

# ===== Helper functions =====
log()   { echo "▸ $1"; }
ok()    { echo "✓ $1"; }
fail()  { echo "✗ $1" >&2; exit 1; }

# ===== Dependency checks =====
for cmd in curl tar jq pnpm; do
  command -v "$cmd" >/dev/null 2>&1 || fail "$cmd is not installed, please install it first"
done

# Verify npm login status
pnpm whoami >/dev/null 2>&1 || fail "Not logged in to npm, please run: pnpm login"

log "Publishing mock-cli@${VERSION} to npm"

# ===== Publish platform sub-packages =====
for platform in "${!TARGETS[@]}"; do
  target=${TARGETS[$platform]}
  pkg_dir="$PLATFORMS_DIR/$platform"
  binary_name="mock-cli"
  [ "$platform" = "win32-x64" ] && binary_name="mock-cli.exe"

  log "Processing platform package: @mock-cli/${platform}"

  # Remove any stale binary
  rm -f "$pkg_dir/$binary_name"

  # Download and extract the binary from the GitHub Release
  asset_url="https://github.com/${REPO}/releases/download/v${VERSION}/mock-cli-${target}-v${VERSION}.tar.gz"
  log "  Downloading: $asset_url"

  # Download to a temp file before extracting (handles GitHub's 302 redirect)
  tmp_tar=$(mktemp -t mock-cli.XXXXXX.tar.gz)
  trap 'rm -f "$tmp_tar"' EXIT

  curl -fSL "$asset_url" -o "$tmp_tar" || fail "Download failed: $asset_url"
  tar -xzf "$tmp_tar" -C "$pkg_dir" || fail "Extraction failed"

  # Ensure the binary exists and is executable (non-Windows)
  [ "$platform" != "win32-x64" ] && chmod +x "$pkg_dir/$binary_name"
  [ -f "$pkg_dir/$binary_name" ] || fail "Binary not found: $pkg_dir/$binary_name"

  ok "  Binary ready: $pkg_dir/$binary_name"

  # Publish the platform sub-package (--no-git-checks because the binary is not in git)
  (cd "$pkg_dir" && pnpm publish --access public --no-git-checks) || fail "Publish failed: @mock-cli/${platform}"

  ok "  Published: @mock-cli/${platform}@${VERSION}"

  # Clean up the binary to avoid polluting the git working tree
  rm -f "$pkg_dir/$binary_name"
done

# ===== Publish the main package =====
log "Publishing main package: mock-cli@${VERSION}"
(cd "$MAIN_PKG_DIR" && pnpm publish --access public --no-git-checks) || fail "Main package publish failed"
ok "Published: mock-cli@${VERSION}"

ok "All done! Users can now install with: npm install -g mock-cli"

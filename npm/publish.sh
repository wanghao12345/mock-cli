#!/bin/bash
# Publish script for npm platform sub-packages and main package.
#
# Usage:
#   ./npm/publish.sh <version> [--main-only]
#   ./npm/publish.sh 0.1.1              # publish all (5 platforms + main)
#   ./npm/publish.sh 0.1.1 --main-only  # publish only the main package
#
# Prerequisites:
#   1. A Granular Access Token configured in ~/.npmrc (see README for setup).
#      npm 2026 security policy requires Granular Access Tokens; classic tokens
#      and OTP-based login are no longer supported for publishing.
#   2. The GitHub Release for the target version is published (with platform tarballs).
#   3. curl, tar, unzip, and pnpm are installed in the environment.

set -e

# ===== Helper functions =====
log()   { echo "▸ $1"; }
ok()    { echo "✓ $1"; }
fail()  { echo "✗ $1" >&2; exit 1; }

# ===== Argument validation =====
VERSION="${1:-}"
if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version> [--main-only]"
  echo "Example: $0 0.1.1"
  echo "Example: $0 0.1.1 --main-only"
  exit 1
fi
shift

# Parse optional flags
MAIN_ONLY=0
while [ $# -gt 0 ]; do
  case "$1" in
    --main-only)
      MAIN_ONLY=1
      shift
      ;;
    *)
      fail "Unknown argument: $1"
      ;;
  esac
done

# ===== Configuration =====
REPO="wanghao12345/mock-cli"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PLATFORMS_DIR="$SCRIPT_DIR/platforms"
MAIN_PKG_DIR="$SCRIPT_DIR/mock-cli"

# Mapping: npm platform name -> Rust target triple
# Format: "npm-platform rust-target" (space-separated, parsed below)
# Uses an indexed array for bash 3.2 compatibility (macOS default).
PLATFORM_TARGETS=(
  "darwin-arm64 aarch64-apple-darwin"
  "darwin-x64 x86_64-apple-darwin"
  "linux-arm64 aarch64-unknown-linux-gnu"
  "linux-x64 x86_64-unknown-linux-gnu"
  "win32-x64 x86_64-pc-windows-msvc"
)

# Publish flags: the Granular Access Token in ~/.npmrc handles authentication,
# so no --otp flag is needed.
PUBLISH_FLAGS="--access public --no-git-checks"

# ===== Dependency checks =====
# Only check for curl/tar/unzip when platform packages need to be downloaded.
if [ "$MAIN_ONLY" = "0" ]; then
  for cmd in curl tar unzip pnpm; do
    command -v "$cmd" >/dev/null 2>&1 || fail "$cmd is not installed, please install it first"
  done
else
  for cmd in pnpm; do
    command -v "$cmd" >/dev/null 2>&1 || fail "$cmd is not installed, please install it first"
  done
fi

# Verify npm authentication via the configured Granular Access Token.
# pnpm whoami reads the token from ~/.npmrc; if it fails, the token is missing
# or expired (tokens expire after 90 days per npm 2026 policy).
if ! pnpm whoami >/dev/null 2>&1; then
  fail "Not authenticated to npm. Configure a Granular Access Token:
  1. Go to https://www.npmjs.com/settings/<username>/tokens (New Token -> Granular Access Token)
  2. Select 'Read and write' with 'Bypass 2FA for automation' enabled
  3. Run: npm config set //registry.npmjs.org/:_authToken=<YOUR_TOKEN>"
fi

log "Publishing @mock-cli/server@${VERSION} to npm"

# ===== Publish platform sub-packages =====
if [ "$MAIN_ONLY" = "0" ]; then
  for entry in "${PLATFORM_TARGETS[@]}"; do
    # Split "npm-platform rust-target" into two variables
    platform="${entry% *}"
    target="${entry#* }"
    pkg_dir="$PLATFORMS_DIR/$platform"
    binary_name="mock-cli"
    is_windows=0
    if [ "$platform" = "win32-x64" ]; then
      binary_name="mock-cli.exe"
      is_windows=1
    fi

    log "Processing platform package: @mock-cli/${platform}"

    # Remove any stale binary
    rm -f "$pkg_dir/$binary_name"

    # Build the asset URL.
    # cargo-dist names assets as "<cargo-pkg-name>-<target>.<ext>" (no version in the name).
    # Unix uses .tar.xz, Windows uses .zip.
    if [ "$is_windows" = "1" ]; then
      asset_name="cli-${target}.zip"
    else
      asset_name="cli-${target}.tar.xz"
    fi
    asset_url="https://github.com/${REPO}/releases/download/v${VERSION}/${asset_name}"
    log "  Downloading: $asset_url"

    # Download to a temp file before extracting (handles GitHub's 302 redirect).
    tmp_archive=$(mktemp -t "mock-cli.${platform}.XXXXXX")
    trap 'rm -f "$tmp_archive"' EXIT

    curl -fSL "$asset_url" -o "$tmp_archive" || fail "Download failed: $asset_url"

    # Extract the archive.
    # Unix tarballs have a top-level dir (e.g. cli-aarch64-apple-darwin/), so we
    # extract to a temp dir and then copy the binary out.
    # Windows zips have the binary at the root, so we extract directly into pkg_dir.
    if [ "$is_windows" = "1" ]; then
      unzip -o "$tmp_archive" "$binary_name" -d "$pkg_dir" >/dev/null || fail "Extraction failed"
    else
      extract_dir=$(mktemp -d -t "mock-cli.extract.XXXXXX")
      trap 'rm -f "$tmp_archive"; rm -rf "$extract_dir"' EXIT
      tar -xJf "$tmp_archive" -C "$extract_dir" || fail "Extraction failed"
      # Find the binary inside the top-level directory
      found_bin=$(find "$extract_dir" -name "$binary_name" -type f | head -1)
      [ -n "$found_bin" ] || fail "Binary not found in archive: $binary_name"
      cp "$found_bin" "$pkg_dir/$binary_name" || fail "Failed to copy binary"
    fi

    # Ensure the binary exists and is executable (non-Windows)
    [ "$is_windows" = "0" ] && chmod +x "$pkg_dir/$binary_name"
    [ -f "$pkg_dir/$binary_name" ] || fail "Binary not found: $pkg_dir/$binary_name"

    ok "  Binary ready: $pkg_dir/$binary_name"

    # Publish the platform sub-package (--no-git-checks because the binary is not in git)
    (cd "$pkg_dir" && pnpm publish $PUBLISH_FLAGS) || fail "Publish failed: @mock-cli/${platform}"

    ok "  Published: @mock-cli/${platform}@${VERSION}"

    # Clean up the binary to avoid polluting the git working tree
    rm -f "$pkg_dir/$binary_name"
  done
else
  log "Skipping platform packages (--main-only)"
fi

# ===== Publish the main package =====
log "Publishing main package: @mock-cli/server@${VERSION}"
(cd "$MAIN_PKG_DIR" && pnpm publish $PUBLISH_FLAGS) || fail "Main package publish failed"
ok "Published: @mock-cli/server@${VERSION}"

ok "All done! Users can now install with: npm install -g @mock-cli/server"

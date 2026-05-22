#!/usr/bin/env bash
set -euo pipefail

REPO="https://github.com/FacileStudio/opus-cli.git"
BIN="opus"

c_cyan='\033[0;36m'
c_green='\033[0;32m'
c_red='\033[0;31m'
c_bold='\033[1m'
c_reset='\033[0m'

info()    { printf "${c_cyan}${c_bold}▸${c_reset} %s\n" "$1"; }
success() { printf "${c_green}${c_bold}✓${c_reset} %s\n" "$1"; }
fail()    { printf "${c_red}${c_bold}✗${c_reset} %s\n" "$1" >&2; exit 1; }

command -v git   >/dev/null 2>&1 || fail "git not found"
command -v cargo >/dev/null 2>&1 || fail "cargo not found — install Rust from https://rustup.rs"

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

info "Cloning opus-cli..."
git clone --depth 1 --quiet "$REPO" "$TMPDIR/opus-cli"

info "Building (release)..."
cargo install --path "$TMPDIR/opus-cli" --force --quiet 2>&1

LOCATION=$(command -v "$BIN" 2>/dev/null || echo "$HOME/.cargo/bin/$BIN")
success "Installed $BIN → $LOCATION"

case ":$PATH:" in
  *":$HOME/.cargo/bin:"*) ;;
  *) printf "\n${c_cyan}Add ~/.cargo/bin to your PATH:${c_reset}\n  export PATH=\"\$HOME/.cargo/bin:\$PATH\"\n\n" ;;
esac

info "Run 'opus' to get started, 'opus upgrade' to update later"

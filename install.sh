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

SKILL_MARKER_START="<!-- opus:start -->"
SKILL_MARKER_END="<!-- opus:end -->"

inject_block() {
  local file="$1"
  local content="$2"
  local block
  block="$(printf '%s\n%s\n%s' "$SKILL_MARKER_START" "$content" "$SKILL_MARKER_END")"

  if [ ! -f "$file" ]; then
    printf '%s\n' "$block" > "$file"
    return
  fi

  if grep -qF "$SKILL_MARKER_START" "$file"; then
    local tmp
    tmp="$(mktemp)"
    awk -v start="$SKILL_MARKER_START" -v end="$SKILL_MARKER_END" '
      $0 == start { skip=1; next }
      $0 == end   { skip=0; next }
      !skip       { print }
    ' "$file" > "$tmp"
    mv "$tmp" "$file"
    printf '\n%s\n' "$block" >> "$file"
  else
    printf '\n%s\n' "$block" >> "$file"
  fi
}

register_skill() {
  local repo_dir="$1"
  local skill_file="$repo_dir/integrations/SKILL.md"

  [ -f "$skill_file" ] || return 0

  local skill_content
  skill_content="$(cat "$skill_file")"

  if command -v claude &>/dev/null; then
    local skill_dir="$HOME/.claude/skills/opus"
    mkdir -p "$skill_dir"
    cp "$skill_file" "$skill_dir/SKILL.md"
    inject_block "$HOME/.claude/CLAUDE.md" "$skill_content"
    info "  ✓ Claude Code skill registered"
  fi

  if command -v codex &>/dev/null; then
    mkdir -p "$HOME/.codex"
    inject_block "$HOME/.codex/AGENTS.md" "$skill_content"
    info "  ✓ Codex skill registered"
  fi
}

command -v git   >/dev/null 2>&1 || fail "git not found"
command -v cargo >/dev/null 2>&1 || fail "cargo not found — install Rust from https://rustup.rs"

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

info "Cloning opus-cli..."
git clone --depth 1 --quiet "$REPO" "$TMPDIR/opus-cli"

info "Building (release)..."
cargo install --path "$TMPDIR/opus-cli" --force --quiet 2>&1

register_skill "$TMPDIR/opus-cli"

LOCATION=$(command -v "$BIN" 2>/dev/null || echo "$HOME/.cargo/bin/$BIN")
success "Installed $BIN → $LOCATION"

case ":$PATH:" in
  *":$HOME/.cargo/bin:"*) ;;
  *) printf "\n${c_cyan}Add ~/.cargo/bin to your PATH:${c_reset}\n  export PATH=\"\$HOME/.cargo/bin:\$PATH\"\n\n" ;;
esac

info "Run 'opus' to get started, 'opus upgrade' to update later"

#!/usr/bin/env bash
# Generates a VHS (https://github.com/charmbracelet/vhs) .tape file for each of the
# Blargg cpu_instrs individual test ROMs, then runs `vhs` on each one to produce a
# clean pass/fail GIF under docs/gifs/. Run from anywhere; paths are resolved
# relative to the repo root.
#
# Requires: vhs, ttyd, ffmpeg, and a Chrome/Chromium install (VHS renders via CDP).
# Run this from your own terminal (not a sandboxed one) if VHS hangs — it needs a
# real Chrome DevTools Protocol connection to a local browser.
#
# Usage: bash docs/vhs/generate-cpu-gifs.sh

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

# git-bash doesn't source the PowerShell/cmd profile that puts cargo on PATH,
# so try rustup's default install location, then fall back to asking Windows
# where it actually is (handles a custom CARGO_HOME).
if ! command -v cargo >/dev/null 2>&1; then
  if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
  elif [ -d "${USERPROFILE:-}/.cargo/bin" ]; then
    export PATH="$USERPROFILE/.cargo/bin:$PATH"
  fi
fi

if ! command -v cargo >/dev/null 2>&1 && command -v where.exe >/dev/null 2>&1; then
  # where.exe prints Windows-style "C:\foo\bar\cargo.exe" paths; convert
  # backslashes to forward slashes first or dirname/PATH won't split it.
  cargo_path="$(where.exe cargo 2>/dev/null | head -n1 | tr -d '\r' | tr '\\' '/')"
  if [ -n "$cargo_path" ]; then
    export PATH="$(dirname "$cargo_path"):$PATH"
  fi
fi

# Windows-only: ttyd hangs vhs silently unless it's given an absolute -w
# working-dir flag, which vhs never passes on Windows
# (https://github.com/charmbracelet/vhs/issues/631). Shim ttyd.cmd (next to
# this script) ahead of the real ttyd.exe on PATH so it gets that flag.
if command -v where.exe >/dev/null 2>&1; then
  export PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd):$PATH"
fi

cargo build --release

mkdir -p docs/vhs docs/gifs

# "<rom path relative to repo root>|<gif slug>"
roms=(
  "blargg/cpu_instrs/individual/01-special.gb|01-special"
  "blargg/cpu_instrs/individual/02-interrupts.gb|02-interrupts"
  "blargg/cpu_instrs/individual/03-op sp,hl.gb|03-op-sp-hl"
  "blargg/cpu_instrs/individual/04-op r,imm.gb|04-op-r-imm"
  "blargg/cpu_instrs/individual/05-op rp.gb|05-op-rp"
  "blargg/cpu_instrs/individual/06-ld r,r.gb|06-ld-r-r"
  "blargg/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb|07-jr-jp-call-ret-rst"
  "blargg/cpu_instrs/individual/08-misc instrs.gb|08-misc-instrs"
  "blargg/cpu_instrs/individual/09-op r,r.gb|09-op-r-r"
  "blargg/cpu_instrs/individual/10-bit ops.gb|10-bit-ops"
  "blargg/cpu_instrs/individual/11-op a,(hl).gb|11-op-a-hl"
)

for entry in "${roms[@]}"; do
  rom_path="${entry%%|*}"
  slug="${entry##*|}"
  tape="docs/vhs/${slug}.tape"
  gif="docs/gifs/${slug}.gif"

  cat > "$tape" <<EOF
Output ${gif}

Set Shell "bash"
Set FontSize 16
Set Width 900
Set Height 480
Set Padding 20
Set TypingSpeed 30ms
Set Theme "Catppuccin Mocha"

Sleep 1s
Type \`./target/release/rusty_gameboy_emulator.exe "${rom_path}"\`
Enter
Sleep 3s
Sleep 4s
EOF

  echo "=== Recording ${slug} ==="
  vhs "$tape"
done

echo "Done. GIFs written to docs/gifs/."

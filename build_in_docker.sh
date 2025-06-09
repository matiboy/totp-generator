#!/usr/bin/env bash
set -e

TARGETS=(
  "x86_64-unknown-linux-gnu"
  "x86_64-unknown-linux-musl"
  "i686-unknown-linux-gnu"
  "aarch64-unknown-linux-gnu"
  "armv7-unknown-linux-gnueabihf"
  "arm-unknown-linux-gnueabihf"
  "x86_64-pc-windows-gnu"
  "i686-pc-windows-gnu"
  "x86_64-apple-darwin"
  "aarch64-apple-darwin"
)

# Show menu
echo "Select targets one by one (type the number). Type 'p' to proceed."
for i in "${!TARGETS[@]}"; do
  printf "%2d) %s\n" "$((i+1))" "${TARGETS[$i]}"
done

SELECTED=()

# Selection loop
while true; do
  read -rp "Enter a target number or 'p' to proceed: " input

  if [[ "$input" == "p" ]]; then
    break
  elif [[ "$input" =~ ^[0-9]+$ ]] && ((input >= 1 && input <= ${#TARGETS[@]})); then
    target="${TARGETS[$((input-1))]}"
    echo "✅ Added: $target"
    SELECTED+=("$target")
  else
    echo "❌ Invalid input"
  fi
done

# Get binary name
BINARY_NAME=$(cargo metadata --no-deps --format-version 1 \
  | jq -r '.packages[0].targets[] | select(.kind[] == "bin") | .name')

if [[ -z "$BINARY_NAME" ]]; then
  echo "❌ Could not determine binary name"
  exit 1
fi

# Build for selected targets
for TARGET in "${SELECTED[@]}"; do
  echo "🎯 Building for $TARGET..."
  cross build --release --target "$TARGET"

  EXT=""
  [[ "$TARGET" == *windows* ]] && EXT=".exe"

  RELEASES_DIR="releases/$TARGET"
  mkdir -p "$RELEASES_DIR"
  cp "target/$TARGET/release/$BINARY_NAME$EXT" "$RELEASES_DIR/"
  echo "✅ Copied to $RELEASES_DIR/$BINARY_NAME$EXT"
done


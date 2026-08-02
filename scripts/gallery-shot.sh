#!/usr/bin/env bash
# Render a yggui example under a headless compositor and photograph it.
#
# A component library that can only be reviewed by rebuilding a host
# application is a component library nobody reviews. This brings up a private
# sway (headless wlroots), runs `cargo run --example <name>`, and writes a PNG —
# so a design change costs a screenshot rather than an afternoon.
#
#   scripts/gallery-shot.sh conversation_gallery out.png [WxH]
#
# Requires sway + grim on PATH. Nothing here touches the user's seat: the
# compositor has no input devices, no bar, no keybindings, and its own socket.
set -euo pipefail

EXAMPLE="${1:-conversation_gallery}"
OUT="${2:-/tmp/yggui-${EXAMPLE}.png}"
SIZE="${3:-2200x1500}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_DIR="${TMPDIR:-/tmp}/yggui-gallery-$EXAMPLE"
: "${XDG_RUNTIME_DIR:=/run/user/$(id -u)}"
export XDG_RUNTIME_DIR

mkdir -p "$RUN_DIR"
CONF="$RUN_DIR/sway.conf"
cat > "$CONF" <<EOF
xwayland disable
default_border none
default_floating_border none
output HEADLESS-1 resolution $SIZE position 0 0
output * bg #101418 solid_color
EOF

sockets() { ls "$XDG_RUNTIME_DIR" 2>/dev/null | grep -E '^wayland-[0-9]+$' || true; }

before="$(sockets | wc -l)"
WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1 setsid sway -c "$CONF" \
  > "$RUN_DIR/sway.log" 2>&1 &
SWAY_PID=$!
# The compositor names its OWN socket; the newest one after it comes up is it.
DISPLAY_NAME=""
for _ in $(seq 1 50); do
  sleep 0.2
  if [ "$(sockets | wc -l)" -gt "$before" ]; then
    DISPLAY_NAME="$(for s in $(sockets); do
      printf '%s %s\n' "$(stat -c %Y "$XDG_RUNTIME_DIR/$s")" "$s"
    done | sort -rn | head -1 | cut -d' ' -f2)"
    break
  fi
done
cleanup() {
  [ -n "${APP_PID:-}" ] && kill "$APP_PID" 2>/dev/null || true
  kill "$SWAY_PID" 2>/dev/null || true
}
trap cleanup EXIT
[ -n "$DISPLAY_NAME" ] || { echo "compositor did not start; see $RUN_DIR/sway.log" >&2; exit 4; }

# Build FIRST, outside the capture window: a cold cargo build would otherwise
# be photographed as an empty screen and read as a broken component.
( cd "$REPO_ROOT" && cargo build -p yggui --example "$EXAMPLE" ) >&2

WAYLAND_DISPLAY="$DISPLAY_NAME" GDK_BACKEND=wayland \
  setsid "$REPO_ROOT/target/debug/examples/$EXAMPLE" > "$RUN_DIR/app.log" 2>&1 &
APP_PID=$!

# Webview first paint is not instant and there is no signal for it; poll for a
# frame that is not the compositor's flat background instead of sleeping blind.
for _ in $(seq 1 40); do
  sleep 0.5
  WAYLAND_DISPLAY="$DISPLAY_NAME" grim "$OUT" 2>/dev/null || continue
  # A png of a single flat colour compresses to almost nothing; a rendered
  # conversation does not.
  bytes="$(stat -c %s "$OUT" 2>/dev/null || echo 0)"
  [ "$bytes" -gt 40000 ] && break
done

if ! [ -s "$OUT" ]; then
  echo "no frame captured; app log:" >&2
  tail -20 "$RUN_DIR/app.log" >&2
  exit 5
fi
echo "$OUT ($(stat -c %s "$OUT") bytes, $DISPLAY_NAME)"

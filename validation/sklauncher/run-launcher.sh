#!/bin/sh
set -eu

phase=${1:-}
case "$phase" in
  bootstrap|reload) ;;
  *)
    echo "Expected launcher phase bootstrap or reload." >&2
    exit 2
    ;;
esac

mkdir -p "$HOME/.minecraft" /results

cleanup() {
  pkill -TERM -u "$(id -u)" java 2>/dev/null || true
  sleep 2
  pkill -KILL -u "$(id -u)" java 2>/dev/null || true
  kill "$openbox_pid" "$xvfb_pid" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

Xvfb "$DISPLAY" -screen 0 1280x800x24 -nolisten tcp >"/results/${phase}-xvfb.log" 2>&1 &
xvfb_pid=$!
sleep 2
openbox >"/results/${phase}-openbox.log" 2>&1 &
openbox_pid=$!

java \
  -Djava.awt.headless=false \
  -Dprism.order=sw \
  -Duser.home="$HOME" \
  -jar /opt/sklauncher/SKlauncher.jar \
  --demo \
  --workDir "$HOME/.minecraft" \
  >"/results/${phase}-launcher.log" 2>&1 &
launcher_pid=$!

window_id=
attempt=0
while [ "$attempt" -lt 120 ]; do
  window_id=$(xdotool search --onlyvisible --name '[Ss][Kk]launcher' 2>/dev/null | head -n 1 || true)
  [ -n "$window_id" ] && break
  if ! kill -0 "$launcher_pid" 2>/dev/null; then
    break
  fi
  attempt=$((attempt + 1))
  sleep 1
done

if [ -z "$window_id" ]; then
  import -display "$DISPLAY" -window root "/results/${phase}-failure.png" 2>/dev/null || true
  echo "SKlauncher did not expose a visible window within 120 seconds." >&2
  exit 1
fi

sleep "${SKLAUNCHER_SETTLE_SECONDS:-30}"
import -display "$DISPLAY" -window root "/results/${phase}.png"
xdotool windowactivate --sync "$window_id" 2>/dev/null || true
xdotool windowclose "$window_id" 2>/dev/null || true

attempt=0
while kill -0 "$launcher_pid" 2>/dev/null && [ "$attempt" -lt 15 ]; do
  attempt=$((attempt + 1))
  sleep 1
done

if kill -0 "$launcher_pid" 2>/dev/null; then
  kill -TERM "$launcher_pid" 2>/dev/null || true
fi

test -d "$HOME/.minecraft/sklauncher"

#!/bin/sh
# Luma's optional cold-boot hook. Installed only when explicitly requested.
APP_DIR=/media/developer/apps/usr/palm/applications/cc.ncls.luma
HOOK=/var/lib/webosbrew/init.d/65-ncls-luma
conflict() {
  for file in /var/lib/webosbrew/init.d/65-luma \
    /var/lib/webosbrew/init.d/64-luma-home \
    /var/lib/webosbrew/init.d/64-ncls-luma-home \
    /var/run/luma-home /var/run/luma-home-target \
    /var/run/ncls-luma-home; do
    [ -e "$file" ] || [ -L "$file" ] || continue
    printf '%s\n' 'Resolve the existing Home or boot integration first.' >&2
    return 0
  done
  return 1
}
case "${1:-}" in
  enable)
    [ "$(id -u)" = 0 ] || { printf '%s\n' 'Run as root.' >&2; exit 1; }
    conflict && exit 1
    [ -f "$APP_DIR/appinfo.json" ] || { printf '%s\n' 'Install Luma first.' >&2; exit 1; }
    [ -x "$APP_DIR/luma-helper" ] && "$APP_DIR/luma-helper" home preflight >/dev/null || exit 1
    mkdir -p /var/lib/webosbrew/init.d
    if [ -e "$HOOK" ] || [ -L "$HOOK" ]; then
      [ "$(readlink "$HOOK")" = "$APP_DIR/autostart.sh" ] || { printf '%s\n' '65-ncls-luma is already used by another file.' >&2; exit 1; }
    fi
    chmod 755 "$APP_DIR/autostart.sh"
    ln -sf "$APP_DIR/autostart.sh" "$HOOK"
    printf '%s\n' 'Luma will open after a full boot.'
    exit 0 ;;
  disable)
    [ "$(id -u)" = 0 ] || exit 1
    if [ "$(readlink "$HOOK")" = "$APP_DIR/autostart.sh" ]; then rm -f "$HOOK"; fi
    printf '%s\n' 'Luma boot launch disabled.'
    exit 0 ;;
esac
conflict && exit 1
[ -x "$APP_DIR/luma-helper" ] && "$APP_DIR/luma-helper" home preflight >/dev/null || exit 1
# A lock lives for this boot only; do not launch twice when the hook is invoked again.
(
    exec 9>/tmp/ncls-luma-boot.lock
    flock -n 9 || exit 0
    [ -f /tmp/ncls-luma-boot-done ] && exit 0
    [ -f "$APP_DIR/appinfo.json" ] || exit 0
    sleep 25
    # -n prints the reply. -t is timing mode, not a firmware-specific flag.
    RESULT="$(/usr/bin/luna-send -n 1 -w 5000 luna://com.webos.service.applicationmanager/launch '{"id":"cc.ncls.luma"}' 2>/dev/null)"
    printf '%s\n' "$RESULT"
    case "$RESULT" in *'"returnValue":true'*|*'"returnValue": true'*) touch /tmp/ncls-luma-boot-done ;; esac
) </dev/null >/tmp/ncls-luma-boot.log 2>&1 &

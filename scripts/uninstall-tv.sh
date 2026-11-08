#!/bin/sh
# Undo lg-hush on the TV: lift the debloat and the Home replacement (LG's Home returns), then
# optionally remove the app.
#
#   scripts/uninstall-tv.sh TV_IP [--remove-app] [--dry-run]
#
# Without --remove-app only the Home replacement and the debloat are disabled.
set -eu
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$here/common.sh"

usage() { sed -n '2,8p' "$0" | sed 's/^# \{0,1\}//'; exit "${1:-0}"; }

TV_IP=""; REMOVE=0
for arg in "$@"; do
  case "$arg" in
    -h|--help) usage 0 ;;
    --remove-app) REMOVE=1 ;;
    --dry-run) DRY_RUN=1 ;;
    -*) die "unknown option: $arg" ;;
    *) [ -z "$TV_IP" ] && TV_IP=$arg || die "unexpected argument: $arg" ;;
  esac
done
[ -n "$TV_IP" ] || usage 1
check_tv

info "Disabling the Home replacement (this also lifts the debloat)"
tv_ssh "if [ -f /var/lib/ncls-luma-home/home.sh ]; then sh /var/lib/ncls-luma-home/home.sh disable; else echo 'Home replacement is not enabled'; fi"

if [ "$REMOVE" = 1 ]; then
  info "Removing the app $APP_ID"
  tv_ssh "luna-send -i luna://com.webos.appInstallService/dev/remove '{\"id\":\"$APP_ID\",\"subscribe\":true}' & p=\$!; sleep 15; kill \$p 2>/dev/null; ls -d /media/*/apps/usr/palm/applications/$APP_ID 2>/dev/null || echo 'app removed'"
fi
info "Done. If anything looks wrong, reboot the TV: without the boot hook LG's Home returns."

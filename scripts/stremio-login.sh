#!/bin/sh
# Sign in to Stremio from this computer and save only the session key on the TV, so the Home row
# shows "Continue Watching" from your library. The password is sent to api.strem.io only, never
# to the TV; the key is written next to Luma's preferences, readable by Luma only.
#
#   scripts/stremio-login.sh TV_IP [--dry-run]
#   scripts/stremio-login.sh TV_IP --logout [--dry-run]
#
# Reads STREMIO_EMAIL and STREMIO_PASSWORD from the environment, or asks for them.
# --logout   remove the saved key from the TV (sign out in Stremio to revoke the session too)
# --dry-run  print what would run, change nothing
set -eu
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$here/common.sh"

usage() { sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'; exit "${1:-0}"; }

TV_IP=""; LOGOUT=0
for arg in "$@"; do
  case "$arg" in
    -h|--help) usage 0 ;;
    --logout) LOGOUT=1 ;;
    --dry-run) DRY_RUN=1 ;;
    -*) die "unknown option: $arg" ;;
    *) [ -z "$TV_IP" ] || die "unexpected argument: $arg"; TV_IP=$arg ;;
  esac
done
[ -n "$TV_IP" ] || usage 1

# Luma keeps its preferences in the first of these that holds preferences.json.
PREF_DIRS="/media/developer/$APP_ID /media/developer/temp/$APP_ID /media/internal/.$APP_ID"
KEY_FILE="preferences.stremio-auth.json"
find_dir="D=; for d in $PREF_DIRS; do [ -f \"\$d/preferences.json\" ] && D=\$d && break; done; [ -n \"\$D\" ] || { echo 'Luma has no preferences on this TV yet: open it once first' >&2; exit 1; }; "

check_tv

if [ "$LOGOUT" = 1 ]; then
  info "Removing the saved Stremio key"
  tv_ssh "${find_dir}rm -f \"\$D/$KEY_FILE\""
  exit 0
fi

json_escape() { printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'; }

if [ "$DRY_RUN" = 1 ]; then
  printf '[dry-run] POST https://api.strem.io/api/login (credentials on stdin)\n'
  printf '[dry-run] ssh root@%s: write the key to <preferences dir>/%s, mode 600, owned by Luma\n' "$TV_IP" "$KEY_FILE"
  exit 0
fi

need curl
email=${STREMIO_EMAIL:-}
password=${STREMIO_PASSWORD:-}
if [ -z "$email" ]; then printf 'Stremio e-mail: ' >&2; read -r email; fi
if [ -z "$password" ]; then
  printf 'Stremio password: ' >&2
  stty -echo 2>/dev/null || true
  read -r password
  stty echo 2>/dev/null || true
  printf '\n' >&2
fi
[ -n "$email" ] && [ -n "$password" ] || die "e-mail and password are required"

info "Signing in to Stremio"
# Credentials go through stdin so they never appear in the process list.
answer=$(printf '{"type":"Login","email":"%s","password":"%s","facebook":false}' \
  "$(json_escape "$email")" "$(json_escape "$password")" |
  curl -fsS --max-time 20 -H 'Content-Type: application/json' --data-binary @- \
    https://api.strem.io/api/login) || die "could not reach api.strem.io"
key=$(printf '%s' "$answer" | sed -n 's/.*"authKey":"\([^"]*\)".*/\1/p')
case "$key" in
  ""|*[!A-Za-z0-9+/=_-]*) die "Stremio did not accept the sign-in (check the e-mail and password)" ;;
esac

info "Saving the key on the TV"
# The key goes through stdin too; the file takes the owner of Luma's preferences.
printf '{"authKey":"%s"}\n' "$key" | ssh -o ConnectTimeout=8 "root@$TV_IP" \
  "${find_dir}umask 077; cat > \"\$D/$KEY_FILE.tmp\" && chown \$(stat -c %u:%g \"\$D/preferences.json\") \"\$D/$KEY_FILE.tmp\" && mv \"\$D/$KEY_FILE.tmp\" \"\$D/$KEY_FILE\""
info "Done: the Home row shows Continue Watching within a minute"

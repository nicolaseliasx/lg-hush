# Shared helpers for the TV scripts. Sourced, not executed.
# Every TV call uses `ssh -tt`: on webOS, luna-send over SSH without a PTY returns empty output.

APP_ID="cc.ncls.luma"
# Where the package lives on the TV: Homebrew Channel (production) or developer mode.
TV_APP_DIRS="/media/cryptofs/apps/usr/palm/applications/$APP_ID /media/developer/apps/usr/palm/applications/$APP_ID"

die() { printf 'error: %s\n' "$*" >&2; exit 1; }
info() { printf '==> %s\n' "$*"; }

DRY_RUN=0

# tv_ssh COMMAND: run COMMAND in the TV's root shell (with a PTY).
tv_ssh() {
  if [ "$DRY_RUN" = 1 ]; then
    printf '[dry-run] ssh -tt root@%s %s\n' "$TV_IP" "$1"
    return 0
  fi
  ssh -tt -o ConnectTimeout=8 "root@$TV_IP" "$1"
}

# tv_app_dir_snippet: shell snippet that sets APP to the installed package directory.
tv_app_dir_snippet() {
  printf 'APP=; for d in %s; do [ -d "$d" ] && APP="$d" && break; done; [ -n "$APP" ] || { echo "%s is not installed on the TV" >&2; exit 1; }; ' \
    "$TV_APP_DIRS" "$APP_ID"
}

need() { command -v "$1" >/dev/null 2>&1 || die "'$1' is required but not installed"; }

check_tv() {
  [ -n "${TV_IP:-}" ] || die "missing TV_IP (the TV's address, e.g. 192.168.1.50)"
  case "$TV_IP" in *[!0-9A-Za-z.:-]*) die "TV_IP looks invalid: $TV_IP" ;; esac
  [ "$DRY_RUN" = 1 ] && return 0
  need ssh
  info "Checking root SSH on $TV_IP"
  ssh -tt -o ConnectTimeout=8 "root@$TV_IP" 'echo connected as $(id -un)' \
    || die "cannot log in as root@$TV_IP (is the TV rooted, on this network, with your SSH key authorised?)"
}

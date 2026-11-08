#!/bin/sh
# Install the lg-hush IPK on a rooted LG webOS TV over SSH, and optionally enable the Home
# replacement and the debloat.
#
#   scripts/install-tv.sh TV_IP PATH_TO.ipk [--home] [--tune] [--dry-run]
#
# --home     also run `home.sh enable` (asks for confirmation on the TV)
# --tune     also run `luma-helper tune apply` (the debloat; needs the Home replacement active)
# --dry-run  print what would run, change nothing
set -eu
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$here/common.sh"

usage() { sed -n '2,10p' "$0" | sed 's/^# \{0,1\}//'; exit "${1:-0}"; }

TV_IP=""; IPK=""; DO_HOME=0; DO_TUNE=0
for arg in "$@"; do
  case "$arg" in
    -h|--help) usage 0 ;;
    --home) DO_HOME=1 ;;
    --tune) DO_TUNE=1 ;;
    --dry-run) DRY_RUN=1 ;;
    -*) die "unknown option: $arg" ;;
    *) if [ -z "$TV_IP" ]; then TV_IP=$arg; elif [ -z "$IPK" ]; then IPK=$arg; else die "unexpected argument: $arg"; fi ;;
  esac
done
[ -n "$TV_IP" ] && [ -n "$IPK" ] || usage 1
[ -f "$IPK" ] || die "IPK not found: $IPK"
case "$IPK" in *.ipk) ;; *) die "expected an .ipk file: $IPK" ;; esac
[ "$DO_TUNE" = 1 ] && [ "$DO_HOME" = 0 ] && die "--tune needs --home (the debloat only runs while Luma is the Home)"

# Verify the checksum when a sha256sums file sits next to the package.
sums="$(dirname -- "$IPK")/sha256sums"
if [ -f "$sums" ]; then
  name=$(basename -- "$IPK")
  expected=$(grep " \*\{0,1\}$name\$" "$sums" | awk '{print $1}' | head -n 1)
  if [ -n "$expected" ]; then
    need sha256sum
    actual=$(sha256sum "$IPK" | awk '{print $1}')
    [ "$expected" = "$actual" ] || die "checksum mismatch for $name (expected $expected, got $actual)"
    info "Checksum OK ($name)"
  fi
fi

check_tv
need scp

info "Copying the package to the TV"
if [ "$DRY_RUN" = 1 ]; then
  printf '[dry-run] scp -O %s root@%s:/tmp/lg-hush.ipk\n' "$IPK" "$TV_IP"
else
  scp -O -o ConnectTimeout=8 "$IPK" "root@$TV_IP:/tmp/lg-hush.ipk"
fi

info "Installing $APP_ID (this can take a minute)"
tv_ssh "rm -f /tmp/lg-hush-install.log; luna-send -i luna://com.webos.appInstallService/dev/install '{\"id\":\"$APP_ID\",\"ipkUrl\":\"/tmp/lg-hush.ipk\",\"subscribe\":true}' > /tmp/lg-hush-install.log 2>&1 & p=\$!; i=0; while [ \$i -lt 120 ]; do sleep 2; i=\$((i+2)); grep -qiE 'installed|failed|error' /tmp/lg-hush-install.log && break; done; kill \$p 2>/dev/null; cat /tmp/lg-hush-install.log; rm -f /tmp/lg-hush.ipk; ! grep -qiE 'failed|errorCode' /tmp/lg-hush-install.log"

info "Checking the installed package"
tv_ssh "$(tv_app_dir_snippet)echo \"installed at \$APP\"; ls \"\$APP\" | head -n 20"

info "Read-only diagnosis (changes nothing)"
tv_ssh "$(tv_app_dir_snippet)\"\$APP/luma-helper\" diagnose"

if [ "$DO_HOME" = 1 ]; then
  info "Enabling the Home replacement (answer y on the TV prompt to continue)"
  tv_ssh "$(tv_app_dir_snippet)sh \"\$APP/home.sh\" enable"
fi
if [ "$DO_TUNE" = 1 ]; then
  info "Applying the debloat"
  tv_ssh "$(tv_app_dir_snippet)\"\$APP/luma-helper\" tune apply && \"\$APP/luma-helper\" tune status"
fi

info "Done. Open Luma from the TV once. To undo: scripts/uninstall-tv.sh $TV_IP"

#!/bin/sh
# Read-only check of a TV: prints `luma-helper diagnose` (webOS release, capabilities and whether
# the Home replacement could be enabled). Changes nothing. Needs the IPK to be installed.
#
#   scripts/diagnose-tv.sh TV_IP
set -eu
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$here/common.sh"
TV_IP=${1:-}
case "$TV_IP" in -h|--help) sed -n '2,6p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;; esac
check_tv
tv_ssh "$(tv_app_dir_snippet)\"\$APP/luma-helper\" diagnose"

#!/bin/sh
# Fork releases do not yet have a verified update feed.
printf '%s\n' 'Updates are disabled until the fork has its own verified feed.' >&2
exit 1

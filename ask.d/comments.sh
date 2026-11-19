#!/bin/sh
# id: 6
# group: MAINTENANCE
# desc: Comments - list the .cmt files
. "$(dirname "$0")/lib.sh"

enter
find example/.comments.d -name '*.cmt' -print

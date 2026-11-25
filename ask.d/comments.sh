#!/bin/sh
# id: 6
# group: MAINTENANCE
# desc: Comments - list all .cmt files
. "$(dirname "$0")/lib.sh"

enter
find . \( -path './target' -o -path './make_deb/target' \) -prune -o -type f -name '*.cmt' -print

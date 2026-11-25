#!/bin/sh
# id: 7
# group: MAINTENANCE
# desc: Comments - create missing .cmt files for source files
. "$(dirname "$0")/lib.sh"

enter
find . \( -path './target' -o -path './make_deb/target' \) -prune -o -type f -name '*.rs' -print | while IFS= read -r src; do
  src=${src#./}
  dir=${src%/*}
  file=${src##*/}
  pair="$dir/.comments.d/$file.cmt"
  [ -e "$pair" ] && continue
  mkdir -p "$dir/.comments.d"
  printf '@1\nAdd a comment for %s.\n' "$src" > "$pair"
  printf 'created %s\n' "$pair"
done

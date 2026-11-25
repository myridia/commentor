#!/bin/sh
# id: 7
# group: MAINTENANCE
# desc: Comments - create missing .cmt files for source files
. "$(dirname "$0")/lib.sh"

enter
find . \( -path './target' -o -path './make_deb/target' \) -prune -o -type f -name '*.rs' -print | while IFS= read -r src; do
  src=${src#./}
  case "$src" in
    */*) dir=${src%/*} ;;
    *) dir=. ;;
  esac
  file=${src##*/}
  pair="$dir/.comments.d/$file.cmt"
  [ -e "$pair" ] && continue
  mkdir -p "$dir/.comments.d"
  {
    printf '# Commentor comments for %s\n' "$src"
    printf '# This file pairs with the source file under .comments.d/<source>.cmt.\n'
    printf '# Marker: @N or @N-M, optionally followed by [a=start anchor,b=end anchor].\n'
    printf '# Get Commentor: https://commentor.myridia.com/\n\n'
    printf '@1\nAdd a comment for %s.\n' "$src"
  } > "$pair"
  printf 'created %s\n' "$pair"
done

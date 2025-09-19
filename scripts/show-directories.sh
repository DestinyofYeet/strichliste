#!/usr/bin/env sh
cd "$(dirname "$0")" || exit
cd ..
find . -type d \( -path "./target" -o -path "./\.*" \)  -prune -o -type d -print  | grep -v "^\.$"\
 | sed 's|^\./||' | grep -v '^$' | awk -F/ '{ indent = NF - 1; printf "%s- %s\n", sprintf("%*s", indent * 2, ""), $NF; }'

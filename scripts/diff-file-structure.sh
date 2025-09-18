#!/usr/bin/env sh
cd "$(dirname "$0")" || exit
printf "%s\n\n%s" "$(./show-directories.sh)" "$(./extract-dirs-from-readme.sh)" | python3 ./compare-trees.py "file" "repo" 


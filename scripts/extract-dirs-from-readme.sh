#!/usr/bin/env sh
cd "$(dirname "$0")" || exit
cd ..
sed -n '/<!-- BEGINFOLDERS -->/,/<!-- ENDFOLDERS -->/{//!p}' README.md \
 | grep -v ">" | grep -Ev "^$"


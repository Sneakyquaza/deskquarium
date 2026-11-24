#!/bin/bash
# especes_assets.sh [clé…] : nettoie les images IA retenues (tools/choix.tsv) en sprites
# assets/especes/<clé>.png (fond lu dans out/brouillons/ia/especes.tsv, vert ou magenta).
cd "$(dirname "$0")/.."
IA=out/brouillons/ia
grep -v '^#' tools/choix.tsv | while IFS=$'\t' read -r cle img opts; do
  [ -n "$1" ] && ! [[ " $* " == *" $cle "* ]] && continue
  fond=$(awk -F'\t' -v c="$cle" '$1==c{print $3}' $IA/especes.tsv)
  echo "== $cle ($img)"
  # shellcheck disable=SC2086
  python3 tools/pixeliser.py "$IA/$img.png" "assets/especes/$cle.png" --fond "${fond:-magenta}" $opts | tail -2
done

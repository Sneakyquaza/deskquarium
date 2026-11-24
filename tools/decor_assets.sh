#!/bin/bash
# decor_assets.sh : nettoie les images IA du décor retenues (out/brouillons/ia/decor_*.png,
# fond magenta) en assets/decor/<nom>.png (pixel art au pixel près, sans yeux).
cd "$(dirname "$0")/.."
IA=out/brouillons/ia
while read -r nom img; do
  [ -z "$nom" ] || [[ "$nom" == \#* ]] && continue
  echo "== $nom ($img)"
  python3 tools/pixeliser.py "$IA/$img.png" "assets/decor/$nom.png" --yeux aucun | tail -1
done <<'LISTE'
sable decor_sable_b
rochers_a decor_rochers_a
rochers_b decor_rochers_b
corail_branche decor_corail_branche_a
corail_eventail decor_corail_eventail_a
corail_cerveau decor_corail_cerveau_a
algue_a decor_algue_a
algue_b decor_algue_b
lointain decor_lointain_a
LISTE

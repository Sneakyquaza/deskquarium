#!/bin/bash
# nourriture_assets.sh [motif] : nettoie les images IA retenues de la nourriture (v0.16,
# out/brouillons/ia/nourriture/, fond magenta) en assets/nourriture/<nom>.png : pixeliser (grille
# d'origine, jamais réduite, sans yeux), puis les restes du fond enfermés (entre les barreaux de
# la cage…) rendus transparents. Les images absentes sont sautées. Codex dessine les petites
# nourritures bien trop grandes (un flocon de 25 pixels, plus gros qu'un néon) : elles sont
# ramenées à l'échelle du jeu (`--largeur`, cases du dessin), jamais plus bas.
cd "$(dirname "$0")/.."
IA=out/brouillons/ia/nourriture
motif=${1:-}
mkdir -p assets/nourriture
while read -r nom img opts; do
  [ -z "$nom" ] || [[ "$nom" == \#* ]] && continue
  [ -n "$motif" ] && [[ "$nom" != *$motif* ]] && continue
  [ -s "$IA/$img.png" ] || { echo "-- $nom : $img absent"; continue; }
  echo "== $nom ($img)"
  python3 tools/pixeliser.py "$IA/$img.png" "assets/nourriture/$nom.png" --yeux aucun $opts | tail -1
  python3 -I - "assets/nourriture/$nom.png" <<'PY'
import sys
from PIL import Image
f = sys.argv[1]
im = Image.open(f).convert("RGBA")
p = im.load()
w, h = im.size
for y in range(h):
    for x in range(w):
        r, g, b, a = p[x, y]
        if a and r > 150 and b > 150 and g < 110:
            p[x, y] = (0, 0, 0, 0)
im.save(f)
print(f, im.size)
PY
done <<'LISTE'
flocons flocons_a --largeur 9 --couleurs 12
granules granules_a --largeur 7 --couleurs 10
vers vers_a --largeur 18 --couleurs 16
crevette crevette_a --largeur 32
calamar calamar_a --largeur 28
cage cage_a
festin festin_a --largeur 40
flocons_icone icone_flocons_a
granules_icone icone_granules_a
vers_icone icone_vers_a
crevette_icone icone_crevette_a
calamar_icone icone_calamar_a
festin_icone icone_festin_a
LISTE

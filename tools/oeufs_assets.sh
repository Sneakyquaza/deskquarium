#!/bin/bash
# oeufs_assets.sh : nettoie les œufs IA retenus (v0.10, out/brouillons/ia/oeuf_*.png, fond
# magenta) en assets/oeufs/<clé>.png ; les trous intérieurs (pixels sombres pris pour le fond)
# sont rebouchés par la couleur voisine.
cd "$(dirname "$0")/.."
IA=out/brouillons/ia
mkdir -p assets/oeufs
while read -r nom img; do
  [ -z "$nom" ] || [[ "$nom" == \#* ]] && continue
  [ -s "$IA/$img.png" ] || { echo "-- $nom : $img absent"; continue; }
  python3 tools/pixeliser.py "$IA/$img.png" "assets/oeufs/$nom.png" --yeux aucun | tail -1
  python3 -I - "assets/oeufs/$nom.png" <<'PY'
import sys
from PIL import Image
f = sys.argv[1]
im = Image.open(f).convert("RGBA")
w, h = im.size
p = im.load()
# Vide relié au bord = dehors ; le reste du vide = trou à reboucher.
dehors = set()
pile = [(x, y) for x in range(w) for y in (0, h - 1)] + [(x, y) for y in range(h) for x in (0, w - 1)]
while pile:
    x, y = pile.pop()
    if (x, y) in dehors or not (0 <= x < w and 0 <= y < h) or p[x, y][3] > 0:
        continue
    dehors.add((x, y))
    pile += [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
trous = [(x, y) for y in range(h) for x in range(w) if p[x, y][3] == 0 and (x, y) not in dehors]
while trous:
    reste = []
    for x, y in trous:
        v = [p[a, b] for a, b in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1))
             if 0 <= a < w and 0 <= b < h and p[a, b][3] == 255]
        if v:
            p[x, y] = min(v, key=lambda c: c[0] + c[1] + c[2])
        else:
            reste.append((x, y))
    if len(reste) == len(trous):
        break
    trous = reste
im.save(f)
print(f, "trous rebouchés")
PY
done <<'LISTE'
riviere oeuf_riviere_a
lagon oeuf_lagon_a
abysses oeuf_abysses_a
cosmique oeuf_cosmique_b
ambre oeuf_ambre_a
dore oeuf_dore_a
arc oeuf_arc_a
eclipse oeuf_eclipse_a
LISTE

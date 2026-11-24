#!/bin/bash
# oeufs_calques.sh : nettoie les calques IA retenus des œufs (v0.11, out/brouillons/ia/v11/)
# en assets/oeufs/calques/<nom>.png : pixeliser (grille d'origine, jamais réduite), puis les
# restes de la couleur de fond enfermés (dedans d'un anneau, trous d'une galaxie) rendus
# transparents ; la fenêtre verte de la coquille cosmique est gardée en vert pur.
cd "$(dirname "$0")/.."
IA=out/brouillons/ia/v11
mkdir -p assets/oeufs/calques
while read -r nom img fond opts; do
  [ -z "$nom" ] || [[ "$nom" == \#* ]] && continue
  [ -s "$IA/$img.png" ] || { echo "-- $nom : $img absent"; continue; }
  python3 tools/pixeliser.py "$IA/$img.png" "assets/oeufs/calques/$nom.png" --fond "$fond" --yeux aucun $opts | tail -1
  python3 -I - "assets/oeufs/calques/$nom.png" "$fond" "$nom" <<'PY'
import sys
from PIL import Image
f, fond, nom = sys.argv[1:4]
im = Image.open(f).convert("RGBA")
p = im.load()
w, h = im.size
for y in range(h):
    for x in range(w):
        r, g, b, a = p[x, y]
        if a == 0:
            continue
        mag = r > 150 and b > 150 and g < 110
        ver = g > 170 and r < 120 and b < 120
        if nom == "cosmique_coque" and ver:
            p[x, y] = (0, 255, 0, 255)
        elif (fond == "magenta" and mag) or (fond == "vert" and ver):
            p[x, y] = (0, 0, 0, 0)
im.save(f)
print(f, im.size)
PY
done <<'LISTE'
cosmique_coque cosmique_coque_a magenta
nebuleuse cosmique_nebuleuse_b magenta --sans-contour
galaxie galaxie_spirale_b vert --sans-contour
trou_noir trou_noir_b vert --sans-contour
ambre_coque ambre_coque_b magenta
fossile fossile_c magenta --sans-contour
gouttes gouttes_ambre_a magenta
soleil soleil_couronne_b magenta --sans-contour
lune lune_d magenta
anneau anneau_diamant_b magenta --sans-contour
oeil oeil_geant_a magenta
lanterne lanterne_a magenta
huitre_bas huitre_bas_a magenta
huitre_haut huitre_haut_a magenta
perle perle_a magenta
tresor tresor_b magenta
prisme prisme_a magenta
eclaboussure eclaboussure_a magenta
LISTE

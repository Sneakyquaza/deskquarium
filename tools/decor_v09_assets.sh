#!/bin/bash
# decor_v09_assets.sh [motif] : nettoie les images IA retenues des décors à thème et des objets
# de la boutique (v0.9, out/brouillons/ia/decor_*.png, fond magenta) en assets/decor/…
# (pixel art au pixel près, sans yeux). Les images absentes sont sautées.
cd "$(dirname "$0")/.."
IA=out/brouillons/ia
motif=${1:-}
while read -r nom img; do
  [ -z "$nom" ] || [[ "$nom" == \#* ]] && continue
  [ -n "$motif" ] && [[ "$nom" != *$motif* ]] && continue
  [ -s "$IA/$img.png" ] || { echo "-- $nom : $img absent"; continue; }
  mkdir -p "assets/decor/$(dirname "$nom")"
  echo "== $nom ($img)"
  python3 tools/pixeliser.py "$IA/$img.png" "assets/decor/$nom.png" --yeux aucun | tail -1
done <<'LISTE'
epave/grand_a decor_epave_grand_a_a
epave/grand_b decor_epave_grand_b_a
epave/petit_a decor_epave_petit_a_a
epave/petit_b decor_epave_petit_b_a
epave/petit_c decor_epave_petit_c_a
epave/lointain decor_epave_lointain_a
ruines/grand_a decor_ruines_grand_a_a
ruines/grand_b decor_ruines_grand_b_a
ruines/petit_a decor_ruines_petit_a_a
ruines/petit_b decor_ruines_petit_b_a
ruines/petit_c decor_ruines_petit_c_a
ruines/lointain decor_ruines_lointain_a
volcan/grand_a decor_volcan_grand_a_a
volcan/grand_b decor_volcan_grand_b_a
volcan/petit_a decor_volcan_petit_a_a
volcan/petit_b decor_volcan_petit_b_a
volcan/petit_c decor_volcan_petit_c_a
volcan/lointain decor_volcan_lointain_a
volcan/algue decor_volcan_algue_a
volcan/sable decor_volcan_sable_a
kelp/grand_a decor_kelp_grand_a_a
kelp/grand_b decor_kelp_grand_b_a
kelp/petit_a decor_kelp_petit_a_a
kelp/petit_b decor_kelp_petit_b_a
kelp/petit_c decor_kelp_petit_c_a
kelp/lointain decor_kelp_lointain_a
kelp/algue_a decor_kelp_algue_a
kelp/algue_b decor_kelp_algue_b
cimetiere/grand_a decor_cimetiere_grand_a_a
cimetiere/grand_b decor_cimetiere_grand_b_a
cimetiere/petit_a decor_cimetiere_petit_a_a
cimetiere/petit_b decor_cimetiere_petit_b_a
cimetiere/petit_c decor_cimetiere_petit_c_a
cimetiere/lointain decor_cimetiere_lointain_a
objets/cailloux decor_item_cailloux_a
objets/crane decor_item_crane_a
objets/anemone decor_item_anemone_a
objets/amphores decor_item_amphores_a
objets/algue_lumineuse decor_item_algue_lumineuse_a
objets/ancre decor_item_ancre_a
objets/corail_rare decor_item_corail_rare_a
objets/casque decor_item_casque_a
objets/coffre decor_item_coffre_a
objets/cristaux decor_item_cristaux_a
objets/epave decor_item_epave_a
objets/geyser decor_item_geyser_a
objets/statue decor_item_statue_a
objets/chateau decor_item_chateau_a
LISTE

#!/bin/bash
# revue_oeufs.sh DOSSIER : tout ce qu'il faut pour relire les œufs v0.11 (planche, repos,
# une éclosion par œuf avec son, ouverture ×10, images des paliers).
cd "$(dirname "$0")/.."
D=${1:-out/captures/v11/revue}
mkdir -p "$D"
A=./target/release/deskquarium
export AQUARIUM_FFMPEG=${AQUARIUM_FFMPEG:-$HOME/opt/ffmpeg/ffmpeg}
F=$AQUARIUM_FFMPEG
$A --planche-oeufs "$D/oeufs_planche.png"
$A --repos-oeufs "$D/oeufs_repos.mp4" --secondes 10
# Une éclosion par œuf, raretés variées (shiny, modificateur, haute rareté).
eclo() { $A --eclosion "$D/eclosion_$1.mp4" --oeuf $2 --espece $3 $4 > "$D/eclosion_$1.txt"; }
eclo 0_riviere_commun 0 clown &
eclo 1_lagon_epique 1 manta &
eclo 2_abysses_legendaire 2 orque &
eclo 3_cosmique_celeste_shiny 3 baleine_stellaire "--forme shiny" &
wait
eclo 4_ambre_prehistorique 4 megalodon &
eclo 5_dore_rare_galaxie 5 meduse "--forme galaxie" &
eclo 6_arc_mythique 6 hydre &
eclo 7_eclipse_irreel_shiny 7 cthulhu "--forme shiny,diamant" &
wait
$A --multi "$D/multi_x10.mp4" --oeuf 2 --nombre 10 --graine ${GRAINE_MULTI:-3} > "$D/multi_x10.txt"
# Paliers : l'image juste après chaque saut (éclosion Céleste shiny).
t=$(sed -n 's/^temps forts : \[\(.*\)\].*/\1/p' "$D/eclosion_3_cosmique_celeste_shiny.txt")
i=0; args=""
for x in 1.2 ${t//,/ }; do
  $F -v error -y -ss $(python3 -c "print($x+0.12)") -i "$D/eclosion_3_cosmique_celeste_shiny.mp4" -frames:v 1 -vf scale=384:288:flags=neighbor "$D/palier_$i.png"
  args="$args -i $D/palier_$i.png"; i=$((i+1))
done
$F -v error -y $args -filter_complex "hstack=inputs=$i" "$D/paliers.png"
echo "fini : $D"

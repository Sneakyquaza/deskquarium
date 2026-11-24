#!/bin/bash
# revue.sh : planches pour la relecture des espèces (out/captures/) :
# especes_planche_x4.png, especes_geantes_x2.png, especes_animation_x4.png (8 espèces).
cd "$(dirname "$0")/.."
B=target/release/aquarium
$B --planche out/captures/especes_planche_x4.png --secondes 2
$B --planche-geantes out/captures/especes_geantes_x2.png --secondes 2
$B --planche-images out/captures/especes_animation_x4.png clown,neon,murene,meduse,manta,crabe,blanc,poulpe
$B --planche-images out/captures/especes_animation_geantes_x4.png kraken,cthulhu,leviathan,hydre
python3 tools/planche_assets.py out/captures/especes_sprites_bruts_x2.png 2

#!/bin/bash
# video_oeufs.sh SORTIE.mp4 : vidéo de présentation des œufs v0.11 (rendus du jeu, son
# compris) : les 8 œufs au repos, les 8 éclosions, une ouverture ×10. 1280x720, H.264.
# AQUARIUM_LANGUE=en pour l'anglais.
cd "$(dirname "$0")/.."
S=${1:-out/captures/v11/oeufs_v11_ingame.mp4}
T=out/captures/v11/video_tmp
mkdir -p "$T"
A=./target/release/aquarium
export AQUARIUM_FFMPEG=${AQUARIUM_FFMPEG:-$HOME/opt/ffmpeg/ffmpeg}
F=$AQUARIUM_FFMPEG
$A --repos-oeufs "$T/00_repos.mp4" --secondes 7
i=1
for e in "0 clown" "1 manta" "2 orque" "3 baleine_stellaire --forme shiny" "4 megalodon" "5 meduse --forme galaxie" "6 hydre" "7 cthulhu --forme shiny"; do
  set -- $e
  $A --eclosion "$T/0${i}_eclosion.mp4" --oeuf $1 --espece $2 ${3:+$3 $4} >/dev/null
  i=$((i+1))
done
$A --multi "$T/09_multi.mp4" --oeuf 2 --nombre 10 --graine ${GRAINE_MULTI:-8} >/dev/null
# Silence pour le repos (pas de son au repos), puis tout au même format.
L=""; n=0
for f in "$T"/0*.mp4; do
  $F -v error -y -i "$f" -f lavfi -i anullsrc=r=44100:cl=stereo \
    -filter_complex "[0:v]scale=1280:720:force_original_aspect_ratio=decrease:flags=neighbor,pad=1280:720:(ow-iw)/2:(oh-ih)/2:color=0x0a1420,fps=30,format=yuv420p[v];[0:a]aresample=44100,aformat=channel_layouts=stereo[a0];[1:a][a0]amix=inputs=2:duration=longest[a]" \
    -map "[v]" -map "[a]" -shortest -c:v libx264 -crf 22 -preset slow -c:a aac -b:a 96k "$T/n_$(basename "$f")" 2>/dev/null \
  || $F -v error -y -i "$f" -f lavfi -i anullsrc=r=44100:cl=stereo \
    -filter_complex "[0:v]scale=1280:720:force_original_aspect_ratio=decrease:flags=neighbor,pad=1280:720:(ow-iw)/2:(oh-ih)/2:color=0x0a1420,fps=30,format=yuv420p[v]" \
    -map "[v]" -map 1:a -shortest -c:v libx264 -crf 22 -preset slow -c:a aac -b:a 96k "$T/n_$(basename "$f")"
  echo "file 'n_$(basename "$f")'" >> "$T/liste.txt.tmp"; n=$((n+1))
done
mv "$T/liste.txt.tmp" "$T/liste.txt"
$F -v error -y -f concat -safe 0 -i "$T/liste.txt" -c copy "$S"
ls -la "$S"

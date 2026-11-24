#!/usr/bin/env bash
# Sons du jeu : rendu genny (tools/sons/batch.json -> out/sons_wav/) puis conversion en
# WAV IMA ADPCM mono 22050 Hz dans assets/sons/ (embarqués par src/son.rs).
# Crête ramenée par son : -1 dBFS (jingles, éclats), -4 dBFS (le reste), -10 dBFS (clics d'UI).
# Silence de début/fin coupé, fondu de 3 ms à l'entrée et 15 ms à la sortie (pas de clic).
set -euo pipefail
cd "$(dirname "$0")/../.."
FF=${FFMPEG:-$HOME/opt/ffmpeg/ffmpeg}
command -v "$FF" >/dev/null || FF=ffmpeg

genny render tools/sons/batch.json --quiet
mkdir -p assets/sons

crete() {
    case "$1" in
        jingle* | eclat* | palier*) echo -1 ;;
        clic | carte) echo -10 ;;
        piece | fele*) echo -6 ;;
        *) echo -4 ;;
    esac
}

for f in out/sons_wav/*.wav; do
    n=$(basename "$f" .wav)
    # Mono 22050 Hz, silence coupé aux deux bouts (fichier intermédiaire laissé dans out/).
    tmp=out/sons_wav/.tmp_$n.wav
    "$FF" -loglevel error -y -i "$f" -ac 1 -ar 22050 \
        -af "silenceremove=start_periods=1:start_threshold=-60dB,areverse,silenceremove=start_periods=1:start_threshold=-60dB,areverse" \
        -c:a pcm_s16le "$tmp"
    d=$("$FF" -i "$tmp" -af volumedetect -f null - 2>&1 | sed -n 's/.*max_volume: \(-\?[0-9.]*\) dB.*/\1/p')
    g=$(awk -v c="$(crete "$n")" -v m="$d" 'BEGIN { print c - m }')
    l=$(awk -v t="$("$FF" -i "$tmp" 2>&1 | sed -n 's/.*Duration: 00:00:\([0-9.]*\).*/\1/p')" 'BEGIN { print t - 0.015 }')
    "$FF" -loglevel error -y -i "$tmp" \
        -af "volume=${g}dB,afade=t=in:d=0.003,afade=t=out:st=${l}:d=0.015" \
        -c:a adpcm_ima_wav "assets/sons/$n.wav"
done
du -cb assets/sons/*.wav | tail -1

#!/usr/bin/env bash
# Fabrique out/livraison/aquarium_windows_<version>.zip : aquarium.exe + LISEZMOI.txt + LICENCES.txt.
set -euo pipefail
cd "$(dirname "$0")/.."
VERSION=${1:-v0.1}
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release --target x86_64-pc-windows-gnu
L=out/livraison
mkdir -p "$L"
cp target/x86_64-pc-windows-gnu/release/aquarium.exe "$L/"
cp livraison/LISEZMOI.txt "$L/"
{
  echo "AQUARIUM - licences"
  echo
  echo "Le programme aquarium est sous licence MIT. Les œufs et décors sont dessinés par"
  echo "le programme. Les sprites des 61 espèces (assets/especes) ont été générés pour ce"
  echo "projet avec un modèle d'image (ChatGPT, OpenAI) puis nettoyés au pixel près et animés"
  echo "par le programme ; designs originaux (Cthulhu et le monstre du Loch Ness : folklore et"
  echo "domaine public, rendus originaux)."
  echo
  echo "Polices embarquées (SIL Open Font License 1.1, textes complets plus bas) :"
  echo "- Pixelify Sans, Copyright 2021 The Pixelify Sans Project Authors"
  echo "- Silkscreen, Copyright 2001 The Silkscreen Project Authors"
  echo
  echo "Bibliothèques Rust incluses dans l'exe (nom version : licence) :"
  cargo tree --target x86_64-pc-windows-gnu -e normal --prefix none --format "{p} : {l}" \
    | sed 's/ (\*)//; s/ (proc-macro)//; s/ (\/home[^)]*)//' | sort -u | grep -v '^aquarium '
  echo
  echo "Textes complets : https://opensource.org/license/mit , https://www.apache.org/licenses/LICENSE-2.0"
  echo
  echo "================ OFL - Pixelify Sans ================"
  cat assets/polices/OFL-PixelifySans.txt
  echo
  echo "================ OFL - Silkscreen ================"
  cat assets/polices/OFL-Silkscreen.txt
} > "$L/LICENCES.txt"
# Textes en fins de ligne Windows dans le zip.
python3 -I - "$L" "aquarium_windows_$VERSION.zip" <<'PY'
import sys, zipfile, pathlib
d = pathlib.Path(sys.argv[1])
with zipfile.ZipFile(d / sys.argv[2], "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
    for n in ["aquarium.exe", "LISEZMOI.txt", "LICENCES.txt"]:
        data = (d / n).read_bytes()
        if n.endswith(".txt") and b"\r\n" not in data:
            data = data.replace(b"\n", b"\r\n")
        z.writestr("aquarium/" + n, data)
PY
ls -la "$L/aquarium_windows_$VERSION.zip"

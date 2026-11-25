#!/usr/bin/env bash
# publier_release.sh vX.Y.Z "notes" : construit l'exe Windows, écrit son empreinte et publie la
# release GitHub (dépôt public Sneakyquaza/deskquarium) que l'appli installe toute seule (src/maj.rs
# lit releases/latest : deskquarium.exe + deskquarium.exe.sha256). Joint aussi le zip complet.
# Copie aquarium.exe (+ son .sha256) : les clients v0.12 et moins ne cherchent que ce nom.
# La version de Cargo.toml doit être celle du tag (sinon l'appli se croirait toujours en retard).
set -euo pipefail
cd "$(dirname "$0")/.."
TAG=${1:?usage: scripts/publier_release.sh vX.Y.Z \"notes\"}
NOTES=${2:-}
export PATH="$HOME/.cargo/bin:$PATH"
VCARGO=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
if [ "v$VCARGO" != "$TAG" ]; then
  echo "Cargo.toml est en $VCARGO, pas ${TAG#v} : change la version d'abord." >&2
  exit 1
fi
COURT=${TAG%.*}   # v0.8.0 -> v0.8 (nom du zip)
scripts/livraison_windows.sh "$COURT"
L=out/livraison
( cd "$L" && cp deskquarium.exe aquarium.exe \
  && sha256sum deskquarium.exe > deskquarium.exe.sha256 && sha256sum aquarium.exe > aquarium.exe.sha256 \
  && cat deskquarium.exe.sha256 aquarium.exe.sha256 )
gh release create "$TAG" "$L/deskquarium.exe" "$L/deskquarium.exe.sha256" \
  "$L/aquarium.exe" "$L/aquarium.exe.sha256" "$L/Deskquarium-$COURT.zip" \
  --repo Sneakyquaza/deskquarium --title "Deskquarium $TAG" --notes "$NOTES"

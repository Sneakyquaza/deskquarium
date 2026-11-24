# tailles.py : recopie dans tools/choix.tsv la taille voulue de chaque espèce
# (Espece::longueur_ecran dans src/especes.rs, pixels écran en 1920x1080 = 2 pixels fins)
# sous la forme « --longueur N » (pixels du sprite) pour tools/pixeliser.py.
import re
from pathlib import Path

racine = Path(__file__).resolve().parent.parent
src = (racine / "src/especes.rs").read_text()
bloc = src[src.index("pub fn longueur_ecran"):]
bloc = bloc[:bloc.index("_ => 2 *")]
longueurs = {}
for m in re.finditer(r'((?:"[a-z_]+"\s*\|?\s*)+)=>\s*(\d+)', bloc):
    for cle in re.findall(r'"([a-z_]+)"', m.group(1)):
        longueurs[cle] = int(m.group(2))
choix = racine / "tools/choix.tsv"
lignes = []
for ln in choix.read_text().splitlines():
    if ln.startswith("#") or not ln.strip():
        lignes.append(ln)
        continue
    cle, img, *reste = ln.split("\t")
    opts = re.sub(r"\s*--(longueur|reduire) \S+", "", reste[0] if reste else "").strip()
    if cle in longueurs:
        opts = f"{opts} --longueur {longueurs[cle] // 2}".strip()
    lignes.append("\t".join([cle, img, opts]))
choix.write_text("\n".join(lignes) + "\n")
print(f"{len(longueurs)} tailles recopiées dans {choix}")

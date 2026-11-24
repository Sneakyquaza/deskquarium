# police_sans_ligatures.py src.ttf dst.ttf : retire les ligatures (liga, clig, dlig) d'une police
# (v0.11 : egui appliquait « fi » de Pixelify Sans, cassé : « fish » s'affichait « Ash »).
# Besoin de fontTools (MIT) dans PYTHONPATH.
import sys
from fontTools.ttLib import TTFont
f = TTFont(sys.argv[1])
n = 0
for fr in f["GSUB"].table.FeatureList.FeatureRecord:
    if fr.FeatureTag in ("liga", "clig", "dlig"):
        fr.Feature.LookupListIndex = []
        fr.Feature.LookupCount = 0
        n += 1
f.save(sys.argv[2])
print(f"{n} fonctions de ligature vidées")

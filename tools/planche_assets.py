# planche_assets.py sortie.png [zoom] : tous les sprites de assets/especes côte à côte (zoom
# entier), yeux (alpha 254) en vert et lumières (253) en cyan à droite de chaque sprite.
import sys, glob, os
import numpy as np
from PIL import Image, ImageDraw
z = int(sys.argv[2]) if len(sys.argv) > 2 else 2
fich = sorted(glob.glob(os.path.join(os.path.dirname(__file__), "../assets/especes/*.png")))
cases = []
for f in fich:
    a = np.asarray(Image.open(f).convert("RGBA")).copy()
    b = a.copy()
    b[a[..., 3] == 254, :3] = [0, 255, 0]
    b[a[..., 3] == 253, :3] = [0, 255, 255]
    a[..., 3] = np.where(a[..., 3] > 0, 255, 0)
    b[..., 3] = np.where(b[..., 3] > 0, 255, 0)
    cases.append((os.path.basename(f)[:-4], Image.fromarray(a), Image.fromarray(b)))
L = 1800
x = y = hl = 0
pos = []
for nom, a, b in cases:
    w, h = (a.width * 2 + 6) * z, a.height * z + 14
    if x + w > L:
        x, y, hl = 0, y + hl, 0
    pos.append((x, y))
    x += w + 8
    hl = max(hl, h)
im = Image.new("RGB", (L, y + hl), (18, 34, 58))
d = ImageDraw.Draw(im)
for (nom, a, b), (x, y) in zip(cases, pos):
    d.text((x, y), f"{nom} {a.width}x{a.height}", fill=(230, 240, 244))
    im.paste(a.resize((a.width * z, a.height * z), Image.NEAREST), (x, y + 12), a.resize((a.width * z, a.height * z), Image.NEAREST))
    im.paste(b.resize((b.width * z, b.height * z), Image.NEAREST), (x + (a.width + 6) * z, y + 12), b.resize((b.width * z, b.height * z), Image.NEAREST))
im.save(sys.argv[1])

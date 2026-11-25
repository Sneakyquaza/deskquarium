#!/usr/bin/env python3
"""yeux_affiner.py X.png [Y.png…] [--voir planche.png] [--ecrire] : remplace les disques d'yeux
(alpha 254, posés par pixeliser.py autour du reflet) par le vrai contour de l'œil.

Bug v0.13 (orque Électrique shiny) : le disque débordait sur le corps (tache bleu nuit restée
naturelle dans un corps recoloré) ou coupait l'iris en deux (moitié recolorée). Ici, croissance
de régions à partir de graines : œil = reflet clair + pupille sombre collée au reflet ; corps =
pixels opaques à 2 rayons du disque. Chaque pixel de la fenêtre rejoint la région voisine dont
la couleur est la plus proche (la frontière suit le bord le plus marqué : cercle sombre, iris).
Sans --ecrire : n'écrit rien (planche --voir seulement)."""
import argparse, heapq, math
from PIL import Image, ImageDraw

ap = argparse.ArgumentParser()
ap.add_argument("pngs", nargs="+")
ap.add_argument("--voir", default="")
ap.add_argument("--ecrire", action="store_true")
a = ap.parse_args()
V8 = [(dx, dy) for dx in (-1, 0, 1) for dy in (-1, 0, 1) if dx or dy]


def morceaux(s):
    s, res = set(s), []
    while s:
        p = s.pop()
        c, pile = [p], [p]
        while pile:
            x, y = pile.pop()
            for dx, dy in V8:
                n = (x + dx, y + dy)
                if n in s:
                    s.remove(n)
                    c.append(n)
                    pile.append(n)
        res.append(c)
    return res


def lum(c):
    return c[0] * 0.3 + c[1] * 0.59 + c[2] * 0.11


def ecart(c, d):
    return math.sqrt(sum((c[i] - d[i]) ** 2 for i in range(3)))


def affiner(im):
    """Masque des yeux affiné (ensemble de pixels) et disques d'origine."""
    w, h = im.size
    px = im.load()
    disques = morceaux([(x, y) for y in range(h) for x in range(w) if px[x, y][3] == 254])
    oeil = set()
    for d in disques:
        cx = sum(p[0] for p in d) / len(d)
        cy = sum(p[1] for p in d) / len(d)
        r = math.sqrt(len(d) / math.pi)
        rmax = 2 * r + 2
        plein = lambda p: 0 <= p[0] < w and 0 <= p[1] < h and px[p][3] >= 253
        fen = {
            (x, y)
            for y in range(int(cy - rmax) - 1, int(cy + rmax) + 2)
            for x in range(int(cx - rmax) - 1, int(cx + rmax) + 2)
            if plein((x, y)) and math.hypot(x - cx, y - cy) <= rmax
        }
        # Graines de l'œil : reflet (clair, peu saturé) le plus proche du centre, pupille
        # (pixels les plus sombres du disque) collée au reflet.
        sat = lambda c: (max(c) - min(c)) / max(1, max(c))
        coeur = [p for p in d if math.hypot(p[0] - cx, p[1] - cy) <= max(1.0, r * 0.5)]
        clairs = [p for p in d if max(px[p][:3]) > 190 and sat(px[p][:3]) < 0.35]
        graines = set()
        if clairs:
            m = min(morceaux(clairs), key=lambda c: min(math.hypot(p[0] - cx, p[1] - cy) for p in c))
            graines |= set(m)
            noir = min(lum(px[p]) for p in d)
            sombres = [p for p in d if lum(px[p]) <= noir + 14]
            for c in morceaux(sombres):
                if any((p[0] + dx, p[1] + dy) in graines or math.hypot(p[0] - cx, p[1] - cy) <= r * 0.5
                       for p in c for dx, dy in V8):
                    graines |= set(c)
        if not graines:
            graines = set(coeur)
        region = {p: 1 for p in graines}
        for p in fen:
            if math.hypot(p[0] - cx, p[1] - cy) >= rmax - 1.5 and p not in region:
                region[p] = 0
        # Croissance : le pixel libre le plus proche (en couleur) d'un voisin déjà pris.
        tas = []
        for p, g in region.items():
            for dx, dy in V8:
                n = (p[0] + dx, p[1] + dy)
                if n in fen and n not in region:
                    heapq.heappush(tas, (ecart(px[n], px[p]), n, g))
        # L'œil ne dépasse pas 1,4 rayon du disque (sinon il file le long des traits sombres).
        loin = lambda p: math.hypot(p[0] - cx, p[1] - cy) > 1.4 * r + 1
        while tas:
            _, p, g = heapq.heappop(tas)
            if p in region or (g == 1 and loin(p)):
                continue
            region[p] = g
            for dx, dy in V8:
                n = (p[0] + dx, p[1] + dy)
                if n in fen and n not in region:
                    heapq.heappush(tas, (ecart(px[n], px[p]), n, g))
        oeil |= {p for p, g in region.items() if g == 1}
    # Couture : un pixel d'œil de la MÊME couleur qu'un pixel du corps voisin passe au corps (de
    # proche en proche), sauf les quasi-noirs (pupilles collées au contour, jamais recolorées
    # par le shiny).
    pile = list(oeil)
    while pile:
        x, y = pile.pop()
        if (x, y) not in oeil or max(px[x, y][:3]) < 51:
            continue
        for q in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
            if (0 <= q[0] < w and 0 <= q[1] < h and q not in oeil and px[q][3] >= 254
                    and px[q][:3] == px[x, y][:3]):
                oeil.discard((x, y))
                pile += [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
                break
    return oeil, disques


tuiles = []
for f in a.pngs:
    im = Image.open(f).convert("RGBA")
    oeil, disques = affiner(im)
    if not disques:
        continue
    if a.ecrire:
        px = im.load()
        w, h = im.size
        for y in range(h):
            for x in range(w):
                if px[x, y][3] == 254 and (x, y) not in oeil:
                    px[x, y] = px[x, y][:3] + (255,)
        for p in oeil:
            if px[p][3] == 255:
                px[p] = px[p][:3] + (254,)
        im.save(f, optimize=True)
    for d in disques:
        cx = round(sum(p[0] for p in d) / len(d))
        cy = round(sum(p[1] for p in d) / len(d))
        R, k = 14, 6
        c = im.crop((cx - R, cy - R, cx + R + 1, cy + R + 1)).convert("RGB")
        c = c.resize((c.width * k, c.height * k), Image.NEAREST)
        avant, apres = c.copy(), c.copy()
        for img, m, coul in ((avant, set(d), (255, 0, 255)), (apres, oeil, (0, 255, 0))):
            dr = ImageDraw.Draw(img)
            for x, y in m:
                if abs(x - cx) <= R and abs(y - cy) <= R:
                    X, Y = (x - cx + R) * k, (y - cy + R) * k
                    dr.rectangle((X + 2, Y + 2, X + 3, Y + 3), fill=coul)
        t = Image.new("RGB", (c.width * 3 + 8, c.height + 12))
        for i, img in enumerate((c, avant, apres)):
            t.paste(img, (i * (c.width + 4), 12))
        ImageDraw.Draw(t).text((2, 0), f.split("/")[-1][:-4], fill=(255, 255, 255))
        tuiles.append(t)
if a.voir and tuiles:
    n = 2
    tw, th = tuiles[0].width, tuiles[0].height
    p = Image.new("RGB", ((tw + 8) * n, (th + 4) * ((len(tuiles) + n - 1) // n)))
    for i, t in enumerate(tuiles):
        p.paste(t, ((i % n) * (tw + 8), (i // n) * (th + 4)))
    p.save(a.voir)

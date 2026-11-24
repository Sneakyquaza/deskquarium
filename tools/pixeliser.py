# pixeliser.py <image_ia.png> <sortie.png> [--largeur L] [--longueur N] [--couleurs N] [--apercu x4.png]
# Image ChatGPT (pixel art sur fond magenta uni) -> sprite au pixel près :
# 1. détourage du fond (magenta, connexe au bord) et boîte de la créature ;
# 2. grille : période mesurée (autocorrélation des bords de couleur) ou imposée par --largeur ;
#    chaque case = médiane du centre de la case (pas de moyenne qui baverait) ;
# 3. franges magenta retirées, trous d'un pixel bouchés, miettes d'alpha supprimées ;
# 4. palette Lab réduite (k-moyennes), pixels orphelins proches de leurs voisins fondus ;
# 5. contour sombre de 1 px ajouté si le bord de la créature n'est pas déjà sombre ;
# 6. PNG RGBA sans semi-transparence, 1 px de marge vide.
import argparse
import numpy as np
from PIL import Image

ap = argparse.ArgumentParser()
ap.add_argument("src")
ap.add_argument("dst")
ap.add_argument("--largeur", type=int, default=0, help="largeur native voulue (0 = grille mesurée)")
ap.add_argument("--couleurs", type=int, default=40)
ap.add_argument("--apercu", default="")
ap.add_argument("--fond", default="magenta", choices=["magenta", "vert"])
ap.add_argument("--vue", default="cote", choices=["cote", "face"], help="où chercher les yeux")
ap.add_argument("--yeux", default="", help="yeux imposés « x,y,r;x,y,r » (pixels du sprite final)")
ap.add_argument("--lumieres", default="", help="lumières (leurre…) « x,y,r;… » (pixels du sprite final)")
ap.add_argument("--nb-yeux", type=int, default=0, help="nombre d'yeux cherchés (défaut 1 de côté, 2 de face)")
ap.add_argument("--longueur", type=int, default=0,
                help="plus grande dimension voulue (pixels du sprite) : réduit le sprite fini à sa taille "
                     "dans l'aquarium (Espece::longueur_ecran / 2 ; --yeux en pixels non réduits)")
a = ap.parse_args()

im = np.asarray(Image.open(a.src).convert("RGB")).astype(np.float32)
H, W = im.shape[:2]
MAG = np.array([255, 0, 255] if a.fond == "magenta" else [0, 255, 0], np.float32)
dmag = np.sqrt(((im - MAG) ** 2).sum(2))
obj = dmag > 110
ys, xs = np.where(obj)
x0, x1 = np.percentile(xs, 0.02), np.percentile(xs, 99.98)
y0, y1 = np.percentile(ys, 0.02), np.percentile(ys, 99.98)


def profils():
    gx = np.abs(np.diff(im, axis=1)).sum(2)[int(y0):int(y1)].sum(0)
    gy = np.abs(np.diff(im, axis=0)).sum(2)[:, int(x0):int(x1)].sum(1)
    return gx, gy


def alignement(g, P, debut, fin):
    """Meilleur décalage de grille de période P sur le profil g : (contraste, origine)."""
    best = (-1e9, 0.0)
    for o in np.arange(0, P, 0.25):
        lignes = np.arange(debut + o, fin, P)
        lignes = lignes[(lignes >= 0) & (lignes < len(g) - 1)]
        if len(lignes) < 4:
            continue
        sur = g[np.round(lignes).astype(int)].mean()
        mil = g[np.clip(np.round(lignes + P / 2).astype(int), 0, len(g) - 1)].mean()
        c = (sur - mil) / (g.mean() + 1e-6)
        if c > best[0]:
            best = (c, o + debut)
    return best


def periode():
    """Période de la grille de l'image IA : plus petit pic fort de l'autocorrélation des sauts
    de couleur, puis affinée au 1/50 de pixel (lignes sur les sauts, milieux de case sans)."""
    g = np.abs(np.diff(im, axis=1)).sum(2)[int(y0):int(y1), int(x0):int(x1)]
    prof = (g > 40).sum(0).astype(float)
    prof -= prof.mean()
    ac = np.correlate(prof, prof, "full")[len(prof) - 1:]
    ac /= ac[0]
    hauts = [p for p in range(3, 41) if ac[p] >= ac[p - 1] and ac[p] >= ac[p + 1]]
    if not hauts:
        return 4.0, 0.0
    m = max(ac[p] for p in hauts)
    p0 = min(p for p in hauts if ac[p] >= 0.85 * m)
    gx, gy = profils()
    best = (-1e9, float(p0))
    for P in np.arange(p0 * 0.92, p0 * 1.08, 0.02):
        c = alignement(gx, P, x0 - P, x1 + P)[0] + alignement(gy, P, y0 - P, y1 + P)[0]
        if c > best[0]:
            best = (c, P)
    return best[1], float(ac[p0])


p_mes, force = periode()
P, m = float(p_mes), 1
if a.largeur:
    # Grille mesurée gardée (pixels exacts) : la cible prend un nombre entier m de cases IA
    # par pixel du sprite ; sinon (grille douteuse) on rééchantillonne à la cible.
    cible = (x1 - x0) / a.largeur
    m = max(1, round(cible / p_mes))
    if not (force > 0.3 and abs(m * p_mes - cible) <= 0.3 * cible):
        P, m = cible, 1
print(f"période mesurée {p_mes:.2f} (pic {force:.2f}), {m} case(s) IA par pixel, période {m * P:.2f}")

# Origine de la grille : décalage qui aligne les lignes de case sur les sauts de couleur.
gx, gy = profils()
_, ox = alignement(gx, m * P, x0 - m * P, x1 + m * P)
_, oy = alignement(gy, m * P, y0 - m * P, y1 + m * P)
Q = m * P
cx = np.arange(ox, x1 + Q, Q)
cy = np.arange(oy, y1 + Q, Q)
cells = np.zeros((len(cy), len(cx), 3), np.float32)
part = np.zeros((len(cy), len(cx)), np.float32)
r = max(1, int(P * 0.25))
for j, y in enumerate(cy):
    for i, x in enumerate(cx):
        echant, parts = [], []
        for sj in range(m):
            for si in range(m):
                yy, xx = int(y + 0.5 + (sj + 0.5) * P), int(x + 0.5 + (si + 0.5) * P)
                yy, xx = min(max(yy, r), H - r - 1), min(max(xx, r), W - r - 1)
                blk = im[yy - r:yy + r + 1, xx - r:xx + r + 1].reshape(-1, 3)
                ob = obj[yy - r:yy + r + 1, xx - r:xx + r + 1].reshape(-1)
                parts.append(ob.mean())
                if ob.mean() > 0.5:
                    echant.append(np.median(blk[ob], 0))
        part[j, i] = np.mean(parts)
        if echant:
            e = np.array(echant)
            # couleur réelle la plus centrale des sous-cases (pas de moyenne qui baverait)
            cells[j, i] = e[np.argmin(((e[:, None] - e[None]) ** 2).sum(2).sum(1))]
        else:
            cells[j, i] = MAG

opq = part > 0.5
# franges teintées magenta au bord -> fond
if a.fond == "magenta":
    fr = (cells[..., 0] > 140) & (cells[..., 2] > 140) & (cells[..., 1] < 100)
else:
    fr = (cells[..., 1] > 150) & (cells[..., 0] < 110) & (cells[..., 2] < 110)


def voisins4(m):
    v = np.zeros(m.shape, int)
    v[1:] += m[:-1]
    v[:-1] += m[1:]
    v[:, 1:] += m[:, :-1]
    v[:, :-1] += m[:, 1:]
    return v


for _ in range(2):
    opq &= ~(fr & (voisins4(opq) < 4))
# franges plus sombres : case du bord nettement plus proche du fond que ses voisines intérieures
dfond = np.sqrt(((cells - MAG) ** 2).sum(2))
for _ in range(2):
    bordure = opq & (voisins4(opq) < 4)
    inter = opq & ~bordure
    somme = np.zeros_like(dfond); nb = np.zeros_like(dfond)
    for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        sh = np.roll(np.roll(inter, dy, 0), dx, 1)
        somme += np.roll(np.roll(dfond, dy, 0), dx, 1) * sh
        nb += sh
    mx_ = cells.max(2); mn_ = cells.min(2)
    teinte_fond = (cells[..., 1] < cells[..., 0] - 30) & (cells[..., 1] < cells[..., 2] - 30) if a.fond == "magenta" \
        else (cells[..., 1] > cells[..., 0] + 30) & (cells[..., 1] > cells[..., 2] + 30)
    frange = bordure & teinte_fond & (nb > 0) & (dfond < 0.8 * somme / np.maximum(nb, 1)) & (mx_ - mn_ > 40)
    opq &= ~frange
# miettes : pixels opaques avec moins de 2 voisins opaques ; trous d'un pixel bouchés
for _ in range(2):
    opq &= voisins4(opq) >= 2
# îlots détachés de moins de 6 cases (miettes de fond ou d'ombre) : retirés
vu_ = np.zeros_like(opq)
for y_, x_ in zip(*np.where(opq)):
    if vu_[y_, x_]:
        continue
    pile_, comp_ = [(y_, x_)], []
    vu_[y_, x_] = True
    while pile_:
        cy_, cx_ = pile_.pop()
        comp_.append((cy_, cx_))
        for dy_ in (-1, 0, 1):
            for dx_ in (-1, 0, 1):
                ny_, nx_ = cy_ + dy_, cx_ + dx_
                if 0 <= ny_ < opq.shape[0] and 0 <= nx_ < opq.shape[1] and opq[ny_, nx_] and not vu_[ny_, nx_]:
                    vu_[ny_, nx_] = True
                    pile_.append((ny_, nx_))
    if len(comp_) < 6:
        for cy_, cx_ in comp_:
            opq[cy_, cx_] = False
trou = ~opq & (voisins4(opq) == 4)
if trou.any():
    pad = np.pad(cells, ((1, 1), (1, 1), (0, 0)), mode="edge")
    for j, i in zip(*np.where(trou)):
        cells[j, i] = np.median(np.stack([pad[j, i + 1], pad[j + 2, i + 1], pad[j + 1, i], pad[j + 1, i + 2]]), 0)
    opq |= trou


def to_lab(c):
    c = c / 255.0
    c = np.where(c > 0.04045, ((c + 0.055) / 1.055) ** 2.4, c / 12.92)
    M = np.array([[0.4124, 0.3576, 0.1805], [0.2126, 0.7152, 0.0722], [0.0193, 0.1192, 0.9505]])
    xyz = c @ M.T / np.array([0.9505, 1.0, 1.089])
    f = np.where(xyz > 0.008856, np.cbrt(xyz), 7.787 * xyz + 16 / 116)
    return np.stack([116 * f[:, 1] - 16, 500 * (f[:, 0] - f[:, 1]), 200 * (f[:, 1] - f[:, 2])], 1)


NC = a.couleurs
px = cells[opq]
lab = to_lab(px)
rng = np.random.default_rng(1)
# initialisation k-moyennes++ légère : points éloignés (garde les petits accents : yeux, reflets)
cent = [lab[rng.integers(len(lab))]]
for _ in range(NC - 1):
    d = np.min(((lab[:, None] - np.array(cent)[None]) ** 2).sum(2), 1)
    cent.append(lab[rng.choice(len(lab), p=d / d.sum())])
cent = np.array(cent)
for _ in range(25):
    lbl = np.argmin(((lab[:, None] - cent[None]) ** 2).sum(2), 1)
    for k in range(NC):
        if (lbl == k).any():
            cent[k] = lab[lbl == k].mean(0)
pal = np.array([px[lbl == k].mean(0) if (lbl == k).any() else [0, 0, 0] for k in range(NC)])
ind = np.full(opq.shape, -1, int)
ind[opq] = lbl
labp = to_lab(pal)
HT, WT = ind.shape
# orphelins (aucun voisin 8 de même couleur) proches de leur voisinage -> couleur voisine la plus proche
for _ in range(3):
    nv = ind.copy()
    for y in range(HT):
        for x in range(WT):
            v = ind[y, x]
            if v < 0:
                continue
            blk = [ind[y + dy, x + dx] for dy in (-1, 0, 1) for dx in (-1, 0, 1)
                   if (dy or dx) and 0 <= y + dy < HT and 0 <= x + dx < WT]
            if v in blk:
                continue
            c = [q for q in blk if q >= 0]
            if len(c) >= 3:
                q = min(c, key=lambda q: float(((labp[q] - labp[v]) ** 2).sum()))
                if np.sqrt(((labp[q] - labp[v]) ** 2).sum()) < 28:
                    nv[y, x] = q
    ind = nv

# restes du fond : couleurs rares de la teinte du fond (≤ 0,5 % des pixels) -> couleur voisine
if a.fond == "magenta":
    tf = (pal[:, 1] < pal[:, 0] - 40) & (pal[:, 1] < pal[:, 2] - 40)
else:
    tf = (pal[:, 1] > pal[:, 0] + 40) & (pal[:, 1] > pal[:, 2] + 40)
compte = np.bincount(ind[ind >= 0], minlength=len(pal))
suspects = [k for k in range(len(pal)) if tf[k] and compte[k] <= 0.005 * (ind >= 0).sum()]
for _ in range(3):
    for y, x in zip(*np.where(np.isin(ind, suspects))):
        vois = [ind[y + dy, x + dx] for dy in (-1, 0, 1) for dx in (-1, 0, 1)
                if (dy or dx) and 0 <= y + dy < HT and 0 <= x + dx < WT and ind[y + dy, x + dx] >= 0
                and ind[y + dy, x + dx] not in suspects]
        if vois:
            ind[y, x] = max(set(vois), key=vois.count)

# contour : si le bord n'est pas déjà sombre, anneau extérieur de 1 px (couleur la plus sombre assombrie)
sol = ind >= 0
lumi = pal @ np.array([0.299, 0.587, 0.114])
bord = sol & (voisins4(sol) < 4)
part_sombre = (lumi[ind[bord]] < 60).mean()
ind = np.pad(ind, 2, constant_values=-1)
if part_sombre < 0.7:
    k0 = int(np.argmin(lumi))
    pal = np.vstack([pal, pal[k0] * 0.55])
    sol = ind >= 0
    anneau = ~sol & (voisins4(sol) > 0)
    ind[anneau] = len(pal) - 1
print(f"bord sombre {part_sombre:.0%}", "contour ajouté" if part_sombre < 0.7 else "contour d'origine gardé")

ys, xs = np.where(ind >= 0)
ind = ind[ys.min() - 1:ys.max() + 2, xs.min() - 1:xs.max() + 2]
out = np.zeros(ind.shape + (4,), np.uint8)
out[ind >= 0, :3] = pal[ind[ind >= 0]].round().clip(0, 255).astype(np.uint8)
out[ind >= 0, 3] = 255


def disques(txt):
    r = []
    for d in filter(None, txt.split(";")):
        x, y, rr = (int(v) for v in d.split(","))
        r.append((x, y, rr))
    return r


def yeux_auto():
    """Yeux : reflet blanc (petite tache claire peu saturée) collé à une pupille très sombre,
    dans la tête (tiers avant pour une vue de côté tournée à droite, haut pour une vue de face).
    Repli : petite tache vive d'une teinte différente du corps (iris lumineux)."""
    rgb = out[..., :3].astype(float) / 255
    mx, mn = rgb.max(2), rgb.min(2)
    sat = np.where(mx > 0, (mx - mn) / np.maximum(mx, 1e-6), 0)
    plein = out[..., 3] > 0
    hh, ww = plein.shape
    yy, xx = np.mgrid[0:hh, 0:ww]
    xs_, ys_ = np.where(plein)[1], np.where(plein)[0]
    bx0, bx1, by0, by1 = xs_.min(), xs_.max(), ys_.min(), ys_.max()
    if a.vue == "cote":
        zone = plein & (xx > bx0 + 0.55 * (bx1 - bx0))
    else:
        zone = plein & (yy < by0 + 0.62 * (by1 - by0))
    sombre = plein & (mx < 0.22)
    pres_sombre = np.zeros_like(sombre)
    for dy in (-1, 0, 1):
        for dx in (-1, 0, 1):
            pres_sombre |= np.roll(np.roll(sombre, dy, 0), dx, 1)
    clair = zone & (mx > 0.82) & (sat < 0.3) & pres_sombre
    cand = []
    vus = np.zeros_like(clair)
    for y, x in zip(*np.where(clair)):
        if vus[y, x]:
            continue
        pile, comp = [(y, x)], []
        vus[y, x] = True
        while pile:
            cy_, cx_ = pile.pop()
            comp.append((cy_, cx_))
            for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                ny, nx = cy_ + dy, cx_ + dx
                if 0 <= ny < hh and 0 <= nx < ww and clair[ny, nx] and not vus[ny, nx]:
                    vus[ny, nx] = True
                    pile.append((ny, nx))
        cy_, cx_ = np.mean(comp, 0)
        iy, ix = int(round(cy_)), int(round(cx_))
        # un œil n'est pas au bord de la silhouette (bout du museau, épine…)
        if not plein[max(0, iy - 2):iy + 3, max(0, ix - 2):ix + 3].all():
            continue
        if a.vue == "cote" and cy_ > by0 + 0.7 * (by1 - by0):
            continue
        if len(comp) <= max(4, plein.sum() // 300):
            # pupille : pixels sombres autour du reflet
            r = 1
            rmax = max(1, round(hh * 0.07))
            while r < rmax and sombre[max(0, int(cy_) - r - 1):int(cy_) + r + 2, max(0, int(cx_) - r - 1):int(cx_) + r + 2].sum() > (2 * r + 1) ** 2 * 0.35:
                r += 1
            score = sombre[max(0, int(cy_) - 2):int(cy_) + 3, max(0, int(cx_) - 2):int(cx_) + 3].sum()
            cand.append((score, int(round(cx_)), int(round(cy_)), r))
    cand.sort(reverse=True)
    n = a.nb_yeux or (1 if a.vue == "cote" else 2)
    gardes = []
    for _, x, y, r in cand:
        # un seul œil par endroit (reflets voisins du même œil)
        if all((x - gx) ** 2 + (y - gy) ** 2 > (2 * max(r, gr) + 2) ** 2 for gx, gy, gr in gardes):
            gardes.append((x, y, r))
    return gardes[:n]


if a.yeux == "aucun":
    yeux = []
elif a.yeux:
    yeux = disques(a.yeux)
else:
    yeux = yeux_auto()
lum = disques(a.lumieres)
# Marquage dans l'alpha : 255 corps, 254 œil (fermé au clignement, luit à travers l'eau),
# 253 lumière (luit, ne cligne pas). Tout reste opaque à l'affichage.
hh, ww = ind.shape
yy, xx = np.mgrid[0:hh, 0:ww]
for liste, val in ((yeux, 254), (lum, 253)):
    for x, y, r in liste:
        m = ((xx - x) ** 2 + (yy - y) ** 2 <= r * r + r) & (ind >= 0)
        out[m, 3] = val
print("yeux", ";".join(f"{x},{y},{r}" for x, y, r in yeux) or "aucun", "lumières", a.lumieres or "aucune")


def reduire(src, F):
    """Réduit le sprite fini d'un facteur F (< 1) sans mélanger les tailles de pixel : chaque
    pixel réduit prend une couleur réelle (médoïde) de sa fenêtre, le bord de la silhouette
    reprend la couleur du contour d'origine, chaque œil et lumière garde au moins un pixel
    (pupille sombre, reflet clair s'il en reste la place)."""
    hs, ws = src.shape[:2]
    pl = src[..., 3] > 0
    bord_s = pl & (voisins4(pl) < 4)
    lum_s = src[..., :3].astype(float) @ np.array([0.299, 0.587, 0.114])
    cont = src[bord_s][:, :3]
    c_cont = cont[np.argmin(cont.astype(float) @ np.array([0.299, 0.587, 0.114]))] if len(cont) else np.zeros(3)
    nh, nw = max(4, round(hs * F)), max(4, round(ws * F))
    o = np.zeros((nh, nw, 4), np.uint8)
    fen = lambda k, n, N: (k * N // n, max(k * N // n + 1, (k + 1) * N // n))
    for j in range(nh):
        y0_, y1_ = fen(j, nh, hs)
        for i in range(nw):
            x0_, x1_ = fen(i, nw, ws)
            w_ = src[y0_:y1_, x0_:x1_].reshape(-1, 4)
            p_ = w_[w_[:, 3] > 0]
            if len(p_) * 2 < len(w_):
                continue
            oe = p_[p_[:, 3] < 255]
            if len(oe) * 3 >= len(p_):
                # œil ou lumière : pupille (le plus sombre) ou lumière (le plus clair)
                l_ = oe[:, :3].astype(float) @ np.array([0.299, 0.587, 0.114])
                o[j, i] = oe[np.argmax(l_)] if oe[0, 3] == 253 else oe[np.argmin(l_)]
                continue
            inter = bord_s[y0_:y1_, x0_:x1_].reshape(-1)[w_[:, 3] > 0]
            e = (p_[~inter] if (~inter).any() else p_)[:, :3].astype(float)
            o[j, i, :3] = e[np.argmin(((e[:, None] - e[None]) ** 2).sum(2).sum(1))]
            o[j, i, 3] = 255
    # chaque œil / lumière de la source garde au moins son pixel central
    for val in (254, 253):
        m_ = src[..., 3] == val
        vus = np.zeros_like(m_)
        for y, x in zip(*np.where(m_)):
            if vus[y, x]:
                continue
            pile, comp = [(y, x)], []
            vus[y, x] = True
            while pile:
                cy_, cx_ = pile.pop()
                comp.append((cy_, cx_))
                for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                    ny, nx = cy_ + dy, cx_ + dx
                    if 0 <= ny < hs and 0 <= nx < ws and m_[ny, nx] and not vus[ny, nx]:
                        vus[ny, nx] = True
                        pile.append((ny, nx))
            cy_, cx_ = np.mean(comp, 0)
            j, i = min(nh - 1, int(cy_ * nh / hs)), min(nw - 1, int(cx_ * nw / ws))
            pts = np.array([src[yy, xx] for yy, xx in comp])
            l_ = pts[:, :3].astype(float) @ np.array([0.299, 0.587, 0.114])
            dedans = [(jj, ii) for jj in range(nh) for ii in range(nw) if o[jj, ii, 3] == val
                      and any(fen(jj, nh, hs)[0] <= yy < fen(jj, nh, hs)[1] and fen(ii, nw, ws)[0] <= xx < fen(ii, nw, ws)[1] for yy, xx in comp)]
            if (j, i) not in dedans:
                o[j, i] = pts[np.argmax(l_)] if val == 253 else pts[np.argmin(l_)]
                dedans.append((j, i))
            if val == 254 and len(dedans) >= 3:
                # reflet : le pixel de l'œil le plus haut à droite
                jj, ii = min(dedans, key=lambda q: q[0] * 2 - q[1])
                o[jj, ii, :3] = pts[np.argmax(l_)][:3]
    pl2 = o[..., 3] > 0
    bord2 = pl2 & (voisins4(pl2) < 4) & (o[..., 3] == 255)
    o[bord2, :3] = c_cont
    ys_, xs_ = np.where(pl2)
    o = o[ys_.min():ys_.max() + 1, xs_.min():xs_.max() + 1]
    return np.pad(o, ((1, 1), (1, 1), (0, 0)))


if a.longueur and a.longueur < 0.97 * max(out.shape[:2]):
    out = reduire(out, (a.longueur - 2) / (max(out.shape[:2]) - 2))
    ind = np.where(out[..., 3] > 0, 0, -1)
Image.fromarray(out, "RGBA").save(a.dst, optimize=True)
nc = len(np.unique(out[ind >= 0][:, :3], axis=0))
print(f"{a.dst} : {out.shape[1]}x{out.shape[0]}, {nc} couleurs")
if a.apercu:
    fond = Image.new("RGBA", (out.shape[1], out.shape[0]), (18, 34, 58, 255))
    o2 = out.copy()
    o2[..., 3] = np.where(out[..., 3] > 0, 255, 0)
    fond.alpha_composite(Image.fromarray(o2, "RGBA"))
    fond.resize((out.shape[1] * 4, out.shape[0] * 4), Image.NEAREST).save(a.apercu)

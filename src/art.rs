//! Boîte à outils du pixel art des espèces : formes (fuseau, tube, ellipse, polygone, trait),
//! ombrage par rampes de 5 tons décalés en teinte, tramage ordonné, yeux. Tout est rastérisé
//! au pixel entier (centres des pixels testés, aucun anticrénelage).

use crate::poisson::{OEIL, VIDE};
use crate::toile::{bayer, hsv, melange, Rgb};

pub type Px = [u8; 4];
/// 5 tons, du plus sombre (0) au reflet (4).
pub type Rampe = [Rgb; 5];

/// Point peint : pixel, coordonnée le long de la forme `t` (0..1, ou -1..1 pour l'ellipse)
/// et en travers `v` (-1 haut/gauche .. 1 bas/droite).
#[derive(Clone, Copy)]
pub struct P {
    pub x: i32,
    pub y: i32,
    pub t: f32,
    pub v: f32,
}

pub fn opaque(c: Rgb) -> Option<Px> {
    Some([c[0], c[1], c[2], 255])
}

pub fn alpha(c: Rgb, a: u8) -> Option<Px> {
    Some([c[0], c[1], c[2], a])
}

pub fn rgb_hsv(c: Rgb) -> (f32, f32, f32) {
    let [r, g, b] = c.map(|k| k as f32 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d <= 1e-6 {
        0.0
    } else if max == r {
        ((g - b) / d).rem_euclid(6.0) / 6.0
    } else if max == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };
    (h, if max <= 0.0 { 0.0 } else { d / max }, max)
}

/// Rapproche la teinte `h` de `cible` de `k` (en tours, par le plus court chemin).
fn vers(h: f32, cible: f32, k: f32) -> f32 {
    let mut d = (cible - h).rem_euclid(1.0);
    if d > 0.5 {
        d -= 1.0;
    }
    h + d.signum() * k.min(d.abs())
}

/// Rampe pixel art à partir d'une couleur de base (ton 2) : ombres plus saturées qui tirent
/// vers le bleu-violet, clairs moins saturés qui tirent vers le jaune.
pub fn rampe(c: Rgb) -> Rampe {
    let (h, s, v) = rgb_hsv(c);
    let gris = s < 0.08;
    let ton = |dh: f32, cible: f32, ds: f32, kv: f32| {
        let hh = if gris { 0.62 } else { vers(h, cible, dh) };
        let ss = if gris {
            (s + ds.max(0.0) * 0.6).clamp(0.0, 1.0)
        } else {
            (s + ds).clamp(0.0, 1.0)
        };
        hsv(hh, ss, (v * kv).clamp(0.0, 1.0))
    };
    let clair = |kv: f32, ds: f32| {
        // Au-delà du blanc : on désature au lieu d'éclaircir.
        let vv = v * kv;
        let ds = ds - (vv - 1.0).max(0.0) * 0.8;
        let hh = if gris { h } else { vers(h, 0.15, 0.03) };
        hsv(hh, (s + ds).clamp(0.0, 1.0), vv.min(1.0))
    };
    [
        ton(0.07, 0.7, 0.15, 0.42),
        ton(0.035, 0.7, 0.08, 0.68),
        c,
        clair(1.2, -0.12),
        clair(1.45, -0.3),
    ]
}

/// Ton de la rampe pour une lumière 0..1, tramée (Bayer) près des seuils.
pub fn ton(r: &Rampe, lum: f32, x: i32, y: i32) -> Rgb {
    let i = (lum.clamp(0.0, 1.0) * 4.0 + (bayer(x, y) - 0.5) * 0.85).round();
    r[i.clamp(0.0, 4.0) as usize]
}

/// Lumière d'un volume éclairé d'en haut selon `v` (-1 dessus .. 1 dessous).
pub fn lum(v: f32) -> f32 {
    (0.6 - 0.36 * v - 0.12 * v * v).clamp(0.0, 1.0)
}

/// Hachage entier → 0..1 (taches, écailles, étoiles : mêmes sur toutes les images).
pub fn hache(x: i32, y: i32, graine: u32) -> f32 {
    let mut h = (x as u32)
        .wrapping_mul(0x8da6_b343)
        .wrapping_add((y as u32).wrapping_mul(0xd816_3841))
        .wrapping_add(graine.wrapping_mul(0xcb1a_b31f));
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h & 0xffff) as f32 / 65536.0
}

pub struct Dessin {
    /// Taille utile (sans la marge) en pixels.
    pub l: i32,
    pub h: i32,
    pub marge: i32,
    pub px: Vec<Px>,
    pub cl: Vec<u8>,
    /// Points d'attache (coordonnées utiles, image tournée à droite).
    pub queue: (i32, i32),
    pub bouche: (i32, i32),
    pub lueur: Option<(i32, i32)>,
}

impl Dessin {
    pub fn new(l: i32, h: i32, marge: i32) -> Dessin {
        let n = ((l + 2 * marge) * (h + 2 * marge)) as usize;
        Dessin {
            l,
            h,
            marge,
            px: vec![[0; 4]; n],
            cl: vec![VIDE; n],
            queue: (0, h / 2),
            bouche: (l, h / 2),
            lueur: None,
        }
    }

    pub fn lt(&self) -> i32 {
        self.l + 2 * self.marge
    }

    pub fn ht(&self) -> i32 {
        self.h + 2 * self.marge
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        // Le dessin peut déborder dans la marge, sauf ses 2 derniers pixels (contour + vide).
        let d = self.marge - 2;
        let dedans = x >= -d && y >= -d && x < self.l + d && y < self.h + d;
        let (xx, yy) = (x + self.marge, y + self.marge);
        dedans.then(|| (yy * self.lt() + xx) as usize)
    }

    /// Pose un pixel : opaque = remplace ; translucide = se mélange à ce qui est dessous.
    pub fn mettre(&mut self, x: i32, y: i32, c: Px, classe: u8) {
        let Some(i) = self.index(x, y) else {
            return;
        };
        if c[3] == 0 {
            return;
        }
        let ancien = self.px[i];
        if c[3] == 255 || self.cl[i] == VIDE {
            self.px[i] = c;
        } else {
            let m = melange(
                [ancien[0], ancien[1], ancien[2]],
                [c[0], c[1], c[2]],
                c[3] as f32 / 255.0,
            );
            self.px[i] = [m[0], m[1], m[2], ancien[3].max(c[3])];
            return;
        }
        self.cl[i] = classe;
    }

    /// Retire un pixel (silhouette rabotée).
    pub fn effacer(&mut self, x: i32, y: i32) {
        if let Some(i) = self.index(x, y) {
            self.px[i] = [0; 4];
            self.cl[i] = VIDE;
        }
    }

    /// Ne peint que là où il y a déjà quelque chose (motifs posés sur une forme).
    pub fn sur(&mut self, x: i32, y: i32, c: Px) {
        if let Some(i) = self.index(x, y) {
            if self.cl[i] != VIDE && self.cl[i] != OEIL {
                let cl = self.cl[i];
                self.mettre(x, y, c, cl);
            }
        }
    }

    fn bornes(&self, x0: f32, x1: f32, y0: f32, y1: f32) -> (i32, i32, i32, i32) {
        let m = self.marge;
        (
            (x0.floor() as i32).max(-m),
            (x1.ceil() as i32).min(self.l + m),
            (y0.floor() as i32).max(-m),
            (y1.ceil() as i32).min(self.h + m),
        )
    }

    /// Corps le long de l'axe x : de `x0` (t = 0, queue) à `x1` (t = 1, tête), ligne médiane
    /// `centre(t)`, demi-hauteurs `haut(t)` et `bas(t)`. L'épaisseur est corrigée dans les pentes.
    #[allow(clippy::too_many_arguments)]
    pub fn fuseau(
        &mut self,
        x0: f32,
        x1: f32,
        centre: impl Fn(f32) -> f32,
        haut: impl Fn(f32) -> f32,
        bas: impl Fn(f32) -> f32,
        classe: u8,
        peint: impl Fn(&P) -> Option<Px>,
    ) {
        let lg = (x1 - x0).max(0.5);
        for x in (x0.floor() as i32)..(x1.ceil() as i32) {
            let t = ((x as f32 + 0.5 - x0) / lg).clamp(0.0, 1.0);
            let e = 0.5 / lg;
            let pente = (centre((t + e).min(1.0)) - centre((t - e).max(0.0))) / (2.0 * e * lg);
            let k = (1.0 + pente * pente).sqrt();
            let (cy, ht, hb) = (centre(t), haut(t).max(0.0) * k, bas(t).max(0.0) * k);
            let mut rien = true;
            for y in ((cy - ht).floor() as i32)..=((cy + hb).ceil() as i32) {
                let d = y as f32 + 0.5 - cy;
                if d >= -ht && d <= hb {
                    let v = if d < 0.0 {
                        d / ht.max(0.01)
                    } else {
                        d / hb.max(0.01)
                    };
                    if let Some(c) = peint(&P { x, y, t, v }) {
                        self.mettre(x, y, c, classe);
                    }
                    rien = false;
                }
            }
            // Bouts fins : au moins un pixel par colonne pour ne pas casser la silhouette.
            if rien && ht + hb > 0.3 {
                let y = cy.floor() as i32;
                if let Some(c) = peint(&P { x, y, t, v: 0.0 }) {
                    self.mettre(x, y, c, classe);
                }
            }
        }
    }

    /// Tube le long d'une ligne brisée `(x, y, rayon)` : tentacules, cous, serpents très courbés.
    /// `t` = 0..1 le long, `v` = -1..1 en travers (côté gauche du sens de parcours = -1).
    pub fn tube(&mut self, pts: &[(f32, f32, f32)], classe: u8, peint: impl Fn(&P) -> Option<Px>) {
        if pts.len() < 2 {
            return;
        }
        let mut cumul = vec![0.0f32];
        for w in pts.windows(2) {
            let d = (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1);
            cumul.push(cumul.last().unwrap() + d);
        }
        let total = cumul.last().copied().unwrap_or(1.0).max(0.01);
        let rmax = pts.iter().map(|p| p.2).fold(0.0, f32::max) + 1.0;
        let (mut ax, mut bx, mut ay, mut by) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for p in pts {
            ax = ax.min(p.0 - rmax);
            bx = bx.max(p.0 + rmax);
            ay = ay.min(p.1 - rmax);
            by = by.max(p.1 + rmax);
        }
        let (x0, x1, y0, y1) = self.bornes(ax, bx, ay, by);
        for y in y0..y1 {
            for x in x0..x1 {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                let mut best = (f32::MAX, 0.0f32, 0.0f32, 0.0f32);
                for (i, w) in pts.windows(2).enumerate() {
                    let (dx, dy) = (w[1].0 - w[0].0, w[1].1 - w[0].1);
                    let l2 = (dx * dx + dy * dy).max(1e-6);
                    let u = (((px - w[0].0) * dx + (py - w[0].1) * dy) / l2).clamp(0.0, 1.0);
                    let (qx, qy) = (w[0].0 + dx * u, w[0].1 + dy * u);
                    let d2 = (px - qx).powi(2) + (py - qy).powi(2);
                    let r = w[0].2 + (w[1].2 - w[0].2) * u;
                    // On garde le segment où le point est le plus « dedans ».
                    let score = d2.sqrt() - r;
                    if score < best.0 {
                        let cote = dx * (py - w[0].1) - dy * (px - w[0].0);
                        let s = if cote < 0.0 { -1.0 } else { 1.0 };
                        let t = (cumul[i] + u * l2.sqrt()) / total;
                        best = (score, t, s * d2.sqrt() / r.max(0.3), r);
                    }
                }
                if best.0 <= 0.0 || (best.3 < 0.55 && best.0 <= 0.55 - best.3) {
                    let p = P {
                        x,
                        y,
                        t: best.1,
                        v: best.2.clamp(-1.0, 1.0),
                    };
                    if let Some(c) = peint(&p) {
                        self.mettre(x, y, c, classe);
                    }
                }
            }
        }
    }

    /// Ellipse tournée de `ang` radians ; `t` et `v` = coordonnées locales -1..1.
    #[allow(clippy::too_many_arguments)]
    pub fn ellipse(
        &mut self,
        cx: f32,
        cy: f32,
        rx: f32,
        ry: f32,
        ang: f32,
        classe: u8,
        peint: impl Fn(&P) -> Option<Px>,
    ) {
        let r = rx.max(ry) + 1.0;
        let (x0, x1, y0, y1) = self.bornes(cx - r, cx + r, cy - r, cy + r);
        let (s, c) = ang.sin_cos();
        for y in y0..y1 {
            for x in x0..x1 {
                let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                let u = (dx * c + dy * s) / rx.max(0.3);
                let v = (-dx * s + dy * c) / ry.max(0.3);
                if u * u + v * v <= 1.0 {
                    if let Some(px) = peint(&P { x, y, t: u, v }) {
                        self.mettre(x, y, px, classe);
                    }
                }
            }
        }
    }

    /// Polygone plein (pair-impair) ; `t` et `v` = position dans la boîte (0..1, -1..1).
    pub fn poly(&mut self, pts: &[(f32, f32)], classe: u8, peint: impl Fn(&P) -> Option<Px>) {
        if pts.len() < 3 {
            return;
        }
        let (mut ax, mut bx, mut ay, mut by) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for p in pts {
            ax = ax.min(p.0);
            bx = bx.max(p.0);
            ay = ay.min(p.1);
            by = by.max(p.1);
        }
        let (x0, x1, y0, y1) = self.bornes(ax, bx, ay, by);
        for y in y0..y1 {
            let py = y as f32 + 0.5;
            for x in x0..x1 {
                let px = x as f32 + 0.5;
                let mut dedans = false;
                let mut j = pts.len() - 1;
                for i in 0..pts.len() {
                    let (a, b) = (pts[i], pts[j]);
                    if (a.1 > py) != (b.1 > py) && px < (b.0 - a.0) * (py - a.1) / (b.1 - a.1) + a.0
                    {
                        dedans = !dedans;
                    }
                    j = i;
                }
                if dedans {
                    let p = P {
                        x,
                        y,
                        t: (px - ax) / (bx - ax).max(0.01),
                        v: (py - ay) / (by - ay).max(0.01) * 2.0 - 1.0,
                    };
                    if let Some(c) = peint(&p) {
                        self.mettre(x, y, c, classe);
                    }
                }
            }
        }
    }

    /// Trait d'un pixel (Bresenham).
    pub fn trait_(&mut self, a: (f32, f32), b: (f32, f32), c: Px, classe: u8) {
        let (mut x0, mut y0) = (a.0.floor() as i32, a.1.floor() as i32);
        let (x1, y1) = (b.0.floor() as i32, b.1.floor() as i32);
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = ((x1 - x0).signum(), (y1 - y0).signum());
        let mut err = dx + dy;
        loop {
            self.mettre(x0, y0, c, classe);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Trait posé seulement sur la forme existante (lignes de détail : ouïes, bouche, écailles).
    pub fn trait_sur(&mut self, a: (f32, f32), b: (f32, f32), c: Px) {
        let n = ((b.0 - a.0).abs().max((b.1 - a.1).abs()).ceil() as i32).max(1);
        for i in 0..=n {
            let u = i as f32 / n as f32;
            let x = (a.0 + (b.0 - a.0) * u).floor() as i32;
            let y = (a.1 + (b.1 - a.1) * u).floor() as i32;
            self.sur(x, y, c);
        }
    }

    /// Œil : `r` = 0 (un pixel), 1 (2x2 avec reflet), 2+ (iris, pupille, reflet).
    pub fn oeil(&mut self, x: f32, y: f32, r: i32, iris: Rgb) {
        let (xi, yi) = (x.floor() as i32, y.floor() as i32);
        let noir = [14, 12, 24, 255];
        let blanc = [250, 250, 255, 255];
        match r {
            0 => self.mettre(xi, yi, noir, OEIL),
            1 => {
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    self.mettre(xi + dx, yi + dy, noir, OEIL);
                }
                self.mettre(xi, yi, blanc, OEIL);
            }
            _ => {
                let rr = r as f32 + 0.35;
                let ri = rampe(iris);
                for dy in -r..=r {
                    for dx in -r..=r {
                        let d = ((dx * dx + dy * dy) as f32).sqrt();
                        if d > rr {
                            continue;
                        }
                        let c = if d <= rr - 1.25 - (r as f32 - 2.0) * 0.6 {
                            noir
                        } else {
                            let k = if dy < 0 { 1 } else { 3 };
                            let c = ri[k];
                            [c[0], c[1], c[2], 255]
                        };
                        self.mettre(xi + dx, yi + dy, c, OEIL);
                    }
                }
                self.mettre(xi - r / 2, yi - r / 2, blanc, OEIL);
                if r >= 4 {
                    self.mettre(xi - r / 2 + 1, yi - r / 2, blanc, OEIL);
                    self.mettre(xi - r / 2, yi - r / 2 + 1, blanc, OEIL);
                    self.mettre(xi + r / 2, yi + r / 2, [200, 200, 220, 255], OEIL);
                }
            }
        }
    }
}

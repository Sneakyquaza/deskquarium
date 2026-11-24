//! Œufs et éclosion. v0.10 : un sprite pixel art IA par œuf, éclosion en phases.
//! v0.11 (œufs spectaculaires) : œufs en CALQUES IA (coquille à fenêtre, contenu intérieur,
//! effets) animés par le code (`calques`), animation au repos propre à chaque œuf, lumière
//! des fêlures qui monte par PALIERS selon le vrai résultat (`paliers`), scène d'éclosion
//! unique par œuf (`themes`), ouverture ×3/×10 (`multi`), sons. Tout au pixel entier.

pub mod calques;
pub mod multi;
pub mod paliers;
mod themes;

use crate::decor::Img;
use crate::jeu::{Alea, Partie, Poisson, Rarete, OEUFS};
use crate::scene::{sprite_ajuste, Aquarium, Nageur};
use crate::son::Son;
use crate::toile::{hsv, melange, rgb_hsv, Rgb, Toile, FIN};
use calques::{calques, fenetre};
use paliers::{couleur_palier, Deroule, Extra, PAUSE};
use std::f32::consts::{PI, TAU};
use std::sync::{Arc, OnceLock};

/// Œufs (ordre de `jeu::OEUFS`).
pub const RIVIERE: usize = 0;
pub const LAGON: usize = 1;
pub const ABYSSES: usize = 2;
pub const COSMIQUE: usize = 3;
pub const AMBRE: usize = 4;
pub const DORE: usize = 5;
pub const ARC: usize = 6;
pub const ECLIPSE: usize = 7;

/// Sprites des œufs, dans l'ordre de `jeu::OEUFS`. Cosmique et ambre sont composés de leurs
/// calques (coquille + nébuleuse, ambre + fossile) : c'est ce dessin qui se fêle et éclate.
pub fn image_oeuf(i: usize) -> &'static Img {
    static O: OnceLock<Vec<Img>> = OnceLock::new();
    let v = O.get_or_init(|| {
        let mut v = vec![
            Img::charger(include_bytes!("../assets/oeufs/riviere.png")),
            Img::charger(include_bytes!("../assets/oeufs/lagon.png")),
            Img::charger(include_bytes!("../assets/oeufs/abysses.png")),
            // Cosmique et ambre : composés de leurs calques juste après.
            Img {
                l: 0,
                h: 0,
                px: Vec::new(),
            },
            Img {
                l: 0,
                h: 0,
                px: Vec::new(),
            },
            Img::charger(include_bytes!("../assets/oeufs/dore.png")),
            Img::charger(include_bytes!("../assets/oeufs/arc.png")),
            Img::charger(include_bytes!("../assets/oeufs/eclipse.png")),
        ];
        for i in [COSMIQUE, AMBRE] {
            let c = if i == COSMIQUE {
                &calques().cosmique_coque
            } else {
                &calques().ambre_coque
            };
            let mut t = Toile::new(c.l as usize, c.h as usize);
            peindre_oeuf(&mut t, i, (0, 0), 1, 0.0, |_| 0, |y| y);
            let px = (0..c.h)
                .flat_map(|y| (0..c.l).map(move |x| (x, y)))
                .map(|(x, y)| {
                    let p = crate::toile::demultiplier(t.px[(y * t.lf() as i32 + x) as usize]);
                    if p[3] > 0 {
                        [p[0], p[1], p[2], 255]
                    } else {
                        [0; 4]
                    }
                })
                .collect();
            v[i] = Img { l: c.l, h: c.h, px };
        }
        v
    });
    &v[i.min(v.len() - 1)]
}

/// Pixel vif (taches lumineuses, étoiles, lave, or) : il anime l'œuf au repos.
fn vif(c: [u8; 4]) -> bool {
    let (_, s, v) = rgb_hsv([c[0], c[1], c[2]]);
    v > 0.72 && s > 0.4
}

/// Petit hachage entier → 0..1 (scintillements sans état).
pub fn bruit(a: i32, b: i32, c: i32) -> f32 {
    let h = (a as u32).wrapping_mul(73_856_093)
        ^ (b as u32).wrapping_mul(19_349_663)
        ^ (c as u32).wrapping_mul(83_492_791);
    (h.wrapping_mul(2_654_435_761) >> 8) as f32 / (1 << 24) as f32
}

/// Fêlures d'un œuf : segments (points du sprite) dans l'ordre où elles s'ouvrent.
pub struct Fissures {
    pts: Vec<(i32, i32)>,
}

impl Fissures {
    /// Réseau de fêlures : un zigzag autour du milieu qui part du centre vers les deux bords,
    /// puis des branches qui montent et descendent.
    pub fn nouvelles(img: &Img, graine: u64) -> Fissures {
        let mut a = Alea(graine ^ 0xf155);
        let dedans =
            |x: i32, y: i32| x >= 0 && y >= 0 && x < img.l && y < img.h && img.px(x, y)[3] > 0;
        let y0 = (img.h as f32 * a.entre(0.3, 0.38)) as i32;
        let cx = img.l / 2;
        let mut pts = Vec::new();
        // Deux bras partent du centre en même temps (points entrelacés).
        let mut bras = [(cx, y0), (cx, y0)];
        for k in 0..img.l {
            let i = k as usize % 2;
            let sens = if i == 0 { 1 } else { -1 };
            let (x, y) = bras[i];
            let ny = (y + a.n(3) as i32 - 1).clamp(y0 - 5, y0 + 5);
            let nx = x + sens;
            if dedans(nx, ny) {
                bras[i] = (nx, ny);
                pts.push((nx, ny));
            }
        }
        // Branches.
        let base: Vec<(i32, i32)> = pts.clone();
        for _ in 0..10 {
            let Some(&(mut x, mut y)) = base.get(a.n(base.len().max(1) as u32) as usize) else {
                break;
            };
            let dy = if a.f() < 0.5 { -1 } else { 1 };
            for _ in 0..(img.h / 6) {
                y += dy;
                x += a.n(3) as i32 - 1;
                if !dedans(x, y) {
                    break;
                }
                pts.push((x, y));
            }
        }
        Fissures { pts }
    }
}

/// Couleur d'un pixel du dessin de l'œuf `i` au repos à l'instant `temps` (calques compris) ;
/// None = vide. (`x`, `sy`) = pixel du dessin.
fn pixel_repos(i: usize, x: i32, sy: i32, temps: f32) -> Option<Rgb> {
    let cq = calques();
    match i {
        COSMIQUE => {
            let p = cq.cosmique_coque.px(x, sy);
            if p[3] == 0 {
                return None;
            }
            if !fenetre(p) {
                return Some([p[0], p[1], p[2]]);
            }
            // Fenêtre : la nébuleuse tourne lentement autour du centre de l'œuf.
            let c = &cq.cosmique_coque;
            let (dx, dy) = (x as f32 - c.l as f32 / 2.0, sy as f32 - c.h as f32 * 0.52);
            let (s, co) = (temps * 0.11).sin_cos();
            let n = &cq.nebuleuse;
            let (u, v) = (dx * co - dy * s, dx * s + dy * co);
            let (u, v) = (
                (u + n.l as f32 / 2.0).floor() as i32,
                (v + n.h as f32 / 2.0).floor() as i32,
            );
            let q = n.px(u.rem_euclid(n.l), v.rem_euclid(n.h));
            let mut col = [q[0], q[1], q[2]];
            // Étoiles de la nébuleuse : scintillent une à une.
            let (_, sat, val) = rgb_hsv(col);
            if val > 0.8 && sat < 0.35 {
                let k = (temps * 2.6 + bruit(u, v, 5) * TAU).sin();
                if k < -0.3 {
                    col = melange(col, [0x40, 0x20, 0x70], 0.6);
                }
            }
            // Étoiles fixes en plus, devant la nébuleuse (elles ne tournent pas).
            let b = bruit(x, sy, 41);
            if b > 0.985 {
                let k = (temps * 3.4 + b * 300.0).sin();
                if k > 0.2 {
                    col = melange(col, [255, 255, 255], k);
                }
            }
            // Étoile filante de temps en temps : une tête blanche et sa traîne en diagonale.
            let cyc = (temps / 4.3).floor();
            let u = temps / 4.3 - cyc;
            if u < 0.16 {
                let k = u / 0.16;
                let ox = bruit(cyc as i32, 1, 7) * c.l as f32 * 0.5;
                let (hx, hy) = (ox + k * c.l as f32 * 0.8, c.h as f32 * (0.2 + 0.5 * k));
                // Trajectoire : pente 0,6 ; distance le long et en travers.
                let (rx, ry) = (x as f32 - hx, sy as f32 - hy);
                let le_long = rx * 0.86 + ry * 0.52;
                let travers = (-rx * 0.52 + ry * 0.86).abs();
                if travers < 0.8 && (-9.0..=0.5).contains(&le_long) {
                    col = melange(col, [255, 250, 220], 1.0 + le_long / 10.0);
                }
            }
            Some(col)
        }
        AMBRE => {
            let p = cq.ambre_coque.px(x, sy);
            if p[3] == 0 {
                return None;
            }
            let mut col = [p[0], p[1], p[2]];
            let a = &cq.ambre_coque;
            // Pixel de bord (contour) : rien dedans.
            let bord = [(2, 0), (-2, 0), (0, 2), (0, -2)].iter().any(|(dx, dy)| {
                let (u, v) = (x + dx, sy + dy);
                u < 0 || v < 0 || u >= a.l || v >= a.h || a.px(u, v)[3] == 0
            });
            if !bord {
                // Le fossile dérive lentement dans l'ambre (parallaxe), assombri et teinté.
                let f = &cq.fossile;
                let (fx, fy) = (
                    (a.l - f.l) / 2 + ((temps * 0.37).sin() * 2.5).round() as i32,
                    a.h * 11 / 20 - f.h / 2 + ((temps * 0.29).cos() * 1.6).round() as i32,
                );
                let (u, v) = (x - fx, sy - fy);
                if u >= 0 && v >= 0 && u < f.l && v < f.h {
                    let q = f.px(u, v);
                    if q[3] > 0 {
                        col = melange(
                            col,
                            melange([q[0], q[1], q[2]], [0x5a, 0x22, 0x06], 0.35),
                            0.72,
                        );
                    }
                }
                // Bulles de sève qui montent (anneau d'un pixel clair).
                for b in 0..6 {
                    let periode = 5.0 + b as f32 * 0.7;
                    let w = ((temps + b as f32 * 1.9) / periode).fract();
                    let bx = (a.l as f32 * (0.3 + 0.42 * bruit(b, 3, 3))
                        + (temps * 1.3 + b as f32).sin() * 1.2)
                        .round() as i32;
                    let by = (a.h as f32 * (0.85 - 0.62 * w)).round() as i32;
                    if (x, sy) == (bx, by) {
                        col = melange(col, [255, 246, 200], 0.75);
                    } else if (x - bx).abs() + (sy - by).abs() == 1 && b % 2 == 0 {
                        col = melange(col, [255, 220, 140], 0.3);
                    }
                }
            }
            // Chaleur qui respire et miroitement qui monte.
            let k = 0.5 + 0.5 * (temps * 0.9).sin();
            col = melange(col, [255, 200, 90], 0.08 * k);
            let onde = (x as f32 * 0.3 + sy as f32 * 0.5 - temps * 2.2).sin();
            if onde > 0.93 {
                col = melange(col, [255, 236, 170], 0.22);
            }
            Some(col)
        }
        _ => {
            let img = image_oeuf(i);
            let p = img.px(x, sy);
            if p[3] == 0 {
                return None;
            }
            Some([p[0], p[1], p[2]])
        }
    }
}

/// Peint le dessin de l'œuf `i` (pixels du dessin en blocs `g` fins) au repos à l'instant
/// `temps` : haut-gauche (`x0`, `y0`), décalage par ligne `dec` (penché), ligne source de
/// chaque ligne dessinée `src_y` (écrasé). Les animations de pixels (taches, bandes, reflets,
/// coraux, mousse, gouttes) sont ici ; les calques autour (couronne, bulles) ailleurs.
fn peindre_oeuf(
    t: &mut Toile,
    i: usize,
    (x0, y0): (i32, i32),
    hh: i32,
    temps: f32,
    dec: impl Fn(i32) -> i32,
    src_y: impl Fn(i32) -> i32,
) {
    let img = match i {
        COSMIQUE => &calques().cosmique_coque,
        AMBRE => &calques().ambre_coque,
        _ => image_oeuf(i),
    };
    let hh = if hh <= 1 { img.h } else { hh };
    // Reflet qui balaie l'œuf (or : joaillier, deux bandes).
    let balaye = match i {
        LAGON => Some((temps * 0.35).rem_euclid(1.6) * 2.0 - 0.6),
        DORE => Some((temps * 0.6).rem_euclid(1.4) * 2.0 - 0.6),
        _ => None,
    };
    for y in 0..hh {
        let sy = src_y(y);
        for x in 0..img.l {
            let Some(mut c) = pixel_repos(i, x, sy, temps) else {
                continue;
            };
            let p = img.px(x, sy);
            let v = vif(p);
            let (h, s, vv) = rgb_hsv(c);
            match i {
                RIVIERE => {
                    // Mousse qui frémit : un pixel de côté par moments.
                    if (0.18..0.42).contains(&h) && s > 0.3 && vv < 0.75 {
                        let dx = ((temps * 2.3 + sy as f32 * 0.7).sin() * 0.7).round() as i32;
                        if let Some(q) = pixel_repos(i, x - dx, sy, temps) {
                            c = q;
                        }
                    }
                    // Reflet de l'eau qui ondule (caustiques).
                    let o = (x as f32 * 0.45 + sy as f32 * 0.2 - temps * 1.5).sin()
                        + (x as f32 * 0.2 - sy as f32 * 0.35 + temps).sin();
                    if o > 1.55 {
                        c = melange(c, [236, 255, 246], 0.28);
                    }
                    // Gouttes qui coulent : tête claire, traîne mouillée plus sombre.
                    for g in 0..3 {
                        let periode = 4.2 + g as f32 * 1.1;
                        let w = ((temps + g as f32 * 1.7) / periode).fract();
                        if w > 0.75 {
                            continue;
                        }
                        let gx = (img.l as f32 * (0.28 + 0.2 * g as f32)) as i32;
                        let gy = (img.h as f32 * (0.08 + 0.85 * (w / 0.75).powf(1.6))) as i32;
                        if x == gx {
                            if sy == gy {
                                c = [235, 252, 255];
                            } else if sy == gy - 1 {
                                c = melange(c, [180, 225, 240], 0.7);
                            } else if (gy - 6..gy - 1).contains(&sy) {
                                c = melange(c, [70, 100, 90], 0.25);
                            }
                        }
                    }
                }
                LAGON => {
                    // Nacre irisée qui glisse.
                    if s < 0.5 && vv > 0.55 {
                        let d = 0.09 * (temps * 1.2 + (x + sy) as f32 * 0.13).sin();
                        c = hsv(h + d, (s + 0.06).min(1.0), vv);
                    }
                }
                ABYSSES if v => {
                    // Taches lumineuses qui respirent en vagues.
                    let k = 0.5 + 0.5 * (temps * 2.2 - sy as f32 * 0.12).sin();
                    c = melange(c, [0xd8, 0xff, 0xff], 0.85 * k);
                }
                ARC => {
                    // Bandes de couleur qui coulent vers le bas.
                    if s > 0.2 {
                        c = hsv(h + temps * 0.15 - sy as f32 * 0.012, s, vv);
                    }
                    // Éclat de prisme : bande blanche bordée d'arc-en-ciel qui traverse.
                    let pos = (temps / 3.7).fract() * (img.l + img.h) as f32 * 1.6 - 10.0;
                    let d = x as f32 + sy as f32 * 0.7 - pos;
                    if (0.0..1.2).contains(&d) {
                        c = melange(c, [255, 255, 255], 0.65);
                    } else if (-2.5..0.0).contains(&d) || (1.2..3.7).contains(&d) {
                        c = melange(c, hsv(d / 6.0 + 0.4, 0.55, 1.0), 0.4);
                    }
                }
                ECLIPSE if v => {
                    // Fissures de lave qui battent.
                    let k = 0.5 + 0.5 * (temps * 1.4 + sy as f32 * 0.05).sin();
                    c = melange(c, [255, 240, 180], 0.55 * k);
                }
                _ => {}
            }
            if let Some(b) = balaye {
                let d = (x as f32 / img.l as f32 + sy as f32 / img.h as f32 * 0.6) - b;
                if (0.0..0.12).contains(&d) || (i == DORE && (0.18..0.21).contains(&d)) {
                    c = melange(c, [255, 255, 255], 0.35);
                }
            }
            t.poser_fin(x0 + x + dec(y), y0 + y, c, 255);
        }
    }
    // Lagon : les coraux (rose vif à droite) se balancent d'un pixel.
    if i == LAGON {
        for y in 0..hh {
            let sy = src_y(y);
            let dx = ((temps * 1.7 - sy as f32 * 0.3).sin() * 1.2).round() as i32;
            if dx == 0 {
                continue;
            }
            for x in img.l / 2..img.l {
                let p = img.px(x, sy);
                let (h, s, _) = rgb_hsv([p[0], p[1], p[2]]);
                if p[3] > 0 && (h > 0.9 || h < 0.04) && s > 0.4 {
                    t.poser_fin(x0 + x + dx + dec(y), y0 + y, [p[0], p[1], p[2]], 255);
                }
            }
        }
    }
}

/// Dessine l'œuf `i` sur la toile (pixels fins) : bas centré en (`cx`, `base`), penché de
/// `penche` pixels en haut, hauteur × `ecrase` (lignes doublées ou sautées, jamais de flou),
/// animation au repos à l'instant `temps`, fêlures ouvertes à `fissures` (0..1) qui laissent
/// passer la lumière `lumiere` (couleur, force 0..1). `blanc` : silhouette blanche (éclair).
#[allow(clippy::too_many_arguments)]
pub fn dessiner_oeuf(
    t: &mut Toile,
    i: usize,
    (cx, base): (i32, i32),
    temps: f32,
    penche: f32,
    ecrase: f32,
    fiss: Option<(&Fissures, f32)>,
    lumiere: Option<(Rgb, f32)>,
    dore: bool,
    blanc: bool,
) {
    let img = image_oeuf(i);
    let hh = ((img.h as f32 * ecrase).round() as i32).max(2);
    let (x0, y0) = (cx - img.l / 2, base - hh);
    // Ligne du dessin pour chaque ligne dessinée, et décalage (penché : le haut bouge).
    let src_y = |y: i32| ((y as f32 / ecrase) as i32).min(img.h - 1);
    let dec = |y: i32| (penche * (1.0 - y as f32 / hh as f32)).round() as i32;
    if blanc {
        for y in 0..hh {
            for x in 0..img.l {
                if img.px(x, src_y(y))[3] > 0 {
                    t.poser_fin(x0 + x + dec(y), y0 + y, [255, 255, 255], 255);
                }
            }
        }
        return;
    }
    peindre_oeuf(t, i, (x0, y0), hh, temps, dec, src_y);
    // Œuf ordinaire devenu doré : teinte d'or sur tout l'œuf (pixels déjà posés).
    if dore {
        for y in 0..hh {
            for x in 0..img.l {
                if img.px(x, src_y(y))[3] == 0 {
                    continue;
                }
                let (px, py) = (x0 + x + dec(y), y0 + y);
                if px < 0 || py < 0 || px >= t.lf() as i32 || py >= t.hf() as i32 {
                    continue;
                }
                let q = t.px[(py * t.lf() as i32 + px) as usize];
                let c = [q[0], q[1], q[2]];
                let l = (c[0] as f32 * 0.3 + c[1] as f32 * 0.59 + c[2] as f32 * 0.11) / 255.0;
                let c = melange(c, melange([0xa0, 0x58, 0x00], [0xff, 0xe8, 0x50], l), 0.9);
                t.poser_fin(px, py, c, 255);
            }
        }
    }
    // Abysses : un œil s'ouvre sur la coquille de temps en temps et regarde autour.
    if i == ABYSSES {
        let cycle = (temps / 6.5).floor() as i32;
        let u = temps / 6.5 - cycle as f32;
        if u < 0.36 {
            let places = [(0.36, 0.42), (0.64, 0.55), (0.45, 0.7), (0.58, 0.3)];
            let (fx, fy) = places[cycle.rem_euclid(4) as usize];
            let ouvert = ((u / 0.36 * PI).sin() * 1.6).min(1.0);
            let regard = ((u * 18.0).sin() * 1.5).round() as i32;
            let (ex, ey) = (
                x0 + (img.l as f32 * fx) as i32,
                y0 + (hh as f32 * fy) as i32,
            );
            petit_oeil(t, (ex, ey), ouvert, regard, 1);
        }
    }
    // Fêlures : trait sombre ; la lumière passe à travers, avec un petit halo.
    if let Some((f, k)) = fiss {
        let n = (f.pts.len() as f32 * k.clamp(0.0, 1.0)) as usize;
        let (lc, lf) = lumiere.unwrap_or(([0, 0, 0], 0.0));
        // Bords sombres d'abord (on lit un trait de fêlure, même sur un œuf clair).
        if lf > 0.05 {
            for &(x, sy) in &f.pts[..n] {
                let y = (sy as f32 * ecrase) as i32;
                let (px, py) = (x0 + x + dec(y), y0 + y);
                for dy in [-1, 1] {
                    t.poser_fin(px, py + dy, [0x14, 0x0c, 0x10], 255);
                }
            }
        }
        for &(x, sy) in &f.pts[..n] {
            let y = (sy as f32 * ecrase) as i32;
            let (px, py) = (x0 + x + dec(y), y0 + y);
            if lf > 0.05 {
                let c = melange(lc, [255, 255, 255], 0.45);
                t.poser_fin(px, py, c, 255);
                for (dx, dy) in [(1, 0), (-1, 0)] {
                    t.eclairer_fin(px + dx, py + dy, lc, (lf * 120.0) as u8);
                }
                if (x + sy).rem_euclid(3) == 0 {
                    for (dx, dy) in [(2, 0), (-2, 0), (0, 2), (0, -2)] {
                        t.eclairer_fin(px + dx, py + dy, lc, (lf * 70.0) as u8);
                    }
                }
            } else {
                t.poser_fin(px, py, [0x10, 0x0c, 0x18], 255);
            }
        }
    }
    // Étincelles à quatre branches : œuf doré (5), pierres de l'œuf d'or (3), rosée (1).
    let n = if dore {
        5
    } else if i == DORE {
        3
    } else if i == RIVIERE {
        1
    } else {
        0
    };
    for k in 0..n {
        let u = (temps * 0.8 + k as f32 * 0.37).rem_euclid(1.0);
        let o = ((1.0 - (2.0 * u - 1.0).abs()) * 3.0).round() as i32;
        let cyc = (temps * 0.8 + k as f32 * 0.37).floor() as i32;
        let (mut x, mut y) = (
            (bruit(k, 1, cyc) * img.l as f32) as i32,
            (bruit(k, 2, cyc) * hh as f32) as i32,
        );
        // Œuf d'or : l'étincelle naît sur une pierre (pixel rouge ou bleu vif) proche.
        if i == DORE && !dore {
            if let Some((gx, gy)) = pierre_proche(x, src_y(y)) {
                (x, y) = (gx, (gy as f32 * ecrase) as i32);
            }
        }
        calques::etoile(t, (x0 + x + dec(y), y0 + y), o, [255, 230, 140], 1);
    }
}

/// Pierre précieuse (pixel rouge ou bleu saturé) de l'œuf d'or la plus proche de (x, y).
fn pierre_proche(x: i32, y: i32) -> Option<(i32, i32)> {
    static P: OnceLock<Vec<(i32, i32)>> = OnceLock::new();
    let v = P.get_or_init(|| {
        let img = image_oeuf(DORE);
        let mut v = Vec::new();
        for yy in 0..img.h {
            for xx in 0..img.l {
                let p = img.px(xx, yy);
                let (h, s, vv) = rgb_hsv([p[0], p[1], p[2]]);
                if p[3] > 0
                    && s > 0.55
                    && vv > 0.45
                    && (h > 0.93 || h < 0.03 || (0.55..0.72).contains(&h))
                {
                    v.push((xx, yy));
                }
            }
        }
        v
    });
    v.iter()
        .min_by_key(|(a, b)| (a - x).pow(2) + (b - y).pow(2))
        .copied()
}

/// Petit œil des abysses posé sur la coquille (`g` = taille d'un pixel) : paupières qui
/// s'ouvrent (`ouvert` 0..1), pupille fendue qui regarde de côté (`regard` -2..2).
pub fn petit_oeil(t: &mut Toile, (cx, cy): (i32, i32), ouvert: f32, regard: i32, g: i32) {
    let demi = (ouvert * 2.4).round() as i32;
    for dx in -4i32..=4 {
        let bord = ((1.0 - (dx as f32 / 4.6).powi(2)).max(0.0).sqrt() * demi as f32).round() as i32;
        for dy in -bord - 1..=bord + 1 {
            let c = if dy.abs() > bord || dx.abs() == 4 {
                [0x06, 0x08, 0x14]
            } else if dx == regard {
                [0x02, 0x02, 0x06]
            } else if (dx - regard).abs() == 1 {
                [0xc8, 0xff, 0x6a]
            } else {
                [0xff, 0xe0, 0x58]
            };
            for j in 0..g {
                for i in 0..g {
                    t.poser_fin(cx + dx * g + i, cy + dy * g + j, c, 255);
                }
            }
        }
    }
    if demi > 0 {
        for j in 0..g {
            for i in 0..g {
                t.poser_fin(cx + (regard - 1) * g + i, cy - g + j, [255, 255, 255], 255);
            }
        }
    }
}

/// Derrière l'œuf de l'Éclipse : soleil et couronne qui vacille, la lune qui glisse devant
/// (toutes les ~12 s), la couronne flambe quand le soleil est couvert.
fn derriere_eclipse(t: &mut Toile, (cx, cy): (i32, i32), temps: f32) {
    let c = calques();
    let w = ((temps / 12.0).fract() * 2.0 - 1.0) * 1.3;
    let couvert = (1.0 - w.abs() * 2.2).max(0.0);
    let s = &c.soleil;
    calques::poser(t, s, (cx - s.l / 2, cy - s.h / 2), 1, 255, |col| {
        let (h, sa, v) = rgb_hsv(col);
        hsv(h, sa, (v * (0.86 + 0.3 * couvert)).min(1.0))
    });
    // Flammes de la couronne qui vacillent (pixels du bord qui s'allument et s'éteignent).
    for y in 0..s.h {
        for x in 0..s.l {
            let p = s.px(x, y);
            if p[3] == 0 {
                continue;
            }
            let k = (temps * 6.0 + bruit(x, y, 12) * TAU).sin();
            if k > 0.75 {
                t.eclairer_fin(cx - s.l / 2 + x, cy - s.h / 2 + y, [255, 230, 150], 90);
            }
        }
    }
    let l = &c.lune;
    let lx = cx + (w * s.l as f32 * 0.45) as i32;
    calques::poser(t, l, (lx - l.l / 2, cy - l.h / 2), 1, 255, |c| c);
}

/// L'œuf agrandi ×`k` (pixels fins en blocs k x k) : dessiné à part puis recopié.
#[allow(clippy::too_many_arguments)]
pub fn dessiner_oeuf_k(
    t: &mut Toile,
    k: i32,
    i: usize,
    (cx, base): (i32, i32),
    temps: f32,
    penche: f32,
    ecrase: f32,
    fiss: Option<(&Fissures, f32)>,
    lumiere: Option<(Rgb, f32)>,
    dore: bool,
    blanc: bool,
) {
    let img = image_oeuf(i);
    let mut tmp = Toile::new((img.l / 2 + 24) as usize, (img.h / 2 + 24) as usize);
    let (tcx, tb) = (tmp.lf() as i32 / 2, tmp.hf() as i32 - 12);
    dessiner_oeuf(
        &mut tmp,
        i,
        (tcx, tb),
        temps,
        penche / k as f32,
        ecrase,
        fiss,
        lumiere,
        dore,
        blanc,
    );
    copier_k(t, &tmp, k, (tcx, tb), (cx, base), 255);
}

/// Recopie `src` agrandie ×`k` (blocs), le point `ancre` de `src` posé en `dest`.
pub fn copier_k(
    t: &mut Toile,
    src: &Toile,
    k: i32,
    (ax, ay): (i32, i32),
    (cx, cy): (i32, i32),
    a: u8,
) {
    let (lf, hf) = (src.lf() as i32, src.hf() as i32);
    for y in 0..hf {
        for x in 0..lf {
            let p = src.px[(y * lf + x) as usize];
            if p[3] == 0 {
                continue;
            }
            let d = crate::toile::demultiplier(p);
            let al = (d[3] as u32 * a as u32 / 255) as u8;
            for dy in 0..k {
                for dx in 0..k {
                    let (px, py) = (cx + (x - ax) * k + dx, cy + (y - ay) * k + dy);
                    t.poser_fin(px, py, [d[0], d[1], d[2]], al);
                }
            }
        }
    }
}

/// Taille d'une vignette d'œuf (pixels monde) : place pour la couronne de l'Éclipse.
pub const VIGNETTE: (usize, usize) = (56, 64);

/// Toile d'une vignette d'œuf (boutique) : l'œuf au repos à l'instant `temps`, légère
/// oscillation, posé sur une petite ombre ; bulles du lagon autour.
pub fn vignette(i: usize, temps: f32) -> Toile {
    let mut t = Toile::new(VIGNETTE.0, VIGNETTE.1);
    let (cx, base) = (VIGNETTE.0 as i32, VIGNETTE.1 as i32 * 2 - 8);
    for dx in -16..=16 {
        let k = 1.0 - (dx as f32 / 16.0).powi(2);
        t.poser_fin(cx + dx, base + 1, [0x04, 0x0c, 0x16], (110.0 * k) as u8);
        t.poser_fin(cx + dx, base + 2, [0x04, 0x0c, 0x16], (60.0 * k) as u8);
    }
    let penche = (temps * 1.3 + i as f32).sin() * 1.2;
    // Petit saut toutes les ~2 s (chaque œuf à son rythme).
    let u = (temps * 0.5 + i as f32 * 0.3).fract();
    let saut = if u < 0.12 {
        -((u / 0.12) * PI).sin() * 4.0
    } else {
        0.0
    };
    let base = base + saut.round() as i32;
    // Derrière l'œuf de l'Éclipse : soleil et couronne, la lune qui glisse devant (vignettes
    // seulement : l'éclosion a sa propre éclipse).
    if i == ECLIPSE {
        let img = image_oeuf(i);
        derriere_eclipse(&mut t, (cx, base - img.h + 12), temps);
    }
    dessiner_oeuf(
        &mut t,
        i,
        (cx, base),
        temps,
        penche,
        1.0,
        None,
        None,
        false,
        false,
    );
    bulles_repos(&mut t, i, (cx, base), temps);
    t
}

/// Petites bulles qui montent autour de l'œuf du lagon (et de la rivière, plus rares).
fn bulles_repos(t: &mut Toile, i: usize, (cx, base): (i32, i32), temps: f32) {
    let n = match i {
        LAGON => 4,
        RIVIERE => 2,
        _ => return,
    };
    let img = image_oeuf(i);
    for b in 0..n {
        let periode = 2.8 + b as f32 * 0.6;
        let w = ((temps + b as f32 * 1.3) / periode).fract();
        let x = cx + img.l / 3 - 6
            + (bruit(b, 7, i as i32) * 16.0) as i32
            + ((temps * 2.0 + b as f32).sin() * 1.5).round() as i32;
        let y = base - 10 - (w * img.h as f32 * 1.1) as i32;
        let a = ((1.0 - w) * 230.0) as u8;
        if b % 2 == 0 {
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                t.poser_fin(x + dx, y + dy, [0xb8, 0xf0, 0xff], a);
            }
            t.poser_fin(x, y - 1, [255, 255, 255], a);
        } else {
            t.poser_fin(x, y, [0xd0, 0xf8, 0xff], a);
        }
    }
}

/// Morceau de coquille qui vole (pixels du vrai dessin), en pixels fins.
struct Eclat {
    vie: f32,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    px: Vec<(i32, i32, Rgb)>,
}

/// Particule (étincelle, poussière) en pixels fins.
struct Etincelle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    vie: f32,
    duree: f32,
    c: Rgb,
}

/// Objet qui vole (pièce, pierre, goutte d'ambre) : un calque, avec gravité et rebond.
struct Objet {
    img: &'static Img,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    vie: f32,
    /// Hauteur du sol où il rebondit (pixels fins), None = traverse.
    sol: Option<f32>,
}

/// Une éclosion en cours : toile 128x96 (pixels monde) agrandie par l'Écloserie.
pub struct Eclosion {
    pub oeuf: usize,
    pub rarete: Rarete,
    pub id: u64,
    pub debut: f64,
    pub toile: Toile,
    poisson: Aquarium,
    pub texture: Option<eframe::egui::TextureHandle>,
    /// Agrandissement du poisson révélé (x1 à x3 selon sa taille).
    zoom: i32,
    pub shiny: bool,
    modif: Option<crate::jeu::Modif>,
    /// Œuf ordinaire devenu doré à l'achat.
    pub dore: bool,
    /// Rareté garantie par la pitié.
    pub pitie: bool,
    d: Deroule,
    fissures: Fissures,
    eclats: Vec<Eclat>,
    etincelles: Vec<Etincelle>,
    objets: Vec<Objet>,
    /// Ondes de choc (début, couleur) : anneaux qui s'élargissent à chaque temps fort.
    ondes: Vec<(f32, Rgb)>,
    /// Éclair blanc plein écran jusqu'à cet instant.
    flash: f32,
    /// Dernier instant dessiné (les particules avancent par petits pas).
    t_prec: f32,
    alea: Alea,
    /// Secousse (modèle « trauma » : force², décroît vite).
    trauma: f32,
    /// Instants déjà fêtés (coups, paliers, éclatement, bandeau…).
    fait: u64,
    /// Sons à jouer (son, hauteur), vidés par l'Écloserie à chaque image.
    pub sons: Vec<(Son, f32)>,
}

const L: i32 = 128;
const HT: i32 = 96;
/// Bas de l'œuf (pixels fins).
const SOL: i32 = HT * 2 - 6;
/// Œuf de l'éclosion agrandi ×2 (pixels monde) : il remplit la scène.
const K: i32 = 2;
/// Fond de la scène.
const FOND: Rgb = [0x0c, 0x22, 0x35];

impl Eclosion {
    /// Prépare l'éclosion du poisson `f` sorti de l'œuf `oeuf`.
    pub fn nouvelle(f: &Poisson, oeuf: usize, debut: f64, dore: bool, pitie: bool) -> Eclosion {
        let (aq, zoom) = Eclosion::creature(f);
        Eclosion {
            oeuf,
            rarete: f.rarete,
            id: f.id,
            debut,
            toile: Toile::new(L as usize, HT as usize),
            poisson: aq,
            texture: None,
            zoom,
            shiny: f.shiny,
            modif: f.modif,
            dore,
            pitie,
            d: Deroule::nouveau(f.rarete, f.shiny, f.modif.is_some(), dore),
            fissures: Fissures::nouvelles(image_oeuf(oeuf), f.id),
            eclats: Vec::new(),
            etincelles: Vec::new(),
            objets: Vec::new(),
            ondes: Vec::new(),
            flash: -1.0,
            t_prec: 0.0,
            alea: Alea(f.id ^ 0xec1a),
            trauma: 0.0,
            fait: 0,
            sons: Vec::new(),
        }
    }

    /// La créature révélée dans un petit aquarium fixe, et son agrandissement.
    fn creature(f: &Poisson) -> (Aquarium, i32) {
        // Petit poisson : x2 ou x3 ; gros : x1, réduit pour tenir.
        let e = crate::scene::espece_de(f);
        let k = crate::poisson::echelle_individu(f.taille);
        let m = 2.0 * crate::poisson::MARGE as f32;
        let fin = FIN as f32;
        // La créature tient dans ~75 % de la scène (sous le bandeau).
        let petit = e.dims.0 as f32 * k + m <= 48.0 * fin && e.dims.1 as f32 * k + m <= 34.0 * fin;
        let tres_petit =
            e.dims.0 as f32 * k + m <= 32.0 * fin && e.dims.1 as f32 * k + m <= 22.0 * fin;
        let (s, zoom) = if tres_petit {
            (crate::scene::sprite_de(f), 3)
        } else if petit {
            (crate::scene::sprite_de(f), 2)
        } else {
            (sprite_ajuste(f, 98 * 2, 68 * 2), 1)
        };
        let (al, ah) = (128 / zoom as usize, 96 / zoom as usize);
        let mut aq = Aquarium::new(al, ah, 9);
        aq.decor = false;
        aq.avec_compteur = false;
        let mut a = Alea(f.id);
        let mut n = Nageur::new(
            f.id,
            f.rarete,
            Arc::new(s),
            al as f32 / 2.0,
            ah as f32 / 2.0,
            &mut a,
        );
        n.fixe = true;
        n.gauche = false;
        n.forme = f.forme();
        aq.nageurs.push(n);
        (aq, zoom)
    }

    /// Change le poisson montré (captures : forme imposée).
    pub fn changer_poisson(&mut self, f: &Poisson) {
        let (aq, zoom) = Eclosion::creature(f);
        self.poisson = aq;
        self.zoom = zoom;
        self.shiny = f.shiny;
        self.modif = f.modif;
        self.d = Deroule::nouveau(f.rarete, f.shiny, f.modif.is_some(), self.dore);
    }

    /// Durée de l'attente avant l'éclatement : plus longue pour les raretés hautes, les
    /// shiny et les modificateurs (la montée fait partie de la récompense).
    pub fn suspense(&self) -> f64 {
        self.d.eclat() as f64
    }

    /// Scène renforcée (haute rareté, shiny, modificateur).
    fn grande(&self) -> bool {
        self.d.fort
    }

    /// Passe directement à l'éclatement (clic).
    pub fn passer(&mut self, maintenant: f64) {
        let s = self.suspense();
        if maintenant - self.debut < s {
            self.debut = maintenant - s;
        }
    }

    /// Le poisson est révélé (texte et boutons sous l'image).
    pub fn revele(&self, maintenant: f64) -> bool {
        maintenant - self.debut >= self.d.total() as f64
    }

    /// Couleur finale (bandeau, rayons) : celle de la rareté.
    fn couleur_finale(&self) -> Rgb {
        self.rarete.couleur()
    }

    fn son(&mut self, s: Son, h: f32) {
        self.sons.push((s, h));
    }

    /// Une fois seulement (bit `b` de `fait`).
    fn une_fois(&mut self, b: u32) -> bool {
        let m = 1u64 << b;
        let neuf = self.fait & m == 0;
        self.fait |= m;
        neuf
    }

    /// Dessine l'image de l'instant `t` (secondes depuis le début) sur `self.toile`.
    pub fn dessiner(&mut self, t: f64) {
        let t = t as f32;
        let dt = (t - self.t_prec).clamp(0.0, 0.25);
        self.t_prec = t;
        // Particules et éclats : petits pas (images régulières même si l'appli rame).
        let pas = (dt / (1.0 / 120.0)).ceil().max(1.0) as usize;
        for _ in 0..pas {
            self.avancer(dt / pas as f32);
        }
        self.trauma = (self.trauma - 1.6 * dt).max(0.0);
        let p = self.d.palier_a(t);
        let couleur = couleur_palier(p, t);
        let mut scene = Toile::new(L as usize, HT as usize);
        scene.rect(0, 0, L, HT, FOND, 255);
        let eclat = self.d.eclat();
        if t < self.d.scene.0 {
            // Lueur en paliers derrière l'œuf, qui monte avec l'attente.
            let force = ((t - 0.6) / (self.d.scene.0 - 0.6)).clamp(0.0, 1.0);
            let pause = self.d.en_pause(t);
            lueur(
                &mut scene,
                couleur,
                if pause { force * 0.4 } else { force },
                40.0,
            );
            self.anticipation(&mut scene, t, couleur);
        } else if t < eclat {
            let u = (t - self.d.scene.0) / (eclat - self.d.scene.0);
            self.scene_oeuf(&mut scene, t, u, couleur);
        } else {
            self.revelation(&mut scene, t - eclat);
        }
        for p in &self.etincelles {
            let a = (255.0 * (1.0 - p.vie / p.duree)).clamp(0.0, 255.0) as u8;
            let (x, y) = (p.x.round() as i32 & !1, p.y.round() as i32 & !1);
            // Étincelles sur la grille monde (comme l'œuf).
            scene.eclairer(x / 2, y / 2, p.c, a);
        }
        for o in &self.objets {
            let a = (255.0 * (1.0 - ((o.vie - 2.2) / 0.6).clamp(0.0, 1.0))) as u8;
            let (x, y) = (o.x.round() as i32 & !1, o.y.round() as i32 & !1);
            calques::poser(
                &mut scene,
                o.img,
                (x - o.img.l / 2, y - o.img.h),
                1,
                a,
                |c| c,
            );
        }
        // Ondes de choc des temps forts : anneau d'un pixel monde qui s'élargit.
        for &(d, c) in &self.ondes {
            let v = t - d;
            if !(0.0..0.5).contains(&v) {
                continue;
            }
            let r = 10.0 + v * 150.0;
            let (ccx, ccy) = (L as f32 / 2.0, (SOL - 90) as f32 / 2.0);
            let a = (220.0 * (1.0 - v / 0.5)) as u8;
            for k in 0..160 {
                let ang = k as f32 / 160.0 * TAU;
                let (x, y) = (ccx + ang.cos() * r, ccy + ang.sin() * r * 0.8);
                scene.eclairer(x.round() as i32, y.round() as i32, c, a);
            }
        }
        if t < self.flash {
            calques::assombrir(&mut scene, [255, 255, 255], 0.85);
        }
        // Secousse de l'écran : décalage entier (pixels monde), force = trauma².
        let k = self.trauma * self.trauma;
        let (ox, oy) = (
            (k * 5.0 * (t * 47.0).sin() * (t * 13.0).cos()).round() as i32 * 2,
            (k * 4.0 * (t * 53.0 + 1.3).sin()).round() as i32 * 2,
        );
        let toile = &mut self.toile;
        toile.rect(0, 0, L, HT, FOND, 255);
        let lf = scene.lf() as i32;
        let hf = scene.hf() as i32;
        for y in 0..hf {
            for x in 0..lf {
                let (sx, sy) = (x - ox, y - oy);
                if sx >= 0 && sy >= 0 && sx < lf && sy < hf {
                    toile.px[(y * lf + x) as usize] = scene.px[(sy * lf + sx) as usize];
                }
            }
        }
    }

    fn avancer(&mut self, dt: f32) {
        for e in &mut self.eclats {
            e.vie += dt;
            e.vy += 300.0 * dt;
            e.x += e.vx * dt;
            e.y += e.vy * dt;
        }
        // Les éclats tombent hors de la scène et s'effacent (jamais posés sur le poisson).
        self.eclats
            .retain(|e| e.vie < 1.4 && e.y < (HT * 2 + 40) as f32);
        for p in &mut self.etincelles {
            p.vie += dt;
            p.vy += 40.0 * dt;
            p.vx *= 1.0 - 1.5 * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
        self.etincelles.retain(|p| p.vie < p.duree);
        for o in &mut self.objets {
            o.vie += dt;
            o.vy += 340.0 * dt;
            o.x += o.vx * dt;
            o.y += o.vy * dt;
            if let Some(s) = o.sol {
                if o.y > s && o.vy > 0.0 {
                    o.y = s;
                    o.vy *= -0.38;
                    o.vx *= 0.6;
                }
            }
        }
        self.objets
            .retain(|o| o.vie < 2.8 && o.y < (HT * 2 + 60) as f32);
    }

    /// Fait jaillir `n` étincelles de `c` depuis (x, y) (pixels fins).
    fn gerbe(&mut self, (x, y): (f32, f32), n: usize, c: Rgb, vitesse: f32) {
        for _ in 0..n {
            let a = &mut self.alea;
            let ang = a.f() * TAU;
            let v = a.entre(0.3, 1.0) * vitesse;
            self.etincelles.push(Etincelle {
                x,
                y,
                vx: ang.cos() * v,
                vy: ang.sin() * v - vitesse * 0.3,
                vie: 0.0,
                duree: a.entre(0.4, 1.1),
                c,
            });
        }
    }

    /// Temps fort : éclair, secousse, onde de choc, gerbe.
    fn temps_fort(&mut self, t: f32, c: Rgb, force: f32) {
        self.flash = t + 0.035;
        self.trauma = (self.trauma + force).min(1.0);
        self.ondes.push((t, c));
        let centre = (L as f32, (SOL - 90) as f32);
        self.gerbe(centre, (20.0 + 40.0 * force) as usize, c, 120.0);
    }

    /// Avant la scène de l'œuf : il tombe, s'écrase, oscille, se fêle (lumière blanche), puis
    /// chaque palier : pause, saut de couleur, secousse ; shiny et modificateur : un temps fort
    /// chacun.
    fn anticipation(&mut self, scene: &mut Toile, t: f32, couleur: Rgb) {
        let img = image_oeuf(self.oeuf);
        let (il, ih) = (img.l * K, img.h * K);
        let (cx, base) = (L, SOL);
        // Ombre au sol.
        for dx in -(il / 2)..=(il / 2) {
            let k = 1.0 - (dx as f32 / (il as f32 / 2.0)).powi(2);
            scene.poser_fin(cx + dx, base + 1, [0x04, 0x0c, 0x16], (120.0 * k) as u8);
            scene.poser_fin(cx + dx, base + 2, [0x04, 0x0c, 0x16], (60.0 * k) as u8);
        }
        // Entrée : tombe d'en haut (0,3 s), s'écrase en touchant le sol, rebondit.
        let (mut y, mut ecrase) = (0.0, 1.0);
        if t < 0.3 {
            let u = t / 0.3;
            y = -120.0 * (1.0 - u * u);
        } else if t < 0.48 {
            let u = (t - 0.3) / 0.18;
            ecrase = 1.0 - 0.14 * (u * PI).sin();
            if self.une_fois(0) {
                self.trauma = (self.trauma + 0.25).min(1.0);
                self.son(Son::Chute, 1.0);
                self.gerbe(((cx - 14) as f32, base as f32), 6, [0x80, 0xa0, 0xb0], 30.0);
                self.gerbe(((cx + 14) as f32, base as f32), 6, [0x80, 0xa0, 0xb0], 30.0);
            }
        }
        let p = self.d.palier_a(t);
        let u = ((t - 0.4) / (paliers::ENTREE - 0.4)).clamp(0.0, 1.0);
        // Oscillation qui grandit avec l'attente et le palier.
        let ampleur = 1.5 + 5.0 * u + 1.6 * p as f32;
        let mut penche = (t * (5.0 + 7.0 * u + p as f32)).sin() * ampleur;
        // Coups secs : l'œuf se fêle d'un cran.
        let mut fiss = 0.0;
        for k in 0..3 {
            let tc = self.d.coups[k];
            if t >= tc {
                fiss = 0.2 * (k + 1) as f32;
                if self.une_fois(1 + k as u32) {
                    self.trauma = (self.trauma + 0.2).min(1.0);
                    let col = OEUFS[self.oeuf].coquille[0];
                    self.gerbe((cx as f32, (base - ih / 2) as f32), 5, col, 50.0);
                    self.son(Son::Fele(k as u8), 0.97 + 0.04 * k as f32);
                    if k == 2 {
                        self.son(Son::Palier(0), 1.0);
                    }
                }
                if t - tc < 0.1 {
                    ecrase = 0.93;
                    penche *= 1.8;
                }
            }
        }
        // Fêlures qui s'élargissent avec chaque palier.
        fiss += 0.4 * p as f32 / paliers::NB_PALIERS as f32
            + 0.1 * ((t - paliers::ENTREE) / 3.0).clamp(0.0, 1.0);
        let mut lumiere = (t > 0.8).then_some((couleur, 0.35 + 0.15 * p as f32 + 0.2 * u));
        // Paliers : pause (l'œuf se fige, la lumière baisse), puis le saut.
        for k in 0..self.d.paliers.len() {
            let d0 = self.d.paliers[k];
            if (d0..d0 + PAUSE).contains(&t) {
                penche = 0.0;
                lumiere = lumiere.map(|(c, f)| (c, f * 0.35));
            }
            if t >= d0 + PAUSE && self.une_fois(8 + k as u32) {
                let c = couleur_palier(k + 1, t);
                self.temps_fort(t, c, 0.3 + 0.1 * k as f32);
                self.son(Son::Palier(k as u8 + 1), 1.0);
            }
        }
        // Temps forts en plus : shiny (étoiles roses et arc-en-ciel), modificateur (sa couleur).
        for k in 0..self.d.extras.len() {
            let (d0, e) = self.d.extras[k];
            if (d0..d0 + PAUSE).contains(&t) {
                penche = 0.0;
                lumiere = lumiere.map(|(c, f)| (c, f * 0.35));
            }
            if t >= d0 + PAUSE && t < d0 + paliers::TEMPS_EXTRA {
                let c = match e {
                    Extra::Shiny => hsv(t * 1.4, 0.45, 1.0),
                    Extra::Modif => self.modif.map_or(couleur, |m| m.couleur()),
                };
                lumiere = Some((c, 1.0));
                if self.une_fois(16 + k as u32) {
                    self.temps_fort(t, c, 0.4);
                    match e {
                        Extra::Shiny => {
                            self.son(Son::Shiny, 1.0);
                            self.gerbe(
                                (cx as f32, (base - ih / 2) as f32),
                                30,
                                [255, 190, 236],
                                150.0,
                            );
                        }
                        Extra::Modif => self.son(Son::Modif, 1.0),
                    }
                }
                if e == Extra::Shiny {
                    let mut a = Alea(self.id ^ (t * 20.0) as u64);
                    for _ in 0..4 {
                        let (x, y) = (a.entre(40.0, 216.0) as i32, a.entre(20.0, 180.0) as i32);
                        calques::etoile(scene, (x, y), 2 + a.n(3) as i32, [255, 200, 240], 2);
                    }
                }
            }
        }
        if self.d.en_pause(t) {
            ecrase = 1.0;
        }
        // La montée finale avant la scène : tremble de plus en plus vite.
        let fin = self.d.scene.0;
        if t > fin - 0.5 {
            let v = (t - (fin - 0.5)) / 0.5;
            penche = (t * 60.0).sin() * (2.0 + 2.0 * v) * if self.grande() { 1.6 } else { 1.0 };
            self.trauma = self.trauma.max(0.2 + 0.25 * v);
            if self.une_fois(20) {
                self.son(Son::Montee, 1.0);
            }
        }
        let ox = (penche * 0.5).round() as i32;
        dessiner_oeuf_k(
            scene,
            K,
            self.oeuf,
            (cx + ox, base + y as i32),
            t,
            penche,
            ecrase,
            Some((&self.fissures, fiss.min(1.0))),
            lumiere,
            self.dore && t >= 0.9,
            // Changement en or : un éclair blanc de deux images.
            self.dore && (0.87..0.92).contains(&t),
        );
        // Œuf ordinaire qui devient doré : à 0,9 s, l'or balaie l'œuf, gerbe et secousse.
        if self.dore && (0.9..1.3).contains(&t) {
            balayage(scene, [0xff, 0xd8, 0x50], (t - 0.9) / 0.4 / 3.0);
            if self.une_fois(6) {
                self.trauma = (self.trauma + 0.5).min(1.0);
                let c = (cx as f32, (base - ih / 2) as f32);
                self.gerbe(c, 40, [0xff, 0xe0, 0x60], 120.0);
                self.son(Son::Tresor, 1.1);
            }
        }
        // Rayons qui sortent des fêlures dès le premier palier (plus nombreux ensuite).
        if let Some((c, f)) = lumiere.filter(|_| p >= 1 && !self.d.en_pause(t)) {
            let n = 6 + 2 * p;
            for k in 0..n {
                let ang = k as f32 / n as f32 * TAU + t * 0.6;
                for r in (il / 4)..(il / 4 + 14 + 4 * p as i32) {
                    let (x, y) = (
                        cx as f32 / 2.0 + ang.cos() * r as f32,
                        (base - ih / 2) as f32 / 2.0 + ang.sin() * r as f32 * 0.8,
                    );
                    let a = (f * 110.0 * (1.0 - (r - il / 4) as f32 / (14 + 4 * p) as f32)) as u8;
                    scene.eclairer(x.round() as i32, y.round() as i32, c, a);
                }
            }
        }
    }

    /// Après l'éclatement (`apres` secondes) : éclair, éclats, rayons, poisson qui rebondit
    /// (ou bondit hors de l'eau), bandeau de rareté qui claque.
    fn revelation(&mut self, scene: &mut Toile, apres: f32) {
        let couleur = self.couleur_finale();
        let img = image_oeuf(self.oeuf);
        let ih = img.h * K;
        let (cx, base) = (L, SOL);
        let centre = (cx as f32, (base - ih / 2) as f32);
        // Éclatement : les morceaux de coquille partent, gerbe d'étincelles, secousse.
        if self.une_fois(24) {
            self.trauma = if self.grande() { 0.95 } else { 0.7 };
            // Les œufs engloutis par leur scène (trou noir, ambre fondu, vague, remous) ne
            // laissent pas de coquille.
            if !matches!(self.oeuf, COSMIQUE | AMBRE | LAGON | RIVIERE) {
                self.eclater();
            }
            let n = if self.grande() { 70 } else { 36 };
            self.gerbe(centre, n, couleur, 150.0);
            self.gerbe(centre, n / 3, [255, 255, 255], 110.0);
            self.son(
                if self.grande() {
                    Son::EclatFort
                } else {
                    Son::Eclat
                },
                1.0,
            );
            self.eclat_theme(centre);
        }
        if apres >= 0.22 && self.une_fois(25) {
            self.son(Son::Jingle(self.rarete.index() as u8), 1.0);
        }
        // Fond propre à l'œuf (galaxie, soleil, huître ouverte, pièces…).
        self.fond_revelation(scene, apres);
        // Éclair qui part du centre et s'efface en ~0,15 s (blanc au cœur, couleur au bord).
        if apres < 0.18 {
            let f = 1.0 - apres / 0.18;
            let (ccx, ccy) = (centre.0 / 2.0, centre.1 / 2.0);
            for y in 0..HT {
                for x in 0..L {
                    let d = ((x as f32 - ccx).powi(2) + ((y as f32 - ccy) * 1.2).powi(2)).sqrt();
                    let k = (1.0 - d / (40.0 + 140.0 * (1.0 - f))).max(0.0);
                    if k > 0.0 {
                        let c = melange(couleur, [255, 255, 255], k);
                        scene.poser(x, y, c, (255.0 * f * (0.35 + 0.65 * k)) as u8);
                    }
                }
            }
        }
        // Rayons tournants (shiny : arc-en-ciel, deux fois plus nombreux), en pixels monde.
        let nr = if self.shiny {
            24
        } else if self.grande() {
            16
        } else {
            12
        };
        for i in 0..nr {
            let ang = i as f32 / nr as f32 * TAU + apres * 0.4;
            let long = (apres * 160.0).min(75.0) as i32;
            let c = if self.shiny || self.d.palier == 4 {
                hsv(i as f32 / nr as f32 + apres * 0.3, 0.55, 1.0)
            } else {
                couleur
            };
            let fort = if self.shiny || self.grande() {
                110.0
            } else {
                80.0
            };
            let vif = 1.0 + 0.8 * (1.0 - apres / 0.5).max(0.0);
            for r in 10..long {
                let (x, y) = (
                    centre.0 / 2.0 + ang.cos() * r as f32,
                    centre.1 / 2.0 + ang.sin() * r as f32 * 0.8,
                );
                let a = (fort * vif * (1.0 - r as f32 / 75.0)).min(255.0) as u8;
                scene.eclairer(x.round() as i32, y.round() as i32, c, a);
            }
        }
        // Shiny : pluie d'étoiles à quatre branches.
        if self.shiny {
            let mut a = Alea(self.id ^ 0x5419);
            for _ in 0..22 {
                let (x, y) = (a.entre(12.0, 244.0), a.entre(12.0, 180.0));
                let (debut, vie) = (a.entre(0.0, 2.5), a.entre(0.5, 0.9));
                let u = (apres - debut).rem_euclid(2.6) / vie;
                if u >= 1.0 {
                    continue;
                }
                let o = ((1.0 - (2.0 * u - 1.0).abs()) * 3.5).round() as i32;
                let c = [[255, 246, 190], [255, 190, 236], [190, 240, 255]][a.n(3) as usize];
                calques::etoile(scene, (x as i32, (y - u * 8.0) as i32), o, c, 2);
            }
        }
        // Modificateur : sa couleur en anneau qui s'élargit.
        if let Some(m) = self.modif {
            let r = (apres * 70.0) % 60.0;
            for k in 0..120 {
                let ang = k as f32 / 120.0 * TAU;
                let (x, y) = (
                    centre.0 / 2.0 + ang.cos() * r,
                    centre.1 / 2.0 + ang.sin() * r * 0.8,
                );
                let a = (200.0 * (1.0 - r / 60.0)) as u8;
                scene.eclairer(x.round() as i32, y.round() as i32, m.couleur(), a);
            }
        }
        // Éclats de coquille, derrière le poisson, qui s'effacent.
        for e in &self.eclats {
            let a = (255.0 * (1.0 - ((e.vie - 0.7) / 0.7).clamp(0.0, 1.0))) as u8;
            let (ex, ey) = (e.x.round() as i32 & !1, e.y.round() as i32 & !1);
            for &(x, y, c) in &e.px {
                scene.poser_fin(ex + x, ey + y, c, a);
            }
        }
        // Poisson révélé : tombe d'un coup et rebondit (ressort amorti), blanc 2 images ;
        // rivière : il bondit hors du remous.
        self.poisson.avancer(1.0 / 60.0);
        self.poisson.dessiner(&Partie::nouvelle(0, 0));
        let u = (apres - 0.04).max(0.0);
        let rebond = if self.oeuf == RIVIERE {
            let w = (u / 0.7).min(1.0);
            (110.0 * (1.0 - w) * (1.0 - w) - 70.0 * (w * PI).sin() * (1.0 - w)).round() as i32
        } else {
            (-44.0 * (-6.0 * u).exp() * (15.0 * u).cos()).round() as i32
        };
        let blanc = u < 0.06;
        let src = &self.poisson.toile;
        let z = self.zoom;
        let (sl, sh) = (src.lf() as i32, src.hf() as i32);
        // Au milieu de la scène, sous le bandeau.
        let (ox, oy) = (L - sl * z / 2, HT + 14 - sh * z / 2 + rebond);
        for y in 0..sh {
            for x in 0..sl {
                let p = src.px[(y * sl + x) as usize];
                if p[3] == 0 {
                    continue;
                }
                let c = crate::toile::demultiplier(p);
                let c = if blanc {
                    [255, 255, 255]
                } else {
                    [c[0], c[1], c[2]]
                };
                for dy in 0..z {
                    for dx in 0..z {
                        scene.poser_fin(ox + x * z + dx, oy + y * z + dy, c, p[3]);
                    }
                }
            }
        }
        // Bandeau de rareté qui claque (×4 puis ×3 puis ×2, petite secousse).
        if apres > 0.3 {
            let v = apres - 0.3;
            if self.une_fois(26) {
                self.trauma = (self.trauma + 0.35).min(1.0);
                self.gerbe((cx as f32, 30.0), 18, couleur, 90.0);
            }
            let k = if v < 0.05 {
                4
            } else if v < 0.1 {
                3
            } else {
                2
            };
            let nom = self.rarete.nom().to_uppercase();
            let large = (v / 0.12).min(1.0);
            // Bande sombre bordée de la couleur, qui s'ouvre depuis le centre.
            let bande = (L as f32 * large) as i32;
            let (y0, hb) = (4, 5 * k + 6);
            let x0 = (L - bande) / 2;
            scene.rect(x0, y0, bande, hb, [0x08, 0x12, 0x20], 230);
            scene.rect(x0, y0, bande, 1, couleur, 255);
            scene.rect(x0, y0 + hb - 1, bande, 1, couleur, 255);
            texte_gros(scene, &nom, (L / 2, y0 + 3 + 2 * k), k, couleur);
            // Sous le bandeau : shiny et modificateur (après un court temps), œuf doré, pitié.
            let mut lignes: Vec<(String, Rgb)> = Vec::new();
            if v > 0.25 {
                if self.shiny {
                    lignes.push((crate::i18n::tr("SHINY !").into(), [0xff, 0xb8, 0xe8]));
                }
                if let Some(m) = self.modif {
                    lignes.push((m.nom().to_uppercase(), m.couleur()));
                }
            }
            if self.dore {
                lignes.push((crate::i18n::tr("ŒUF DORÉ !").into(), [0xff, 0xdc, 0x60]));
            }
            if self.pitie {
                lignes.push((crate::i18n::tr("GARANTI !").into(), [0xa8, 0xf0, 0xff]));
            }
            for (j, (l, c)) in lignes.iter().enumerate() {
                // Sur sa propre plaque sombre, sous le bandeau.
                let w = crate::toile::largeur_texte(l) + 8;
                let y = y0 + hb + 2 + j as i32 * 9;
                scene.rect(L / 2 - w / 2, y, w, 9, [0x08, 0x12, 0x20], 220);
                scene.rect(L / 2 - w / 2, y + 8, w, 1, *c, 200);
                texte_gros(scene, l, (L / 2, y + 2), 1, *c);
            }
        }
    }

    /// Découpe le vrai dessin de l'œuf en morceaux (cellules autour de points tirés) qui
    /// partent du centre, vers le haut surtout.
    fn eclater(&mut self) {
        let img = image_oeuf(self.oeuf);
        let (x0, y0) = (L - img.l * K / 2, SOL - img.h * K);
        let n = 16;
        let a = &mut self.alea;
        let graines: Vec<(i32, i32)> = (0..n)
            .map(|_| (a.n(img.l as u32) as i32, a.n(img.h as u32) as i32))
            .collect();
        let mut morceaux: Vec<Vec<(i32, i32, Rgb)>> = vec![Vec::new(); n];
        for y in 0..img.h {
            for x in 0..img.l {
                let p = img.px(x, y);
                if p[3] == 0 {
                    continue;
                }
                let k = (0..n)
                    .min_by_key(|&k| (graines[k].0 - x).pow(2) + (graines[k].1 - y).pow(2))
                    .unwrap_or(0);
                let mut c = [p[0], p[1], p[2]];
                if self.dore {
                    // Même or que l'œuf doré.
                    let l = (c[0] as f32 * 0.3 + c[1] as f32 * 0.59 + c[2] as f32 * 0.11) / 255.0;
                    c = melange(c, melange([0xa0, 0x58, 0x00], [0xff, 0xe8, 0x50], l), 0.9);
                }
                for (bx, by) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    morceaux[k].push(((x - graines[k].0) * K + bx, (y - graines[k].1) * K + by, c));
                }
            }
        }
        let (mx, my) = (img.l as f32 / 2.0, img.h as f32 / 2.0);
        for (k, px) in morceaux.into_iter().enumerate() {
            if px.is_empty() {
                continue;
            }
            let (gx, gy) = (graines[k].0 as f32, graines[k].1 as f32);
            let (dx, dy) = (gx - mx, gy - my);
            let d = dx.hypot(dy).max(1.0);
            let v = a.entre(70.0, 150.0);
            self.eclats.push(Eclat {
                x: x0 as f32 + gx * K as f32,
                y: y0 as f32 + gy * K as f32,
                vie: 0.0,
                vx: dx / d * v + a.entre(-20.0, 20.0),
                vy: dy / d * v - a.entre(60.0, 140.0),
                px,
            });
        }
    }
}

/// Lueur en paliers derrière l'œuf (anneaux pleins au bord tramé, pas de dégradé lisse).
fn lueur(t: &mut Toile, c: Rgb, force: f32, rayon: f32) {
    if force <= 0.0 {
        return;
    }
    let c = melange(c, FOND, 0.45);
    calques::halo(t, (L, SOL - 90), rayon * 2.0, c, force * 0.8, 2);
}

/// Raretés hautes pendant la montée : bandes de la couleur de la rareté qui balaient l'écran.
fn balayage(t: &mut Toile, c: Rgb, u: f32) {
    let pos = (u * 3.0).fract() * (L + 40) as f32 - 20.0;
    for y in 0..HT {
        for x in 0..L {
            let d = (x as f32 + y as f32 * 0.5 - pos).abs();
            if d < 6.0 && (x + y) % 2 == 0 {
                t.eclairer(x, y, c, (60.0 * (1.0 - d / 6.0)) as u8);
            }
        }
    }
}

/// Texte de la petite police agrandi ×`k` (pixels monde), centré en `cx`, contour sombre.
pub fn texte_gros(t: &mut Toile, s: &str, (cx, y): (i32, i32), k: i32, c: Rgb) {
    let w = crate::toile::largeur_texte(s);
    let mut tmp = Toile::new((w + 4) as usize, 9);
    tmp.texte(2, 2, s, c, 255, Some([0x04, 0x08, 0x10]));
    let x0 = cx - (w + 4) * k / 2;
    for yy in 0..9 {
        for xx in 0..w + 4 {
            let p = tmp.px[(yy * 2 * tmp.lf() as i32 + xx * 2) as usize];
            if p[3] == 0 {
                continue;
            }
            let d = crate::toile::demultiplier(p);
            t.rect(
                x0 + xx * k,
                y - 2 * k + yy * k,
                k,
                k,
                [d[0], d[1], d[2]],
                d[3],
            );
        }
    }
}

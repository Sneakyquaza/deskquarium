//! Calques pixel art des œufs (v0.11) : éléments dessinés À PART par l'IA sur fond uni
//! (coquille à fenêtre, contenu intérieur, effets : trou noir, galaxie, soleil, lune, perle…),
//! que le code anime (rotation, parallaxe, masques, lueurs en paliers, déformation). Tout reste
//! sur une grille de pixels entiers : `g` = taille d'un pixel du calque en pixels fins.

use crate::decor::Img;
use crate::toile::{bayer, Rgb, Toile};
use std::sync::OnceLock;

pub struct Calques {
    /// Coquille de cristal de l'œuf cosmique : fenêtre = vert pur (#00FF00).
    pub cosmique_coque: Img,
    pub nebuleuse: Img,
    pub galaxie: Img,
    pub trou_noir: Img,
    /// Œuf d'ambre vide (le fossile est un calque à part, en parallaxe).
    pub ambre_coque: Img,
    pub fossile: Img,
    pub gouttes: Vec<Img>,
    pub soleil: Img,
    pub lune: Img,
    pub anneau: Img,
    pub oeil: Img,
    pub lanterne: Img,
    pub huitre_bas: Img,
    pub huitre_haut: Img,
    pub perle: Img,
    pub tresor: Vec<Img>,
    pub prisme: Img,
    pub eclaboussure: Img,
}

/// Les calques, décodés une fois.
pub fn calques() -> &'static Calques {
    static C: OnceLock<Calques> = OnceLock::new();
    C.get_or_init(|| {
        let c = |b: &[u8]| Img::charger(b);
        Calques {
            cosmique_coque: c(include_bytes!(
                "../../assets/oeufs/calques/cosmique_coque.png"
            )),
            nebuleuse: c(include_bytes!("../../assets/oeufs/calques/nebuleuse.png")),
            galaxie: c(include_bytes!("../../assets/oeufs/calques/galaxie.png")),
            trou_noir: c(include_bytes!("../../assets/oeufs/calques/trou_noir.png")),
            ambre_coque: c(include_bytes!("../../assets/oeufs/calques/ambre_coque.png")),
            fossile: c(include_bytes!("../../assets/oeufs/calques/fossile.png")),
            gouttes: ilots(&c(include_bytes!("../../assets/oeufs/calques/gouttes.png"))),
            soleil: c(include_bytes!("../../assets/oeufs/calques/soleil.png")),
            lune: c(include_bytes!("../../assets/oeufs/calques/lune.png")),
            anneau: c(include_bytes!("../../assets/oeufs/calques/anneau.png")),
            oeil: c(include_bytes!("../../assets/oeufs/calques/oeil.png")),
            lanterne: c(include_bytes!("../../assets/oeufs/calques/lanterne.png")),
            huitre_bas: c(include_bytes!("../../assets/oeufs/calques/huitre_bas.png")),
            huitre_haut: c(include_bytes!("../../assets/oeufs/calques/huitre_haut.png")),
            perle: c(include_bytes!("../../assets/oeufs/calques/perle.png")),
            tresor: ilots(&c(include_bytes!("../../assets/oeufs/calques/tresor.png"))),
            prisme: c(include_bytes!("../../assets/oeufs/calques/prisme.png")),
            eclaboussure: c(include_bytes!(
                "../../assets/oeufs/calques/eclaboussure.png"
            )),
        }
    })
}

/// Pixel « fenêtre » de la coquille cosmique (vert pur gardé par le nettoyage).
pub fn fenetre(p: [u8; 4]) -> bool {
    p[3] > 0 && p[1] > 200 && p[0] < 90 && p[2] < 90
}

/// Découpe une planche (pièces, gouttes) en îlots 8-connexes recadrés, du plus gros au plus
/// petit ; les miettes (< 6 pixels) sont ignorées.
pub fn ilots(img: &Img) -> Vec<Img> {
    let (l, h) = (img.l, img.h);
    let mut vu = vec![false; (l * h) as usize];
    let mut v = Vec::new();
    for y0 in 0..h {
        for x0 in 0..l {
            if vu[(y0 * l + x0) as usize] || img.px(x0, y0)[3] == 0 {
                continue;
            }
            let mut pile = vec![(x0, y0)];
            vu[(y0 * l + x0) as usize] = true;
            let mut pts = Vec::new();
            while let Some((x, y)) = pile.pop() {
                pts.push((x, y));
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let (a, b) = (x + dx, y + dy);
                        if a >= 0 && b >= 0 && a < l && b < h {
                            let i = (b * l + a) as usize;
                            if !vu[i] && img.px(a, b)[3] > 0 {
                                vu[i] = true;
                                pile.push((a, b));
                            }
                        }
                    }
                }
            }
            if pts.len() < 6 {
                continue;
            }
            let x1 = pts.iter().map(|p| p.0).min().unwrap_or(0);
            let y1 = pts.iter().map(|p| p.1).min().unwrap_or(0);
            let x2 = pts.iter().map(|p| p.0).max().unwrap_or(0);
            let y2 = pts.iter().map(|p| p.1).max().unwrap_or(0);
            let (il, ih) = (x2 - x1 + 1, y2 - y1 + 1);
            let mut px = vec![[0u8; 4]; (il * ih) as usize];
            for (x, y) in pts {
                px[((y - y1) * il + x - x1) as usize] = img.px(x, y);
            }
            v.push(Img { l: il, h: ih, px });
        }
    }
    v.sort_by_key(|i| -(i.l * i.h));
    v
}

/// Pose `img` (coin haut-gauche (`x`, `y`) en pixels fins), chaque pixel du calque en bloc
/// `g` x `g`, opacité `a`, couleur passée par `f`.
pub fn poser(t: &mut Toile, img: &Img, (x, y): (i32, i32), g: i32, a: u8, f: impl Fn(Rgb) -> Rgb) {
    for sy in 0..img.h {
        for sx in 0..img.l {
            let p = img.px(sx, sy);
            if p[3] == 0 {
                continue;
            }
            let c = f([p[0], p[1], p[2]]);
            for dy in 0..g {
                for dx in 0..g {
                    t.poser_fin(x + sx * g + dx, y + sy * g + dy, c, a);
                }
            }
        }
    }
}

/// Pose `img` tourné de `angle` (radians) et mis à l'échelle `e`, centré en (`cx`, `cy`)
/// (pixels fins), au plus proche : chaque case de sortie est un bloc `g` x `g` aligné sur la
/// grille (jamais de demi-pixel ni de flou). `ecrase` < 1 aplatit verticalement (disque vu de
/// biais). `lum` : ajouté en lumière au lieu d'être posé.
#[allow(clippy::too_many_arguments)]
pub fn poser_tourne(
    t: &mut Toile,
    img: &Img,
    (cx, cy): (i32, i32),
    g: i32,
    angle: f32,
    e: f32,
    ecrase: f32,
    a: u8,
    lum: bool,
    f: impl Fn(Rgb, i32, i32) -> Rgb,
) {
    if e <= 0.01 || a == 0 {
        return;
    }
    let (s, c) = angle.sin_cos();
    let r = ((img.l.max(img.h) as f32 * e * 0.75) as i32 + 1) * g;
    let (ox, oy) = (cx - cx.rem_euclid(g), cy - cy.rem_euclid(g));
    let (mx, my) = (img.l as f32 / 2.0, img.h as f32 / 2.0);
    let mut y = -r;
    while y <= r {
        let mut x = -r;
        while x <= r {
            let (u, v) = (
                (x as f32 + g as f32 / 2.0) / g as f32,
                (y as f32 + g as f32 / 2.0) / g as f32 / ecrase.max(0.05),
            );
            // Rotation inverse puis échelle inverse : coordonnées dans le calque.
            let (sx, sy) = ((u * c + v * s) / e + mx, (-u * s + v * c) / e + my);
            let (sx, sy) = (sx.floor() as i32, sy.floor() as i32);
            if sx >= 0 && sy >= 0 && sx < img.l && sy < img.h {
                let p = img.px(sx, sy);
                if p[3] > 0 {
                    let col = f([p[0], p[1], p[2]], sx, sy);
                    for dy in 0..g {
                        for dx in 0..g {
                            if lum {
                                t.eclairer_fin(ox + x + dx, oy + y + dy, col, a);
                            } else {
                                t.poser_fin(ox + x + dx, oy + y + dy, col, a);
                            }
                        }
                    }
                }
            }
            x += g;
        }
        y += g;
    }
}

/// Lueur en paliers (3 anneaux de lumière pleins, bord tramé) : jamais de dégradé lisse.
/// Centre et rayon en pixels fins, cases de `g` pixels.
pub fn halo(t: &mut Toile, (cx, cy): (i32, i32), r: f32, c: Rgb, force: f32, g: i32) {
    if r < 1.0 || force <= 0.0 {
        return;
    }
    let n = (r / g as f32).ceil() as i32 + 1;
    let (ox, oy) = (cx - cx.rem_euclid(g), cy - cy.rem_euclid(g));
    for j in -n..=n {
        for i in -n..=n {
            let d = ((i * g) as f32).hypot((j * g) as f32);
            let k = 1.0 - d / r;
            if k <= 0.0 {
                continue;
            }
            // Trois paliers ; la frontière entre deux paliers est tramée (Bayer).
            let palier = ((k * 3.0) + bayer(i, j) * 0.6 - 0.3)
                .floor()
                .clamp(0.0, 2.0);
            let a = (force * (50.0 + 55.0 * palier)).min(255.0) as u8;
            for dy in 0..g {
                for dx in 0..g {
                    t.eclairer_fin(ox + i * g + dx, oy + j * g + dy, c, a);
                }
            }
        }
    }
}

/// Assombrit toute la toile vers `c` (nuit, éclipse, abysses) : mélange plein, sans trame.
pub fn assombrir(t: &mut Toile, c: Rgb, k: f32) {
    let k = (k.clamp(0.0, 1.0) * 256.0) as u32;
    for p in &mut t.px {
        for i in 0..3 {
            p[i] = ((p[i] as u32 * (256 - k) + c[i] as u32 * k) >> 8) as u8;
        }
        p[3] = 255;
    }
}

/// Déformation de l'espace autour de (`cx`, `cy`) (pixels fins) : chaque case `g` x `g`
/// reprend la couleur d'une case plus éloignée (aspiration, `force` > 0) et tournée de
/// `tourbillon` radians près du centre. Au plus proche, par cases entières.
pub fn deformer(
    t: &mut Toile,
    (cx, cy): (i32, i32),
    rayon: f32,
    force: f32,
    tourbillon: f32,
    g: i32,
) {
    let src = t.px.clone();
    let (lf, hf) = (t.lf() as i32, t.hf() as i32);
    let mut y = 0;
    while y < hf {
        let mut x = 0;
        while x < lf {
            let (dx, dy) = ((x - cx) as f32, (y - cy) as f32);
            let d = dx.hypot(dy);
            let k = (1.0 - d / rayon).max(0.0);
            if k > 0.0 {
                let f = 1.0 + force * k * k;
                let a = tourbillon * k * k;
                let (s, c) = a.sin_cos();
                let (sx, sy) = ((dx * c - dy * s) * f, (dx * s + dy * c) * f);
                let (sx, sy) = (cx + sx.round() as i32, cy + sy.round() as i32);
                let (sx, sy) = (sx - sx.rem_euclid(g), sy - sy.rem_euclid(g));
                for j in 0..g {
                    for i in 0..g {
                        let (px, py) = (x + i, y + j);
                        if px < lf && py < hf {
                            let (qx, qy) = ((sx + i).clamp(0, lf - 1), (sy + j).clamp(0, hf - 1));
                            t.px[(py * lf + px) as usize] = src[(qy * lf + qx) as usize];
                        }
                    }
                }
            }
            x += g;
        }
        y += g;
    }
}

/// Étoile à quatre branches (étincelle de joaillier), en cases `g`.
pub fn etoile(t: &mut Toile, (x, y): (i32, i32), taille: i32, c: Rgb, g: i32) {
    let (x, y) = (x - x.rem_euclid(g), y - y.rem_euclid(g));
    for dy in 0..g {
        for dx in 0..g {
            t.eclairer_fin(x + dx, y + dy, [255, 255, 255], 255);
        }
    }
    for s in 1..=taille {
        let a = (230 - s * 40).max(50) as u8;
        for (i, j) in [(s, 0), (-s, 0), (0, s), (0, -s)] {
            for dy in 0..g {
                for dx in 0..g {
                    t.eclairer_fin(x + i * g + dx, y + j * g + dy, c, a);
                }
            }
        }
    }
}

//! Décor en pixel art IA (lot 4, même recette que les créatures : `tools/pixeliser.py`) :
//! bande de sable, rochers, coraux, algues (ondulent), récif lointain. Images embarquées
//! `assets/decor/*.png`, pixels fins, alpha 0 ou 255.

use crate::toile::{melange, Rgb, Toile};
use std::sync::OnceLock;

pub struct Img {
    pub l: i32,
    pub h: i32,
    pub px: Vec<[u8; 4]>,
}

impl Img {
    fn charger(b: &[u8]) -> Img {
        let i = image::load_from_memory(b)
            .expect("décor embarqué lisible")
            .to_rgba8();
        Img {
            l: i.width() as i32,
            h: i.height() as i32,
            px: i.pixels().map(|p| p.0).collect(),
        }
    }

    pub fn px(&self, x: i32, y: i32) -> [u8; 4] {
        self.px[(y * self.l + x) as usize]
    }

    /// Première ligne opaque de chaque colonne (profil du dessus), `h` si vide.
    pub fn dessus(&self) -> Vec<i32> {
        (0..self.l)
            .map(|x| {
                (0..self.h)
                    .find(|&y| self.px(x, y)[3] > 0)
                    .unwrap_or(self.h)
            })
            .collect()
    }
}

pub struct Decor {
    pub sable: Img,
    /// Rochers et coraux posés sur le sable.
    pub pierres: Vec<Img>,
    pub algues: Vec<Img>,
    pub lointain: Img,
}

pub fn decor() -> &'static Decor {
    static D: OnceLock<Decor> = OnceLock::new();
    D.get_or_init(|| Decor {
        sable: Img::charger(include_bytes!("../assets/decor/sable.png")),
        pierres: vec![
            Img::charger(include_bytes!("../assets/decor/rochers_a.png")),
            Img::charger(include_bytes!("../assets/decor/rochers_b.png")),
            Img::charger(include_bytes!("../assets/decor/corail_branche.png")),
            Img::charger(include_bytes!("../assets/decor/corail_eventail.png")),
            Img::charger(include_bytes!("../assets/decor/corail_cerveau.png")),
        ],
        algues: vec![
            Img::charger(include_bytes!("../assets/decor/algue_a.png")),
            Img::charger(include_bytes!("../assets/decor/algue_b.png")),
        ],
        lointain: Img::charger(include_bytes!("../assets/decor/lointain.png")),
    })
}

/// Couleur du décor fondue dans l'eau (`k` = part de l'eau) : décor et créatures forment une
/// seule illustration.
pub fn dans_l_eau(c: [u8; 4], eau: Rgb, k: f32) -> Rgb {
    melange([c[0], c[1], c[2]], eau, k)
}

/// Sous la ligne du sable (pixels fins, par colonne fine) ? Hors de la toile : oui.
fn sous_le_sable(sol: &[i32], x: i32, y: i32) -> bool {
    x < 0 || sol.get(x as usize).is_none_or(|&s| y >= s)
}

/// Pose une image en pixels fins, coin haut gauche (x0, y0), retournée si `miroir`. Ce qui
/// passe sous la ligne du sable `sol` est caché : le bas est enfoui.
pub fn poser(
    t: &mut Toile,
    img: &Img,
    (x0, y0): (i32, i32),
    miroir: bool,
    eau: (Rgb, f32),
    sol: &[i32],
) {
    for y in 0..img.h {
        for x in 0..img.l {
            let p = img.px(if miroir { img.l - 1 - x } else { x }, y);
            if p[3] > 0 && !sous_le_sable(sol, x0 + x, y0 + y) {
                t.poser_fin(x0 + x, y0 + y, dans_l_eau(p, eau.0, eau.1), 255);
            }
        }
    }
}

/// Ombre de contact sur le sable au pied d'un élément large de `l` pixels fins centré en `cx`.
pub fn ombre(t: &mut Toile, cx: i32, l: i32, sol: &[i32]) {
    let r = (l / 2).max(2) as f32;
    for dx in -(r as i32)..=(r as i32) {
        let x = cx + dx;
        let Some(&s) = sol.get(x.max(0) as usize) else {
            continue;
        };
        if s == i32::MAX {
            continue;
        }
        // v0.8 : ombre de contact plus marquée (la relecture la voyait à peine).
        let k = 1.0 - (dx as f32 / r).powi(2);
        // Bande sombre pleine sur 4 lignes puis fondu : visible sous chaque élément à 1080p.
        for dy in -2..9 {
            let a = (165.0 * k.sqrt() * (1.0 - (dy - 2).max(0) as f32 / 7.0)) as u8;
            if a > 0 {
                t.poser_fin(x, s + dy, [0x08, 0x0e, 0x16], a);
            }
        }
    }
}

/// Algue qui ondule : chaque ligne décalée d'une vague qui grandit vers le haut (le pied ne
/// bouge pas), `base` = bas de l'algue en pixels fins. Seules les `visible` lignes du haut
/// sortent du sable (algues de hauteurs variées sans réduire le dessin).
#[allow(clippy::too_many_arguments)]
pub fn algue(
    t: &mut Toile,
    img: &Img,
    (cx, base): (i32, i32),
    visible: i32,
    temps: f32,
    phase: f32,
    eau: (Rgb, f32),
    miroir: bool,
    sol: &[i32],
) {
    let h = visible.clamp(1, img.h) as f32;
    let debut = img.h - visible.clamp(1, img.h);
    for y in debut..img.h {
        let f = (img.h - 1 - y) as f32 / h;
        let dx =
            ((temps * 0.9 + phase + y as f32 * 0.045).sin() * 7.0 * f.powf(1.5)).round() as i32;
        for x in 0..img.l {
            let p = img.px(if miroir { img.l - 1 - x } else { x }, y);
            if p[3] > 0 {
                let (xx, yy) = (cx - img.l / 2 + x + dx, base - img.h + y);
                if !sous_le_sable(sol, xx, yy) {
                    t.poser_fin(xx, yy, dans_l_eau(p, eau.0, eau.1), 255);
                }
            }
        }
    }
}

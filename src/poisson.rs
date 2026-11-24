//! Sprite d'un individu : l'espèce (catalogue `especes.rs`) dessine chaque image, ici on ajoute
//! la marge, le contour « sel-out », les classes de pixels et les halos des effets de rareté
//! (dessinés au rendu dans `scene.rs`).

use crate::art::Dessin;
use crate::especes::{Espece, Mouvement, Taille};
use crate::jeu::Alea;
use crate::toile::melange;

/// Classe d'un pixel du sprite (sert aux effets : arc-en-ciel sur le corps, pas sur l'œil…).
pub const VIDE: u8 = 0;
pub const CONTOUR: u8 = 1;
pub const CORPS: u8 = 2;
pub const NAGEOIRE: u8 = 3;
pub const OEIL: u8 = 4;

pub struct Image {
    /// RGBA non prémultiplié.
    pub px: Vec<[u8; 4]>,
    pub classe: Vec<u8>,
    /// Halo à 1 et 2 pixels autour de la silhouette (lueurs).
    pub halo1: Vec<(i32, i32)>,
    pub halo2: Vec<(i32, i32)>,
}

pub struct Sprite {
    pub l: i32,
    pub h: i32,
    pub images: Vec<Image>,
    /// Point d'attache de la queue (traînée), bouche (bulles), leurre lumineux, dans l'image 0,
    /// tournée à droite.
    pub queue: (i32, i32),
    pub bouche: (i32, i32),
    pub lueur: Option<(i32, i32)>,
    /// Étoiles du corps (Céleste) : position et phase.
    pub etoiles: Vec<(i32, i32, f32)>,
    pub mouvement: Mouvement,
    pub taille: Taille,
}

pub const MARGE: i32 = 3;

/// Facteur d'échelle d'un individu : ±10 % autour de la taille de l'espèce selon `taille`
/// (0,6..1,5, celle qui multiplie aussi le revenu).
pub fn echelle_individu(taille: f32) -> f32 {
    0.9 + 0.2 * ((taille - 0.6) / 0.9).clamp(0.0, 1.0)
}

/// Sprite d'une espèce, variante et échelle données (déterministe).
pub fn sprite(e: &Espece, variante: usize, echelle: f32) -> Sprite {
    let pal = e.palette(variante);
    let l = ((e.dims.0 as f32 * echelle).round() as i32).max(4);
    let h = ((e.dims.1 as f32 * echelle).round() as i32).max(4);
    let n = e.images.max(1) as usize;
    let mut images = Vec::with_capacity(n);
    let (mut queue, mut bouche, mut lueur) = ((0, 0), (0, 0), None);
    let mut etoiles = Vec::new();
    for k in 0..n {
        let mut d = Dessin::new(l, h, MARGE);
        (e.art)(&mut d, k as f32 / n as f32, &pal);
        if k == 0 {
            (queue, bouche, lueur) = (
                (d.queue.0 + MARGE, d.queue.1 + MARGE),
                (d.bouche.0 + MARGE, d.bouche.1 + MARGE),
                d.lueur.map(|(x, y)| (x + MARGE, y + MARGE)),
            );
        }
        let (w, hh) = (d.lt(), d.ht());
        let (mut px, mut cl) = (d.px, d.cl);
        if k == 0 && e.rarete == crate::jeu::Rarete::Celeste {
            let mut a = Alea(e.cle.len() as u64 * 977 + variante as u64);
            for _ in 0..((l + h) / 6).max(4) {
                for _essai in 0..30 {
                    let (x, y) = (a.n(w as u32) as i32, a.n(hh as u32) as i32);
                    if cl[(y * w + x) as usize] == CORPS {
                        etoiles.push((x, y, a.f()));
                        break;
                    }
                }
            }
        }
        contourer(&mut px, &mut cl, w, hh);
        let (halo1, halo2) = halos(&cl, w, hh);
        images.push(Image {
            px,
            classe: cl,
            halo1,
            halo2,
        });
    }
    Sprite {
        l: l + 2 * MARGE,
        h: h + 2 * MARGE,
        images,
        queue,
        bouche,
        lueur,
        etoiles,
        mouvement: e.mouvement,
        taille: e.taille,
    }
}

/// Contour « sel-out » : pixels vides voisins (4-connexité) = teinte sombre du voisin.
fn contourer(px: &mut [[u8; 4]], cl: &mut [u8], w: i32, h: i32) {
    let avant = cl.to_vec();
    let src = px.to_vec();
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            if avant[i] != VIDE {
                continue;
            }
            let mut voisin = None;
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (xx, yy) = (x + dx, y + dy);
                if xx >= 0 && yy >= 0 && xx < w && yy < h {
                    let j = (yy * w + xx) as usize;
                    if avant[j] != VIDE && avant[j] != OEIL {
                        voisin = Some(src[j]);
                        break;
                    }
                }
            }
            if let Some(v) = voisin {
                let c = melange([v[0], v[1], v[2]], [10, 14, 34], 0.68);
                px[i] = [c[0], c[1], c[2], 255];
                cl[i] = CONTOUR;
            }
        }
    }
}

type Points = Vec<(i32, i32)>;

fn halos(cl: &[u8], w: i32, h: i32) -> (Points, Points) {
    let plein =
        |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h && cl[(y * w + x) as usize] != VIDE;
    let mut h1 = Vec::new();
    let mut h2 = Vec::new();
    for y in -2..h + 2 {
        for x in -2..w + 2 {
            if plein(x, y) {
                continue;
            }
            let mut d = 9;
            for dy in -2i32..=2 {
                for dx in -2i32..=2 {
                    if plein(x + dx, y + dy) {
                        d = d.min(
                            dx.abs().max(dy.abs()) + if dx.abs() + dy.abs() == 4 { 1 } else { 0 },
                        );
                    }
                }
            }
            match d {
                1 => h1.push((x, y)),
                2 => h2.push((x, y)),
                _ => {}
            }
        }
    }
    (h1, h2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::especes::catalogue;

    #[test]
    fn sprites_deterministes_et_bornes() {
        for e in catalogue() {
            for var in 0..e.variantes.len() {
                let s = sprite(e, var, 1.0);
                assert_eq!(s.images.len(), e.images as usize, "{}", e.cle);
                for im in &s.images {
                    assert!(
                        im.classe.contains(&CONTOUR) && im.classe.contains(&CORPS),
                        "{} sans corps ou contour",
                        e.cle
                    );
                    // Le contour ne touche aucun bord (marge pour les halos).
                    let bord = |x: i32, y: i32| im.classe[(y * s.l + x) as usize] == VIDE;
                    assert!(
                        (0..s.l).all(|x| bord(x, 0) && bord(x, s.h - 1))
                            && (0..s.h).all(|y| bord(0, y) && bord(s.l - 1, y)),
                        "{} touche le bord",
                        e.cle
                    );
                }
                let s2 = sprite(e, var, 1.0);
                for k in 0..s.images.len() {
                    assert_eq!(s2.images[k].px, s.images[k].px, "{} image {k}", e.cle);
                }
            }
        }
    }
}

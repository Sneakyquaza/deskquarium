//! Poissons qui penchent la tête (v0.16) : rotation du pixel art façon RotSprite. La v0.7
//! avait retiré toute inclinaison (rotation au plus proche = traits cassés, scintillement) ;
//! ici chaque image est agrandie ×8 par Scale2x (EPX, qui suit les bords au lieu de faire des
//! escaliers), tournée, puis relue au centre de chaque pixel : les contours restent continus,
//! sans couleur nouvelle (palette et classes du dessin gardées : yeux, lumières, contour).
//! Angles par crans de 7,5°, sprites penchés fabriqués à la demande, peu à peu (budget de
//! temps par image), gardés dans un cache borné.

use crate::poisson::{halos, Image, Sprite, CONTOUR, CORPS, NAGEOIRE, VIDE};
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

/// Un cran d'inclinaison (radians) : 7,5°.
pub const CRAN: f32 = 7.5 * std::f32::consts::PI / 180.0;
/// Cache : au plus ces octets d'images penchées (les plus anciennes partent d'abord) ; une
/// inclinaison inutilisée depuis `OUBLI` secondes est oubliée.
const MAX_OCTETS: usize = 12 << 20;
const OUBLI: f32 = 60.0;

/// Pixel du dessin : index de palette, et la classe quand la palette ne la porte pas.
fn cle(im: &Image, i: usize) -> u16 {
    match &im.classes {
        None => im.idx[i] as u16,
        Some(c) => im.idx[i] as u16 | (c[i] as u16) << 8,
    }
}

/// Scale2x (EPX) : chaque pixel devient 2x2 ; un coin prend la couleur de ses deux voisins
/// s'ils sont égaux (diagonale lissée sans couleur nouvelle). Hors de l'image : vide.
fn scale2x(src: &[u16], w: usize, h: usize) -> Vec<u16> {
    let at = |x: isize, y: isize| -> u16 {
        if x < 0 || y < 0 || x >= w as isize || y >= h as isize {
            0
        } else {
            src[y as usize * w + x as usize]
        }
    };
    let mut d = vec![0u16; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let (xi, yi) = (x as isize, y as isize);
            let p = at(xi, yi);
            let (a, b, c, dd) = (
                at(xi, yi - 1),
                at(xi + 1, yi),
                at(xi - 1, yi),
                at(xi, yi + 1),
            );
            let e0 = if c == a && c != dd && a != b { a } else { p };
            let e1 = if a == b && a != c && b != dd { b } else { p };
            let e2 = if dd == c && dd != b && c != a { c } else { p };
            let e3 = if b == dd && b != a && dd != c { dd } else { p };
            let o = (2 * y) * (2 * w) + 2 * x;
            d[o] = e0;
            d[o + 1] = e1;
            d[o + 2 * w] = e2;
            d[o + 2 * w + 1] = e3;
        }
    }
    d
}

/// Taille de l'image penchée (même parité que l'originale : le centre ne bouge pas).
fn dims(l: i32, h: i32, a: f32) -> (i32, i32) {
    let (c, s) = (a.cos().abs(), a.sin().abs());
    let mut nl = (l as f32 * c + h as f32 * s).ceil() as i32 + 2;
    let mut nh = (l as f32 * s + h as f32 * c).ceil() as i32 + 2;
    nl += (nl - l).rem_euclid(2);
    nh += (nh - h).rem_euclid(2);
    (nl, nh)
}

/// Point du dessin (tourné à droite) dans l'image penchée de `a` radians (tête vers le haut
/// si `a` > 0).
pub fn tourner_point(
    (x, y): (i32, i32),
    (l, h): (i32, i32),
    (nl, nh): (i32, i32),
    a: f32,
) -> (i32, i32) {
    let (u, v) = (
        x as f32 + 0.5 - l as f32 / 2.0,
        y as f32 + 0.5 - h as f32 / 2.0,
    );
    let (c, s) = (a.cos(), a.sin());
    (
        (nl as f32 / 2.0 + u * c + v * s).floor() as i32,
        (nh as f32 / 2.0 - u * s + v * c).floor() as i32,
    )
}

/// Une image penchée de `a` radians (RotSprite : ×8 par Scale2x, rotation, relue au centre).
fn tourner_image(im: &Image, l: i32, h: i32, a: f32) -> Image {
    let (w, hh) = (l as usize, h as usize);
    let src: Vec<u16> = (0..w * hh).map(|i| cle(im, i)).collect();
    // ×4 gardé, le dernier ×2 lu à la demande (16 fois moins de mémoire qu'un ×8 entier).
    let x2 = scale2x(&src, w, hh);
    let x4 = scale2x(&x2, w * 2, hh * 2);
    let (w4, h4) = (w as isize * 4, hh as isize * 4);
    let at4 = |x: isize, y: isize| -> u16 {
        if x < 0 || y < 0 || x >= w4 || y >= h4 {
            0
        } else {
            x4[(y * w4 + x) as usize]
        }
    };
    // Pixel du ×8 : coin (qx, qy) du pixel (x, y) du ×4, même règle que `scale2x`.
    let at8 = |x8: isize, y8: isize| -> u16 {
        let (x, y) = (x8.div_euclid(2), y8.div_euclid(2));
        let p = at4(x, y);
        let (ha, b, c, d) = (at4(x, y - 1), at4(x + 1, y), at4(x - 1, y), at4(x, y + 1));
        match (x8.rem_euclid(2), y8.rem_euclid(2)) {
            (0, 0) if c == ha && c != d && ha != b => ha,
            (1, 0) if ha == b && ha != c && b != d => b,
            (0, 1) if d == c && d != b && c != ha => c,
            (1, 1) if b == d && b != ha && d != c => d,
            _ => p,
        }
    };
    let (nl, nh) = dims(l, h, a);
    let (c, s) = (a.cos(), a.sin());
    let mut sortie = vec![0u16; (nl * nh) as usize];
    for oy in 0..nh {
        for ox in 0..nl {
            let (xx, yy) = (
                ox as f32 + 0.5 - nl as f32 / 2.0,
                oy as f32 + 0.5 - nh as f32 / 2.0,
            );
            // Rotation inverse : où était ce point dans le dessin droit.
            let u = xx * c - yy * s + l as f32 / 2.0;
            let v = xx * s + yy * c + h as f32 / 2.0;
            let k = at8((u * 8.0).floor() as isize, (v * 8.0).floor() as isize);
            sortie[(oy * nl + ox) as usize] = k;
        }
    }
    let classe = |k: u16| -> u8 {
        match &im.classes {
            None => im.cls[k as usize],
            Some(_) => (k >> 8) as u8,
        }
    };
    // Contour refermé : en biais, la ligne d'un pixel du contour ne se touche plus que par
    // les coins ; un pixel du corps qui touche le vide prend la couleur du contour voisin
    // (silhouette inchangée, jamais de corps à nu).
    let lu = sortie.clone();
    for oy in 0..nh {
        for ox in 0..nl {
            let i = (oy * nl + ox) as usize;
            if !matches!(classe(lu[i]), CORPS | NAGEOIRE) {
                continue;
            }
            let k = |dx: i32, dy: i32| -> u16 {
                let (x, y) = (ox + dx, oy + dy);
                if x < 0 || y < 0 || x >= nl || y >= nh {
                    0
                } else {
                    lu[(y * nl + x) as usize]
                }
            };
            if ![(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .any(|&(dx, dy)| classe(k(dx, dy)) == VIDE)
            {
                continue;
            }
            let mut voisins: Vec<u16> = (-1..=1)
                .flat_map(|dy| (-1..=1).map(move |dx| (dx, dy)))
                .map(|(dx, dy)| k(dx, dy))
                .filter(|&v| classe(v) == CONTOUR)
                .collect();
            voisins.sort_unstable();
            if let Some(v) = voisins
                .chunk_by(|a, b| a == b)
                .max_by_key(|g| g.len())
                .map(|g| g[0])
            {
                sortie[i] = v;
            }
        }
    }
    let cl: Vec<u8> = sortie.iter().map(|&k| classe(k)).collect();
    let (halo1, halo2) = halos(&cl, nl, nh);
    let rangs = cl
        .chunks(nl as usize)
        .map(|ligne| {
            let a = ligne.iter().position(|&k| k != VIDE).unwrap_or(0);
            let z = ligne.iter().rposition(|&k| k != VIDE).map_or(0, |z| z + 1);
            (a as u16, z.max(a) as u16)
        })
        .collect();
    Image {
        idx: sortie.iter().map(|&k| k as u8).collect(),
        pal: Arc::clone(&im.pal),
        cls: Arc::clone(&im.cls),
        nb_corps: cl.iter().filter(|&&k| k == CORPS).count() as u32,
        classes: im.classes.as_ref().map(|_| cl),
        halo1,
        halo2,
        rangs,
    }
}

/// Sprite penché de `a` radians à partir de ses images déjà tournées (points d'attache,
/// leurre, étoiles tournés avec lui).
fn assembler(s: &Sprite, a: f32, images: Vec<Image>) -> Sprite {
    let (l, h) = (s.l, s.h);
    let n = dims(l, h, a);
    let p = |q: (i32, i32)| tourner_point(q, (l, h), n, a);
    Sprite {
        l: n.0,
        h: n.1,
        images,
        queue: s.queue.iter().map(|&q| p(q)).collect(),
        bouche: p(s.bouche),
        lueur: s.lueur.map(p),
        etoiles: s
            .etoiles
            .iter()
            .map(|&(x, y, ph)| {
                let (x, y) = p((x, y));
                (x, y, ph)
            })
            .collect(),
        mouvement: s.mouvement,
        taille: s.taille,
        paupiere: s.paupiere,
        sombre: s.sombre,
    }
}

/// Sprite entier penché de `cran` crans (tests, planches).
pub fn pencher(s: &Sprite, cran: i8) -> Sprite {
    let a = cran as f32 * CRAN;
    let images = s
        .images
        .iter()
        .map(|im| tourner_image(im, s.l, s.h, a))
        .collect();
    assembler(s, a, images)
}

fn octets(s: &Sprite) -> usize {
    s.images
        .iter()
        .map(|im| {
            im.idx.len()
                + im.classes.as_ref().map_or(0, Vec::len)
                + (im.halo1.len() + im.halo2.len()) * 4
                + im.rangs.len() * 4
        })
        .sum()
}

struct Entree {
    source: Weak<Sprite>,
    sprite: Arc<Sprite>,
    octets: usize,
    vu: f32,
}

/// Sprite penché en fabrication (une image après l'autre).
struct EnCours {
    source: Weak<Sprite>,
    cran: i8,
    images: Vec<Image>,
}

/// Cache des sprites penchés, par sprite d'origine (adresse, vérifiée par un `Weak`) et cran.
#[derive(Default)]
pub struct Penches {
    faits: HashMap<(usize, i8), Entree>,
    demandes: Vec<(Weak<Sprite>, i8)>,
    en_cours: Option<EnCours>,
    temps: f32,
}

impl Penches {
    /// Sprite `s` penché de `cran` crans s'il est prêt ; sinon il est demandé (fabriqué par
    /// `travailler`) et None.
    pub fn obtenir(&mut self, s: &Arc<Sprite>, cran: i8) -> Option<Arc<Sprite>> {
        let cle = (Arc::as_ptr(s) as usize, cran);
        if let Some(e) = self.faits.get_mut(&cle) {
            if std::ptr::eq(e.source.as_ptr(), Arc::as_ptr(s)) && e.source.strong_count() > 0 {
                e.vu = self.temps;
                return Some(Arc::clone(&e.sprite));
            }
            self.faits.remove(&cle);
        }
        let deja = self
            .demandes
            .iter()
            .any(|(w, c)| *c == cran && std::ptr::eq(w.as_ptr(), Arc::as_ptr(s)));
        if !deja {
            self.demandes.push((Arc::downgrade(s), cran));
        }
        None
    }

    /// Fabrique les sprites demandés, une image après l'autre, pendant au plus `budget`
    /// (None : tout, pour des captures identiques d'une fois à l'autre) ; oublie les vieux.
    pub fn travailler(&mut self, temps: f32, budget: Option<Duration>) {
        self.temps = temps;
        let debut = Instant::now();
        loop {
            if budget.is_some_and(|b| debut.elapsed() >= b) {
                break;
            }
            let Some(ec) = &mut self.en_cours else {
                let Some((w, cran)) = self.demandes.pop() else {
                    break;
                };
                if w.strong_count() > 0 {
                    self.en_cours = Some(EnCours {
                        source: w,
                        cran,
                        images: Vec::new(),
                    });
                }
                continue;
            };
            let Some(s) = ec.source.upgrade() else {
                self.en_cours = None;
                continue;
            };
            let a = ec.cran as f32 * CRAN;
            if ec.images.len() < s.images.len() {
                let im = tourner_image(&s.images[ec.images.len()], s.l, s.h, a);
                ec.images.push(im);
                continue;
            }
            let ec = self.en_cours.take().expect("en cours");
            let sprite = assembler(&s, a, ec.images);
            self.faits.insert(
                (Arc::as_ptr(&s) as usize, ec.cran),
                Entree {
                    source: ec.source,
                    octets: octets(&sprite),
                    sprite: Arc::new(sprite),
                    vu: temps,
                },
            );
        }
        // Oubli : sprites d'origine disparus, inclinaisons inutilisées, puis les plus vieilles
        // tant que le cache dépasse sa taille.
        self.faits
            .retain(|_, e| e.source.strong_count() > 0 && temps - e.vu < OUBLI);
        let mut total: usize = self.faits.values().map(|e| e.octets).sum();
        while total > MAX_OCTETS {
            let Some((&k, _)) = self.faits.iter().min_by(|a, b| a.1.vu.total_cmp(&b.1.vu)) else {
                break;
            };
            total -= self.faits.remove(&k).map_or(0, |e| e.octets);
        }
        self.demandes.retain(|(w, _)| w.strong_count() > 0);
    }

    /// Octets des images penchées gardées (mesure).
    pub fn octets(&self) -> usize {
        self.faits.values().map(|e| e.octets).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::especes::par_cle;
    use crate::poisson::sprite;

    #[test]
    fn penche_sans_couleur_nouvelle_ni_trou() {
        let e = par_cle("clown").unwrap();
        let s = sprite(e, 0, 1.0);
        let im0 = &s.images[0];
        let n0 = im0.classes().filter(|&c| c != VIDE).count() as f32;
        for cran in [0i8, -4, -2, -1, 1, 3, 4] {
            let p = pencher(&s, cran);
            let im = &p.images[0];
            // Même palette (aucune couleur inventée), aire presque gardée.
            assert!(Arc::ptr_eq(&im.pal, &im0.pal));
            let n = im.classes().filter(|&c| c != VIDE).count() as f32;
            assert!(
                (0.9..1.1).contains(&(n / n0)),
                "cran {cran} : aire {n} / {n0}"
            );
            // Contour continu : aucun pixel du corps ne touche le vide (le contour l'entoure).
            let cl: Vec<u8> = im.classes().collect();
            let (l, h) = (p.l, p.h);
            let mut a_nu = 0;
            for y in 1..h - 1 {
                for x in 1..l - 1 {
                    if cl[(y * l + x) as usize] == CORPS
                        && [(1, 0), (-1, 0), (0, 1), (0, -1)]
                            .iter()
                            .any(|(dx, dy)| cl[((y + dy) * l + x + dx) as usize] == VIDE)
                    {
                        a_nu += 1;
                    }
                }
            }
            let bord = cl.iter().filter(|&&c| c == CONTOUR).count();
            assert!(
                a_nu * 50 < bord,
                "cran {cran} : {a_nu} pixels du corps à nu"
            );
        }
    }

    #[test]
    fn tete_vers_le_haut_et_symetrie() {
        let e = par_cle("blanc").unwrap();
        let s = sprite(e, 0, 1.0);
        let (l, h) = (s.l, s.h);
        for cran in [-4i8, -1, 1, 4] {
            let a = cran as f32 * CRAN;
            let n = dims(l, h, a);
            // La bouche (à droite) monte quand le cran est positif, descend sinon ; la queue
            // fait l'inverse, d'autant.
            let b = tourner_point(s.bouche, (l, h), n, a);
            let q = tourner_point(s.queue[0], (l, h), n, a);
            let (db, dq) = (
                (b.1 - n.1 / 2) - (s.bouche.1 - h / 2),
                (q.1 - n.1 / 2) - (s.queue[0].1 - h / 2),
            );
            assert_eq!(db.signum(), -(cran.signum() as i32), "cran {cran}");
            assert_eq!(dq.signum(), cran.signum() as i32, "cran {cran}");
            // Cran opposé : image miroir haut/bas de la même taille.
            assert_eq!(dims(l, h, -a), n);
        }
        assert_eq!(dims(l, h, 0.0).0 % 2, l % 2);
    }

    #[test]
    fn cache_borne_et_progressif() {
        let e = par_cle("clown").unwrap();
        let s = Arc::new(sprite(e, 0, 1.0));
        let mut c = Penches::default();
        assert!(c.obtenir(&s, 2).is_none());
        // Budget nul : rien n'est fabriqué.
        c.travailler(0.0, Some(Duration::ZERO));
        assert!(c.obtenir(&s, 2).is_none());
        c.travailler(0.0, None);
        let p = c.obtenir(&s, 2).expect("fait");
        assert_eq!(p.images.len(), s.images.len());
        assert!(c.octets() > 0);
        // Inutilisé longtemps : oublié ; sprite d'origine disparu : oublié aussi.
        c.travailler(OUBLI + 1.0, None);
        assert_eq!(c.octets(), 0);
        c.obtenir(&s, 1);
        c.travailler(30.0, None);
        assert!(c.octets() > 0);
        drop(s);
        c.travailler(30.0, None);
        assert_eq!(c.octets(), 0);
    }
}

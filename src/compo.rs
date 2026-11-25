//! Envoi de la toile à l'écran (v0.12) : seules les cases de la toile qui ont changé depuis
//! l'image précédente sont recomposées (fond d'écran + toile agrandie au plus proche, même
//! formule que `fond::composer`) et recopiées à l'écran. Logique pure (testée sous Linux) ;
//! `calque.rs` s'en sert sous Windows, `bench.rs` la mesure.

use crate::toile::Toile;

/// Côté d'une case en pixels fins (32 pixels écran en 1080p).
pub const CASE: usize = 16;

/// Hauteur maximale d'un rectangle envoyé, en cases (128 lignes en 1080p).
const BANDE: usize = 4;

/// Rectangle en pixels écran (coin haut gauche, largeur, hauteur).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub l: i32,
    pub h: i32,
}

impl Rect {
    /// `self` est entièrement dans `autre`.
    pub fn dans(&self, autre: &Rect) -> bool {
        self.x >= autre.x
            && self.y >= autre.y
            && self.x + self.l <= autre.x + autre.l
            && self.y + self.h <= autre.y + autre.h
    }
}

/// Toile déjà envoyée et cases qui restent à envoyer.
#[derive(Default)]
pub struct Compo {
    /// Toile telle qu'elle doit être à l'écran (copie de la dernière reçue).
    pub px: Vec<[u8; 4]>,
    /// Taille en pixels fins et agrandissement (pixels écran par pixel fin).
    pub tl: usize,
    pub th: usize,
    pub k: usize,
    /// Cases changées pas encore envoyées (ligne par ligne).
    sales: Vec<bool>,
    nl: usize,
    nh: usize,
}

impl Compo {
    /// Reçoit une nouvelle toile : les cases qui diffèrent de la précédente sont recopiées et
    /// marquées. Nouvelle taille : tout est marqué.
    pub fn recevoir(&mut self, toile: &Toile, k: usize) {
        let (tl, th) = (toile.lf(), toile.hf());
        if tl != self.tl || th != self.th || k != self.k || self.px.len() != toile.px.len() {
            self.tl = tl;
            self.th = th;
            self.k = k.max(1);
            self.px.clear();
            self.px.extend_from_slice(&toile.px);
            self.nl = tl.div_ceil(CASE);
            self.nh = th.div_ceil(CASE);
            self.sales = vec![true; self.nl * self.nh];
            return;
        }
        for cy in 0..self.nh {
            let (y0, y1) = (cy * CASE, ((cy + 1) * CASE).min(th));
            for cx in 0..self.nl {
                let (x0, x1) = (cx * CASE, ((cx + 1) * CASE).min(tl));
                let mut change = false;
                for y in y0..y1 {
                    let (a, b) = (y * tl + x0, y * tl + x1);
                    if self.px[a..b] != toile.px[a..b] {
                        self.px[a..b].copy_from_slice(&toile.px[a..b]);
                        change = true;
                    }
                }
                if change {
                    self.sales[cy * self.nl + cx] = true;
                }
            }
        }
    }

    /// Tout est à renvoyer (fond d'écran changé, mire terminée…).
    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn tout_salir(&mut self) {
        self.sales.fill(true);
    }

    /// Cases sales en rectangles écran (bornés à `l` x `h`), puis oubliées ; celles qui sont
    /// entièrement dans un rectangle de `caches` (écran recouvert) restent sales pour plus tard.
    pub fn rectangles(&mut self, l: i32, h: i32, caches: &[Rect]) -> Vec<Rect> {
        let c = (CASE * self.k) as i32;
        let (nl, nh) = (self.nl, self.nh);
        let case = |cx: usize, cy: usize| {
            let (x, y) = (cx as i32 * c, cy as i32 * c);
            Rect {
                x,
                y,
                l: c.min(l - x),
                h: c.min(h - y),
            }
        };
        let mut a_envoyer = vec![false; nl * nh];
        for cy in 0..nh {
            for cx in 0..nl {
                let i = cy * nl + cx;
                if !self.sales[i] {
                    continue;
                }
                let r = case(cx, cy);
                if r.l <= 0 || r.h <= 0 {
                    // Hors de la fenêtre : rien à montrer.
                    self.sales[i] = false;
                    continue;
                }
                if caches.iter().any(|m| r.dans(m)) {
                    continue;
                }
                self.sales[i] = false;
                a_envoyer[i] = true;
            }
        }
        // Bandes de BANDE cases de haut au plus : le tampon de composition reste petit.
        fusionner(&a_envoyer, nl, nh)
            .into_iter()
            .flat_map(|(cx, cy, w, ht)| {
                (cy..cy + ht).step_by(BANDE).map(move |y0| {
                    let (x, y) = (cx as i32 * c, y0 as i32 * c);
                    Rect {
                        x,
                        y,
                        l: (w as i32 * c).min(l - x),
                        h: ((BANDE.min(cy + ht - y0)) as i32 * c).min(h - y),
                    }
                })
            })
            .collect()
    }

    /// Compose le rectangle `r` (pixels écran) : fond d'écran (BGRA en u32, `fl` x `fh`, noir
    /// au-delà) puis la toile agrandie, en BGRA ligne par ligne dans `out` (r.l x r.h).
    /// Même résultat que `fond::composer` au pixel près.
    pub fn composer(&self, fond: &[u32], fl: usize, fh: usize, r: Rect, out: &mut Vec<u32>) {
        let (w, h) = (r.l.max(0) as usize, r.h.max(0) as usize);
        out.clear();
        out.resize(w * h, 0xff00_0000);
        let k = self.k.max(1);
        let fond_ok = fond.len() >= fl * fh;
        let x0 = r.x.max(0) as usize;
        for yy in 0..h {
            let y = r.y.max(0) as usize + yy;
            let ligne = &mut out[yy * w..(yy + 1) * w];
            if y < fh && fond_ok {
                let (a, b) = (x0.min(fl), (x0 + w).min(fl));
                ligne[..b - a].copy_from_slice(&fond[y * fl + a..y * fl + b]);
            }
            let ty = y / k;
            if ty < self.th {
                melanger_ligne(ligne, &self.px[ty * self.tl..(ty + 1) * self.tl], x0, k);
            }
        }
    }
}

/// Toile (RGBA prémultiplié, ligne `rang`, agrandie `k` fois) posée sur la ligne d'écran
/// `ligne` (BGRA) qui commence au pixel écran `x0` : d = s + (d × (255 − a) + 127) / 255,
/// plafonné à 255 (formule de `fond::composer`).
fn melanger_ligne(ligne: &mut [u32], rang: &[[u8; 4]], x0: usize, k: usize) {
    let w = ligne.len();
    #[allow(unused_mut)]
    let mut fait = 0;
    #[cfg(target_arch = "x86_64")]
    if (k == 1 || k == 2) && x0.is_multiple_of(k) {
        // Sûr : SSE2 fait partie de toute machine x86_64.
        fait = unsafe { sse2::melanger(ligne, rang, x0, k) };
    }
    // Le reste (bords, autres agrandissements) pixel par pixel, sans division.
    let (mut tx, mut reste) = ((x0 + fait) / k, k - (x0 + fait) % k);
    for d in &mut ligne[fait..w] {
        let Some(s) = rang.get(tx) else {
            break;
        };
        *d = melanger_px(*d, *s);
        reste -= 1;
        if reste == 0 {
            tx += 1;
            reste = k;
        }
    }
}

#[inline]
fn melanger_px(d: u32, s: [u8; 4]) -> u32 {
    let inv = 255 - s[3] as u32;
    let c = |v: u32, sk: u8| (sk as u32 + (v * inv + 127) / 255).min(255);
    0xff00_0000
        | c(d & 0xff, s[2])
        | c((d >> 8) & 0xff, s[1]) << 8
        | c((d >> 16) & 0xff, s[0]) << 16
}

#[cfg(target_arch = "x86_64")]
mod sse2 {
    use std::arch::x86_64::*;

    /// Même formule que `melanger_px`, 4 pixels écran à la fois (agrandissement 1 ou 2, `x0`
    /// multiple de `k`). Renvoie le nombre de pixels faits (multiple de 4, dans la toile).
    pub unsafe fn melanger(ligne: &mut [u32], rang: &[[u8; 4]], x0: usize, k: usize) -> usize {
        unsafe {
            let tx0 = x0 / k;
            if tx0 >= rang.len() {
                return 0;
            }
            // Pixels écran faisables : dans la ligne et dans la toile.
            let n = ligne.len().min((rang.len() - tx0) * k) / 4 * 4;
            let z = _mm_setzero_si128();
            let c255 = _mm_set1_epi16(255);
            let c127 = _mm_set1_epi16(127);
            let c1 = _mm_set1_epi16(1);
            // Voies 16 bits d'un pixel : B G R A ; l'opacité de sortie vaut 255.
            let rvb = _mm_set_epi16(0, -1, -1, -1, 0, -1, -1, -1);
            let alpha = _mm_set_epi16(255, 0, 0, 0, 255, 0, 0, 0);
            let src = rang.as_ptr().add(tx0) as *const u8;
            let dst = ligne.as_mut_ptr() as *mut __m128i;
            let un = |t16: __m128i, d16: __m128i| {
                // RGBA -> BGRA dans chaque pixel (voies 2, 1, 0, 3), opacité sur les 4 voies.
                let s = _mm_shufflehi_epi16(_mm_shufflelo_epi16(t16, 0xc6), 0xc6);
                let a = _mm_shufflehi_epi16(_mm_shufflelo_epi16(t16, 0xff), 0xff);
                let inv = _mm_and_si128(_mm_sub_epi16(c255, a), rvb);
                let s = _mm_or_si128(_mm_and_si128(s, rvb), alpha);
                let x = _mm_add_epi16(_mm_mullo_epi16(d16, inv), c127);
                // x / 255 exact pour x < 65 536 : (x + 1 + (x >> 8)) >> 8.
                let q =
                    _mm_srli_epi16(_mm_add_epi16(_mm_add_epi16(x, c1), _mm_srli_epi16(x, 8)), 8);
                _mm_add_epi16(q, s)
            };
            let mut i = 0;
            while i < n {
                // Quatre pixels de toile alignés sur les quatre pixels écran.
                let t = if k == 2 {
                    let p = _mm_loadl_epi64(src.add(i / 2 * 4) as *const __m128i);
                    _mm_unpacklo_epi32(p, p)
                } else {
                    _mm_loadu_si128(src.add(i * 4) as *const __m128i)
                };
                let d = _mm_loadu_si128(dst.add(i / 4));
                let bas = un(_mm_unpacklo_epi8(t, z), _mm_unpacklo_epi8(d, z));
                let haut = un(_mm_unpackhi_epi8(t, z), _mm_unpackhi_epi8(d, z));
                // Plafond à 255 par la réduction saturée en octets.
                _mm_storeu_si128(dst.add(i / 4), _mm_packus_epi16(bas, haut));
                i += 4;
            }
            n
        }
    }
}

/// Cases propres tolérées entre deux suites (moins d'appels GDI pour un peu plus de pixels).
const ECART: usize = 3;

/// Cases marquées (grille `nl` x `nh`) en rectangles de cases (x, y, largeur, hauteur) sans
/// recouvrement : suites horizontales (petits écarts comblés), puis suites identiques des
/// lignes suivantes réunies. Toute case marquée est couverte une fois.
pub fn fusionner(marques: &[bool], nl: usize, nh: usize) -> Vec<(usize, usize, usize, usize)> {
    let mut faits: Vec<(usize, usize, usize, usize)> = Vec::new();
    // Rectangles encore ouverts (qui peuvent descendre d'une ligne).
    let mut ouverts: Vec<usize> = Vec::new();
    for cy in 0..nh {
        let mut suites = Vec::new();
        let mut cx = 0;
        while cx < nl {
            if !marques[cy * nl + cx] {
                cx += 1;
                continue;
            }
            let x0 = cx;
            while cx < nl && marques[cy * nl + cx] {
                cx += 1;
            }
            // Deux suites séparées de peu de cases propres : une seule (moins d'appels GDI).
            match suites.last_mut() {
                Some((a, w)) if x0 - (*a + *w) <= ECART => *w = cx - *a,
                _ => suites.push((x0, cx - x0)),
            }
        }
        let mut encore = Vec::new();
        for (x0, w) in suites {
            match ouverts.iter().position(|&i| {
                let f = faits[i];
                f.0 == x0 && f.2 == w && f.1 + f.3 == cy
            }) {
                Some(p) => {
                    let i = ouverts.remove(p);
                    faits[i].3 += 1;
                    encore.push(i);
                }
                None => {
                    faits.push((x0, cy, w, 1));
                    encore.push(faits.len() - 1);
                }
            }
        }
        ouverts = encore;
    }
    faits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeu::Alea;

    #[test]
    fn fusion_couvre_exactement_les_cases() {
        let mut a = Alea(5);
        for _ in 0..200 {
            let (nl, nh) = (1 + (a.f() * 20.0) as usize, 1 + (a.f() * 12.0) as usize);
            let marques: Vec<bool> = (0..nl * nh).map(|_| a.f() < 0.4).collect();
            let r = fusionner(&marques, nl, nh);
            let mut vu = vec![0u8; nl * nh];
            for &(x, y, w, h) in &r {
                assert!(w > 0 && h > 0 && x + w <= nl && y + h <= nh);
                for yy in y..y + h {
                    for xx in x..x + w {
                        vu[yy * nl + xx] += 1;
                    }
                }
            }
            for i in 0..nl * nh {
                assert!(vu[i] <= 1, "case {i} couverte {} fois", vu[i]);
                assert!(!marques[i] || vu[i] == 1, "case {i} oubliée");
            }
        }
        // Un bloc plein = un seul rectangle ; rien = aucun.
        assert_eq!(fusionner(&[true; 12], 4, 3), vec![(0, 0, 4, 3)]);
        assert!(fusionner(&[false; 12], 4, 3).is_empty());
        // Petit écart comblé, grand écart gardé.
        let m = [true, false, false, true, false, false, false, false, true];
        assert_eq!(fusionner(&m, 9, 1), vec![(0, 0, 4, 1), (8, 0, 1, 1)]);
    }

    /// Rectangles quelconques (bords non alignés), agrandissements 1 à 3 : chemin rapide et
    /// pixel par pixel identiques à `fond::composer`.
    #[test]
    fn composition_identique_tout_agrandissement() {
        let mut a = Alea(21);
        for k in 1..=3usize {
            let mut toile = Toile::new(23, 11);
            for p in toile.px.iter_mut() {
                let al = [0u8, 255, (a.f() * 255.0) as u8][(a.f() * 3.0) as usize % 3];
                let v = (a.f() * al as f32) as u8;
                *p = [v, al / 2, v / 3, al];
            }
            let (l, h) = (toile.lf() * k + 5, toile.hf() * k);
            let fond_rgba = image::RgbaImage::from_fn(l as u32, h as u32, |x, y| {
                image::Rgba([(x * 13) as u8, (y * 7) as u8, (x + y) as u8, 255])
            });
            let fond: Vec<u32> = fond_rgba
                .pixels()
                .map(|c| 0xff00_0000 | (c[0] as u32) << 16 | (c[1] as u32) << 8 | c[2] as u32)
                .collect();
            let mut c = Compo::default();
            c.recevoir(&toile, k);
            let attendu = crate::fond::composer(&fond_rgba, &toile, k * crate::toile::FIN);
            let mut out = Vec::new();
            for _ in 0..200 {
                let x = (a.f() * l as f32) as i32;
                let y = (a.f() * h as f32) as i32;
                let r = Rect {
                    x,
                    y,
                    l: 1 + (a.f() * (l as i32 - x) as f32) as i32,
                    h: 1 + (a.f() * (h as i32 - y) as f32) as i32,
                };
                let r = Rect {
                    l: r.l.min(l as i32 - x),
                    h: r.h.min(h as i32 - y),
                    ..r
                };
                c.composer(&fond, l, h, r, &mut out);
                for yy in 0..r.h {
                    for xx in 0..r.l {
                        let e = out[(yy * r.l + xx) as usize];
                        let p = attendu.get_pixel((r.x + xx) as u32, (r.y + yy) as u32);
                        assert_eq!(
                            [e as u8, (e >> 8) as u8, (e >> 16) as u8],
                            [p[2], p[1], p[0]],
                            "k {k} rect {r:?} pixel {xx},{yy}"
                        );
                    }
                }
            }
        }
    }

    /// Image écran tenue à jour par les seuls rectangles envoyés = composition complète de la
    /// dernière toile (`fond::composer`, au pixel près), écrans recouverts compris une fois
    /// découverts.
    #[test]
    fn envoi_partiel_identique_a_la_composition_complete() {
        let mut toile = Toile::new(35, 19);
        let (tl, th, k) = (toile.lf(), toile.hf(), 2usize);
        let (l, h) = ((tl * k) as i32 - 3, (th * k) as i32);
        let mut a = Alea(9);
        let fond_rgba = image::RgbaImage::from_fn(l as u32, h as u32, |x, y| {
            image::Rgba([(x * 7) as u8, (y * 5) as u8, (x ^ y) as u8, 255])
        });
        let fond: Vec<u32> = fond_rgba
            .pixels()
            .map(|c| 0xff00_0000 | (c[0] as u32) << 16 | (c[1] as u32) << 8 | c[2] as u32)
            .collect();
        let mut ecran = vec![0u32; (l * h) as usize];
        let mut c = Compo::default();
        let mut out = Vec::new();
        let cache = Rect {
            x: 0,
            y: 0,
            l: 64,
            h,
        };
        for image in 0..30 {
            // Quelques pixels changés, parfois semi-transparents.
            for _ in 0..(a.f() * 40.0) as usize {
                let i = (a.f() * toile.px.len() as f32) as usize % toile.px.len();
                let al = [0u8, 90, 255][(a.f() * 3.0) as usize % 3];
                let v = (a.f() * al as f32) as u8;
                toile.px[i] = if a.f() < 0.1 {
                    // Couleur plus forte que l'opacité : plafonnée comme `fond::composer`.
                    [255, 200, 255, 90]
                } else {
                    [v, v / 2, al / 3, al]
                };
            }
            c.recevoir(&toile, k);
            // Les 10 premières images, la gauche est recouverte.
            let caches = if image < 10 { vec![cache] } else { vec![] };
            for r in c.rectangles(l, h, &caches) {
                assert!(r.x >= 0 && r.y >= 0 && r.x + r.l <= l && r.y + r.h <= h);
                c.composer(&fond, l as usize, h as usize, r, &mut out);
                for yy in 0..r.h as usize {
                    let d = (r.y as usize + yy) * l as usize + r.x as usize;
                    ecran[d..d + r.l as usize]
                        .copy_from_slice(&out[yy * r.l as usize..(yy + 1) * r.l as usize]);
                }
            }
        }
        let attendu = crate::fond::composer(&fond_rgba, &toile, k * crate::toile::FIN);
        for (i, p) in attendu.pixels().enumerate() {
            let e = ecran[i];
            assert_eq!(
                [e as u8, (e >> 8) as u8, (e >> 16) as u8],
                [p[2], p[1], p[0]],
                "pixel {i}"
            );
        }
        // Rien n'a changé : rien à envoyer.
        c.recevoir(&toile, k);
        assert!(c.rectangles(l, h, &[]).is_empty());
    }
}

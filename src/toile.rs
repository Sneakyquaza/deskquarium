//! Toile pixel art en mémoire (RGBA prémultiplié), primitives de dessin et petite police bitmap.

pub type Rgb = [u8; 3];

/// Finesse de la toile : chaque pixel « monde » (décor, texte, effets) fait FIN x FIN pixels
/// fins ; les sprites des espèces sont posés au pixel fin (deux fois plus de détail).
pub const FIN: usize = 2;

#[derive(Clone)]
pub struct Toile {
    /// Taille en pixels monde.
    pub l: usize,
    pub h: usize,
    /// RGBA prémultiplié, ligne par ligne, en pixels fins (`lf()` x `hf()`).
    pub px: Vec<[u8; 4]>,
}

impl Toile {
    pub fn new(l: usize, h: usize) -> Toile {
        Toile {
            l,
            h,
            px: vec![[0; 4]; l * h * FIN * FIN],
        }
    }

    /// Largeur et hauteur en pixels fins.
    pub fn lf(&self) -> usize {
        self.l * FIN
    }

    pub fn hf(&self) -> usize {
        self.h * FIN
    }

    pub fn effacer(&mut self) {
        self.px.fill([0; 4]);
    }

    #[inline]
    fn idx(&self, x: i32, y: i32) -> Option<usize> {
        (x >= 0 && y >= 0 && (x as usize) < self.lf() && (y as usize) < self.hf())
            .then(|| y as usize * self.lf() + x as usize)
    }

    /// Pose une couleur avec une opacité `a` (0..255), par-dessus (« source-over »), sur le
    /// pixel monde (x, y).
    #[inline]
    pub fn poser(&mut self, x: i32, y: i32, c: Rgb, a: u8) {
        let f = FIN as i32;
        for dy in 0..f {
            for dx in 0..f {
                self.poser_fin(x * f + dx, y * f + dy, c, a);
            }
        }
    }

    /// Pose une couleur sur un pixel fin.
    #[inline]
    pub fn poser_fin(&mut self, x: i32, y: i32, c: Rgb, a: u8) {
        if a == 0 {
            return;
        }
        if let Some(i) = self.idx(x, y) {
            if a == 255 {
                self.px[i] = [c[0], c[1], c[2], 255];
                return;
            }
            let d = &mut self.px[i];
            let a16 = a as u16;
            let inv = 255 - a16;
            for k in 0..3 {
                d[k] = ((c[k] as u16 * a16 + d[k] as u16 * inv + 127) / 255) as u8;
            }
            d[3] = ((a16 * 255 + d[3] as u16 * inv + 127) / 255) as u8;
        }
    }

    /// Ajoute de la lumière (lueurs) sur un pixel monde.
    #[inline]
    pub fn eclairer(&mut self, x: i32, y: i32, c: Rgb, a: u8) {
        let f = FIN as i32;
        for dy in 0..f {
            for dx in 0..f {
                self.eclairer_fin(x * f + dx, y * f + dy, c, a);
            }
        }
    }

    /// Ajoute de la lumière sur un pixel fin : additif plafonné, rend aussi le pixel plus opaque.
    #[inline]
    pub fn eclairer_fin(&mut self, x: i32, y: i32, c: Rgb, a: u8) {
        if a == 0 {
            return;
        }
        if let Some(i) = self.idx(x, y) {
            let d = &mut self.px[i];
            for k in 0..3 {
                d[k] = (d[k] as u16 + c[k] as u16 * a as u16 / 255).min(255) as u8;
            }
            d[3] = (d[3] as u16 + a as u16).min(255) as u8;
            // Prémultiplié : la couleur ne dépasse jamais l'opacité.
            let m = d[0].max(d[1]).max(d[2]);
            if m > d[3] {
                d[3] = m;
            }
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, l: i32, h: i32, c: Rgb, a: u8) {
        for yy in y..y + h {
            for xx in x..x + l {
                self.poser(xx, yy, c, a);
            }
        }
    }

    /// Recopie une autre toile (même taille) par-dessus.
    pub fn superposer(&mut self, autre: &Toile) {
        for (d, s) in self.px.iter_mut().zip(&autre.px) {
            let a = s[3] as u16;
            if a == 0 {
                continue;
            }
            let inv = 255 - a;
            for k in 0..4 {
                d[k] = (s[k] as u16 + (d[k] as u16 * inv + 127) / 255).min(255) as u8;
            }
        }
    }

    /// Image RGBA non prémultipliée agrandie `e` fois (pixels monde, au plus proche), pour
    /// les PNG. `e` multiple de FIN.
    pub fn agrandie_rgba(&self, e: usize) -> Vec<u8> {
        let k = (e / FIN).max(1);
        let lf = self.lf();
        let mut out = Vec::with_capacity(lf * k * self.hf() * k * 4);
        for y in 0..self.hf() * k {
            for x in 0..lf * k {
                out.extend_from_slice(&demultiplier(self.px[(y / k) * lf + x / k]));
            }
        }
        out
    }

    /// Texte dans la petite police (5 px de haut, majuscules + chiffres + k d n e s).
    /// Contour sombre d'un pixel si `contour` ; renvoie la largeur dessinée.
    pub fn texte(&mut self, x: i32, y: i32, s: &str, c: Rgb, a: u8, contour: Option<Rgb>) -> i32 {
        if let Some(o) = contour {
            for (dx, dy) in [
                (-1, 0),
                (1, 0),
                (0, -1),
                (0, 1),
                (1, 1),
                (-1, 1),
                (1, -1),
                (-1, -1),
            ] {
                self.texte_brut(x + dx, y + dy, s, o, (a as u16 * 3 / 4) as u8);
            }
        }
        self.texte_brut(x, y, s, c, a)
    }

    fn texte_brut(&mut self, x: i32, y: i32, s: &str, c: Rgb, a: u8) -> i32 {
        let mut cx = x;
        for ch in police(s) {
            let g = glyphe(ch);
            let lignes: Vec<&str> = g.split('|').collect();
            let decal = lignes.len() as i32 - 5;
            for (j, ligne) in lignes.iter().enumerate() {
                for (i, b) in ligne.bytes().enumerate() {
                    if b == b'#' {
                        self.poser(cx + i as i32, y + j as i32 - decal, c, a);
                    }
                }
            }
            cx += lignes[0].len() as i32 + 1;
        }
        cx - x - 1
    }
}

/// Majuscules partout, sauf les suffixes des nombres (« 1,2 k », « +3 Md/s ») qui gardent
/// leurs minuscules : un mot garde ses minuscules s'il contient un chiffre ou suit un nombre.
fn police(s: &str) -> Vec<char> {
    let mut out = Vec::new();
    let mut prec_nombre = false;
    for (i, mot) in s.split(' ').enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let nombre = mot.chars().any(|c| c.is_ascii_digit());
        let garder = nombre || prec_nombre;
        out.extend(mot.chars().map(|c| {
            if garder {
                c
            } else {
                c.to_uppercase().next().unwrap_or(c)
            }
        }));
        prec_nombre = nombre;
    }
    out
}

pub fn largeur_texte(s: &str) -> i32 {
    police(s)
        .into_iter()
        .map(|c| glyphe(c).split('|').next().unwrap_or("").len() as i32 + 1)
        .sum::<i32>()
        - 1
}

pub fn demultiplier(p: [u8; 4]) -> [u8; 4] {
    if p[3] == 0 {
        return [0; 4];
    }
    let a = p[3] as u16;
    [
        (p[0] as u16 * 255 / a).min(255) as u8,
        (p[1] as u16 * 255 / a).min(255) as u8,
        (p[2] as u16 * 255 / a).min(255) as u8,
        p[3],
    ]
}

/// Glyphes 3x5 (`#` = pixel) ; lignes en plus au-dessus pour les accents.
fn glyphe(c: char) -> &'static str {
    match c {
        '0' => "###|#.#|#.#|#.#|###",
        '1' => ".#.|##.|.#.|.#.|###",
        '2' => "##.|..#|.#.|#..|###",
        '3' => "##.|..#|.#.|..#|##.",
        '4' => "#.#|#.#|###|..#|..#",
        '5' => "###|#..|##.|..#|##.",
        '6' => ".##|#..|###|#.#|###",
        '7' => "###|..#|.#.|.#.|.#.",
        '8' => "###|#.#|###|#.#|###",
        '9' => "###|#.#|###|..#|##.",
        'k' => "#..|#.#|##.|#.#|#.#",
        'd' => "..#|.##|#.#|#.#|.##",
        'n' => "...|##.|#.#|#.#|#.#",
        'e' => "...|.##|###|#..|.##",
        's' => "...|.##|##.|..#|##.",
        ',' => "..|..|..|.#|#.",
        '.' => ".|.|.|.|#",
        ':' => ".|#|.|#|.",
        '+' => "...|.#.|###|.#.|...",
        '-' => "...|...|###|...|...",
        '/' => "..#|..#|.#.|#..|#..",
        '!' => "#|#|#|.|#",
        '\'' => "#|#|.|.|.",
        ' ' => "..|..|..|..|..",
        'É' | 'é' => "..#|###|#..|##.|#..|###",
        'È' | 'è' => "#..|###|#..|##.|#..|###",
        'Œ' | 'œ' => ".####|#.#..|#.##.|#.#..|.####",
        'Ö' | 'ö' => "#.#|.#.|#.#|#.#|#.#|.#.",
        'Ê' | 'ê' => ".#.|###|#..|##.|#..|###",
        'À' | 'à' => "#..|.#.|#.#|###|#.#|#.#",
        c if c.is_ascii_lowercase() => glyphe(c.to_ascii_uppercase()),
        // Lettres accentuées sans glyphe propre : la lettre de base.
        'Â' | 'â' | 'Ä' | 'ä' => glyphe('A'),
        'Ë' | 'ë' => glyphe('E'),
        'Î' | 'î' | 'Ï' | 'ï' => glyphe('I'),
        'Ô' | 'ô' => glyphe('O'),
        'Û' | 'û' | 'Ù' | 'ù' | 'Ü' | 'ü' => glyphe('U'),
        '&' => ".#.|#.#|.#.|#.#|.##",
        'A' => ".#.|#.#|###|#.#|#.#",
        'B' => "##.|#.#|##.|#.#|##.",
        'C' | 'Ç' | 'ç' => ".##|#..|#..|#..|.##",
        'D' => "##.|#.#|#.#|#.#|##.",
        'E' => "###|#..|##.|#..|###",
        'F' => "###|#..|##.|#..|#..",
        'G' => ".##|#..|#.#|#.#|.##",
        'H' => "#.#|#.#|###|#.#|#.#",
        'I' => "###|.#.|.#.|.#.|###",
        'J' => "..#|..#|..#|#.#|.#.",
        'K' => "#.#|#.#|##.|#.#|#.#",
        'L' => "#..|#..|#..|#..|###",
        'M' => "#...#|##.##|#.#.#|#...#|#...#",
        'N' => "#..#|##.#|#.##|#..#|#..#",
        'O' => ".#.|#.#|#.#|#.#|.#.",
        'P' => "##.|#.#|##.|#..|#..",
        'Q' => ".#.|#.#|#.#|##.|.##",
        'R' => "##.|#.#|##.|#.#|#.#",
        'S' => ".##|#..|.#.|..#|##.",
        'T' => "###|.#.|.#.|.#.|.#.",
        'U' => "#.#|#.#|#.#|#.#|###",
        'V' => "#.#|#.#|#.#|#.#|.#.",
        'W' => "#...#|#...#|#.#.#|##.##|#...#",
        'X' => "#.#|#.#|.#.|#.#|#.#",
        'Y' => "#.#|#.#|.#.|.#.|.#.",
        'Z' => "###|..#|.#.|#..|###",
        _ => "###|###|###|###|###",
    }
}

/// Couleurs : outils HSV et mélanges.
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

pub fn hsv(h: f32, s: f32, v: f32) -> Rgb {
    let h = h.rem_euclid(1.0) * 6.0;
    let i = h.floor();
    let f = h - i;
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - s * f), v * (1.0 - s * (1.0 - f)));
    let (r, g, b) = match i as i32 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]
}

#[inline]
pub fn melange(a: Rgb, b: Rgb, t: f32) -> Rgb {
    // Calcul entier (appelé pour chaque pixel des sprites) : t sur 256 crans.
    let t = (t.clamp(0.0, 1.0) * 256.0) as u32;
    [0, 1, 2].map(|k| ((a[k] as u32 * (256 - t) + b[k] as u32 * t + 128) >> 8) as u8)
}

/// Seuil de tramage ordonné (Bayer 4x4) dans [0, 1) : sert à quantifier les dégradés en pixels.
#[inline]
pub fn bayer(x: i32, y: i32) -> f32 {
    const M: [u8; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];
    (M[((y & 3) * 4 + (x & 3)) as usize] as f32 + 0.5) / 16.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn melange_premultiplie() {
        let mut t = Toile::new(2, 1);
        t.poser(0, 0, [255, 0, 0], 255);
        assert_eq!(t.px[0], [255, 0, 0, 255]);
        t.poser(1, 0, [255, 255, 255], 128);
        // Pixel monde (1, 0) = pixels fins 2..4 de la première ligne.
        assert_eq!(t.px[FIN], [128, 128, 128, 128]);
        assert_eq!(t.px[1], [255, 0, 0, 255]);
        assert_eq!(demultiplier(t.px[FIN]), [255, 255, 255, 128]);
        // Hors toile : ignoré.
        t.poser(-1, 5, [1, 2, 3], 255);
    }

    #[test]
    fn police_couvre_le_compteur() {
        for c in "0123456789,.+-/: kMdBnTQe!'ABCDEFGHIJKLMNOPQRSTUVWXYZéèêàçœö".chars() {
            assert_ne!(glyphe(c), glyphe('\u{1}'), "glyphe manquant : {c}");
        }
        // Fiche d'un poisson (v0.9) : noms d'espèces, raretés et modificateurs.
        let mut noms: Vec<String> = crate::especes::catalogue()
            .iter()
            .map(|e| e.nom.to_string())
            .collect();
        noms.extend(
            crate::jeu::Rarete::TOUTES
                .iter()
                .map(|r| r.nom().to_string()),
        );
        noms.extend(crate::jeu::Modif::TOUS.iter().map(|m| m.nom().to_string()));
        for n in noms {
            for c in police(&n) {
                assert_ne!(glyphe(c), glyphe('\u{1}'), "glyphe manquant : {c} ({n})");
            }
        }
        assert_eq!(largeur_texte("1"), 3);
        assert_eq!(largeur_texte("M"), 5);
    }
}

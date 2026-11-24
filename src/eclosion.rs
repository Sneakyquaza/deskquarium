//! Œufs et éclosion (v0.10) : un sprite pixel art IA par œuf (`assets/oeufs/`, même recette
//! que les créatures), une petite animation au repos propre à chaque œuf, et l'éclosion en
//! phases (game feel : anticipation, montée, éclatement avec arrêt sur image, révélation qui
//! rebondit, bandeau de rareté qui claque ; plus long et plus fort pour les raretés hautes,
//! les shiny et les modificateurs). Tout est dessiné au pixel fin, sans flou.

use crate::decor::Img;
use crate::jeu::{Alea, Partie, Poisson, Rarete, OEUFS};
use crate::scene::{sprite_ajuste, Aquarium, Nageur};
use crate::toile::{hsv, melange, rgb_hsv, Rgb, Toile, FIN};
use std::f32::consts::TAU;
use std::sync::{Arc, OnceLock};

/// Sprites des œufs, dans l'ordre de `jeu::OEUFS`.
pub fn image_oeuf(i: usize) -> &'static Img {
    static O: OnceLock<Vec<Img>> = OnceLock::new();
    let v = O.get_or_init(|| {
        vec![
            Img::charger(include_bytes!("../assets/oeufs/riviere.png")),
            Img::charger(include_bytes!("../assets/oeufs/lagon.png")),
            Img::charger(include_bytes!("../assets/oeufs/abysses.png")),
            Img::charger(include_bytes!("../assets/oeufs/cosmique.png")),
            Img::charger(include_bytes!("../assets/oeufs/ambre.png")),
            Img::charger(include_bytes!("../assets/oeufs/dore.png")),
            Img::charger(include_bytes!("../assets/oeufs/arc.png")),
            Img::charger(include_bytes!("../assets/oeufs/eclipse.png")),
        ]
    });
    &v[i.min(v.len() - 1)]
}

/// Pixel vif (taches lumineuses, étoiles, lave, or) : il anime l'œuf au repos.
fn vif(c: [u8; 4]) -> bool {
    let (_, s, v) = rgb_hsv([c[0], c[1], c[2]]);
    v > 0.72 && s > 0.4
}

/// Petit hachage entier → 0..1 (scintillements sans état).
fn bruit(a: i32, b: i32, c: i32) -> f32 {
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
    let hh = ((img.h as f32 * ecrase).round() as i32).max(1);
    let (x0, y0) = (cx - img.l / 2, base - hh);
    // Ligne du dessin pour chaque ligne dessinée, et décalage (penché : le haut bouge).
    let src_y = |y: i32| ((y as f32 / ecrase) as i32).min(img.h - 1);
    let dec = |y: i32| (penche * (1.0 - y as f32 / hh as f32)).round() as i32;
    // Reflet qui balaie l'œuf (nacre, ambre, or) : bande en diagonale.
    let balaye = match i {
        0 | 1 | 4 => Some((temps * 0.35).rem_euclid(1.6) * 2.0 - 0.6),
        5 => Some((temps * 0.6).rem_euclid(1.4) * 2.0 - 0.6),
        _ => None,
    };
    // Éclipse : couronne d'or qui respire autour de l'œuf.
    if i == 7 && !blanc {
        let a = 0.5 + 0.5 * (temps * 1.4).sin();
        let (ccx, ccy) = (cx as f32, (base - hh / 2) as f32);
        let (rx, ry) = (img.l as f32 * 0.62, hh as f32 * 0.58);
        for k in 0..90 {
            let ang = k as f32 / 90.0 * TAU;
            let (x, y) = (ccx + ang.cos() * rx, ccy + ang.sin() * ry);
            let al = ((110.0 + 140.0 * a) * (0.6 + 0.4 * (ang * 3.0 + temps * 2.0).sin())) as u8;
            t.eclairer_fin(x.round() as i32, y.round() as i32, [0xff, 0xc8, 0x50], al);
        }
    }
    for y in 0..hh {
        let sy = src_y(y);
        for x in 0..img.l {
            let p = img.px(x, sy);
            if p[3] == 0 {
                continue;
            }
            let mut c: Rgb = [p[0], p[1], p[2]];
            if blanc {
                t.poser_fin(x0 + x + dec(y), y0 + y, [255, 255, 255], 255);
                continue;
            }
            let v = vif(p);
            match i {
                // Abysses : les taches lumineuses pulsent en vagues.
                2 if v => {
                    let k = 0.5 + 0.5 * (temps * 2.2 - sy as f32 * 0.12).sin();
                    c = melange(c, [0xd8, 0xff, 0xff], 0.85 * k);
                }
                // Cosmique : les étoiles scintillent une à une.
                3 if v => {
                    let ph = bruit(x, sy, 3) * TAU;
                    let k = (temps * 3.0 + ph).sin();
                    if k > 0.6 {
                        c = [255, 255, 255];
                    } else if k < -0.4 {
                        c = melange(c, [0x30, 0x18, 0x60], 0.5);
                    }
                }
                // Arc-en-ciel : les bandes tournent lentement.
                6 => {
                    let (h, s, vv) = rgb_hsv(c);
                    if s > 0.2 {
                        c = hsv(h + temps * 0.12, s, vv);
                    }
                }
                // Éclipse : les veines d'or respirent.
                7 if v => {
                    let k = 0.5 + 0.5 * (temps * 1.4).sin();
                    c = melange(c, [255, 240, 180], 0.5 * k);
                }
                // Ambre : chaleur qui respire.
                4 => {
                    let k = 0.5 + 0.5 * (temps * 0.9).sin();
                    c = melange(c, [255, 200, 90], 0.08 * k);
                }
                _ => {}
            }
            if let Some(b) = balaye {
                let d = (x as f32 / img.l as f32 + sy as f32 / img.h as f32 * 0.6) - b;
                if (0.0..0.12).contains(&d) {
                    c = melange(c, [255, 255, 255], 0.35);
                }
            }
            // Œuf ordinaire devenu doré : teinte d'or sur tout l'œuf.
            if dore {
                let l = (c[0] as f32 * 0.3 + c[1] as f32 * 0.59 + c[2] as f32 * 0.11) / 255.0;
                c = melange(c, melange([0xa0, 0x58, 0x00], [0xff, 0xe8, 0x50], l), 0.9);
            }
            t.poser_fin(x0 + x + dec(y), y0 + y, c, 255);
        }
    }
    // Fêlures : trait sombre ; la lumière passe à travers, avec un petit halo.
    if let (Some((f, k)), false) = (fiss, blanc) {
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
    // Œuf doré : étincelles à quatre branches autour.
    // (rivière : une goutte de rosée qui brille de temps en temps)
    if (dore || i == 0) && !blanc {
        for k in 0..if dore { 5 } else { 1 } {
            let u = (temps * 0.8 + k as f32 * 0.37).rem_euclid(1.0);
            let o = ((1.0 - (2.0 * u - 1.0).abs()) * 3.0).round() as i32;
            let (x, y) = (
                x0 + (bruit(k, 1, 9) * img.l as f32) as i32,
                y0 + (bruit(k, 2, 9) * hh as f32) as i32,
            );
            t.eclairer_fin(x, y, [255, 255, 230], 255);
            for s in 1..=o {
                for (dx, dy) in [(s, 0), (-s, 0), (0, s), (0, -s)] {
                    t.eclairer_fin(x + dx, y + dy, [255, 230, 140], (200 - s * 50) as u8);
                }
            }
        }
    }
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
    let mut tmp = Toile::new(((img.l + 48) / 2) as usize, ((img.h + 48) / 2) as usize);
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
    let (lf, hf) = (tmp.lf() as i32, tmp.hf() as i32);
    for y in 0..hf {
        for x in 0..lf {
            let p = tmp.px[(y * lf + x) as usize];
            if p[3] == 0 {
                continue;
            }
            let d = crate::toile::demultiplier(p);
            for dy in 0..k {
                for dx in 0..k {
                    let (px, py) = (cx + (x - tcx) * k + dx, base + (y - tb) * k + dy);
                    t.poser_fin(px, py, [d[0], d[1], d[2]], d[3]);
                }
            }
        }
    }
}

/// Toile d'une vignette d'œuf (boutique) : l'œuf au repos à l'instant `temps`, légère
/// oscillation, posé sur une petite ombre.
pub fn vignette(i: usize, temps: f32) -> Toile {
    let mut t = Toile::new(48, 52);
    let (cx, base) = (48, 98);
    for dx in -16..=16 {
        let k = 1.0 - (dx as f32 / 16.0).powi(2);
        t.poser_fin(cx + dx, base + 1, [0x04, 0x0c, 0x16], (110.0 * k) as u8);
        t.poser_fin(cx + dx, base + 2, [0x04, 0x0c, 0x16], (60.0 * k) as u8);
    }
    let penche = (temps * 1.3 + i as f32).sin() * 1.2;
    // Petit saut toutes les ~2 s (chaque œuf à son rythme).
    let u = (temps * 0.5 + i as f32 * 0.3).fract();
    let saut = if u < 0.12 {
        -((u / 0.12) * std::f32::consts::PI).sin() * 4.0
    } else {
        0.0
    };
    let base = base + saut.round() as i32;
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
    t
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
    fissures: Fissures,
    eclats: Vec<Eclat>,
    etincelles: Vec<Etincelle>,
    /// Dernier instant dessiné (les particules avancent par petits pas).
    t_prec: f32,
    alea: Alea,
    /// Secousse (modèle « trauma » : force², décroît vite).
    trauma: f32,
    /// Instants déjà fêtés (coups, éclatement, bandeau).
    fait: u32,
}

const L: i32 = 128;
const HT: i32 = 96;
/// Bas de l'œuf (pixels fins).
const SOL: i32 = HT * 2 - 6;
/// Œuf de l'éclosion agrandi ×2 (pixels monde) : il remplit la scène.
const K: i32 = 2;

impl Eclosion {
    /// Prépare l'éclosion du poisson `f` sorti de l'œuf `oeuf`.
    pub fn nouvelle(f: &Poisson, oeuf: usize, debut: f64, dore: bool) -> Eclosion {
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
            fissures: Fissures::nouvelles(image_oeuf(oeuf), f.id),
            eclats: Vec::new(),
            etincelles: Vec::new(),
            t_prec: 0.0,
            alea: Alea(f.id ^ 0xec1a),
            trauma: 0.0,
            fait: 0,
        }
    }

    /// Change le poisson montré (captures : forme imposée).
    pub fn changer_poisson(&mut self, f: &Poisson) {
        let n = Eclosion::nouvelle(f, self.oeuf, self.debut, self.dore);
        self.poisson = n.poisson;
        self.zoom = n.zoom;
        self.shiny = f.shiny;
        self.modif = f.modif;
    }

    /// Durée de l'attente avant l'éclatement : plus longue pour les raretés hautes, les
    /// shiny et les modificateurs (la montée fait partie de la récompense).
    pub fn suspense(&self) -> f64 {
        let r = self.rarete.index() as f64;
        2.0 + (r - 2.0).max(0.0) * 0.22
            + if self.shiny { 0.6 } else { 0.0 }
            + if self.modif.is_some() { 0.3 } else { 0.0 }
            + if self.dore { 0.4 } else { 0.0 }
    }

    /// Rareté « haute » : traitement spécial (couleur qui balaie, secousses plus fortes).
    fn grande(&self) -> bool {
        self.rarete >= Rarete::Legendaire || self.shiny || self.modif.is_some()
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
        maintenant - self.debut >= self.suspense() + 0.25
    }

    /// Couleur « taquine » de la lumière : passe par les raretés inférieures avant la bonne.
    fn couleur(&self, t: f32, s: f32) -> Rgb {
        let etapes = self.rarete.index() + 1;
        let pas = (s - 0.7) / etapes as f32;
        let k = (((t - 0.7) / pas).floor().max(0.0) as usize).min(self.rarete.index());
        Rarete::TOUTES[k].couleur()
    }

    /// Dessine l'image de l'instant `t` (secondes depuis le début) sur `self.toile`.
    pub fn dessiner(&mut self, t: f64) {
        let s = self.suspense() as f32;
        let t = t as f32;
        let dt = (t - self.t_prec).clamp(0.0, 0.25);
        self.t_prec = t;
        // Particules et éclats : petits pas (images régulières même si l'appli rame).
        let pas = (dt / (1.0 / 120.0)).ceil().max(1.0) as usize;
        for _ in 0..pas {
            self.avancer(dt / pas as f32);
        }
        self.trauma = (self.trauma - 1.6 * dt).max(0.0);
        let couleur = self.couleur(t, s);
        let mut scene = Toile::new(L as usize, HT as usize);
        scene.rect(0, 0, L, HT, [0x0c, 0x22, 0x35], 255);
        // Lueur tramée derrière l'œuf, qui monte avec l'attente.
        let force = if t < s {
            ((t - 0.4) / (s - 0.4)).clamp(0.0, 1.0)
        } else {
            1.0
        };
        lueur(&mut scene, couleur, force, if t < s { 40.0 } else { 52.0 });
        if t < s {
            self.anticipation(&mut scene, t, s, couleur);
        } else {
            self.revelation(&mut scene, t - s, couleur);
        }

        for p in &self.etincelles {
            let a = (255.0 * (1.0 - p.vie / p.duree)).clamp(0.0, 255.0) as u8;
            let (x, y) = (p.x.round() as i32, p.y.round() as i32);
            scene.eclairer_fin(x, y, p.c, a);
            if p.duree > 0.6 {
                scene.eclairer_fin(x + 1, y, p.c, a / 2);
                scene.eclairer_fin(x, y + 1, p.c, a / 2);
            }
        }
        // Secousse de l'écran : décalage entier (pixels fins), force = trauma².
        let k = self.trauma * self.trauma;
        let (ox, oy) = (
            (k * 11.0 * (t * 47.0).sin() * (t * 13.0).cos()).round() as i32,
            (k * 9.0 * (t * 53.0 + 1.3).sin()).round() as i32,
        );
        let toile = &mut self.toile;
        toile.rect(0, 0, L, HT, [0x0c, 0x22, 0x35], 255);
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

    /// Avant l'éclatement : l'œuf tombe et s'écrase un peu, oscille de plus en plus, se fêle
    /// progressivement (la lumière passe par les fêlures), quelques coups secs, puis tremble.
    fn anticipation(&mut self, scene: &mut Toile, t: f32, s: f32, couleur: Rgb) {
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
            ecrase = 1.0 - 0.14 * (u * std::f32::consts::PI).sin();
            if self.fait & 1 == 0 {
                self.fait |= 1;
                self.trauma = (self.trauma + 0.25).min(1.0);
                self.gerbe(((cx - 14) as f32, base as f32), 6, [0x80, 0xa0, 0xb0], 30.0);
                self.gerbe(((cx + 14) as f32, base as f32), 6, [0x80, 0xa0, 0xb0], 30.0);
            }
        }
        let montee = (s - 0.65).max(0.5);
        let mut penche;
        let fiss;
        let lumiere;
        if t < montee {
            // Oscillation qui grandit, et des coups secs (l'œuf se fêle d'un cran).
            let u = ((t - 0.4) / (montee - 0.4)).clamp(0.0, 1.0);
            penche = (t * (5.0 + 7.0 * u)).sin() * (1.5 + 7.0 * u);
            let coups = [0.45f32, 0.7, 0.92];
            let mut f = 0.0;
            for (k, &c) in coups.iter().enumerate() {
                let tc = 0.4 + (montee - 0.4) * c;
                if t >= tc {
                    f = 0.22 * (k + 1) as f32;
                    let bit = 2 << k;
                    if self.fait & bit == 0 {
                        self.fait |= bit;
                        self.trauma = (self.trauma + 0.2).min(1.0);
                        let col = OEUFS[self.oeuf].coquille[0];
                        self.gerbe((cx as f32, (base - ih / 2) as f32), 5, col, 50.0);
                    }
                    if t - tc < 0.1 {
                        ecrase = 0.93;
                        penche *= 1.8;
                    }
                }
            }
            fiss = f;
            lumiere = (t > 0.7).then_some((couleur, 0.3 + 0.4 * u));
        } else {
            // Montée : tremble vite, se fêle partout, lumière forte ; raretés hautes : plus fort.
            let u = (t - montee) / (s - montee);
            let fort = if self.grande() { 1.6 } else { 1.0 };
            penche = (t * 60.0).sin() * 2.0 * fort;
            ecrase = 1.0 + 0.04 * (t * 30.0).sin() * u;
            fiss = 0.66 + 0.34 * u;
            lumiere = Some((couleur, 0.7 + 0.3 * u));
            self.trauma = self.trauma.max(0.25 + 0.25 * u * fort);
            if self.grande() && self.alea.f() < 0.5 {
                let x = cx as f32 + self.alea.entre(-12.0, 12.0);
                self.gerbe((x, (base - ih / 2) as f32), 1, couleur, 70.0);
            }
            // Raretés hautes : la couleur de la rareté balaie l'écran.
            if self.grande() {
                balayage(scene, couleur, u);
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
            Some((&self.fissures, fiss)),
            lumiere,
            self.dore && t >= 0.9,
            // Changement en or : un éclair blanc de deux images.
            self.dore && (0.87..0.92).contains(&t),
        );
        // Œuf ordinaire qui devient doré : à 0,9 s, l'or balaie l'œuf, gerbe et secousse.
        if self.dore && (0.9..1.3).contains(&t) {
            balayage(scene, [0xff, 0xd8, 0x50], (t - 0.9) / 0.4 / 3.0);
            if self.fait & 64 == 0 {
                self.fait |= 64;
                self.trauma = (self.trauma + 0.5).min(1.0);
                let c = (cx as f32, (base - ih / 2) as f32);
                self.gerbe(c, 40, [0xff, 0xe0, 0x60], 120.0);
            }
        }
        // Rayons fins qui sortent des fêlures quand elles sont grandes ouvertes.
        if let Some((c, f)) = lumiere.filter(|_| fiss > 0.6) {
            let n = 8;
            for k in 0..n {
                let ang = k as f32 / n as f32 * TAU + t * 0.6;
                for r in (il / 2)..(il / 2 + 26) {
                    if r % 2 == 0 {
                        let (x, y) = (
                            cx as f32 + ang.cos() * r as f32,
                            (base - ih / 2) as f32 + ang.sin() * r as f32 * 0.8,
                        );
                        let a = (f * 120.0 * (1.0 - (r - il / 2) as f32 / 26.0)) as u8;
                        scene.eclairer_fin(x.round() as i32, y.round() as i32, c, a);
                    }
                }
            }
        }
    }

    /// Après l'éclatement (`apres` secondes) : arrêt sur image blanc, éclats qui volent,
    /// rayons, poisson qui rebondit, bandeau de rareté qui claque.
    fn revelation(&mut self, scene: &mut Toile, apres: f32, couleur: Rgb) {
        let img = image_oeuf(self.oeuf);
        let ih = img.h * K;
        let (cx, base) = (L, SOL);
        let centre = (cx as f32, (base - ih / 2) as f32);
        // Éclatement : les morceaux de coquille partent, gerbe d'étincelles, secousse.
        if self.fait & 16 == 0 {
            self.fait |= 16;
            self.trauma = if self.grande() { 0.95 } else { 0.7 };
            self.eclater();
            let n = if self.grande() { 70 } else { 36 };
            self.gerbe(centre, n, couleur, 150.0);
            self.gerbe(centre, n / 3, [255, 255, 255], 110.0);
        }
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
        // Rayons tournants (shiny : arc-en-ciel, deux fois plus nombreux).
        let nr = if self.shiny {
            24
        } else if self.grande() {
            16
        } else {
            12
        };
        for i in 0..nr {
            let ang = i as f32 / nr as f32 * TAU + apres * 0.4;
            let long = (apres * 320.0).min(150.0) as i32;
            let c = if self.shiny {
                hsv(i as f32 / nr as f32 + apres * 0.3, 0.55, 1.0)
            } else {
                couleur
            };
            let fort = if self.shiny || self.grande() {
                120.0
            } else {
                85.0
            };
            // Rayons pleins de 2 pixels, les plus vifs dans la première demi-seconde.
            let vif = 1.0 + 0.8 * (1.0 - apres / 0.5).max(0.0);
            for r in 20..long {
                let (x, y) = (
                    centre.0 + ang.cos() * r as f32,
                    centre.1 + ang.sin() * r as f32 * 0.8,
                );
                let a = (fort * vif * (1.0 - r as f32 / 150.0)).min(255.0) as u8;
                let (x, y) = (x.round() as i32, y.round() as i32);
                scene.eclairer_fin(x, y, c, a);
                if i % 2 == 0 {
                    scene.eclairer_fin(x + 1, y, c, a / 2);
                    scene.eclairer_fin(x, y + 1, c, a / 2);
                }
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
                let o = ((1.0 - (2.0 * u - 1.0).abs()) * 7.0).round() as i32;
                let (x, y) = (x as i32, (y - u * 8.0) as i32);
                let c = [[255, 246, 190], [255, 190, 236], [190, 240, 255]][a.n(3) as usize];
                scene.eclairer_fin(x, y, [255, 255, 255], 255);
                for k in 1..=o {
                    let ak = (230 - k * 28).max(40) as u8;
                    for (dx, dy) in [(k, 0), (-k, 0), (0, k), (0, -k)] {
                        scene.eclairer_fin(x + dx, y + dy, c, ak);
                    }
                }
            }
        }
        // Modificateur : sa couleur en anneau qui s'élargit.
        if let Some(m) = self.modif {
            let r = (apres * 140.0) % 120.0;
            for k in 0..120 {
                let ang = k as f32 / 120.0 * TAU;
                let (x, y) = (centre.0 + ang.cos() * r, centre.1 + ang.sin() * r * 0.8);
                let a = (200.0 * (1.0 - r / 120.0)) as u8;
                scene.eclairer_fin(x.round() as i32, y.round() as i32, m.couleur(), a);
            }
        }
        // Éclats de coquille, derrière le poisson, qui s'effacent.
        for e in &self.eclats {
            let a = (255.0 * (1.0 - ((e.vie - 0.7) / 0.7).clamp(0.0, 1.0))) as u8;
            for &(x, y, c) in &e.px {
                scene.poser_fin(e.x.round() as i32 + x, e.y.round() as i32 + y, c, a);
            }
        }
        // Poisson révélé : tombe d'un coup et rebondit (ressort amorti), blanc 2 images.
        self.poisson.avancer(1.0 / 60.0);
        self.poisson.dessiner(&Partie::nouvelle(0, 0));
        let u = (apres - 0.04).max(0.0);
        let rebond = (-44.0 * (-6.0 * u).exp() * (15.0 * u).cos()).round() as i32;
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
            if self.fait & 32 == 0 {
                self.fait |= 32;
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
            // Sous le bandeau : shiny et modificateur (après un court temps), œuf doré.
            let mut lignes: Vec<(String, Rgb)> = Vec::new();
            if v > 0.25 {
                if self.shiny {
                    lignes.push(("SHINY !".into(), [0xff, 0xb8, 0xe8]));
                }
                if let Some(m) = self.modif {
                    lignes.push((m.nom().to_uppercase(), m.couleur()));
                }
            }
            if self.dore {
                lignes.push(("ŒUF DORÉ !".into(), [0xff, 0xdc, 0x60]));
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

/// Lueur tramée en anneaux derrière l'œuf (pas de dégradé lisse).
fn lueur(t: &mut Toile, c: Rgb, force: f32, rayon: f32) {
    let (cx, cy) = (L / 2, (SOL - 70) / 2);
    for y in 0..HT {
        for x in 0..L {
            let d = (((x - cx) as f32).powi(2) + ((y - cy) as f32 * 1.2).powi(2)).sqrt();
            let a = (1.0 - d / rayon).max(0.0) * force;
            if a > crate::toile::bayer(x, y) {
                t.eclairer(
                    x,
                    y,
                    melange(c, [0x0c, 0x22, 0x35], 0.55),
                    (40.0 + a * 90.0) as u8,
                );
            }
        }
    }
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
fn texte_gros(t: &mut Toile, s: &str, (cx, y): (i32, i32), k: i32, c: Rgb) {
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

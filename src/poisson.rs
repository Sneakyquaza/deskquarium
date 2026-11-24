//! Génome d'un poisson (tiré de sa graine) → sprite pixel art : silhouette, motif, contour,
//! 4 images de nage. Les effets de rareté sont dessinés au rendu (`scene.rs`) à partir des
//! classes de pixels et des halos calculés ici.

use crate::jeu::{Alea, Rarete};
use crate::toile::{hsv, melange, Rgb};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Forme {
    Fuseau,
    Disque,
    Allonge,
    Globe,
    Fleche,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Queue {
    Fourche,
    Eventail,
    Ronde,
    Voile,
    Lyre,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Nageoire {
    Aucune,
    Petite,
    Haute,
    Longue,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Motif {
    Uni,
    Rayures(u8),
    Points,
    Degrade,
    Bande,
    Tete,
}

#[derive(Clone, Debug)]
pub struct Genome {
    pub rarete: Rarete,
    pub forme: Forme,
    pub queue: Queue,
    pub dorsale: Nageoire,
    pub ventrale: Nageoire,
    pub motif: Motif,
    /// Longueur du corps (sans la queue), en pixels de toile.
    pub longueur: i32,
    pub base: Rgb,
    pub ombre: Rgb,
    pub clair: Rgb,
    pub ventre: Rgb,
    pub accent: Rgb,
    pub nageoire: Rgb,
    pub teinte: f32,
    graine_motif: u64,
}

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
    /// Point d'attache de la queue (traînée), bouche (bulles), dans l'image 0, tournée à droite.
    pub queue: (i32, i32),
    pub bouche: (i32, i32),
    /// Étoiles du corps (Céleste) : position et phase.
    pub etoiles: Vec<(i32, i32, f32)>,
    pub nom: String,
}

const MARGE: i32 = 3;

pub fn genome(graine: u64, rarete: Rarete, taille: f32) -> Genome {
    let mut a = Alea(graine);
    let r = rarete.index();
    let choisir = |a: &mut Alea, n: usize| a.n(n as u32) as usize;
    let forme = [
        Forme::Fuseau,
        Forme::Disque,
        Forme::Allonge,
        Forme::Globe,
        Forme::Fleche,
    ][choisir(&mut a, 5)];
    // Les raretés hautes ont des queues et nageoires plus spectaculaires.
    let queues: &[Queue] = match r {
        0 => &[Queue::Fourche, Queue::Eventail, Queue::Ronde],
        1..=2 => &[Queue::Fourche, Queue::Eventail, Queue::Ronde, Queue::Voile],
        _ => &[Queue::Fourche, Queue::Voile, Queue::Lyre, Queue::Voile],
    };
    let queue = queues[choisir(&mut a, queues.len())];
    let dorsales: &[Nageoire] = match r {
        0 => &[Nageoire::Aucune, Nageoire::Petite, Nageoire::Petite],
        1..=2 => &[Nageoire::Petite, Nageoire::Haute, Nageoire::Longue],
        _ => &[Nageoire::Haute, Nageoire::Longue],
    };
    let dorsale = dorsales[choisir(&mut a, dorsales.len())];
    let ventrale = if r >= 3 || a.f() < 0.5 {
        Nageoire::Petite
    } else {
        Nageoire::Aucune
    };
    let motif = match choisir(&mut a, 7) {
        0 | 1 => Motif::Uni,
        2 => Motif::Rayures(3 + a.n(3) as u8),
        3 => Motif::Points,
        4 => Motif::Degrade,
        5 => Motif::Bande,
        _ => Motif::Tete,
    };
    let bonus = [0, 0, 1, 2, 3, 4, 5, 6][r];
    let longueur = (8.0 + 9.0 * taille).round() as i32 + bonus;

    // Palette : rampe ombre/base/clair décalée en teinte (ombres froides, lumières chaudes).
    let (mut h, mut s, mut v) = (a.f(), a.entre(0.3, 0.6), a.entre(0.6, 0.85));
    if r >= 1 {
        s = a.entre(0.5 + 0.04 * r as f32, 0.85);
        v = a.entre(0.7, 0.95);
    }
    let mut h2 = (h + a.entre(0.35, 0.65)).rem_euclid(1.0);
    let (mut s2, mut v2) = (a.entre(0.6, 0.95), a.entre(0.75, 1.0));
    match rarete {
        Rarete::Legendaire => {
            h2 = a.entre(0.11, 0.14);
            s2 = 0.85;
            v2 = 1.0;
        }
        Rarete::Mythique => {
            h = a.entre(-0.03, 0.02);
            s = 0.85;
            v = a.entre(0.55, 0.75);
            h2 = a.entre(0.05, 0.09);
        }
        Rarete::Celeste => {
            h = a.entre(0.68, 0.76);
            s = 0.75;
            v = 0.5;
            h2 = a.entre(0.82, 0.9);
            s2 = 0.7;
        }
        Rarete::Irreel => {
            s = 0.12;
            v = 0.97;
            h2 = a.f();
        }
        _ => {}
    }
    let motif = match rarete {
        Rarete::Celeste => Motif::Degrade,
        Rarete::Irreel => Motif::Uni,
        _ => motif,
    };
    let base = hsv(h, s, v);
    let accent = hsv(h2, s2, v2);
    Genome {
        rarete,
        forme,
        queue,
        dorsale,
        ventrale,
        motif,
        longueur,
        base,
        ombre: hsv(h + 0.03, (s * 1.1).min(1.0), v * 0.55),
        clair: hsv(h - 0.04, s * 0.7, (v * 1.15 + 0.1).min(1.0)),
        ventre: hsv(h - 0.06, s * 0.3, 0.96),
        accent,
        nageoire: if rarete == Rarete::Legendaire {
            accent
        } else {
            melange(
                if a.f() < 0.5 { base } else { accent },
                [255, 255, 255],
                0.3,
            )
        },
        teinte: h,
        graine_motif: a.suivant(),
    }
}

/// Profil du corps (0..1) le long de u (0 = attache de la queue, 1 = museau).
fn profil(f: Forme, u: f32) -> f32 {
    let ell = |u: f32| (1.0 - (2.0 * u - 1.0).powi(2)).max(0.0).sqrt();
    match f {
        Forme::Fuseau => {
            (std::f32::consts::PI * u.powf(0.75))
                .sin()
                .max(0.0)
                .powf(0.7)
                * 0.85
                + 0.15 * ell(u)
        }
        Forme::Disque | Forme::Globe => ell(u * 0.92 + 0.04),
        Forme::Allonge => (u * 6.0).min((1.0 - u) * 4.0).clamp(0.0, 1.0).sqrt(),
        Forme::Fleche => {
            if u < 0.62 {
                0.35 + 0.65 * (u / 0.62).powf(0.6)
            } else {
                ((1.0 - u) / 0.38).powf(0.9)
            }
        }
    }
}

/// Hauteur du corps / longueur.
fn rapport(f: Forme) -> f32 {
    match f {
        Forme::Fuseau => 0.42,
        Forme::Disque => 0.78,
        Forme::Allonge => 0.24,
        Forme::Globe => 0.68,
        Forme::Fleche => 0.36,
    }
}

pub fn nom(g: &Genome) -> String {
    let nom = match (g.forme, g.queue) {
        (_, Queue::Voile) => "Voilier",
        (_, Queue::Lyre) => "Lyrin",
        (Forme::Fuseau, _) => "Fuselier",
        (Forme::Disque, _) => "Discus",
        (Forme::Allonge, _) => "Anguillon",
        (Forme::Globe, _) => "Globulin",
        (Forme::Fleche, _) => "Dardeur",
    };
    let couleurs = [
        "rubis",
        "mandarine",
        "safran",
        "citron",
        "émeraude",
        "jade",
        "lagon",
        "azur",
        "saphir",
        "améthyste",
        "orchidée",
        "corail",
    ];
    let adj = match g.rarete {
        Rarete::Celeste => "stellaire",
        Rarete::Irreel => "prismatique",
        Rarete::Mythique => "braise",
        _ => {
            let [r, v, b] = g.base;
            let (mx, mn) = (r.max(v).max(b), r.min(v).min(b));
            if (mx - mn) < 50 {
                "perle"
            } else {
                couleurs[(g.teinte.rem_euclid(1.0) * 12.0) as usize % 12]
            }
        }
    };
    let motif = match g.motif {
        Motif::Rayures(_) => " rayé",
        Motif::Points => " moucheté",
        Motif::Degrade if g.rarete != Rarete::Celeste => " irisé",
        Motif::Bande => " ligné",
        Motif::Tete => " masqué",
        _ => "",
    };
    let royal = if g.rarete == Rarete::Legendaire {
        " royal"
    } else {
        ""
    };
    format!("{nom} {adj}{motif}{royal}")
}

/// Longueur et demi-ouverture de la queue.
fn dims_queue(g: &Genome) -> (i32, f32) {
    let l = g.longueur as f32;
    let hb = l * rapport(g.forme);
    let (t, s) = match g.queue {
        Queue::Fourche => (0.32, 0.55),
        Queue::Eventail => (0.3, 0.6),
        Queue::Ronde => (0.28, 0.5),
        Queue::Voile => (0.52, 0.75),
        Queue::Lyre => (0.5, 0.8),
    };
    (((l * t).round() as i32).max(3), (hb * s).max(2.0))
}

fn hauteur_nageoire(n: Nageoire, l: i32) -> i32 {
    match n {
        Nageoire::Aucune => 0,
        Nageoire::Petite => (l as f32 * 0.12).round().max(1.0) as i32,
        Nageoire::Haute => (l as f32 * 0.3).round().max(2.0) as i32,
        Nageoire::Longue => (l as f32 * 0.2).round().max(2.0) as i32,
    }
}

pub fn sprite(g: &Genome) -> Sprite {
    let l = g.longueur;
    let hb = l as f32 * rapport(g.forme);
    let (lq, ouv) = dims_queue(g);
    let fd = hauteur_nageoire(g.dorsale, l);
    let fv = hauteur_nageoire(g.ventrale, l);
    let haut = (hb * 0.55).ceil() as i32;
    let bas = (hb * 0.45).ceil() as i32;
    let dessus = (haut + fd).max(ouv.ceil() as i32 + 2);
    let chute = if g.queue == Queue::Voile {
        (ouv * 0.35).ceil() as i32
    } else {
        0
    };
    let dessous = (bas + fv).max(ouv.ceil() as i32 + 2 + chute);
    let w = MARGE * 2 + lq + l;
    let hgt = MARGE * 2 + dessus + dessous + 1;
    let x0 = MARGE + lq;
    let cy = (MARGE + dessus) as f32 + 0.5;
    let mut a = Alea(g.graine_motif);
    // Points du motif tirés une fois (mêmes sur toutes les images).
    let points: Vec<(f32, f32)> = (0..(l / 2).max(3))
        .map(|_| (a.entre(0.12, 0.78), a.entre(-0.7, 0.6)))
        .collect();
    let mut etoiles = Vec::new();
    let mut images = Vec::new();
    for k in 0..4 {
        let battement = [0.0f32, 1.0, 0.0, -1.0][k];
        let amp = if lq >= 8 { 2.0 } else { 1.0 };
        let mut px = vec![[0u8; 4]; (w * hgt) as usize];
        let mut cl = vec![VIDE; (w * hgt) as usize];
        for y in 0..hgt {
            for x in 0..w {
                let i = (y * w + x) as usize;
                let dy = y as f32 + 0.5 - cy;
                // Corps.
                let u = (x - x0) as f32 / l as f32 + 0.5 / l as f32;
                if (0.0..1.0).contains(&u) {
                    let p = profil(g.forme, u);
                    // Arrière du corps : suit un peu la queue.
                    let decal = if u < 0.22 {
                        (battement * 0.6 * (0.22 - u) / 0.22).round()
                    } else {
                        0.0
                    };
                    let dyc = dy - decal;
                    let (t, b) = (hb * 0.55 * p, hb * 0.45 * p);
                    if dyc >= -t && dyc <= b && t + b >= 0.8 {
                        let n = (dyc + t) / (t + b).max(0.01);
                        let mut c = couleur_corps(g, u, n, dyc, &points, x, y);
                        // Liseré clair en haut, museau plus sombre.
                        if dyc - 1.0 < -t {
                            c = melange(c, g.clair, 0.6);
                        }
                        if u > 0.95 && n > 0.45 && n < 0.7 {
                            c = g.ombre;
                        }
                        px[i] = [c[0], c[1], c[2], 255];
                        cl[i] = CORPS;
                        continue;
                    }
                    // Dorsale : bord avant raide, bord arrière en pente vers la queue.
                    let (d0, d1) = match g.dorsale {
                        Nageoire::Longue => (0.08, 0.72),
                        _ => (0.3, 0.68),
                    };
                    if fd > 0 && u > d0 && u < d1 && dyc < -t {
                        let tt = (d1 - u) / (d1 - d0);
                        let forme = if tt < 0.3 {
                            tt / 0.3
                        } else {
                            1.0 - (tt - 0.3) / 0.7 * 0.8
                        };
                        let ondule = if g.dorsale == Nageoire::Longue {
                            (battement * 0.5 * (1.0 - tt)).round()
                        } else {
                            0.0
                        };
                        if dyc >= -t - fd as f32 * forme + ondule {
                            px[i] = nageoire(g, dyc + t, fd);
                            cl[i] = NAGEOIRE;
                            continue;
                        }
                    }
                    // Ventrale (anale) petite, à l'arrière du ventre.
                    if fv > 0 && u > 0.2 && u < 0.48 && dyc > b {
                        let tt = (0.48 - u) / 0.28;
                        let forme = if tt < 0.4 {
                            tt / 0.4
                        } else {
                            1.0 - (tt - 0.4) / 0.6
                        };
                        if dyc <= b + fv as f32 * forme {
                            px[i] = nageoire(g, dyc - b, fv);
                            cl[i] = NAGEOIRE;
                            continue;
                        }
                    }
                }
                // Queue.
                if x < x0 + 1 && x >= x0 - lq {
                    let v = ((x0 - x) as f32 - 0.5) / lq as f32;
                    let v = v.max(0.0);
                    let mut decal = (battement * amp * v.powf(1.2)).round();
                    if g.queue == Queue::Voile {
                        // Le voile retombe un peu en arrière.
                        decal += (v * v * ouv * 0.35).round();
                    }
                    let dq = dy - decal;
                    let serre = 1.0 - 0.2 * battement.abs();
                    let base_ouv = (hb * 0.12).max(0.8);
                    let s = match g.queue {
                        Queue::Fourche => base_ouv + (ouv - base_ouv) * v,
                        Queue::Eventail => base_ouv + (ouv - base_ouv) * v.powf(0.7),
                        Queue::Ronde => {
                            base_ouv
                                + (ouv - base_ouv)
                                    * (v * 2.6).min(1.0)
                                    * (1.0 - (v - 0.7).max(0.0) * 1.6)
                        }
                        Queue::Voile => base_ouv + (ouv - base_ouv) * (v * 1.6).min(1.0).powf(0.7),
                        Queue::Lyre => base_ouv + (ouv - base_ouv) * v,
                    } * serre;
                    let encoche = match g.queue {
                        Queue::Fourche => (v - 0.45).max(0.0) * ouv * 1.4,
                        Queue::Lyre => (v - 0.3).max(0.0) * ouv * 1.6,
                        _ => -1.0,
                    };
                    // Bord de fuite : convexe (éventail), ondulé (voile).
                    let r = (dq.abs() / s.max(0.5)).min(1.0);
                    let bout = match g.queue {
                        Queue::Eventail => v <= 0.78 + 0.22 * (1.0 - r * r).sqrt(),
                        Queue::Voile => v <= 0.72 + 0.28 * (r * 3.2 + battement * 0.5).cos().abs(),
                        _ => true,
                    };
                    if bout && dq.abs() <= s && dq.abs() >= encoche {
                        // Rayons de la queue : une ligne sur deux plus foncée.
                        let mut c = nageoire(g, dq.abs(), ouv as i32);
                        if g.queue != Queue::Ronde
                            && (dq.round() as i32).rem_euclid(3) == 0
                            && v > 0.2
                        {
                            let o = melange([c[0], c[1], c[2]], g.ombre, 0.25);
                            c = [o[0], o[1], o[2], c[3]];
                        }
                        px[i] = c;
                        cl[i] = NAGEOIRE;
                    }
                }
            }
        }
        // Nageoire pectorale (sur le corps) qui bat en alternance.
        let up = 0.66;
        let xp = x0 + (up * l as f32) as i32;
        let yp = (cy + hb * 0.1) as i32;
        let lp = (l / 6).max(1);
        for j in 0..lp {
            let yy = yp + if k % 2 == 0 { j / 2 } else { 0 };
            let xx = xp - j;
            let i = (yy * w + xx) as usize;
            if cl.get(i) == Some(&CORPS) {
                let c = melange(g.nageoire, g.clair, 0.3);
                px[i] = [c[0], c[1], c[2], 255];
                cl[i] = NAGEOIRE;
            }
        }
        // Œil.
        let ue = if g.forme == Forme::Globe { 0.76 } else { 0.82 };
        let xe = x0 + (ue * l as f32) as i32;
        let te = hb * 0.55 * profil(g.forme, ue);
        let ye = (cy - te * 0.35).floor() as i32;
        let pupille = [18, 20, 34, 255];
        let mut oeil = |x: i32, y: i32, c: [u8; 4]| {
            let i = (y * w + x) as usize;
            if i < px.len() {
                px[i] = c;
                cl[i] = OEIL;
            }
        };
        if l >= 20 {
            oeil(xe, ye, [250, 250, 255, 255]);
            oeil(xe + 1, ye, pupille);
            oeil(xe, ye + 1, pupille);
            oeil(xe + 1, ye + 1, pupille);
        } else {
            oeil(xe, ye, pupille);
            if l >= 14 {
                oeil(xe - 1, ye, [240, 240, 250, 255]);
            }
        }
        if k == 0 && g.rarete == Rarete::Celeste {
            for _ in 0..(l / 3).max(3) {
                for _essai in 0..20 {
                    let (x, y) = (a.n(w as u32) as i32, a.n(hgt as u32) as i32);
                    if cl[(y * w + x) as usize] == CORPS {
                        etoiles.push((x, y, a.f()));
                        break;
                    }
                }
            }
        }
        contourer(&mut px, &mut cl, w, hgt);
        let (halo1, halo2) = halos(&cl, w, hgt);
        images.push(Image {
            px,
            classe: cl,
            halo1,
            halo2,
        });
    }
    Sprite {
        l: w,
        h: hgt,
        images,
        queue: (x0 - lq + 1, cy as i32),
        bouche: (x0 + l, (cy + hb * 0.1) as i32),
        etoiles,
        nom: nom(g),
    }
}

fn couleur_corps(
    g: &Genome,
    u: f32,
    n: f32,
    dy: f32,
    points: &[(f32, f32)],
    x: i32,
    y: i32,
) -> Rgb {
    // Contre-ombrage : dos plus sombre, ventre clair.
    let mut c = if n < 0.35 {
        melange(g.ombre, g.base, n / 0.35 * 0.8 + 0.2)
    } else if n < 0.68 {
        g.base
    } else {
        melange(g.base, g.ventre, ((n - 0.68) / 0.32 * 1.4).min(0.85))
    };
    let tram = crate::toile::bayer(x, y);
    match g.motif {
        Motif::Uni => {}
        Motif::Rayures(pas) => {
            let col = (u * g.longueur as f32) as i32;
            if col.rem_euclid(pas as i32) == 0 && u > 0.12 && u < 0.86 && n < 0.85 {
                c = melange(g.accent, g.ombre, 0.35);
            }
        }
        Motif::Points => {
            for &(pu, pn) in points {
                let du = (u - pu) * g.longueur as f32;
                let dn = dy - pn * g.longueur as f32 * rapport(g.forme) * 0.45;
                if du.abs() < 0.6 && dn.abs() < 0.6 {
                    c = g.accent;
                }
            }
        }
        Motif::Degrade => {
            // Dégradé tramé de la queue vers la tête (nébuleuse pour Céleste).
            let t = u + (n - 0.5) * 0.4;
            if g.rarete == Rarete::Celeste {
                let cyan = [0x5c, 0xe8, 0xf0];
                if t > 0.55 + tram * 0.25 {
                    c = melange(c, g.accent, 0.7);
                }
                if t > 0.82 + tram * 0.2 {
                    c = melange(c, cyan, 0.6);
                }
            } else if t > 0.35 + tram * 0.4 {
                c = melange(c, g.accent, 0.65);
            }
        }
        Motif::Bande => {
            if (n - 0.45).abs() < 0.12 && u > 0.08 && u < 0.9 {
                c = melange(g.accent, [255, 255, 255], 0.2);
            }
        }
        Motif::Tete => {
            if u > 0.7 && n < 0.8 {
                c = melange(g.accent, g.ombre, 0.2);
            }
        }
    }
    c
}

fn nageoire(g: &Genome, depuis_base: f32, ampleur: i32) -> [u8; 4] {
    // Plus transparente vers le bord.
    let t = (depuis_base.abs() / ampleur.max(1) as f32).clamp(0.0, 1.0);
    let c = melange(g.nageoire, g.clair, t * 0.35);
    [c[0], c[1], c[2], if t > 0.6 { 200 } else { 235 }]
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

    #[test]
    fn sprites_deterministes_et_bornes() {
        for (i, r) in Rarete::TOUTES.iter().enumerate() {
            for graine in 0..40u64 {
                let g = genome(
                    graine * 7919 + i as u64,
                    *r,
                    0.6 + (graine % 10) as f32 * 0.1,
                );
                let s = sprite(&g);
                assert_eq!(s.images.len(), 4);
                assert!(
                    s.l > 10 && s.l < 60 && s.h > 6 && s.h < 60,
                    "{} {}x{}",
                    s.nom,
                    s.l,
                    s.h
                );
                // Un œil, un contour, et le contour ne touche pas le bord (marge pour les halos).
                let im = &s.images[0];
                assert!(im.classe.contains(&OEIL) && im.classe.contains(&CONTOUR));
                for x in 0..s.l {
                    assert_eq!(im.classe[x as usize], VIDE);
                    assert_eq!(im.classe[((s.h - 1) * s.l + x) as usize], VIDE);
                }
                let s2 = sprite(&genome(
                    graine * 7919 + i as u64,
                    *r,
                    0.6 + (graine % 10) as f32 * 0.1,
                ));
                assert_eq!(s2.images[2].px, s.images[2].px);
            }
        }
    }
}

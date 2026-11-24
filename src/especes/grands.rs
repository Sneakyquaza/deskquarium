//! Légendaires et mythiques : grands requins, cétacés, calmar géant, baudroie, dragon de mer
//! feuillu, cœlacanthe, requin-lutin, isopode géant, hydre, cadborosaure.

use super::recif::{calmar, requin};
use super::{
    caudale, esp, membrane, plier, tentacule, tourner, v, Caudale, Espece, Mouvement as M, Pal,
    Taille as T, V_SOMBRE,
};
use crate::art::{alpha, hache, lum, opaque, ton, Dessin, Px, Rampe, P};
use crate::jeu::Rarete::{Legendaire as L, Mythique as Y};
use crate::poisson::{CORPS, NAGEOIRE, OEIL};
use std::f32::consts::{PI, TAU};

pub fn especes() -> Vec<Espece> {
    vec![
        esp(
            "blanc",
            "Grand requin blanc",
            L,
            T::Enorme,
            M::Lent,
            (104, 36),
            4,
            [
                [0x6c, 0x74, 0x7c],
                [0xf0, 0xf0, 0xec],
                [0x34, 0x38, 0x40],
                [0x40, 0x18, 0x20],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("ancien", 0.08, 1.5, 0.8),
                    v("fantôme", 0.0, 0.3, 1.3),
                ]
            },
            blanc,
        ),
        esp(
            "orque",
            "Orque",
            L,
            T::Enorme,
            M::Lent,
            (116, 40),
            6,
            [
                [0x1c, 0x1c, 0x28],
                [0xf0, 0xf0, 0xf4],
                [0x78, 0x7c, 0x88],
                [0x08, 0x08, 0x10],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("blanche", 0.0, 0.0, 6.0),
                    v("des glaces", 0.55, 4.0, 2.2),
                ]
            },
            orque,
        ),
        esp(
            "requin_baleine",
            "Requin-baleine",
            L,
            T::Enorme,
            M::Lent,
            (150, 42),
            4,
            [
                [0x3c, 0x50, 0x6c],
                [0xe0, 0xe8, 0xec],
                [0x24, 0x30, 0x44],
                [0x18, 0x14, 0x20],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("doré", -0.45, 1.3, 1.2),
                    v("nocturne", 0.1, 0.8, 0.6),
                ]
            },
            requin_baleine,
        ),
        esp(
            "narval",
            "Narval",
            L,
            T::Grande,
            M::Lent,
            (84, 24),
            6,
            [
                [0x7c, 0x84, 0x90],
                [0xe8, 0xe8, 0xe4],
                [0x3c, 0x40, 0x4c],
                [0xf4, 0xec, 0xd0],
            ],
            V_SOMBRE,
            narval,
        ),
        esp(
            "calamar_geant",
            "Calamar géant",
            L,
            T::Enorme,
            M::Jet,
            (140, 44),
            6,
            [
                [0xc0, 0x40, 0x34],
                [0x70, 0x14, 0x18],
                [0xe8, 0x90, 0x70],
                [0xe0, 0xc8, 0x40],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("des abysses", 0.75, 0.8, 0.8),
                    v("opale", 0.0, 0.25, 1.3),
                ]
            },
            calamar_geant,
        ),
        esp(
            "baudroie",
            "Baudroie abyssale",
            L,
            T::Moyenne,
            M::Nage,
            (36, 30),
            4,
            [
                [0x40, 0x34, 0x34],
                [0x80, 0x64, 0x58],
                [0x18, 0x10, 0x14],
                [0xf8, 0xf4, 0xe8],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("d'encre", 0.6, 0.6, 0.6),
                    v("rouille", 0.02, 1.8, 1.4),
                ]
            },
            baudroie,
        ),
        esp(
            "dragon_feuillu",
            "Dragon de mer feuillu",
            Y,
            T::Moyenne,
            M::Debout,
            (38, 26),
            4,
            [
                [0xd8, 0xb0, 0x40],
                [0x8c, 0xa8, 0x40],
                [0xe8, 0x84, 0x38],
                [0xf8, 0xf0, 0xc0],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("corail", -0.1, 1.1, 1.0),
                    v("algue", 0.15, 1.0, 0.85),
                ]
            },
            dragon_feuillu,
        ),
        esp(
            "coelacanthe",
            "Cœlacanthe",
            Y,
            T::Grande,
            M::Lent,
            (54, 22),
            4,
            [
                [0x34, 0x4c, 0x80],
                [0xe0, 0xe8, 0xf0],
                [0x20, 0x2c, 0x4c],
                [0xc0, 0xd8, 0x70],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("brun", -0.55, 0.8, 0.8),
                    v("fossile", 0.0, 0.2, 1.1),
                ]
            },
            coelacanthe,
        ),
        esp(
            "lutin",
            "Requin-lutin",
            Y,
            T::Grande,
            M::Lent,
            (64, 22),
            4,
            [
                [0xd8, 0x94, 0x9c],
                [0xf0, 0xcc, 0xcc],
                [0xa0, 0x5c, 0x6c],
                [0x60, 0x18, 0x28],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("spectral", 0.6, 0.5, 1.1),
                    v("sanguin", 0.0, 1.6, 0.85),
                ]
            },
            lutin,
        ),
        esp(
            "bosse",
            "Baleine à bosse",
            Y,
            T::Enorme,
            M::Lent,
            (176, 56),
            6,
            [
                [0x34, 0x3c, 0x4c],
                [0xe8, 0xec, 0xf0],
                [0x20, 0x24, 0x30],
                [0x9c, 0xa4, 0xb0],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("blanche", 0.0, 0.1, 3.5),
                    v("bleue", 0.02, 1.6, 1.3),
                ]
            },
            bosse,
        ),
        esp(
            "isopode",
            "Isopode géant",
            Y,
            T::Moyenne,
            M::Marche,
            (36, 16),
            4,
            [
                [0xb8, 0xa8, 0xc0],
                [0xe8, 0xe0, 0xe8],
                [0x5c, 0x50, 0x6c],
                [0x20, 0x18, 0x28],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("doré", -0.6, 1.5, 1.05),
                    v("de jais", 0.0, 0.5, 0.45),
                ]
            },
            isopode,
        ),
        esp(
            "hydre",
            "Hydre",
            Y,
            T::Enorme,
            M::Flotte,
            (150, 112),
            8,
            [
                [0x3c, 0x7c, 0x48],
                [0xd8, 0xc8, 0x68],
                [0x20, 0x40, 0x2c],
                [0xff, 0x40, 0x30],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("d'obsidienne", 0.4, 0.4, 0.5),
                    v("pourpre", 0.5, 1.0, 1.1),
                ]
            },
            hydre,
        ),
        esp(
            "cadborosaure",
            "Cadborosaure",
            Y,
            T::Enorme,
            M::Ondule,
            (150, 44),
            8,
            [
                [0x6c, 0x5c, 0x38],
                [0xc8, 0xb0, 0x78],
                [0x2c, 0x24, 0x18],
                [0xc0, 0x50, 0x30],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("verdâtre", 0.15, 1.0, 0.9),
                    v("cendré", 0.0, 0.2, 1.1),
                ]
            },
            cadborosaure,
        ),
    ]
}

fn blanc(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let cy = h * 0.5;
    let (x0, x1) = (l * 0.15, l - 0.5);
    let hb = h * 0.3;
    // Limite dos/ventre irrégulière, très nette.
    requin(
        d,
        k,
        &p.r[0],
        &p.r[1],
        &p.r[2],
        (x0, x1),
        cy,
        hb,
        1.0,
        |t| 0.12 + 0.12 * (t * 19.0).sin() + 0.08 * (t * 41.0).sin(),
    );
    let b = (k * TAU).sin();
    let c = |t: f32| cy + plier(t, b, hb * 0.35);
    let x = |t: f32| x0 + t * (x1 - x0);
    // Gueule entrouverte et dents.
    let (bx, by) = (x(0.9), c(0.9) + hb * 0.35);
    for i in 0..((l * 0.07) as i32) {
        let xx = bx + i as f32;
        let yy = by + i as f32 * 0.25;
        d.sur(xx as i32, yy as i32, opaque(p.c[3]).unwrap());
        if i % 2 == 0 {
            d.sur(xx as i32, yy as i32 - 1, [0xf8, 0xf8, 0xf0, 255]);
        }
    }
    d.oeil(x(0.92), c(0.92) - hb * 0.25, 1, [0, 0, 0]);
    d.mettre(
        x(0.92) as i32,
        (c(0.92) - hb * 0.25) as i32,
        [20, 20, 28, 255],
        OEIL,
    );
    d.queue = ((x0 - l * 0.08) as i32, cy as i32);
    d.bouche = (x1 as i32, c(1.0) as i32);
}

/// Cétacé générique (orque, narval, baleines) : corps, caudale horizontale, dorsale.
/// `peint` colore le corps ; renvoie la géométrie (x, centre, haut, bas) pour les détails.
pub struct Cetace {
    pub x0: f32,
    pub x1: f32,
    pub hb: f32,
    /// Position, hauteur (fraction de h) et pente arrière de la dorsale (0 = pas de dorsale).
    pub dorsale: (f32, f32, f32),
    /// Tête : 0 = arrondie (orque), 1 = plate et large (baleine).
    pub tete: f32,
}

pub fn cetace<'a>(
    d: &mut Dessin,
    k: f32,
    o: &Cetace,
    dos: &'a Rampe,
    peint: impl Fn(&P) -> Option<Px>,
) -> impl Fn(f32) -> (f32, f32, f32, f32) + 'a {
    let (l, h) = (d.l as f32, d.h as f32);
    let b = (k * TAU).sin();
    let cy = h * 0.5;
    let (x0, x1, hb, tete) = (l * o.x0, l * o.x1, h * o.hb, o.tete);
    let lg = x1 - x0;
    let geo = move |t: f32| {
        let c = cy + b * h * 0.1 * (1.0 - t / 0.7).max(0.0).powi(2);
        let a = (t / 0.5).min(1.0).powf(0.7) * 0.9 + 0.1;
        let nez = ((t - 0.85) / 0.15).clamp(0.0, 1.0);
        let haut = hb * a * (1.0 - nez.powf(1.5 - tete * 0.6) * (0.6 - tete * 0.25));
        let bas = hb * a * (1.0 - nez.powi(2) * 0.45) * (1.0 + tete * 0.15);
        (x0 + t * lg, c, haut, bas)
    };
    // Caudale horizontale vue de profil.
    let (_, c0, _, _) = geo(0.0);
    let or = (x0 + 1.0, c0);
    let a = -b * 0.55;
    let lame: Vec<(f32, f32)> = [
        (0.0, -1.5),
        (-lg * 0.11, -h * 0.09),
        (-lg * 0.14, -h * 0.02),
        (-lg * 0.12, h * 0.08),
        (0.0, 1.5),
    ]
    .iter()
    .map(|&(px, py)| tourner(or, (or.0 + px, or.1 + py), a))
    .collect();
    d.poly(&lame, NAGEOIRE, |q| {
        opaque(ton(dos, 0.35 + 0.2 * q.t, q.x, q.y))
    });
    let (td, hd, pente) = o.dorsale;
    if hd > 0.0 {
        let (xa, ca, ha, _) = geo(td - 0.06);
        let (xb, cb, hbb, _) = geo(td);
        let (xc, cc, hc, _) = geo(td + 0.07);
        d.poly(
            &[
                (xa, ca - ha + 1.0),
                (xb - pente * h, cb - hbb - h * hd),
                (xb + 1.0, cb - hbb - h * hd * 0.9),
                (xc, cc - hc + 1.0),
            ],
            NAGEOIRE,
            |q| opaque(ton(dos, 0.3 + 0.3 * q.t, q.x, q.y)),
        );
    }
    d.fuseau(
        x0,
        x1,
        |t| geo(t).1,
        |t| geo(t).2,
        |t| geo(t).3,
        CORPS,
        peint,
    );
    geo
}

fn orque(d: &mut Dessin, k: f32, p: &Pal) {
    let o = Cetace {
        x0: 0.14,
        x1: 0.99,
        hb: 0.27,
        dorsale: (0.55, 0.42, 0.0),
        tete: 0.0,
    };
    let (noir, blanc, gris) = (&p.r[0], &p.r[1], &p.r[2]);
    let geo = cetace(d, k, &o, noir, |q| {
        let t = q.t;
        // Ventre blanc qui remonte en pointe sur le flanc arrière.
        let flanc = (t > 0.25 && t < 0.5) && q.v > 0.55 - (t - 0.25) * 2.2 * (0.5 - t) * 4.0;
        let ventre = q.v > 0.5 && t > 0.3;
        // Tache blanche derrière l'œil.
        let tache = {
            let (dt, dv) = ((t - 0.85) / 0.06, (q.v + 0.3) / 0.22);
            dt * dt + dv * dv < 1.0
        };
        // Selle grise derrière la dorsale.
        let selle = t > 0.4 && t < 0.52 && q.v < -0.45;
        if tache || ventre || flanc {
            return opaque(ton(blanc, 0.7 - q.v * 0.15, q.x, q.y));
        }
        if selle {
            return opaque(ton(gris, lum(q.v) + 0.1, q.x, q.y));
        }
        opaque(ton(noir, lum(q.v) * 0.75 + 0.1, q.x, q.y))
    });
    let (xp, cp, _, bp) = geo(0.75);
    let b = (k * TAU).sin();
    d.poly(
        &[
            (xp + 2.0, cp + bp * 0.4),
            (xp - d.l as f32 * 0.08, cp + bp + d.h as f32 * 0.22 + b),
            (xp - d.l as f32 * 0.02, cp + bp + d.h as f32 * 0.2 + b),
            (xp + 3.0, cp + bp * 0.7),
        ],
        NAGEOIRE,
        |q| opaque(ton(noir, 0.35, q.x, q.y)),
    );
    let (xe, ce, he, _) = geo(0.9);
    d.oeil(xe, ce - he * 0.1, 0, [0, 0, 0]);
    let (xb, cb, _, _) = geo(1.0);
    d.trait_sur(
        (xe - 2.0, ce + he * 0.25),
        (xb, cb + he * 0.15),
        [0x50, 0x50, 0x58, 255],
    );
    d.queue = (geo(0.0).0 as i32 - 6, geo(0.0).1 as i32);
    d.bouche = (xb as i32, cb as i32);
}

fn requin_baleine(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let cy = h * 0.5;
    let (x0, x1) = (l * 0.16, l - 0.5);
    let hb = h * 0.3;
    requin(
        d,
        k,
        &p.r[0],
        &p.r[1],
        &p.r[2],
        (x0, x1),
        cy,
        hb,
        0.9,
        |_| 0.35,
    );
    let b = (k * TAU).sin();
    let c = |t: f32| cy + plier(t, b, hb * 0.35);
    let x = |t: f32| x0 + t * (x1 - x0);
    // Tête large et plate : on rabote le dessus du museau et on dessine la bouche terminale.
    for xi in (x(0.9) as i32)..(x1 as i32) {
        let t = (xi as f32 - x0) / (x1 - x0);
        let (ht, _) = super::recif::profil_requin(t, hb);
        let rab = ((t - 0.9) / 0.1).clamp(0.0, 1.0) * ht * 0.3;
        let y0 = (c(t) - ht).floor() as i32;
        for y in y0..(y0 + rab as i32) {
            d.effacer(xi, y);
        }
    }
    d.trait_sur(
        (x(0.93), c(0.93) + hb * 0.15),
        (x1, c(1.0) + hb * 0.05),
        opaque(p.c[3]).unwrap(),
    );
    // Points blancs en damier et rayures pâles, crêtes le long du flanc.
    for xi in (x0 as i32)..(x1 as i32) {
        let t = (xi as f32 - x0) / (x1 - x0);
        let (ht, hbas) = super::recif::profil_requin(t, hb);
        let top = c(t) - ht;
        for y in (top as i32)..((c(t) + hbas * 0.2) as i32) {
            let ligne = ((y as f32 - top) / 3.0) as i32;
            let raie = xi % 9 == 0 && t < 0.88;
            let point = (xi + ligne * 2) % 4 == 0 && (y as f32 - top) as i32 % 3 == 1;
            if (point || raie) && t > 0.06 {
                d.sur(xi, y, [0xe4, 0xec, 0xf0, 255]);
            }
        }
        if t > 0.2 && t < 0.85 {
            d.sur(xi, (c(t) - ht * 0.35) as i32, opaque(p.r[0][3]).unwrap());
        }
    }
    d.oeil(x(0.95), c(0.95) - hb * 0.05, 0, [0, 0, 0]);
    d.queue = ((x0 - l * 0.06) as i32, cy as i32);
    d.bouche = (x1 as i32, c(1.0) as i32);
}

fn narval(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let o = Cetace {
        x0: 0.1,
        x1: 0.72,
        hb: 0.3,
        dorsale: (0.0, 0.0, 0.0),
        tete: 0.0,
    };
    let geo = cetace(d, k, &o, &p.r[0], |q| {
        // Dos marbré de taches sombres, ventre clair.
        if q.v > 0.35 {
            return opaque(ton(&p.r[1], 0.7 - q.v * 0.2, q.x, q.y));
        }
        let tache = hache(q.x / 2, q.y / 2, 41) < 0.35 - q.v * 0.2;
        let r = if tache { &p.r[2] } else { &p.r[0] };
        opaque(ton(r, lum(q.v) + 0.1, q.x, q.y))
    });
    // Défense torsadée.
    let (xb, cb, hb, _) = geo(0.98);
    let (ax, ay) = (xb, cb - hb * 0.15);
    let n = (l - ax) as i32;
    for i in 0..n {
        let x = ax + i as f32;
        let y = ay - i as f32 * 0.06;
        let c = if i % 3 == 0 { p.r[3][1] } else { p.r[3][3] };
        d.mettre(x as i32, y as i32, [c[0], c[1], c[2], 255], CORPS);
        if i < n / 3 {
            d.mettre(x as i32, y as i32 + 1, opaque(p.r[3][1]).unwrap(), CORPS);
        }
    }
    let (xp, cp, _, bp) = geo(0.75);
    d.poly(
        &[
            (xp + 1.0, cp + bp * 0.5),
            (xp - l * 0.06, cp + bp + h * 0.12),
            (xp + 1.0, cp + bp * 0.8),
        ],
        NAGEOIRE,
        |q| opaque(ton(&p.r[2], 0.45, q.x, q.y)),
    );
    let (xe, ce, he, _) = geo(0.88);
    d.oeil(xe, ce - he * 0.05, 0, [0, 0, 0]);
    d.queue = (geo(0.0).0 as i32 - 4, geo(0.0).1 as i32);
    d.bouche = (l as i32 - 1, ay as i32);
}

fn calamar_geant(d: &mut Dessin, k: f32, p: &Pal) {
    calmar(d, k, p, 4, 0.55);
}

fn baudroie(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let b = (k * TAU).sin();
    let (cx, cy) = (l * 0.5, h * 0.58);
    let (rx, ry) = (l * 0.34, h * 0.33);
    let q = caudale(Caudale::Ronde, cx - rx + 1.0, cy, l * 0.2, h * 0.16, b);
    d.poly(&q, NAGEOIRE, membrane(&p.r[1], 220));
    d.poly(
        &[
            (cx - rx * 0.7, cy - ry * 0.6),
            (cx - rx * 0.95, cy - ry * 0.95 + b * 0.5),
            (cx - rx * 0.3, cy - ry * 0.85),
        ],
        NAGEOIRE,
        membrane(&p.r[1], 220),
    );
    // Gueule immense ouverte : mâchoire du bas qui avance.
    let ouv = 0.5 + 0.2 * (k * TAU * 2.0).sin();
    d.ellipse(cx, cy, rx, ry, 0.0, CORPS, |q| {
        let bouche = q.t > 0.25 && (q.v - 0.15).abs() < ouv * (q.t - 0.25) * 1.2;
        if bouche {
            return opaque(ton(&p.r[2], 0.3 + q.t * 0.2, q.x, q.y));
        }
        // Peau rugueuse avec petits points pâles.
        if hache(q.x, q.y, 19) < 0.06 {
            return opaque(ton(&p.r[1], 0.8, q.x, q.y));
        }
        opaque(ton(&p.r[0], lum(q.v) * 0.8 + 0.1, q.x, q.y))
    });
    // Dents en aiguille sur les deux bords de la gueule.
    for i in 0..9 {
        let t = 0.3 + i as f32 * 0.08;
        for s in [-1.0f32, 1.0] {
            let x = cx + t * rx;
            let bord = cy + ry * (0.15 + s * ouv * (t - 0.25) * 1.2);
            let pointe = bord - s * (1.0 + (i % 2) as f32);
            d.trait_((x, bord), (x, pointe), opaque(p.c[3]).unwrap(), OEIL);
        }
    }
    // Leurre : tige qui part du front et se courbe en avant, boule lumineuse.
    let base = (cx + rx * 0.3, cy - ry * 0.95);
    let tige = tentacule(
        base,
        -PI * 0.42,
        l * 0.34,
        0.5,
        0.5,
        0.0,
        0.0,
        3.2 + b * 0.3,
    );
    d.tube(&tige, NAGEOIRE, |q| opaque(ton(&p.r[1], 0.5, q.x, q.y)));
    let bout = *tige.last().unwrap();
    d.ellipse(bout.0, bout.1 + 1.0, 1.6, 1.6, 0.0, CORPS, |_| {
        opaque([0xc8, 0xff, 0xf0])
    });
    d.lueur = Some((bout.0 as i32, bout.1 as i32 + 1));
    d.oeil(cx + rx * 0.35, cy - ry * 0.5, 0, [0, 0, 0]);
    d.mettre(
        (cx + rx * 0.35) as i32 - 1,
        (cy - ry * 0.5) as i32,
        [0x80, 0xa0, 0xa0, 255],
        OEIL,
    );
    d.queue = ((cx - rx) as i32 - 3, cy as i32);
    d.bouche = ((cx + rx) as i32, cy as i32);
}

fn dragon_feuillu(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let pt = |x: f32, y: f32, r: f32| (x * l, y * h, r * h);
    let corps = [
        pt(0.99, 0.32, 0.04),
        pt(0.84, 0.3, 0.05),
        pt(0.74, 0.26, 0.11),
        pt(0.64, 0.33, 0.1),
        pt(0.52, 0.45, 0.13),
        pt(0.38, 0.5, 0.12),
        pt(0.24, 0.52, 0.08),
        pt(0.12, 0.6, 0.05),
        pt(0.05, 0.75, 0.04),
        pt(0.1, 0.86, 0.03),
    ];
    // Feuilles : lobes attachés au corps, qui ondulent avec le courant.
    let feuilles = [
        (3, -1.0, 0.32),
        (4, -1.0, 0.36),
        (4, 1.0, 0.3),
        (5, -1.0, 0.3),
        (5, 1.0, 0.34),
        (6, -1.0, 0.24),
        (6, 1.0, 0.22),
        (7, 1.0, 0.2),
        (2, -1.0, 0.22),
    ];
    for (i, &(j, s, long)) in feuilles.iter().enumerate() {
        let (x, y, _) = corps[j];
        let ang = s * PI / 2.0 + 0.5 * (ph + i as f32).sin() * 0.4 - 0.3;
        let pts = tentacule(
            (x, y),
            ang,
            long * h * 1.6,
            1.1,
            0.4,
            2.0,
            ph + i as f32,
            0.0,
        );
        d.tube(&pts, NAGEOIRE, |q| {
            let r = if i % 3 == 0 { &p.r[2] } else { &p.r[1] };
            alpha(ton(r, 0.6 - q.v * 0.2, q.x, q.y), 235)
        });
        // Petite feuille plate au bout.
        let bout = pts[pts.len() - 1];
        d.ellipse(bout.0, bout.1, 2.2, 1.3, ang + 1.2, NAGEOIRE, |q| {
            alpha(ton(&p.r[1], 0.55 - q.v * 0.2, q.x, q.y), 235)
        });
    }
    d.tube(&corps, CORPS, |q| {
        let anneau = (q.t * 26.0).fract() < 0.2;
        let lu = 0.68 - q.v * 0.4 - if anneau { 0.25 } else { 0.0 };
        if q.v > 0.4 {
            return opaque(ton(&p.r[3], lu, q.x, q.y));
        }
        opaque(ton(&p.r[0], lu, q.x, q.y))
    });
    // Petites taches claires.
    for t in [0.45f32, 0.55, 0.65] {
        let (x, y, _) = corps[(t * 9.0) as usize];
        d.sur(x as i32, y as i32, opaque(p.c[3]).unwrap());
    }
    d.oeil(l * 0.74, h * 0.25, 0, [0, 0, 0]);
    d.queue = ((l * 0.08) as i32, (h * 0.8) as i32);
    d.bouche = ((l * 0.99) as i32, (h * 0.32) as i32);
}

fn coelacanthe(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let b = (k * TAU).sin();
    let cy = h * 0.5;
    let (x0, x1) = (l * 0.2, l - 0.5);
    let lg = x1 - x0;
    let hb = h * 0.3;
    let c = |t: f32| cy + plier(t, b, 1.2);
    let pr = |t: f32| (PI * (0.12 + 0.8 * t)).sin().max(0.0).powf(0.5);
    let x = |t: f32| x0 + t * lg;
    // Queue à trois lobes (le lobe central en pinceau).
    let q = caudale(Caudale::Ronde, x0 + 0.5, c(0.0), l * 0.18, h * 0.38, b);
    d.poly(&q, NAGEOIRE, |q| {
        opaque(ton(&p.r[0], 0.35 + 0.25 * q.t, q.x, q.y))
    });
    d.ellipse(
        x0 - l * 0.2,
        c(0.0) + b * 1.2,
        l * 0.06,
        h * 0.08,
        0.0,
        NAGEOIRE,
        |q| opaque(ton(&p.r[0], 0.5 - q.v * 0.2, q.x, q.y)),
    );
    // Nageoires lobées : un moignon charnu terminé par un éventail.
    let lobee = |d: &mut Dessin, t: f32, dessous: bool, ang: f32, long: f32| {
        let s = if dessous { 1.0 } else { -1.0 };
        let base = (x(t), c(t) + s * hb * pr(t) * 0.8);
        let a = ang + 0.25 * b * s;
        let bout = (base.0 + a.cos() * long, base.1 + s * a.sin() * long);
        d.tube(
            &[(base.0, base.1, 1.8), (bout.0, bout.1, 1.2)],
            NAGEOIRE,
            |q| opaque(ton(&p.r[0], 0.5 - q.v * 0.15, q.x, q.y)),
        );
        d.ellipse(bout.0, bout.1, long * 0.45, 1.6, a * s, NAGEOIRE, |q| {
            alpha(ton(&p.r[2], 0.5, q.x, q.y), 230)
        });
    };
    lobee(d, 0.28, false, PI * 0.8, l * 0.1);
    lobee(d, 0.3, true, PI * 0.75, l * 0.1);
    lobee(d, 0.52, true, PI * 0.7, l * 0.09);
    // Première dorsale en éventail.
    d.poly(
        &[
            (x(0.5), c(0.5) - hb * pr(0.5) + 1.0),
            (x(0.48), c(0.48) - hb - h * 0.2),
            (x(0.58), c(0.58) - hb - h * 0.18),
            (x(0.62), c(0.62) - hb * pr(0.62) + 1.0),
        ],
        NAGEOIRE,
        |q| alpha(ton(&p.r[2], 0.45, q.x, q.y), 230),
    );
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t),
        CORPS,
        |q| {
            // Taches blanches irrégulières et écailles en losanges.
            let tache = hache(q.x / 3, q.y / 2, 61) < 0.22;
            if tache {
                return opaque(ton(&p.r[1], 0.6, q.x, q.y));
            }
            let ecaille = (q.x + q.y) % 3 == 0;
            opaque(ton(
                &p.r[0],
                lum(q.v) + if ecaille { 0.15 } else { 0.0 },
                q.x,
                q.y,
            ))
        },
    );
    lobee(d, 0.7, true, PI * 0.7, l * 0.11);
    d.oeil(x(0.88), c(0.88) - hb * 0.2, 2, p.c[3]);
    d.queue = (x0 as i32 - 6, c(0.0) as i32);
    d.bouche = (x1 as i32, c(1.0) as i32);
}

fn lutin(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let cy = h * 0.45;
    let (x0, x1) = (l * 0.14, l * 0.78);
    let hb = h * 0.24;
    requin(
        d,
        k,
        &p.r[0],
        &p.r[1],
        &p.r[2],
        (x0, x1),
        cy,
        hb,
        0.4,
        |_| 0.4,
    );
    let b = (k * TAU).sin();
    let c = |t: f32| cy + plier(t, b, hb * 0.35);
    // Long museau plat en lame au-dessus de la bouche.
    d.fuseau(
        x1 - 2.0,
        l - 0.5,
        |t| c(1.0) - hb * 0.35 - t * 0.6,
        |t| 1.6 - t * 0.8,
        |t| 1.2 - t * 0.6,
        CORPS,
        |q| opaque(ton(&p.r[0], 0.65 - q.v * 0.25, q.x, q.y)),
    );
    // Mâchoires projetées en avant, pleines de dents.
    let ouv = 0.6 + 0.4 * (k * TAU).cos();
    let (mx, my) = (x1 - l * 0.02, c(1.0) + hb * 0.3);
    let jaw = [
        (mx - l * 0.06, my - 1.0),
        (mx + l * 0.07, my + ouv * 1.5),
        (mx + l * 0.06, my + 2.0 + ouv * 2.5),
        (mx - l * 0.06, my + 2.5),
    ];
    d.poly(&jaw, CORPS, |q| {
        opaque(ton(&p.r[2], 0.45 + q.v * 0.2, q.x, q.y))
    });
    for i in 0..5 {
        let x = mx - l * 0.03 + i as f32 * 2.0;
        d.mettre(
            x as i32,
            (my + ouv * 0.8 + i as f32 * 0.2) as i32,
            [0xf8, 0xf0, 0xe8, 255],
            OEIL,
        );
    }
    d.oeil(x1 - l * 0.03, c(1.0) - hb * 0.4, 0, [0, 0, 0]);
    d.queue = ((x0 - l * 0.05) as i32, cy as i32);
    d.bouche = (l as i32 - 1, my as i32);
}

fn bosse(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let o = Cetace {
        x0: 0.1,
        x1: 0.99,
        hb: 0.27,
        dorsale: (0.38, 0.1, 0.02),
        tete: 1.0,
    };
    let (dos, blanc) = (&p.r[0], &p.r[1]);
    let geo = cetace(d, k, &o, dos, |q| {
        // Sillons de la gorge : rayures claires sous la tête.
        if q.v > 0.25 && q.t > 0.62 {
            let sillon = (q.y as f32 * 1.0 + q.t * 3.0) as i32 % 2 == 0;
            return opaque(ton(if sillon { blanc } else { dos }, 0.55, q.x, q.y));
        }
        if q.v > 0.55 {
            return opaque(ton(blanc, 0.6, q.x, q.y));
        }
        // Tubercules sur la tête.
        if q.t > 0.88 && q.v < -0.2 && hache(q.x, q.y, 3) < 0.25 {
            return opaque(ton(dos, 0.95, q.x, q.y));
        }
        opaque(ton(dos, lum(q.v) * 0.8 + 0.12, q.x, q.y))
    });
    // Immense pectorale blanche festonnée.
    let (xp, cp, _, bp) = geo(0.72);
    let b = (k * TAU).sin();
    let ang = 0.55 + 0.2 * b;
    let pts: Vec<(f32, f32)> = (0..=10)
        .map(|i| {
            let u = i as f32 / 10.0;
            (-u * l * 0.3, (if i % 2 == 0 { 0.0 } else { 0.8 }) + 1.5)
        })
        .chain((0..=10).rev().map(|i| {
            let u = i as f32 / 10.0;
            (-u * l * 0.3, -h * 0.06 * (1.0 - u * 0.6))
        }))
        .map(|(x, y)| tourner((xp, cp + bp * 0.6), (xp + x, cp + bp * 0.6 + y), ang))
        .collect();
    d.poly(&pts, NAGEOIRE, |q| {
        opaque(ton(blanc, 0.55 + 0.15 * q.v, q.x, q.y))
    });
    let (xe, ce, _, be) = geo(0.84);
    d.oeil(xe, ce + be * 0.15, 0, [0, 0, 0]);
    let (xb, cb, _, _) = geo(1.0);
    d.trait_sur(
        (xe - l * 0.06, ce + be * 0.2),
        (xb, cb + be * 0.05),
        [0x18, 0x18, 0x20, 255],
    );
    d.queue = (geo(0.0).0 as i32 - 8, geo(0.0).1 as i32);
    d.bouche = (xb as i32, cb as i32);
}

fn isopode(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = (k * 4.0) as i32;
    let sol = h - 0.5;
    // Pattes : 7 paires, alternées.
    for i in 0..7 {
        let x = l * (0.2 + i as f32 * 0.1);
        let lev = if (i + ph) % 2 == 0 { 1.0 } else { 0.0 };
        d.trait_(
            (x, h * 0.6),
            (x + 1.0 - lev, sol - lev),
            opaque(p.c[2]).unwrap(),
            NAGEOIRE,
        );
    }
    // Uropodes (queue) et antennes.
    d.poly(
        &[(l * 0.12, h * 0.55), (0.5, h * 0.8), (l * 0.1, h * 0.75)],
        NAGEOIRE,
        |q| opaque(ton(&p.r[0], 0.4, q.x, q.y)),
    );
    let ant = (ph % 2) as f32 * 0.8;
    d.trait_(
        (l * 0.92, h * 0.45),
        (l - 0.5, h * 0.15 + ant),
        opaque(p.c[2]).unwrap(),
        NAGEOIRE,
    );
    d.trait_(
        (l * 0.9, h * 0.5),
        (l - 0.5, h * 0.62),
        opaque(p.c[2]).unwrap(),
        NAGEOIRE,
    );
    // Carapace segmentée en dôme.
    let (cx, cy) = (l * 0.5, h * 0.62);
    d.ellipse(cx, cy, l * 0.42, h * 0.5, 0.0, CORPS, |q| {
        if q.v > 0.25 {
            return None;
        }
        let seg = ((q.t + 1.0) * 4.5).fract();
        let bord = seg < 0.16;
        let lu = 0.62 - q.v * 0.35 - if bord { 0.35 } else { 0.0 } + seg * 0.12;
        opaque(ton(&p.r[0], lu, q.x, q.y))
    });
    // Tête et gros yeux composés.
    d.ellipse(l * 0.88, h * 0.55, l * 0.07, h * 0.22, 0.0, CORPS, |q| {
        opaque(ton(&p.r[0], 0.55 - q.v * 0.3, q.x, q.y))
    });
    d.ellipse(l * 0.89, h * 0.48, 1.4, 1.4, 0.0, OEIL, |q| {
        opaque(if q.t < -0.2 && q.v < -0.2 {
            p.r[3][4]
        } else {
            p.c[3]
        })
    });
    d.queue = ((l * 0.08) as i32, (h * 0.7) as i32);
    d.bouche = ((l * 0.95) as i32, (h * 0.6) as i32);
}

/// Écailles de reptile : petits arcs plus sombres en quinconce.
pub fn ecailles(r: &Rampe, q: &P, base: f32, pas: i32) -> Option<Px> {
    let rang = q.y.div_euclid(pas);
    let col = (q.x + if rang % 2 == 0 { 0 } else { pas }).rem_euclid(pas * 2);
    let bord = q.y.rem_euclid(pas) == pas - 1 || col == 0;
    opaque(ton(r, base - if bord { 0.25 } else { 0.0 }, q.x, q.y))
}

fn hydre(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let (cx, cy) = (l * 0.5, h * 0.72);
    // Queue qui s'enroule vers le bas à gauche.
    let queue = tentacule(
        (cx - l * 0.12, cy + h * 0.1),
        PI * 0.85,
        l * 0.42,
        9.0,
        1.5,
        1.6,
        ph,
        -1.5,
    );
    d.tube(&queue, CORPS, |q| ecailles(&p.r[0], q, 0.6 - q.v * 0.3, 3));
    // Cinq cous qui se balancent, chacun avec sa tête.
    let cous = [-1.0f32, -0.5, 0.0, 0.5, 1.0];
    for (i, &u) in cous.iter().enumerate() {
        let depart = (cx + u * l * 0.1, cy - h * 0.12);
        let ang = -PI / 2.0 + u * 0.95 + 0.12 * (ph + i as f32 * 1.3).sin();
        let long = h * (0.52 - u.abs() * 0.08);
        let cou = tentacule(
            depart,
            ang,
            long,
            5.0,
            2.6,
            1.2,
            ph + i as f32 * 1.7,
            -u * 1.2,
        );
        d.tube(&cou, CORPS, |q| {
            if q.v > 0.45 {
                return opaque(ton(&p.r[1], 0.6, q.x, q.y));
            }
            ecailles(&p.r[0], q, 0.62 - q.v * 0.3, 3)
        });
        let (hx, hy, _) = *cou.last().unwrap();
        let (ax, ay, _) = cou[cou.len() - 2];
        // Les têtes regardent vers l'extérieur, un peu vers le bas.
        let n = (hx - ax).hypot(hy - ay).max(0.01);
        let cote = if u < 0.0 {
            -1.0
        } else if u > 0.0 {
            1.0
        } else {
            0.6
        };
        let (vx, vy) = (
            0.35 * (hx - ax) / n + 0.65 * cote,
            0.35 * (hy - ay) / n + 0.2,
        );
        let dir = vy.atan2(vx);
        tete_serpent(
            d,
            (hx, hy),
            dir,
            h * 0.06,
            p,
            (i % 2 == 0) as i32 as f32 * (0.3 + 0.3 * (ph * 2.0).sin()),
        );
    }
    // Corps massif.
    d.ellipse(cx, cy, l * 0.2, h * 0.17, 0.0, CORPS, |q| {
        if q.v > 0.35 {
            return opaque(ton(&p.r[1], 0.55 - q.v * 0.1, q.x, q.y));
        }
        ecailles(&p.r[0], q, lum(q.v) + 0.05, 3)
    });
    // Pattes griffues.
    for s in [-1.0f32, 1.0] {
        let pts = [
            (cx + s * l * 0.12, cy + h * 0.06, 3.5),
            (cx + s * l * 0.17, cy + h * 0.18, 2.5),
            (cx + s * l * 0.2, cy + h * 0.24, 1.5),
        ];
        d.tube(&pts, CORPS, |q| ecailles(&p.r[2], q, 0.5, 3));
    }
    d.queue = (
        queue.last().unwrap().0 as i32,
        queue.last().unwrap().1 as i32,
    );
    d.bouche = (cx as i32, (h * 0.05) as i32);
}

/// Tête de serpent/dragon orientée selon `dir` : museau, mâchoire qui s'ouvre (`ouv` 0..1),
/// œil luisant, petite corne.
pub fn tete_serpent(d: &mut Dessin, o: (f32, f32), dir: f32, r: f32, p: &Pal, ouv: f32) {
    // Tête tournée vers la gauche : on la retourne pour garder la mâchoire en bas.
    let fy = if dir.cos() < 0.0 { -1.0 } else { 1.0 };
    let pt = |x: f32, y: f32| tourner(o, (o.0 + x * r, o.1 + y * r * fy), dir);
    let crane = [
        pt(-1.0, -1.1),
        pt(1.2, -0.9),
        pt(2.6, -0.4),
        pt(2.8, 0.1),
        pt(0.6, 0.3),
        pt(-1.0, 0.9),
    ];
    let machoire = [
        pt(-0.6, 0.3),
        pt(0.5, 0.3 + ouv * 0.4),
        pt(2.4, 0.4 + ouv * 1.4),
        pt(2.0, 0.9 + ouv * 1.4),
        pt(-0.4, 1.0),
    ];
    d.poly(&machoire, CORPS, |q| {
        opaque(ton(&p.r[1], 0.5 - q.v * 0.1, q.x, q.y))
    });
    if ouv > 0.15 {
        let gueule = [pt(0.4, 0.3), pt(2.6, 0.15), pt(2.2, 0.4 + ouv * 1.2)];
        d.poly(&gueule, CORPS, |_| opaque([0x50, 0x10, 0x18]));
    }
    d.poly(&crane, CORPS, |q| ecailles(&p.r[0], q, 0.7 - q.v * 0.3, 2));
    let corne = [pt(-0.6, -0.9), pt(-1.8, -1.9), pt(-0.1, -1.0)];
    d.poly(&corne, CORPS, |q| {
        opaque(ton(&p.r[1], 0.6 - q.v * 0.2, q.x, q.y))
    });
    let (ex, ey) = pt(0.9, -0.45);
    d.mettre(ex as i32, ey as i32, opaque(p.c[3]).unwrap(), OEIL);
    d.mettre(ex as i32 + 1, ey as i32, opaque(p.r[3][4]).unwrap(), OEIL);
}

fn cadborosaure(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    // Corps en bosses verticales qui avancent (vague), de la queue (gauche) au cou (droite).
    let x0 = l * 0.02;
    let x1 = l * 0.8;
    let cy = h * 0.62;
    let c = |t: f32| cy - h * 0.2 * (t * TAU * 1.6 - ph).sin().max(-0.4) * (0.4 + 0.6 * t.min(0.8));
    let hb = h * 0.14;
    let pr = |t: f32| (t * 4.0).min(1.0) * (0.6 + 0.4 * (1.0 - (t - 0.7).abs()));
    // Nageoires (deux paires).
    for (t, sombre) in [(0.72f32, 0.0f32), (0.3, 0.1)] {
        let (x, y) = (x0 + t * (x1 - x0), c(t) + hb * 0.6);
        let a = 0.3 * (ph + t * 6.0).sin();
        let pts: Vec<(f32, f32)> = [
            (1.5, -1.0),
            (-l * 0.06, h * 0.18),
            (-l * 0.02, h * 0.2),
            (2.0, 1.5),
        ]
        .iter()
        .map(|&(px, py)| tourner((x, y), (x + px, y + py), a))
        .collect();
        d.poly(&pts, NAGEOIRE, |q| {
            opaque(ton(&p.r[2], 0.5 - sombre + 0.2 * q.t, q.x, q.y))
        });
    }
    // Queue en nageoire.
    d.poly(
        &[
            (x0 + 3.0, c(0.0) - 1.0),
            (0.5, c(0.0) - h * 0.14),
            (x0 + 1.0, c(0.0) + 1.0),
            (0.5, c(0.0) + h * 0.14),
        ],
        NAGEOIRE,
        |q| opaque(ton(&p.r[2], 0.5, q.x, q.y)),
    );
    // Crête dentelée le long du dos.
    d.fuseau(
        x0 + l * 0.05,
        x1,
        |t| c(t) - hb * pr(t),
        |t| if (t * 40.0).fract() < 0.5 { 2.2 } else { 0.8 },
        |_| 0.5,
        NAGEOIRE,
        |q| opaque(ton(&p.r[3], 0.45, q.x, q.y)),
    );
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t),
        CORPS,
        |q| {
            if q.v > 0.4 {
                return opaque(ton(&p.r[1], 0.6, q.x, q.y));
            }
            ecailles(&p.r[0], q, lum(q.v) + 0.05, 3)
        },
    );
    // Long cou dressé et tête de cheval avec crinière.
    let (nx, ny) = (x1, c(1.0));
    let cou = [
        (nx - 2.0, ny, hb * 1.1),
        (l * 0.86, h * 0.35, hb * 0.95),
        (l * 0.89, h * 0.17, hb * 0.85),
    ];
    d.tube(&cou, CORPS, |q| ecailles(&p.r[0], q, 0.62 - q.v * 0.3, 3));
    for i in 0..7 {
        let u = i as f32 / 6.0;
        let (x, y) = (l * (0.85 - u * 0.02), h * (0.14 + u * 0.32));
        let lg = 4.0 + 2.0 * (ph + u * 5.0).sin();
        d.trait_(
            (x, y),
            (x - lg, y + 1.5),
            opaque(p.r[3][1 + i % 2]).unwrap(),
            NAGEOIRE,
        );
    }
    // Tête : crâne allongé de cheval, naseau, oreilles.
    let (hx, hy) = (l * 0.9, h * 0.15);
    d.ellipse(hx, hy, l * 0.035, h * 0.1, 0.2, CORPS, volume(&p.r[0]));
    d.poly(
        &[
            (hx, hy - h * 0.08),
            (hx + l * 0.09, hy + h * 0.05),
            (hx + l * 0.085, hy + h * 0.12),
            (hx, hy + h * 0.1),
        ],
        CORPS,
        |q| opaque(ton(&p.r[0], 0.6 - q.v * 0.25, q.x, q.y)),
    );
    d.poly(
        &[
            (hx - 1.0, hy - h * 0.06),
            (hx - 2.0, hy - h * 0.16),
            (hx + 1.0, hy - h * 0.08),
        ],
        CORPS,
        |q| opaque(ton(&p.r[0], 0.5, q.x, q.y)),
    );
    d.oeil(hx + l * 0.012, hy - h * 0.02, 0, [0, 0, 0]);
    d.sur(
        (hx + l * 0.08) as i32,
        (hy + h * 0.07) as i32,
        opaque(p.c[2]).unwrap(),
    );
    d.queue = (1, c(0.0) as i32);
    d.bouche = ((hx + l * 0.09) as i32, (hy + h * 0.08) as i32);
}

fn volume(r: &Rampe) -> impl Fn(&P) -> Option<Px> + '_ {
    super::volume(r)
}

//! Célestes (cosmiques, translucides, ciel étoilé dans le corps) et irréels (créatures
//! colossales des légendes et des abysses, dessins originaux).

use super::grands::{cetace, ecailles, tete_serpent, Cetace};
use super::recif::{cloche, vague};
use super::{
    caudale, esp, plier, tentacule, tourner, v, Caudale, Espece, Mouvement as M, Pal, Taille as T,
    V_COSMIQUE,
};
use crate::art::{alpha, hache, lum, opaque, ton, Dessin, Px, P};
use crate::jeu::Rarete::{Celeste as C, Irreel as I};
use crate::poisson::{CORPS, NAGEOIRE, OEIL};
use crate::toile::{hsv, melange, Rgb};
use std::f32::consts::{PI, TAU};

pub fn especes() -> Vec<Espece> {
    let mut liste = vec![
        esp(
            "baleine_stellaire",
            "Baleine stellaire",
            C,
            T::Enorme,
            M::Lent,
            (156, 50),
            6,
            [
                [0x28, 0x24, 0x78],
                [0xd0, 0x60, 0xd8],
                [0x40, 0xd8, 0xf0],
                [0xff, 0xff, 0xff],
            ],
            V_COSMIQUE,
            baleine_stellaire,
        ),
        esp(
            "meduse_nebuleuse",
            "Méduse nébuleuse",
            C,
            T::Grande,
            M::Pulse,
            (38, 58),
            6,
            [
                [0x5c, 0x3c, 0xc8],
                [0xf0, 0x70, 0xd0],
                [0x50, 0xe8, 0xf0],
                [0xff, 0xff, 0xff],
            ],
            V_COSMIQUE,
            meduse_nebuleuse,
        ),
        esp(
            "raie_aurore",
            "Raie aurore",
            C,
            T::Grande,
            M::Plane,
            (68, 50),
            6,
            [
                [0x30, 0xe0, 0x90],
                [0x90, 0x50, 0xf0],
                [0x10, 0x28, 0x40],
                [0xe0, 0xff, 0xf0],
            ],
            V_COSMIQUE,
            raie_aurore,
        ),
        esp(
            "carpe_comete",
            "Carpe-comète",
            C,
            T::Moyenne,
            M::Nage,
            (44, 20),
            4,
            [
                [0xf8, 0x70, 0x30],
                [0xf8, 0xf0, 0xe8],
                [0x60, 0xd0, 0xff],
                [0xff, 0xf0, 0xa0],
            ],
            V_COSMIQUE,
            carpe_comete,
        ),
        esp(
            "kraken",
            "Kraken",
            I,
            T::Colossale,
            M::Abysse,
            (230, 180),
            8,
            [
                [0x84, 0x20, 0x34],
                [0xec, 0xb0, 0xa4],
                [0x34, 0x08, 0x18],
                [0xff, 0xd0, 0x40],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("des glaces", 0.55, 0.5, 1.1),
                    v("d'encre", 0.75, 0.6, 0.6),
                ]
            },
            kraken,
        ),
        esp(
            "cthulhu",
            "Cthulhu",
            I,
            T::Colossale,
            M::Flotte,
            (196, 172),
            6,
            [
                [0x3c, 0x60, 0x4c],
                [0x2c, 0x34, 0x40],
                [0x14, 0x20, 0x1c],
                [0xff, 0x50, 0x30],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("abyssal", 0.3, 0.8, 0.8),
                    v("de corail", -0.4, 1.0, 1.2),
                ]
            },
            cthulhu,
        ),
        esp(
            "leviathan",
            "Léviathan",
            I,
            T::Colossale,
            M::Traverse,
            (340, 66),
            8,
            [
                [0x1c, 0x5c, 0x68],
                [0xc8, 0x98, 0x50],
                [0x0c, 0x24, 0x2c],
                [0xff, 0xc0, 0x30],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("de braise", -0.5, 1.4, 1.0),
                    v("d'argent", 0.0, 0.2, 1.4),
                ]
            },
            leviathan,
        ),
        esp(
            "oeil",
            "Œil des abysses",
            I,
            T::Colossale,
            M::Flotte,
            (150, 150),
            8,
            [
                [0x5c, 0x2c, 0x4c],
                [0xec, 0xe4, 0xd8],
                [0xa0, 0x50, 0xff],
                [0xc0, 0x24, 0x34],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("vert", 0.3, 1.0, 0.9),
                    v("pâle", 0.0, 0.3, 1.4),
                ]
            },
            oeil,
        ),
        esp(
            "bloop",
            "Le Bloop",
            I,
            T::Colossale,
            M::Lent,
            (264, 112),
            6,
            [
                [0xc4, 0xc0, 0xbc],
                [0xe8, 0xe4, 0xe0],
                [0x58, 0xd0, 0xff],
                [0x30, 0x28, 0x34],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("spectral", 0.55, 2.0, 1.0),
                    v("cendré", 0.0, 0.5, 0.75),
                ]
            },
            bloop,
        ),
        esp(
            "grand_abyssal",
            "Le Grand Abyssal",
            I,
            T::Colossale,
            M::Traverse,
            (360, 58),
            8,
            [
                [0x30, 0x34, 0x3c],
                [0x90, 0x8c, 0x94],
                [0x34, 0x08, 0x12],
                [0xec, 0xe8, 0xdc],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("blême", 0.0, 0.5, 2.4),
                    v("brun", 0.05, 3.0, 1.2),
                ]
            },
            grand_abyssal,
        ),
        esp(
            "lusca",
            "Lusca",
            I,
            T::Enorme,
            M::Nage,
            (130, 58),
            6,
            [
                [0x70, 0x7c, 0x88],
                [0xa8, 0x5c, 0x5c],
                [0xec, 0xec, 0xe4],
                [0xf0, 0xc8, 0x40],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("des trous bleus", 0.45, 1.6, 0.9),
                    v("corail", -0.55, 1.3, 1.1),
                ]
            },
            lusca,
        ),
    ];
    let mut j = esp(
        "jormungandr",
        "Jörmungandr",
        I,
        T::Colossale,
        M::Traverse,
        (420, 74),
        8,
        [
            [0x2c, 0x70, 0x48],
            [0xe8, 0xc0, 0x50],
            [0x10, 0x30, 0x20],
            [0xff, 0xe0, 0x60],
        ],
        const {
            &[
                v("", 0.0, 1.0, 1.0),
                v("de cendre", 0.0, 0.2, 0.9),
                v("de cuivre", -0.3, 1.2, 1.2),
            ]
        },
        jormungandr,
    );
    // Le serpent-monde est le plus rare de tous.
    j.poids = 0.35;
    liste.push(j);
    liste
}

/// Ciel profond translucide : nébuleuses (deux rampes) et étoiles fixes.
fn cosmos(p: &Pal, q: &P, base: f32, a: u8) -> Option<Px> {
    let (x, y) = (q.x as f32, q.y as f32);
    let neb = (x * 0.09 + (y * 0.13).sin() * 1.7).sin() + (y * 0.07 - x * 0.04).cos() * 0.8;
    let etoile = hache(q.x, q.y, 101);
    if etoile < 0.018 {
        return opaque([255, 255, 255]);
    }
    if etoile < 0.04 {
        return alpha(p.r[2][4], 240);
    }
    let r = if neb > 0.9 {
        &p.r[1]
    } else if neb < -0.9 {
        &p.r[2]
    } else {
        &p.r[0]
    };
    let k = if neb.abs() > 0.9 { 0.45 } else { 0.0 };
    alpha(ton(r, base * (0.7 + k) + 0.05, q.x, q.y), a)
}

fn baleine_stellaire(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let o = Cetace {
        x0: 0.1,
        x1: 0.99,
        hb: 0.27,
        dorsale: (0.38, 0.07, 0.02),
        tete: 1.0,
    };
    let geo = cetace(d, k, &o, &p.r[0], |q| {
        // Bord lumineux, intérieur de ciel étoilé.
        if q.v.abs() > 0.86 {
            return alpha(ton(&p.r[2], 0.7, q.x, q.y), 230);
        }
        if q.v > 0.3 && q.t > 0.6 && (q.y + (q.t * 4.0) as i32) % 3 == 0 {
            return alpha(ton(&p.r[2], 0.55, q.x, q.y), 210);
        }
        cosmos(p, q, lum(q.v), 215)
    });
    // Constellation sur le flanc.
    let pts = [
        (0.3, -0.2),
        (0.4, 0.1),
        (0.52, -0.3),
        (0.62, 0.05),
        (0.7, -0.15),
    ];
    let pos: Vec<(f32, f32)> = pts
        .iter()
        .map(|&(t, v)| {
            let (x, c, ht, _) = geo(t);
            (x, c + v * ht)
        })
        .collect();
    for w in pos.windows(2) {
        d.trait_sur(w[0], w[1], [0xb0, 0xe8, 0xff, 120]);
    }
    for &(x, y) in &pos {
        d.sur(x as i32, y as i32, [255, 255, 255, 255]);
    }
    // Longue pectorale de lumière.
    let (xp, cp, _, bp) = geo(0.72);
    let ang = 0.5 + 0.2 * (k * TAU).sin();
    let pts: Vec<(f32, f32)> = [
        (0.0, 1.5),
        (-l * 0.24, 1.0),
        (-l * 0.25, -1.0),
        (0.0, -h * 0.05),
    ]
    .iter()
    .map(|&(x, y)| tourner((xp, cp + bp * 0.6), (xp + x, cp + bp * 0.6 + y), ang))
    .collect();
    d.poly(&pts, NAGEOIRE, |q| cosmos(p, q, 0.7, 200));
    let (xe, ce, _, be) = geo(0.85);
    d.oeil(xe, ce + be * 0.1, 1, p.c[2]);
    d.queue = (geo(0.0).0 as i32 - 8, geo(0.0).1 as i32);
    d.bouche = (geo(1.0).0 as i32, geo(1.0).1 as i32);
}

fn meduse_nebuleuse(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let c = (k * TAU).sin().max(0.0);
    let cx = l * 0.5;
    let (rx, ry) = (l * 0.46 * (1.0 - 0.15 * c), h * 0.22 * (1.0 + 0.1 * c));
    let cy = ry + 0.5;
    let bas = cy + ry * 0.25;
    for i in 0..9 {
        let u = -0.9 + 1.8 * i as f32 / 8.0;
        let long = h - bas - 1.5 - (i % 3) as f32 * 4.0;
        let pts = tentacule(
            (cx + u * rx * 0.95, bas),
            PI / 2.0 + u * 0.12,
            long,
            0.6,
            0.45,
            1.0,
            k * TAU + i as f32,
            0.0,
        );
        let teinte = i as f32 / 9.0;
        d.tube(&pts, NAGEOIRE, |q| {
            let col = melange(p.c[1], p.c[2], (q.t + teinte * 0.5).min(1.0));
            if hache(q.x, q.y, 3) < 0.05 {
                return opaque([255, 255, 255]);
            }
            alpha(col, (230.0 - q.t * 120.0) as u8)
        });
    }
    for (i, u) in [-0.2f32, 0.05, 0.25].iter().enumerate() {
        let pts = tentacule(
            (cx + u * rx, bas - 1.0),
            PI / 2.0,
            h * 0.42,
            2.2,
            0.8,
            1.3,
            k * TAU * 0.8 + i as f32 * 2.0,
            0.0,
        );
        d.tube(&pts, NAGEOIRE, |q| cosmos(p, q, 0.6 + q.v * 0.2, 220));
    }
    cloche(d, cx, cy, rx, ry, &p.r[0], &p.r[3], 0);
    d.ellipse(cx, cy, rx * 0.92, ry * 0.92, 0.0, CORPS, |q| {
        if q.v > 0.25 {
            return None;
        }
        cosmos(p, q, 0.75 - q.v * 0.4, 215)
    });
    d.queue = (cx as i32, h as i32 - 2);
    d.bouche = (cx as i32, bas as i32);
}

fn raie_aurore(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let cy = h * 0.5;
    let s = 0.82 + 0.18 * ph.cos();
    let rec = 0.06 * ph.sin();
    // Ruban de queue ondulant.
    let queue: Vec<(f32, f32, f32)> = (0..=14)
        .map(|i| {
            let u = i as f32 / 14.0;
            (
                l * 0.3 * (1.0 - u) + 0.5,
                cy + (ph + u * 5.0).sin() * u * 3.0,
                1.4 - u,
            )
        })
        .collect();
    d.tube(&queue, NAGEOIRE, |q| {
        let c = melange(p.c[0], p.c[1], q.t);
        alpha(c, (240.0 - q.t * 100.0) as u8)
    });
    for sg in [-1.0f32, 1.0] {
        let aile = [
            (l * 0.92, cy + sg * h * 0.06),
            (l * 0.74, cy + sg * h * 0.15),
            (l * (0.5 - rec), cy + sg * h * 0.5 * s),
            (l * (0.42 - rec), cy + sg * h * 0.46 * s),
            (l * 0.4, cy + sg * h * 0.28 * s),
            (l * 0.3, cy + sg * h * 0.1),
            (l * 0.26, cy),
            (l * 0.92, cy),
        ];
        d.poly(&aile, CORPS, |q| {
            // Rideaux d'aurore : bandes verticales qui ondulent et passent du vert au violet.
            let y = ((q.y as f32 + 0.5 - cy) / h).abs() * 2.0;
            let rideau =
                ((q.x as f32 * 0.35 + (q.y as f32 * 0.2 + ph).sin() * 2.0).sin() + 1.0) * 0.5;
            let c = melange(p.c[0], p.c[1], y.min(1.0));
            let c = melange(p.c[2], c, 0.45 + rideau * 0.55);
            if hache(q.x, q.y, 7) < 0.02 {
                return opaque([255, 255, 255]);
            }
            alpha(c, (200.0 + rideau * 40.0) as u8)
        });
    }
    d.trait_(
        (l * 0.3, cy),
        (l * 0.88, cy),
        [p.c[3][0], p.c[3][1], p.c[3][2], 160],
        CORPS,
    );
    for sg in [-1.0f32, 1.0] {
        d.ellipse(
            l * 0.95,
            cy + sg * h * 0.06,
            l * 0.05,
            h * 0.03,
            sg * 0.3,
            CORPS,
            |q| alpha(ton(&p.r[0], 0.7 - q.v * sg * 0.2, q.x, q.y), 230),
        );
        d.oeil(l * 0.86, cy + sg * h * 0.08 - 0.5, 0, [0, 0, 0]);
    }
    d.queue = (1, cy as i32);
    d.bouche = ((l * 0.98) as i32, cy as i32);
}

fn carpe_comete(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let b = (k * TAU).sin();
    let cy = h * 0.5;
    let (x0, x1) = (l * 0.42, l - 0.5);
    let lg = x1 - x0;
    let hb = h * 0.28;
    let c = |t: f32| cy + plier(t, b, 1.2);
    let pr = |t: f32| (PI * (0.12 + 0.8 * t)).sin().max(0.0).powf(0.55);
    // Queue de comète : longs voiles qui se défont en poussière d'étoiles.
    for (dy, a) in [(-1.0f32, 0.8f32), (1.0, 1.0)] {
        let q = caudale(
            Caudale::Voile,
            x0 + 0.5,
            c(0.0) + dy,
            l * 0.42,
            h * 0.36,
            b * a,
        );
        d.poly(&q, NAGEOIRE, |q| {
            let fin = 1.0 - q.t;
            if fin > 0.55 && hache(q.x, q.y, 5) < fin - 0.55 {
                return None;
            }
            let c = melange(p.c[3], p.c[2], fin);
            alpha(c, (230.0 - fin * 110.0) as u8)
        });
    }
    d.poly(
        &[
            (x0 + lg * 0.3, c(0.3) - hb * pr(0.3) + 0.5),
            (x0 + lg * 0.4, c(0.4) - hb - h * 0.18),
            (x0 + lg * 0.7, c(0.7) - hb * pr(0.7) + 0.5),
        ],
        NAGEOIRE,
        |q| alpha(melange(p.c[3], p.c[2], q.t), 210),
    );
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t),
        CORPS,
        |q| {
            // Robe de koï : blanc nacré à grandes taches orangées, écailles qui brillent.
            let tache =
                ((q.x as f32 * 0.25).sin() + (q.y as f32 * 0.4 + q.x as f32 * 0.1).cos()) > 0.6;
            let r = if tache { &p.r[0] } else { &p.r[1] };
            let ecaille = (q.x + (q.y / 2) * 2) % 4 == 0 && q.y % 2 == 0;
            opaque(ton(r, lum(q.v) + if ecaille { 0.2 } else { 0.0 }, q.x, q.y))
        },
    );
    // Barbillons.
    d.trait_(
        (x1 - 1.0, c(1.0) + 1.0),
        (x1 + 0.4, c(1.0) + 3.0),
        [0xf8, 0xe0, 0xc0, 220],
        NAGEOIRE,
    );
    d.oeil(x0 + lg * 0.86, c(0.86) - hb * 0.25, 1, p.c[2]);
    d.queue = ((x0 - l * 0.2) as i32, c(0.0) as i32);
    d.bouche = (x1 as i32, c(1.0) as i32);
}

fn kraken(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let (cx, cy) = (l * 0.5, h * 0.8);
    let bras = |d: &mut Dessin, i: usize, fond: bool| {
        let u = -1.0 + 2.0 * i as f32 / 7.0;
        let depart = (cx + u * l * 0.12, cy - h * 0.12 + u.abs() * h * 0.05);
        let ang = -PI / 2.0 + u * 1.05 + 0.08 * (ph + i as f32).sin();
        let long = h * (0.9 - u.abs() * 0.25) * if fond { 0.85 } else { 1.0 };
        let enroule = if i.is_multiple_of(2) { 3.5 } else { -3.5 };
        let pts = tentacule(
            depart,
            ang,
            long,
            if fond { 8.0 } else { 9.5 },
            1.2,
            2.4,
            ph + i as f32 * 1.3,
            enroule,
        );
        d.tube(&pts, CORPS, |q| {
            // Ventouses en rangées sur la face intérieure.
            let cote = if u > 0.0 { -q.v } else { q.v };
            if cote > 0.3 && ((q.t * 70.0) as i32) % 3 == 0 && q.t < 0.92 {
                return opaque(ton(&p.r[1], 0.65 - (cote - 0.3), q.x, q.y));
            }
            let lu = 0.62 - q.v.abs() * 0.3 - if fond { 0.25 } else { 0.0 } + q.v * 0.1;
            opaque(ton(&p.r[0], lu, q.x, q.y))
        });
    };
    // Bras du fond, plus sombres.
    for i in [1, 3, 5, 6] {
        bras(d, i, true);
    }
    // Manteau énorme, à moitié sous l'écran.
    d.ellipse(cx, cy + h * 0.05, l * 0.2, h * 0.27, 0.0, CORPS, |q| {
        let tache = hache(q.x / 3, q.y / 3, 77) < 0.22;
        let lu = lum(q.v) * 0.9 + 0.08 - if tache { 0.2 } else { 0.0 };
        opaque(ton(&p.r[0], lu, q.x, q.y))
    });
    for i in [0, 2, 4, 7] {
        bras(d, i, false);
    }
    // Deux yeux d'or, pupille en fente horizontale.
    for s in [-1.0f32, 1.0] {
        let (ex, ey) = (cx + s * l * 0.1, cy - h * 0.06);
        d.ellipse(ex, ey, 7.5, 6.5, 0.0, OEIL, |q| {
            if q.v.abs() < 0.22 && q.t.abs() < 0.75 {
                return opaque([10, 6, 12]);
            }
            opaque(ton(&p.r[3], 0.75 - q.v * 0.35, q.x, q.y))
        });
        d.mettre(ex as i32 - 3, ey as i32 - 3, [255, 255, 240, 255], OEIL);
        // Paupière épaisse.
        d.ellipse(ex, ey - 6.0, 8.5, 2.5, 0.0, CORPS, |q| {
            opaque(ton(&p.r[2], 0.6 - q.v * 0.2, q.x, q.y))
        });
    }
    d.queue = (cx as i32, (h * 0.95) as i32);
    d.bouche = (cx as i32, (h * 0.6) as i32);
}

fn cthulhu(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let cx = l * 0.5;
    // Ailes membraneuses de chauve-souris, déployées derrière, qui battent lentement.
    let bat = 0.06 * ph.sin();
    for s in [-1.0f32, 1.0] {
        let epaule = (cx + s * l * 0.08, h * 0.38);
        let doigts: Vec<(f32, f32)> = (0..4)
            .map(|i| {
                let a = -PI / 2.0 + s * (0.55 + i as f32 * 0.32 + bat * 4.0);
                let long = l * (0.46 - i as f32 * 0.05);
                (
                    epaule.0 + a.cos() * long * 0.98,
                    epaule.1 + a.sin() * long * 0.75 + h * 0.05,
                )
            })
            .collect();
        let mut contour = vec![epaule];
        contour.push((epaule.0 + s * l * 0.1, h * 0.12 - bat * h));
        for (i, &dp) in doigts.iter().enumerate() {
            contour.push(dp);
            // Feston de la membrane entre deux doigts.
            if i + 1 < doigts.len() {
                let n = doigts[i + 1];
                contour.push((
                    (dp.0 + n.0) / 2.0 - s * l * 0.015,
                    (dp.1 + n.1) / 2.0 - h * 0.05,
                ));
            }
        }
        contour.push((cx + s * l * 0.1, h * 0.62));
        d.poly(&contour, NAGEOIRE, |q| {
            let x = (q.x as f32 + 0.5 - epaule.0).abs() / (l * 0.5);
            opaque(ton(&p.r[1], 0.55 - x * 0.35 + 0.1 * -q.v, q.x, q.y))
        });
        for &dp in &doigts {
            d.tube(
                &[(epaule.0, epaule.1, 2.2), (dp.0, dp.1, 0.6)],
                NAGEOIRE,
                |q| opaque(ton(&p.r[0], 0.4 - q.v * 0.2, q.x, q.y)),
            );
        }
    }
    // Jambes repliées et bras griffus.
    for s in [-1.0f32, 1.0] {
        let jambe = [
            (cx + s * l * 0.06, h * 0.72, 8.0),
            (cx + s * l * 0.13, h * 0.84, 6.5),
            (cx + s * l * 0.08, h * 0.97, 4.0),
        ];
        d.tube(&jambe, CORPS, |q| ecailles(&p.r[0], q, 0.5 - q.v * 0.25, 4));
        let bras = [
            (cx + s * l * 0.1, h * 0.42, 6.0),
            (cx + s * l * 0.19, h * 0.55, 5.0),
            (cx + s * l * 0.15, h * 0.68 + bat * h, 3.5),
        ];
        d.tube(&bras, CORPS, |q| ecailles(&p.r[0], q, 0.55 - q.v * 0.25, 4));
        let (gx, gy, _) = bras[2];
        for g in 0..4 {
            let a = PI / 2.0 + s * (g as f32 - 1.5) * 0.35;
            let bout = (gx + a.cos() * 7.0, gy + a.sin() * 7.0);
            d.tube(&[(gx, gy, 1.5), (bout.0, bout.1, 0.5)], CORPS, |q| {
                opaque(ton(&p.r[2], 0.7 - q.t * 0.3, q.x, q.y))
            });
        }
    }
    // Torse massif.
    d.ellipse(cx, h * 0.55, l * 0.13, h * 0.2, 0.0, CORPS, |q| {
        let cote = (q.v * 6.0).fract() < 0.18 && q.v > -0.2;
        ecailles(&p.r[0], q, lum(q.v) + if cote { -0.2 } else { 0.08 }, 4)
    });
    // Tentacules du visage, en barbe, qui ondulent.
    for i in 0..9 {
        let u = -1.0 + 2.0 * i as f32 / 8.0;
        let depart = (cx + u * l * 0.07, h * 0.27);
        let long = h * (0.3 - u.abs() * 0.1);
        let pts = tentacule(
            depart,
            PI / 2.0 + u * 0.25,
            long,
            3.8,
            0.9,
            1.8,
            ph + i as f32 * 1.1,
            u * 1.5,
        );
        d.tube(&pts, CORPS, |q| {
            if q.v > 0.4 && ((q.t * 30.0) as i32) % 2 == 0 {
                return opaque(ton(&p.r[0], 0.85, q.x, q.y));
            }
            opaque(ton(&p.r[0], 0.6 - q.v * 0.3 - q.t * 0.15, q.x, q.y))
        });
    }
    // Tête de céphalopode, crâne bombé.
    d.ellipse(cx, h * 0.17, l * 0.11, h * 0.14, 0.0, CORPS, |q| {
        let ride = ((q.v + 1.0) * 3.5).fract() < 0.15;
        opaque(ton(
            &p.r[0],
            lum(q.v) + 0.12 - if ride { 0.2 } else { 0.0 },
            q.x,
            q.y,
        ))
    });
    // Yeux luisants : deux grands et deux petits.
    for (dx, dy, r) in [
        (-0.045, 0.2, 3.0f32),
        (0.045, 0.2, 3.0),
        (-0.08, 0.15, 1.4),
        (0.08, 0.15, 1.4),
    ] {
        let (ex, ey) = (cx + dx * l, h * dy);
        d.ellipse(ex, ey, r + 0.5, r * 0.7 + 0.3, 0.0, OEIL, |q| {
            opaque(if q.v < -0.2 { p.r[3][4] } else { p.c[3] })
        });
    }
    d.queue = (cx as i32, (h * 0.95) as i32);
    d.bouche = (cx as i32, (h * 0.35) as i32);
}

/// Serpent colossal qui ondule (léviathan, Grand Abyssal, Jörmungandr).
#[allow(clippy::too_many_arguments)]
fn serpent(
    d: &mut Dessin,
    k: f32,
    (x0, x1): (f32, f32),
    cy: f32,
    amp: f32,
    hb: impl Fn(f32) -> f32,
    peint: impl Fn(&P) -> Option<Px>,
) {
    let c = move |t: f32| cy + vague(t, k, amp, 0.55);
    d.fuseau(x0, x1, c, &hb, |t| hb(t) * 0.92, CORPS, peint);
}

fn leviathan(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let (x0, x1) = (l * 0.04, l * 0.92);
    let lg = x1 - x0;
    let cy = h * 0.55;
    let amp = h * 0.16;
    let hb0 = h * 0.17;
    let hb = move |t: f32| {
        hb0 * (0.25 + 0.75 * (t / 0.3).min(1.0)) * (1.0 - ((t - 0.95) / 0.05).clamp(0.0, 1.0) * 0.3)
    };
    let c = move |t: f32| cy + vague(t, k, amp, 0.55);
    // Crête d'épines dorsales.
    for i in 0..26 {
        let t = 0.1 + i as f32 * 0.032;
        let (x, y) = (x0 + t * lg, c(t) - hb(t) + 1.0);
        let haut = h * 0.12 * (0.6 + 0.4 * (1.0 - (t - 0.6).abs()));
        d.poly(
            &[(x - 3.0, y + 1.0), (x - 3.5, y - haut), (x + 2.5, y + 1.0)],
            NAGEOIRE,
            |q| opaque(ton(&p.r[1], 0.3 + 0.4 * (1.0 - q.v) * 0.5, q.x, q.y)),
        );
    }
    // Nageoire de queue en éventail.
    let qx = x0 + 1.0;
    d.poly(
        &[
            (qx + 6.0, c(0.0) - 2.0),
            (0.5, c(0.0) - h * 0.3 + (ph).sin() * 3.0),
            (qx + 2.0, c(0.0)),
            (0.5, c(0.0) + h * 0.28 + ph.sin() * 3.0),
            (qx + 6.0, c(0.0) + 2.0),
        ],
        NAGEOIRE,
        |q| opaque(ton(&p.r[0], 0.4 + 0.3 * q.t, q.x, q.y)),
    );
    // Nageoires-ailes pectorales.
    for (t, sombre) in [(0.8f32, 0.2f32), (0.78, 0.0)] {
        let (x, y) = (x0 + t * lg, c(t) + hb(t) * 0.4);
        let a = 0.35 * (ph + t).sin();
        let pts: Vec<(f32, f32)> = [
            (4.0, -2.0),
            (-lg * 0.07, h * 0.42),
            (-lg * 0.03, h * 0.38),
            (-lg * 0.05, h * 0.25),
            (2.0, 3.0),
        ]
        .iter()
        .map(|&(px, py)| tourner((x, y), (x + px, y + py), a))
        .collect();
        d.poly(&pts, NAGEOIRE, |q| {
            let nervure = (q.x - q.y / 2).rem_euclid(5) == 0;
            opaque(ton(
                &p.r[0],
                0.45 - sombre + if nervure { -0.2 } else { 0.1 },
                q.x,
                q.y,
            ))
        });
    }
    serpent(d, k, (x0, x1), cy, amp, hb, |q| {
        // Plaques ventrales dorées, écailles sombres sur le dos.
        if q.v > 0.35 {
            let plaque = q.x.rem_euclid(4) == 0;
            return opaque(ton(
                &p.r[1],
                0.62 - q.v * 0.2 - if plaque { 0.25 } else { 0.0 },
                q.x,
                q.y,
            ));
        }
        ecailles(&p.r[0], q, lum(q.v) + 0.08, 4)
    });
    // Tête de dragon marin.
    let dir = (c(1.0) - c(0.97)).atan2(lg * 0.03);
    tete_serpent(
        d,
        (x1 - 2.0, c(1.0) - 1.0),
        dir,
        h * 0.13,
        p,
        0.4 + 0.4 * (ph * 2.0).sin().max(0.0),
    );
    d.queue = (2, c(0.0) as i32);
    d.bouche = ((x1 + h * 0.3) as i32, c(1.0) as i32);
}

fn grand_abyssal(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let (x0, x1) = (l * 0.03, l * 0.86);
    let lg = x1 - x0;
    let cy = h * 0.55;
    let amp = h * 0.14;
    let hb0 = h * 0.16;
    let hb = move |t: f32| {
        hb0 * (0.3 + 0.7 * (t / 0.25).min(1.0)) * (1.0 + 0.5 * ((t - 0.85) / 0.15).clamp(0.0, 1.0))
    };
    let c = move |t: f32| cy + vague(t, k, amp, 0.6);
    // Seule nageoire : une petite caudale ronde.
    let q = caudale(
        Caudale::Ronde,
        x0 + 1.0,
        c(0.0),
        l * 0.04,
        h * 0.14,
        ph.sin(),
    );
    d.poly(&q, NAGEOIRE, |q| {
        opaque(ton(&p.r[0], 0.35 + 0.2 * q.t, q.x, q.y))
    });
    serpent(d, k, (x0, x1), cy, amp, hb, |q| {
        // Peau lisse, anneaux à peine marqués, ventre pâle.
        let anneau = ((q.t * 46.0).fract() < 0.08) as i32 as f32;
        if q.v > 0.5 {
            return opaque(ton(&p.r[1], 0.55 - q.v * 0.15 - anneau * 0.15, q.x, q.y));
        }
        opaque(ton(&p.r[0], lum(q.v) + 0.08 - anneau * 0.15, q.x, q.y))
    });
    // Gueule béante : deux mâchoires en V, gosier sombre, dents en aiguilles.
    let ouv = 0.75 + 0.25 * (ph * 0.5).sin();
    let (hx, hy) = (x1 - 2.0, c(1.0));
    let r = hb(1.0);
    let haut = [
        (hx - r * 1.2, hy - r * 0.9),
        (hx + lg * 0.07, hy - r * 0.5 - ouv * r * 0.9),
        (hx + lg * 0.075, hy - r * 0.3 - ouv * r * 0.8),
        (hx, hy - r * 0.1),
    ];
    let bas = [
        (hx - r * 1.2, hy + r * 0.9),
        (hx, hy + r * 0.25),
        (hx + lg * 0.07, hy + r * 0.3 + ouv * r * 0.9),
        (hx + lg * 0.06, hy + r * 0.6 + ouv * r * 0.9),
    ];
    let gosier = [
        (hx - r * 0.6, hy),
        (hx + lg * 0.072, hy - r * 0.35 - ouv * r * 0.8),
        (hx + lg * 0.068, hy + r * 0.35 + ouv * r * 0.9),
    ];
    d.poly(&gosier, CORPS, |q| {
        opaque(ton(&p.r[2], 0.2 + q.t * 0.4, q.x, q.y))
    });
    d.poly(&haut, CORPS, |q| {
        opaque(ton(&p.r[0], 0.6 - q.v * 0.2, q.x, q.y))
    });
    d.poly(&bas, CORPS, |q| {
        opaque(ton(&p.r[1], 0.5 - q.v * 0.2, q.x, q.y))
    });
    // Dents.
    for i in 0..9 {
        let u = i as f32 / 8.0;
        let xt = hx + u * lg * 0.068;
        let yh = hy - r * 0.15 - ouv * r * 0.8 * u - r * 0.25 * u;
        let yb = hy + r * 0.3 + ouv * r * 0.9 * u;
        d.trait_(
            (xt, yh),
            (xt + 0.5, yh + 3.0 + (i % 2) as f32),
            opaque(p.c[3]).unwrap(),
            OEIL,
        );
        d.trait_(
            (xt, yb),
            (xt + 0.5, yb - 3.0 - (i % 2) as f32),
            opaque(p.c[3]).unwrap(),
            OEIL,
        );
    }
    // Petit œil laiteux.
    d.ellipse(hx - r * 0.4, hy - r * 0.75, 1.8, 1.4, 0.0, OEIL, |q| {
        opaque(if q.t < 0.0 && q.v < 0.0 {
            [255, 255, 255]
        } else {
            [0xb8, 0xc8, 0xd0]
        })
    });
    d.queue = (2, c(0.0) as i32);
    d.bouche = ((hx + lg * 0.07) as i32, hy as i32);
}

fn bloop(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let cy = h * 0.46;
    let (x0, x1) = (l * 0.02, l * 0.98);
    let lg = x1 - x0;
    // Corps en goutte : avant bulbeux, longue queue effilée.
    let c = move |t: f32| cy + h * 0.05 * (t * TAU * 0.8 - ph).sin() * (1.0 - t);
    let pr = |t: f32| {
        let avant = ((t - 0.15) / 0.6).clamp(0.0, 1.0);
        let a = 0.04 + 0.96 * avant.powf(0.9);
        a * (1.0 - ((t - 0.88) / 0.12).clamp(0.0, 1.0).powf(1.5) * 0.8)
    };
    let hb = h * 0.34;
    // Filaments pendants sous le corps.
    for i in 0..10 {
        let t = 0.4 + i as f32 * 0.05;
        let (x, y) = (x0 + t * lg, c(t) + hb * pr(t) * 0.9);
        let long = h * (0.18 + 0.12 * hache(i, 0, 5));
        let pts = tentacule(
            (x, y),
            PI / 2.0 + 0.4,
            long,
            0.6,
            0.45,
            1.0,
            ph + i as f32,
            0.0,
        );
        d.tube(&pts, NAGEOIRE, |q| {
            alpha(ton(&p.r[1], 0.7, q.x, q.y), (200.0 - q.t * 120.0) as u8)
        });
    }
    // Nageoire de queue en voile.
    let q = caudale(
        Caudale::Voile,
        x0 + lg * 0.04,
        c(0.0),
        lg * 0.08,
        h * 0.2,
        ph.sin(),
    );
    d.poly(&q, NAGEOIRE, |q| {
        alpha(ton(&p.r[1], 0.5 + 0.2 * q.t, q.x, q.y), 200)
    });
    // Jupe de nageoire ondulante tout le long (haut et bas).
    for s in [-1.0f32, 1.0] {
        d.fuseau(
            x0 + lg * 0.3,
            x0 + lg * 0.86,
            |t| c(t) + s * hb * pr(t) * 0.92,
            |t| {
                if s < 0.0 {
                    3.0 + 2.5 * (t * 40.0 - ph * 2.0).sin()
                } else {
                    0.5
                }
            },
            |t| {
                if s > 0.0 {
                    3.0 + 2.5 * (t * 40.0 - ph * 2.0).sin()
                } else {
                    0.5
                }
            },
            NAGEOIRE,
            |q| alpha(ton(&p.r[1], 0.6, q.x, q.y), 190),
        );
    }
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t),
        CORPS,
        |q| {
            // Peau pâle translucide, organes devinés, points bioluminescents en rangées.
            let rangee = (q.v + 0.55).abs() < 0.08 || (q.v - 0.35).abs() < 0.08;
            if rangee && q.x % 5 == 0 && q.t > 0.2 {
                return opaque(p.r[2][4]);
            }
            let organe = {
                let (dt, dv) = ((q.t - 0.62) / 0.12, (q.v - 0.05) / 0.35);
                dt * dt + dv * dv < 1.0
            };
            let lu = lum(q.v) * 0.75 + 0.2 - if organe { 0.15 } else { 0.0 };
            alpha(ton(&p.r[0], lu, q.x, q.y), 230)
        },
    );
    // Rangée de petits yeux noirs et longue fente de bouche.
    for i in 0..5 {
        let t = 0.86 + i as f32 * 0.022;
        let (x, y) = (x0 + t * lg, c(t) - hb * pr(t) * 0.25);
        d.oeil(x, y, if i == 0 { 1 } else { 0 }, [0, 0, 0]);
    }
    let (bx, by) = (x0 + 0.8 * lg, c(0.8) + hb * pr(0.8) * 0.3);
    d.trait_sur((bx, by), (x1, c(1.0) + hb * 0.2), opaque(p.c[3]).unwrap());
    d.queue = (2, c(0.0) as i32);
    d.bouche = (x1 as i32, c(1.0) as i32);
}

fn lusca(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let cy = h * 0.42;
    let (x0, x1) = (l * 0.38, l - 0.5);
    let lg = x1 - x0;
    let hb = h * 0.2;
    // Moitié arrière : huit tentacules qui traînent et ondulent.
    for i in 0..8 {
        let u = -1.0 + 2.0 * i as f32 / 7.0;
        let depart = (x0 + 4.0, cy + u * hb * 0.6);
        let pts = tentacule(
            depart,
            PI + u * 0.35,
            l * (0.42 - u.abs() * 0.08),
            4.0,
            0.8,
            2.0,
            ph + i as f32 * 0.8,
            if i % 2 == 0 { 1.5 } else { -1.5 },
        );
        let fond = i % 2 == 1;
        d.tube(&pts, CORPS, |q| {
            if q.v > 0.35 && ((q.t * 40.0) as i32) % 2 == 0 {
                return opaque(ton(&p.r[2], 0.55, q.x, q.y));
            }
            let lu = 0.6 - q.v * 0.25 - if fond { 0.2 } else { 0.0 };
            opaque(ton(&p.r[1], lu, q.x, q.y))
        });
    }
    // Moitié avant : requin (dorsale, ouïes, gueule).
    let pr = |t: f32| {
        (0.75 + 0.25 * (t * 2.0).min(1.0))
            * (1.0 - ((t - 0.7) / 0.3).clamp(0.0, 1.0).powf(1.6) * 0.65)
    };
    let c = |t: f32| cy + plier(t, ph.sin(), 1.0);
    let x = |t: f32| x0 + t * lg;
    d.poly(
        &[
            (x(0.25), c(0.25) - hb + 1.0),
            (x(0.32), c(0.3) - hb - h * 0.26),
            (x(0.36), c(0.36) - hb - h * 0.25),
            (x(0.52), c(0.5) - hb * pr(0.5) + 1.0),
        ],
        NAGEOIRE,
        |q| opaque(ton(&p.r[0], 0.35 + 0.3 * q.t, q.x, q.y)),
    );
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t) * 0.85,
        CORPS,
        |q| {
            // Le gris du requin passe au rouge du poulpe vers l'arrière.
            if q.v > 0.3 {
                return opaque(ton(&p.r[2], 0.75 - q.v * 0.2, q.x, q.y));
            }
            let r = if q.t < 0.18 + hache(q.x, q.y, 3) * 0.12 {
                &p.r[1]
            } else {
                &p.r[0]
            };
            opaque(ton(r, lum(q.v) + 0.05, q.x, q.y))
        },
    );
    for i in 0..5 {
        let t = 0.68 + i as f32 * 0.03;
        d.trait_sur(
            (x(t), c(t) - hb * 0.1),
            (x(t) - 0.5, c(t) + hb * 0.45),
            opaque(p.r[0][0]).unwrap(),
        );
    }
    let pe = (x(0.62), c(0.62) + hb * 0.4);
    d.poly(
        &[
            (pe.0 + 2.0, pe.1 - 1.0),
            (pe.0 - lg * 0.18, pe.1 + h * 0.3),
            (pe.0 - lg * 0.05, pe.1 + 2.0),
        ],
        NAGEOIRE,
        |q| opaque(ton(&p.r[0], 0.4, q.x, q.y)),
    );
    // Gueule dentue et œil doré.
    let (bx, by) = (x(0.86), c(0.86) + hb * 0.4);
    for i in 0..(lg * 0.12) as i32 {
        let (xx, yy) = (bx + i as f32, by - i as f32 * 0.2);
        d.sur(xx as i32, yy as i32, [0x30, 0x10, 0x14, 255]);
        if i % 2 == 0 {
            d.sur(xx as i32, yy as i32 - 1, [0xf8, 0xf8, 0xf0, 255]);
            d.sur(xx as i32, yy as i32 + 1, [0xf8, 0xf8, 0xf0, 255]);
        }
    }
    d.oeil(x(0.9), c(0.9) - hb * 0.35, 1, p.c[3]);
    d.queue = (2, cy as i32);
    d.bouche = (x1 as i32, c(1.0) as i32);
}

fn jormungandr(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let (x0, x1) = (l * 0.01, l * 0.9);
    let lg = x1 - x0;
    let cy = h * 0.56;
    let amp = h * 0.17;
    let hb0 = h * 0.17;
    let hb = move |t: f32| hb0 * (0.15 + 0.85 * (t / 0.35).min(1.0).sqrt());
    let c = move |t: f32| cy + vague(t, k, amp, 0.5);
    // Crête translucide continue.
    d.fuseau(
        x0 + lg * 0.05,
        x1 - lg * 0.02,
        |t| c(t) - hb(t) + 0.5,
        |t| 2.0 + 2.0 * ((t * 60.0).fract() - 0.5).abs() * 2.0,
        |_| 0.5,
        NAGEOIRE,
        |q| alpha(ton(&p.r[0], 0.7, q.x, q.y), 200),
    );
    serpent(d, k, (x0, x1), cy, amp, hb, |q| {
        // Anneaux de runes dorées.
        let pos = q.t * 14.0;
        let anneau = pos.fract() < 0.07;
        if anneau && q.t > 0.05 && q.t < 0.95 {
            let rune = (q.y + (pos as i32) * 3).rem_euclid(4) < 2;
            return opaque(ton(&p.r[1], if rune { 0.9 } else { 0.55 }, q.x, q.y));
        }
        if q.v > 0.45 {
            return opaque(ton(&p.r[1], 0.45 - q.v * 0.15, q.x, q.y));
        }
        ecailles(&p.r[0], q, lum(q.v) + 0.08, 4)
    });
    // Tête cornue aux yeux d'or.
    let dir = (c(1.0) - c(0.97)).atan2(lg * 0.03);
    let o = (x1 - 3.0, c(1.0) - 1.0);
    let r = h * 0.14;
    tete_serpent(d, o, dir, r, p, 0.3 + 0.3 * (ph * 2.0).sin().max(0.0));
    for s in [0.0f32, 1.0] {
        let base = tourner(o, (o.0 - r * (0.2 + s * 0.6), o.1 - r * 0.9), dir);
        let pts = tentacule(base, PI * 1.2 + dir, r * 2.2, 2.2, 0.5, 0.0, 0.0, 2.5);
        d.tube(&pts, CORPS, |q| {
            opaque(ton(&p.r[1], 0.7 - q.t * 0.3, q.x, q.y))
        });
    }
    d.queue = (2, c(0.0) as i32);
    d.bouche = ((o.0 + r * 2.6) as i32, o.1 as i32);
}

fn oeil(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let (cx, cy) = (l * 0.5, h * 0.5);
    // Tentacules tout autour, certains terminés par un petit œil.
    for i in 0..11 {
        let a = i as f32 / 11.0 * TAU + 0.2;
        let depart = (cx + a.cos() * l * 0.26, cy + a.sin() * h * 0.24);
        let pts = tentacule(
            depart,
            a,
            l * 0.26,
            5.0,
            1.0,
            2.2,
            ph + i as f32 * 0.9,
            if i % 2 == 0 { 2.0 } else { -2.0 },
        );
        d.tube(&pts, CORPS, |q| {
            let pli = ((q.t * 20.0).fract() < 0.2) as i32 as f32;
            opaque(ton(&p.r[0], 0.55 - q.v * 0.25 - pli * 0.2, q.x, q.y))
        });
        if i % 3 == 0 {
            let (x, y, _) = *pts.last().unwrap();
            d.ellipse(x, y, 2.3, 2.3, 0.0, OEIL, |q| {
                opaque(if q.t * q.t + q.v * q.v < 0.3 {
                    [10, 6, 12]
                } else {
                    p.c[2]
                })
            });
        }
    }
    // Masse de chair autour de l'œil.
    d.ellipse(cx, cy, l * 0.32, h * 0.3, 0.0, CORPS, |q| {
        let pli = ((q.t * q.t + q.v * q.v).sqrt() * 9.0).fract() < 0.18;
        opaque(ton(
            &p.r[0],
            lum(q.v) + 0.05 - if pli { 0.2 } else { 0.0 },
            q.x,
            q.y,
        ))
    });
    // Le globe : blanc veiné, iris immense, pupille fendue verticale.
    let r = l * 0.22;
    let regard = 0.12 * (ph * 0.5).sin();
    d.ellipse(cx, cy, r, r, 0.0, OEIL, |q| {
        let (u, w) = (q.t - regard, q.v);
        let dist = (u * u + w * w).sqrt();
        if dist < 0.58 {
            if u.abs() < 0.1 * (1.0 - (w / 0.55).powi(2)).max(0.0).sqrt() + 0.02 && dist < 0.52 {
                return opaque([8, 4, 12]);
            }
            // Iris en anneaux tramés.
            let anneau = (dist * 14.0).fract() < 0.3;
            return opaque(ton(
                &p.r[2],
                0.9 - dist * 0.9 + if anneau { 0.15 } else { 0.0 },
                q.x,
                q.y,
            ));
        }
        // Veines rouges.
        let veine = ((q.t * 7.0 + (q.v * 5.0).sin() * 0.8).sin().abs() < 0.06) && dist > 0.65;
        if veine {
            return opaque(p.c[3]);
        }
        opaque(ton(&p.r[1], 0.75 - q.v * 0.35 - dist * 0.2, q.x, q.y))
    });
    d.ellipse(
        cx - r * 0.35 + regard * r,
        cy - r * 0.35,
        r * 0.1,
        r * 0.07,
        -0.5,
        OEIL,
        |_| opaque([255, 255, 255]),
    );
    // Clignement : les paupières se ferment sur deux images.
    let ferme = match (k * 8.0).round() as i32 {
        5 => 0.55,
        6 => 1.0,
        _ => 0.0,
    };
    if ferme > 0.0 {
        for s in [-1.0f32, 1.0] {
            d.ellipse(cx, cy, r * 1.05, r * 1.05, 0.0, CORPS, |q| {
                let bord = if s < 0.0 {
                    -1.0 + ferme * 1.05
                } else {
                    1.0 - ferme * 1.05
                };
                let dedans = if s < 0.0 { q.v < bord } else { q.v > bord };
                dedans.then(|| {
                    let c: Rgb = ton(&p.r[0], 0.6 - q.v * 0.2, q.x, q.y);
                    [c[0], c[1], c[2], 255]
                })
            });
        }
    }
    d.queue = (cx as i32, (cy + r) as i32);
    d.bouche = (cx as i32, (cy + r) as i32);
    let _ = hsv;
}

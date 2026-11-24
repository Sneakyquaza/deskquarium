//! Rares et épiques : méduse, poulpe, murène, anguille, rascasse volante, tortue, calamar,
//! raie manta, requin-marteau, espadon, phoque, dauphin, poisson-lune.

use super::{
    caudale, contre, esp, membrane, plier, tentacule, tourner, v, Caudale, Espece, Mouvement as M,
    Pal, Taille as T, V_POISSON, V_SOMBRE, V_VIF,
};
use crate::art::{alpha, hache, lum, opaque, ton, Dessin, Rampe, P};
use crate::jeu::Rarete::{Epique as E, Rare as R};
use crate::poisson::{CORPS, NAGEOIRE};
use crate::toile::melange;
use std::f32::consts::{PI, TAU};

pub fn especes() -> Vec<Espece> {
    vec![
        esp(
            "meduse",
            "Méduse",
            R,
            T::Moyenne,
            M::Pulse,
            (22, 34),
            6,
            [
                [0xe8, 0x9c, 0xd8],
                [0xf8, 0xc8, 0xec],
                [0xb0, 0x50, 0xa8],
                [0xff, 0xf0, 0xff],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("lune", 0.55, 0.6, 1.0),
                    v("dorée", 0.2, 0.9, 1.0),
                    v("cristal", 0.0, 0.15, 1.1),
                ]
            },
            meduse,
        ),
        esp(
            "poulpe",
            "Poulpe",
            R,
            T::Moyenne,
            M::Fond,
            (30, 26),
            8,
            [
                [0xd0, 0x4c, 0x34],
                [0xf4, 0xa0, 0x78],
                [0xf0, 0xc8, 0x40],
                [0x7c, 0x20, 0x20],
            ],
            V_VIF,
            poulpe,
        ),
        esp(
            "murene",
            "Murène",
            R,
            T::Moyenne,
            M::Ondule,
            (56, 12),
            8,
            [
                [0x88, 0xa0, 0x30],
                [0x2c, 0x38, 0x14],
                [0x50, 0x14, 0x14],
                [0xf0, 0xe0, 0x50],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("léopard", -0.12, 1.1, 1.1),
                    v("ruban", 0.4, 1.2, 1.0),
                ]
            },
            murene,
        ),
        esp(
            "anguille",
            "Anguille",
            R,
            T::Moyenne,
            M::Ondule,
            (50, 8),
            8,
            [
                [0x5c, 0x50, 0x34],
                [0xd8, 0xc0, 0x68],
                [0x30, 0x28, 0x1c],
                [0x20, 0x18, 0x10],
            ],
            V_SOMBRE,
            anguille,
        ),
        esp(
            "rascasse",
            "Rascasse volante",
            R,
            T::Moyenne,
            M::Lent,
            (30, 26),
            4,
            [
                [0xb8, 0x3c, 0x28],
                [0xf4, 0xe4, 0xd0],
                [0x4c, 0x18, 0x18],
                [0xe0, 0x84, 0x64],
            ],
            V_POISSON,
            rascasse,
        ),
        esp(
            "tortue",
            "Tortue de mer",
            R,
            T::Moyenne,
            M::Plane,
            (38, 20),
            6,
            [
                [0x7c, 0x5c, 0x30],
                [0xb8, 0x94, 0x50],
                [0x84, 0x90, 0x5c],
                [0xe8, 0xd8, 0x9c],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("verte", 0.15, 0.9, 0.95),
                    v("imbriquée", -0.03, 1.2, 1.1),
                ]
            },
            tortue,
        ),
        esp(
            "calamar",
            "Calamar",
            R,
            T::Moyenne,
            M::Jet,
            (34, 12),
            6,
            [
                [0xec, 0xb4, 0xa4],
                [0xa8, 0x38, 0x30],
                [0xf8, 0xdc, 0xd4],
                [0x30, 0x60, 0x80],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("luciole", 0.5, 0.7, 1.0),
                    v("rouge", 0.0, 1.8, 0.85),
                ]
            },
            calamar,
        ),
        esp(
            "manta",
            "Raie manta",
            E,
            T::Grande,
            M::Plane,
            (60, 46),
            6,
            [
                [0x2c, 0x34, 0x44],
                [0xe4, 0xe8, 0xf0],
                [0x18, 0x1c, 0x28],
                [0x90, 0x9c, 0xb0],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("noire", 0.0, 0.6, 0.6),
                    v("des récifs", 0.45, 2.0, 1.4),
                ]
            },
            manta,
        ),
        esp(
            "marteau",
            "Requin-marteau",
            E,
            T::Grande,
            M::Nage,
            (72, 28),
            4,
            [
                [0x80, 0x88, 0x80],
                [0xec, 0xec, 0xe4],
                [0x50, 0x56, 0x58],
                [0x30, 0x24, 0x28],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("bronze", -0.1, 2.0, 1.0),
                    v("albinos", 0.0, 0.1, 1.35),
                ]
            },
            marteau,
        ),
        esp(
            "espadon",
            "Espadon",
            E,
            T::Grande,
            M::Nage,
            (80, 24),
            4,
            [
                [0x34, 0x38, 0x68],
                [0xd4, 0xdc, 0xe4],
                [0x48, 0x4c, 0x58],
                [0x80, 0x88, 0xb8],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("pourpre", 0.12, 1.2, 1.0),
                    v("d'acier", 0.0, 0.3, 1.2),
                ]
            },
            espadon,
        ),
        esp(
            "phoque",
            "Phoque",
            E,
            T::Grande,
            M::Fonce,
            (52, 20),
            6,
            [
                [0x8c, 0x90, 0x98],
                [0xe0, 0xdc, 0xcc],
                [0x40, 0x40, 0x48],
                [0x14, 0x10, 0x18],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("moine", 0.05, 1.5, 0.75),
                    v("annelé", 0.0, 0.2, 1.15),
                ]
            },
            phoque,
        ),
        esp(
            "dauphin",
            "Dauphin",
            E,
            T::Grande,
            M::Fonce,
            (62, 24),
            6,
            [
                [0x6c, 0x7c, 0x90],
                [0xe0, 0xe4, 0xe8],
                [0x40, 0x48, 0x58],
                [0x24, 0x20, 0x28],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("rose", -0.62, 1.6, 1.2),
                    v("tacheté", 0.0, 0.9, 0.85),
                ]
            },
            dauphin,
        ),
        esp(
            "mola",
            "Poisson-lune",
            E,
            T::Grande,
            M::Lent,
            (40, 52),
            4,
            [
                [0x98, 0xa4, 0xb0],
                [0xd8, 0xe0, 0xe4],
                [0x58, 0x64, 0x74],
                [0xf0, 0xf0, 0xf0],
            ],
            V_POISSON,
            mola,
        ),
    ]
}

/// Cloche translucide d'une méduse : `rx`, `ry`, centre ; `cote` = rampe.
#[allow(clippy::too_many_arguments)]
pub fn cloche(d: &mut Dessin, cx: f32, cy: f32, rx: f32, ry: f32, r: &Rampe, bord: &Rampe, a: u8) {
    d.ellipse(cx, cy, rx, ry, 0.0, CORPS, |q| {
        if q.v > 0.32 {
            return None;
        }
        // Bord festonné.
        let feston = (q.t * 7.0 * PI).cos().abs();
        if q.v > 0.18 && feston < 0.35 {
            return None;
        }
        let rim = q.t * q.t + q.v * q.v > 0.72;
        if rim {
            return alpha(ton(bord, 0.85, q.x, q.y), 235);
        }
        alpha(ton(r, 0.75 - q.v * 0.4 - q.t.abs() * 0.2, q.x, q.y), a)
    });
}

fn meduse(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let c = (k * TAU).sin().max(0.0);
    let cx = l * 0.5;
    let (rx, ry) = (l * 0.46 * (1.0 - 0.16 * c), h * 0.33 * (1.0 + 0.12 * c));
    let cy = ry + 0.5;
    let bas = cy + ry * 0.25;
    // Tentacules fins depuis le bord.
    for i in 0..7 {
        let u = -0.85 + 1.7 * i as f32 / 6.0;
        let x0 = cx + u * rx * 0.95;
        let long = h - bas - 1.5 - (i % 2) as f32 * 3.0;
        let pts = tentacule(
            (x0, bas),
            PI / 2.0 + u * 0.15,
            long,
            0.45,
            0.45,
            0.9,
            k * TAU + i as f32,
            0.0,
        );
        d.tube(&pts, NAGEOIRE, |q| {
            alpha(ton(&p.r[0], 0.7, q.x, q.y), (210.0 - q.t * 90.0) as u8)
        });
    }
    // Bras buccaux en rubans plissés.
    for (i, u) in [-0.22f32, 0.0, 0.22].iter().enumerate() {
        let long = h * 0.52 - i as f32 * 1.5;
        let pts = tentacule(
            (cx + u * rx, bas - 1.0),
            PI / 2.0 + u * 0.4,
            long,
            1.6,
            0.7,
            1.4,
            k * TAU * 0.8 + i as f32 * 2.0,
            0.0,
        );
        d.tube(&pts, NAGEOIRE, |q| {
            let pli = ((q.t * 14.0).fract() < 0.3) as i32 as f32;
            alpha(ton(&p.r[1], 0.55 + q.v * 0.2 - pli * 0.25, q.x, q.y), 225)
        });
    }
    cloche(d, cx, cy, rx, ry, &p.r[0], &p.r[3], 200);
    // Gonades : quatre anneaux visibles par transparence.
    for (dx, dy) in [(-0.3, -0.05), (0.3, -0.05), (-0.12, -0.35), (0.12, -0.35)] {
        d.ellipse(
            cx + dx * rx,
            cy + dy * ry,
            rx * 0.16,
            ry * 0.18,
            0.0,
            CORPS,
            |q| (q.t * q.t + q.v * q.v > 0.3).then(|| [p.c[2][0], p.c[2][1], p.c[2][2], 170]),
        );
    }
    d.queue = (cx as i32, h as i32 - 2);
    d.bouche = (cx as i32, bas as i32);
}

fn poulpe(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let (cx, cy) = (l * 0.38, h * 0.32);
    // Bras : six visibles, qui ondulent en éventail vers l'arrière et le bas.
    let bras = [
        (0.15f32, 1.0f32),
        (0.75, 0.95),
        (1.3, 0.9),
        (1.9, 0.95),
        (2.5, 0.85),
        (2.95, 0.8),
    ];
    for (i, &(ang, kl)) in bras.iter().enumerate() {
        let depart = (cx + l * 0.1 + ang.cos() * 2.0, cy + h * 0.25);
        let a = ang + 0.15 * (ph + i as f32).sin();
        let pts = tentacule(
            depart,
            a,
            l * 0.55 * kl,
            2.2,
            0.45,
            2.2,
            ph + i as f32 * 0.9,
            if i % 2 == 0 { 2.5 } else { -2.0 },
        );
        let dos = i >= 3;
        d.tube(&pts, CORPS, |q| {
            // Ventouses claires sur le côté intérieur.
            if q.v > 0.35 && ((q.t * 30.0) as i32) % 2 == 0 {
                return opaque(ton(&p.r[1], 0.7, q.x, q.y));
            }
            let lu = 0.62 - q.v * 0.3 - if dos { 0.15 } else { 0.0 };
            opaque(ton(&p.r[0], lu, q.x, q.y))
        });
    }
    // Manteau en sac, incliné vers l'arrière.
    let gonfle = 1.0 + 0.06 * ph.sin();
    d.ellipse(cx, cy, l * 0.24 * gonfle, h * 0.27, -0.55, CORPS, |q| {
        let mut lu = 0.68 - q.v * 0.35 + q.t * 0.1;
        if hache(q.x / 2, q.y / 2, 7) < 0.2 {
            lu -= 0.3;
        }
        if hache(q.x, q.y, 2) < 0.08 {
            return opaque(ton(&p.r[1], 0.8, q.x, q.y));
        }
        opaque(ton(&p.r[0], lu, q.x, q.y))
    });
    // Tête et œil doré à pupille horizontale.
    d.ellipse(
        cx + l * 0.16,
        cy + h * 0.2,
        l * 0.13,
        h * 0.14,
        0.0,
        CORPS,
        volume(&p.r[0]),
    );
    let (ex, ey) = (cx + l * 0.2, cy + h * 0.14);
    d.oeil(ex, ey, 2, p.c[2]);
    d.mettre(
        ex as i32 - 1,
        ey as i32,
        [10, 10, 16, 255],
        crate::poisson::OEIL,
    );
    d.mettre(
        ex as i32 + 1,
        ey as i32,
        [10, 10, 16, 255],
        crate::poisson::OEIL,
    );
    d.queue = ((cx - l * 0.2) as i32, (cy - h * 0.15) as i32);
    d.bouche = ((cx + l * 0.2) as i32, (cy + h * 0.3) as i32);
}

fn volume(r: &Rampe) -> impl Fn(&P) -> Option<crate::art::Px> + '_ {
    super::volume(r)
}

/// Corps de serpent/anguille qui ondule : vague qui court de la tête vers la queue.
pub fn vague(t: f32, k: f32, amp: f32, longueur_onde: f32) -> f32 {
    amp * (t * TAU / longueur_onde - k * TAU).sin() * (0.35 + 0.65 * (1.0 - t))
}

fn murene(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let cy = h * 0.55;
    let (x0, x1) = (0.5, l - 0.5);
    let hb = h * 0.24;
    let c = |t: f32| cy + vague(t, k, h * 0.18, 0.9);
    let pr = |t: f32| (t * 8.0).min(1.0) * (1.0 + 0.35 * ((t - 0.75) / 0.25).clamp(0.0, 1.0));
    let ouvre = 0.5 + 0.5 * (k * TAU * 2.0).sin();
    // Nageoire dorsale continue.
    d.fuseau(
        x0,
        x1 - l * 0.1,
        |t| c(t) - hb * pr(t) - 0.6,
        |t| (1.2 + 0.4 * t) * (t * 10.0).min(1.0),
        |_| 0.6,
        NAGEOIRE,
        |q| opaque(ton(&p.r[0], 0.55, q.x, q.y)),
    );
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t) * 0.95,
        CORPS,
        |q| {
            // Gueule ouverte : intérieur sombre et dents.
            let tete = q.t > 0.92;
            if tete && q.v > -0.05 - ouvre * 0.15 && q.v < 0.3 + ouvre * 0.2 && q.t > 0.95 {
                let dent = (q.v - 0.3 - ouvre * 0.2).abs() < 0.2
                    || (q.v + 0.05 + ouvre * 0.15).abs() < 0.2;
                if dent && q.x % 2 == 0 {
                    return opaque([0xf4, 0xf0, 0xe0]);
                }
                return opaque(ton(&p.r[2], 0.3, q.x, q.y));
            }
            // Marbrures réticulées.
            let cell = hache(q.x / 3, q.y / 2 + (q.x / 3) % 2, 13);
            let lu = lum(q.v) + if cell < 0.35 { -0.35 } else { 0.0 };
            if cell < 0.35 && q.v < 0.6 {
                return opaque(ton(&p.r[1], lu + 0.4, q.x, q.y));
            }
            opaque(ton(&p.r[0], lu, q.x, q.y))
        },
    );
    let ex = x1 - l * 0.06;
    d.oeil(ex, c(0.94) - hb * 0.75, 1, p.c[3]);
    d.mettre(
        ex as i32,
        (c(0.94) - hb * 0.75) as i32,
        opaque(p.c[3]).unwrap(),
        crate::poisson::OEIL,
    );
    d.queue = (1, c(0.0) as i32);
    d.bouche = (x1 as i32, c(1.0) as i32);
}

fn anguille(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let cy = h * 0.5;
    let (x0, x1) = (0.5, l - 0.5);
    let hb = h * 0.24;
    let c = |t: f32| cy + vague(t, k, h * 0.2, 0.7);
    let pr = |t: f32| (t * 6.0).min(1.0) * (1.0 - ((t - 0.9) / 0.1).clamp(0.0, 1.0) * 0.45);
    // Nageoires dorsale et anale continues jusqu'à la queue pointue.
    for s in [-1.0f32, 1.0] {
        d.fuseau(
            x0,
            x1 - l * 0.35,
            |t| c(t) + s * (hb * pr(t) + 0.4),
            |t| {
                if s < 0.0 {
                    0.9 * (t * 5.0).min(1.0)
                } else {
                    0.3
                }
            },
            |t| {
                if s > 0.0 {
                    0.9 * (t * 5.0).min(1.0)
                } else {
                    0.3
                }
            },
            NAGEOIRE,
            |q| alpha(ton(&p.r[2], 0.6, q.x, q.y), 220),
        );
    }
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t),
        CORPS,
        contre(&p.r[0], &p.r[1], 0.3),
    );
    let pe = (x1 - l * 0.12, c(0.88) + 0.5);
    d.poly(
        &[
            (pe.0, pe.1),
            (pe.0 - 2.0, pe.1 + 1.5),
            (pe.0 - 0.5, pe.1 + 2.0),
        ],
        NAGEOIRE,
        membrane(&p.r[1], 220),
    );
    d.oeil(x1 - l * 0.04, c(0.96) - 0.5, 0, [0, 0, 0]);
    d.queue = (1, c(0.0) as i32);
    d.bouche = (x1 as i32, c(1.0) as i32);
}

fn rascasse(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let b = (k * TAU).sin();
    let cy = h * 0.45;
    let (x0, x1) = (l * 0.28, l - 0.5);
    let lg = x1 - x0;
    let hb = h * 0.17;
    let c = |t: f32| cy + plier(t, b, 0.6);
    let pr = |t: f32| (PI * (0.1 + 0.82 * t)).sin().max(0.0).powf(0.6);
    let x = |t: f32| x0 + t * lg;
    let rayure = |q: &P, base: f32| {
        let r = if ((q.x as f32 + q.y as f32 * 0.3) as i32 / 2) % 2 == 0 {
            &p.r[0]
        } else {
            &p.r[1]
        };
        opaque(ton(r, base, q.x, q.y))
    };
    // Grandes pectorales en éventail, faites d'épines rayées et de membrane.
    let pe = (x(0.62), c(0.62) + hb * 0.3);
    for i in 0..8 {
        let a = PI * 0.62 + i as f32 * 0.13 + b * 0.05;
        let long = l * (0.42 + 0.06 * (i % 2) as f32);
        let bout = (pe.0 + a.cos() * long, pe.1 + a.sin() * long * 0.8);
        let pts = [(pe.0, pe.1, 0.8), (bout.0, bout.1, 0.4)];
        d.tube(&pts, NAGEOIRE, |q| {
            let bande = ((q.t * 6.0) as i32) % 2 == 0;
            let r = if bande { &p.r[3] } else { &p.r[1] };
            alpha(ton(r, 0.55, q.x, q.y), 230)
        });
    }
    // Épines dorsales venimeuses.
    for i in 0..9 {
        let t = 0.2 + i as f32 * 0.065;
        let base = (x(t), c(t) - hb * pr(t));
        let long = h * (0.42 - (i as f32 - 2.0).abs() * 0.025);
        let bout = (base.0 - long * 0.35, base.1 - long);
        d.tube(
            &[(base.0, base.1, 0.7), (bout.0, bout.1, 0.35)],
            NAGEOIRE,
            |q| {
                let bande = ((q.t * 5.0) as i32) % 2 == 0;
                alpha(ton(&p.r[if bande { 0 } else { 1 }], 0.55, q.x, q.y), 240)
            },
        );
    }
    let q = caudale(Caudale::Ronde, x0 + 0.5, c(0.0), l * 0.2, h * 0.16, b);
    d.poly(&q, NAGEOIRE, |q| {
        let point = hache(q.x, q.y, 3) < 0.25;
        alpha(ton(&p.r[if point { 2 } else { 3 }], 0.6, q.x, q.y), 210)
    });
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t),
        CORPS,
        |q| rayure(q, lum(q.v)),
    );
    // Tentacules au-dessus de l'œil.
    let ex = x(0.84);
    d.trait_(
        (ex, c(0.84) - hb * 0.9),
        (ex + 1.0, c(0.84) - hb * 0.9 - 3.0),
        opaque(p.c[1]).unwrap(),
        NAGEOIRE,
    );
    d.oeil(ex, c(0.84) - hb * 0.3, 1, p.c[0]);
    d.queue = (x0 as i32 - 3, c(0.0) as i32);
    d.bouche = (x1 as i32, c(1.0) as i32);
}

fn tortue(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let (cx, cy) = (l * 0.46, h * 0.5);
    let (rx, ry) = (l * 0.3, h * 0.36);
    // Nageoire avant (au loin) : derrière la carapace.
    let nageoire = |d: &mut Dessin, ang: f32, long: f32, sombre: f32| {
        let o = (cx + rx * 0.5, cy + ry * 0.35);
        let pts: Vec<(f32, f32)> = [
            (0.0, -1.6),
            (long * 0.5, -1.4),
            (long, 0.0),
            (long * 0.45, 1.8),
            (0.0, 1.8),
        ]
        .iter()
        .map(|&(x, y)| tourner(o, (o.0 - x, o.1 + y), ang))
        .collect();
        d.poly(&pts, NAGEOIRE, |q| {
            let ecaille = hache(q.x, q.y, 5) < 0.15;
            opaque(ton(
                &p.r[2],
                0.5 - sombre + if ecaille { 0.25 } else { 0.0 },
                q.x,
                q.y,
            ))
        });
    };
    nageoire(d, -0.9 + 0.7 * (ph + 0.6).sin(), l * 0.36, 0.2);
    // Patte arrière.
    let pa = (cx - rx * 0.8, cy + ry * 0.4);
    d.poly(
        &[
            (pa.0 + 2.0, pa.1 - 1.0),
            (pa.0 - 4.0, pa.1 + 1.5 + ph.sin()),
            (pa.0 + 1.0, pa.1 + 2.0),
        ],
        NAGEOIRE,
        |q| opaque(ton(&p.r[2], 0.45, q.x, q.y)),
    );
    // Tête et cou.
    let tx = cx + rx + l * 0.1;
    d.ellipse(tx, cy + ry * 0.05, l * 0.1, h * 0.17, 0.15, CORPS, |q| {
        let ecaille = hache(q.x, q.y, 9) < 0.18;
        opaque(ton(
            &p.r[2],
            lum(q.v) + if ecaille { 0.25 } else { 0.0 },
            q.x,
            q.y,
        ))
    });
    // Plastron clair dessous, carapace bombée à écailles.
    d.ellipse(cx, cy + ry * 0.35, rx * 0.95, ry * 0.45, 0.0, CORPS, |q| {
        opaque(ton(&p.r[3], 0.55 - q.v * 0.2, q.x, q.y))
    });
    d.ellipse(cx, cy, rx, ry, 0.0, CORPS, |q| {
        if q.v > 0.45 {
            return None;
        }
        // Écailles (plaques) : réseau de bords sombres.
        let (u, w) = ((q.t + 1.0) * 3.0, (q.v + 1.0) * 2.2);
        let decal = if (w as i32) % 2 == 0 { 0.5 } else { 0.0 };
        let bord_u = ((u + decal).fract() - 0.5).abs() > 0.38;
        let bord_v = (w.fract() - 0.5).abs() > 0.4;
        let bord = q.t * q.t + q.v * q.v > 0.8;
        let lu = lum(q.v * 0.8);
        if bord_u || bord_v || bord {
            return opaque(ton(&p.r[0], lu - 0.35, q.x, q.y));
        }
        opaque(ton(&p.r[1], lu, q.x, q.y))
    });
    // Nageoire avant proche, devant la carapace.
    nageoire(d, -0.6 + 0.8 * ph.sin(), l * 0.42, 0.0);
    d.oeil(tx + l * 0.04, cy - h * 0.04, 0, [0, 0, 0]);
    d.sur(
        (tx + l * 0.1) as i32 - 1,
        (cy + h * 0.06) as i32,
        [0x30, 0x28, 0x18, 255],
    );
    d.queue = ((cx - rx) as i32, cy as i32);
    d.bouche = ((tx + l * 0.1) as i32, cy as i32);
}

fn calamar(d: &mut Dessin, k: f32, p: &Pal) {
    calmar(d, k, p, 1, 0.36);
}

/// Calmar vu de profil, manteau vers l'avant : rayon de l'œil, longueur des tentacules (fraction).
pub fn calmar(d: &mut Dessin, k: f32, p: &Pal, oeil: i32, tentacules: f32) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let cy = h * 0.5;
    let geant = tentacules > 0.4;
    // Le manteau pointe vers l'avant (sens de la nage), les bras traînent derrière.
    let (x0, x1) = (l * if geant { 0.4 } else { 0.36 }, l - 0.5);
    let hb = h * if geant { 0.2 } else { 0.26 } * (1.0 + 0.08 * ph.sin());
    let pr = |t: f32| (t * 5.0).min(1.0).sqrt() * (1.0 - t).powf(0.55).min(1.0) * 1.15;
    // Nageoires en losange au bout du manteau.
    let xf = x1 - l * 0.1;
    let f = h * if geant { 0.3 } else { 0.42 } + 0.8 * ph.cos();
    d.poly(
        &[(xf - l * 0.12, cy), (xf, cy - f), (x1, cy), (xf, cy + f)],
        NAGEOIRE,
        |q| alpha(ton(&p.r[2], 0.55 + 0.2 * q.t, q.x, q.y), 215),
    );
    // Bras et deux longues tentacules à massue.
    let nb = if geant { 8 } else { 6 };
    for i in 0..nb {
        let u = -1.0 + 2.0 * i as f32 / (nb - 1) as f32;
        let pts = tentacule(
            (x0 + 1.0, cy + u * hb * 0.5),
            PI + u * 0.25,
            l * 0.3,
            if geant { 2.0 } else { 0.9 },
            0.4,
            0.8,
            ph + i as f32,
            0.0,
        );
        d.tube(&pts, NAGEOIRE, |q| {
            let ventouse = geant && q.v > 0.3 && ((q.t * 40.0) as i32) % 2 == 0;
            let r = if ventouse { &p.r[2] } else { &p.r[0] };
            alpha(ton(r, 0.55 - q.v * 0.15, q.x, q.y), 235)
        });
    }
    for s in [-1.0f32, 1.0] {
        let pts = tentacule(
            (x0, cy + s * 0.5),
            PI + s * 0.12,
            l * tentacules,
            if geant { 1.2 } else { 0.5 },
            if geant { 1.0 } else { 0.9 },
            0.6,
            ph * 1.3 + s,
            0.0,
        );
        let bout = *pts.last().unwrap();
        d.tube(&pts, NAGEOIRE, |q| alpha(ton(&p.r[0], 0.5, q.x, q.y), 230));
        if geant {
            d.ellipse(bout.0 + 2.0, bout.1, 4.0, 2.0, 0.0, NAGEOIRE, |q| {
                let v = ((q.x + q.y) % 2 == 0) as i32 as f32;
                alpha(ton(&p.r[2], 0.4 + v * 0.3, q.x, q.y), 235)
            });
        }
    }
    d.fuseau(
        x0,
        x1,
        |_| cy,
        |t| hb * pr(t),
        |t| hb * pr(t),
        CORPS,
        |q| {
            // Chromatophores : points qui changent de taille.
            let ch = hache(q.x, q.y, 17);
            if ch < 0.12 + 0.06 * ph.sin() {
                return alpha(ton(&p.r[1], 0.5, q.x, q.y), 240);
            }
            alpha(ton(&p.r[0], lum(q.v) + 0.1, q.x, q.y), 235)
        },
    );
    d.oeil(x0 + l * 0.04, cy - 0.5, oeil, p.c[3]);
    d.queue = (x1 as i32, cy as i32);
    d.bouche = (x0 as i32, cy as i32);
}

fn manta(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let cy = h * 0.5;
    // Vue de dessus : le battement rapproche les pointes des ailes (raccourci) et les recourbe.
    let s = 0.82 + 0.18 * ph.cos();
    let rec = 0.06 * ph.sin();
    let aile = |sg: f32| -> Vec<(f32, f32)> {
        vec![
            (l * 0.9, cy + sg * h * 0.08),
            (l * 0.72, cy + sg * h * 0.16),
            (l * (0.52 - rec), cy + sg * h * 0.5 * s),
            (l * (0.43 - rec), cy + sg * h * 0.47 * s),
            (l * 0.4, cy + sg * h * 0.3 * s),
            (l * 0.3, cy + sg * h * 0.12),
            (l * 0.2, cy + sg * h * 0.04),
            (l * 0.2, cy),
            (l * 0.9, cy),
        ]
    };
    // Queue fine.
    let queue: Vec<(f32, f32, f32)> = (0..=8)
        .map(|i| {
            let u = i as f32 / 8.0;
            (
                l * 0.22 * (1.0 - u) + 0.5,
                cy + (ph + u * 3.0).sin() * u * 1.5,
                0.8 - 0.3 * u,
            )
        })
        .collect();
    d.tube(&queue, NAGEOIRE, |q| opaque(ton(&p.r[2], 0.5, q.x, q.y)));
    for sg in [-1.0f32, 1.0] {
        d.poly(&aile(sg), CORPS, |q| {
            // Plus clair au milieu du dos (bosse), sombre vers le bout des ailes ;
            // taches blanches en chevron sur les épaules.
            let x = q.x as f32 / l;
            let y = (q.y as f32 + 0.5 - cy).abs() / h;
            let epaule =
                x > 0.5 && x < 0.7 && y > 0.1 && y < 0.24 && (x - 0.6 + y * 0.6).abs() < 0.07;
            if epaule {
                return opaque(ton(&p.r[1], 0.6, q.x, q.y));
            }
            let bord = ((x - 0.45) * 2.0).max(0.0) * (y < 0.2) as i32 as f32;
            let lu = 0.72 - y * 1.3 + bord * 0.2 - if sg > 0.0 { 0.12 } else { 0.0 };
            opaque(ton(&p.r[0], lu, q.x, q.y))
        });
    }
    // Liseré clair du bord d'attaque (ventre blanc qui dépasse).
    for sg in [-1.0f32, 1.0] {
        let a = aile(sg);
        d.trait_(
            (a[1].0 - 0.5, a[1].1),
            (a[2].0 + 0.5, a[2].1),
            [p.c[3][0], p.c[3][1], p.c[3][2], 200],
            CORPS,
        );
    }
    // Lobes céphaliques enroulés devant la bouche.
    for sg in [-1.0f32, 1.0] {
        d.ellipse(
            l * 0.93,
            cy + sg * h * 0.08,
            l * 0.06,
            h * 0.035,
            sg * 0.3,
            CORPS,
            |q| opaque(ton(&p.r[0], 0.4 - q.v * sg * 0.2, q.x, q.y)),
        );
        d.oeil(l * 0.84, cy + sg * h * 0.1 - 0.5, 0, [0, 0, 0]);
    }
    // Arête du dos.
    d.trait_(
        (l * 0.28, cy),
        (l * 0.8, cy),
        [p.c[3][0], p.c[3][1], p.c[3][2], 90],
        CORPS,
    );
    d.queue = (1, cy as i32);
    d.bouche = ((l * 0.97) as i32, cy as i32);
}

/// Corps de requin vu de profil : renvoie (centre, haut, bas) pour `t` (0 queue .. 1 museau).
pub fn profil_requin(t: f32, hb: f32) -> (f32, f32) {
    let dos = if t < 0.6 {
        0.18 + 0.82 * (t / 0.6).powf(0.8)
    } else {
        1.0 - 0.6 * ((t - 0.6) / 0.4).powf(1.6)
    };
    let ventre = if t < 0.55 {
        0.15 + 0.85 * (t / 0.55).powf(0.9)
    } else {
        1.0 - 0.8 * ((t - 0.55) / 0.45).powf(2.2)
    };
    (hb * dos.max(0.05), hb * 0.85 * ventre.max(0.05))
}

/// Requin complet (marteau, blanc, lutin…) : nageoires, corps à contre-ombrage, ouïes.
/// Renvoie la fonction de centre utilisée, les bornes du corps et la demi-hauteur.
#[allow(clippy::too_many_arguments)]
pub fn requin(
    d: &mut Dessin,
    k: f32,
    dos: &Rampe,
    ventre: &Rampe,
    bouts: &Rampe,
    (x0, x1): (f32, f32),
    cy: f32,
    hb: f32,
    aileron: f32,
    seuil: impl Fn(f32) -> f32,
) {
    let b = (k * TAU).sin();
    let lg = x1 - x0;
    let c = |t: f32| cy + plier(t, b, hb * 0.35);
    let x = |t: f32| x0 + t * lg;
    let haut = |t: f32| c(t) - profil_requin(t, hb).0;
    let bas = |t: f32| c(t) + profil_requin(t, hb).1;
    let fin = |q: &P| opaque(ton(dos, 0.4 + 0.25 * (1.0 - q.t), q.x, q.y));
    // Caudale hétérocerque : lobe du haut bien plus long.
    let (qx, qy) = (x(0.0) + 0.5, c(0.0));
    let dy = b * hb * 0.25;
    d.poly(
        &[
            (qx + 1.0, qy - hb * 0.15),
            (qx - lg * 0.12, qy - hb * 1.25 + dy),
            (qx - lg * 0.15, qy - hb * 1.3 + dy),
            (qx - lg * 0.06, qy - hb * 0.3 + dy * 0.5),
            (qx - lg * 0.09, qy + hb * 0.75 + dy),
            (qx + 1.0, qy + hb * 0.2),
        ],
        NAGEOIRE,
        |q| opaque(ton(dos, 0.35 + 0.3 * q.t - 0.15 * q.v, q.x, q.y)),
    );
    // Première dorsale en faucille, seconde dorsale et anale petites.
    let d1 = [
        (x(0.5), haut(0.5) + 1.0),
        (x(0.56), haut(0.56) - hb * aileron * 0.6),
        (x(0.58), haut(0.6) - hb * aileron),
        (x(0.6), haut(0.6) - hb * aileron * 0.95),
        (x(0.6), haut(0.62) - hb * aileron * 0.4),
        (x(0.68), haut(0.68) + 1.0),
    ];
    d.poly(&d1, NAGEOIRE, fin);
    d.poly(
        &[
            (x(0.16), haut(0.16) + 0.5),
            (x(0.17), haut(0.17) - hb * 0.35),
            (x(0.22), haut(0.22) + 0.5),
        ],
        NAGEOIRE,
        fin,
    );
    d.poly(
        &[
            (x(0.14), bas(0.14) - 0.5),
            (x(0.15), bas(0.15) + hb * 0.3),
            (x(0.21), bas(0.21) - 0.5),
        ],
        NAGEOIRE,
        fin,
    );
    // Pelvienne.
    d.poly(
        &[
            (x(0.34), bas(0.34) - 0.5),
            (x(0.32), bas(0.32) + hb * 0.35),
            (x(0.4), bas(0.4) - 0.5),
        ],
        NAGEOIRE,
        fin,
    );
    d.fuseau(
        x0,
        x1,
        c,
        |t| profil_requin(t, hb).0,
        |t| profil_requin(t, hb).1,
        CORPS,
        |q| {
            if q.v > seuil(q.t) {
                opaque(ton(ventre, 0.75 - q.v * 0.25, q.x, q.y))
            } else {
                opaque(ton(dos, lum(q.v) + 0.05, q.x, q.y))
            }
        },
    );
    // Ouïes : cinq fentes.
    for i in 0..5 {
        let t = 0.74 + i as f32 * 0.022;
        let xx = x(t);
        d.trait_sur(
            (xx, c(t) - profil_requin(t, hb).0 * 0.2),
            (xx - 0.5, c(t) + profil_requin(t, hb).1 * 0.5),
            [bouts[0][0], bouts[0][1], bouts[0][2], 200],
        );
    }
    // Pectorale longue, par-dessus le corps.
    let pe = (x(0.72), c(0.72) + hb * 0.35);
    d.poly(
        &[
            (pe.0 + 2.0, pe.1 - 1.0),
            (pe.0 - lg * 0.14, pe.1 + hb * 0.95 + b * 0.5),
            (pe.0 - lg * 0.1, pe.1 + hb * 0.9 + b * 0.5),
            (pe.0 - lg * 0.03, pe.1 + 1.5),
        ],
        NAGEOIRE,
        |q| opaque(ton(dos, 0.35 + 0.25 * (1.0 - q.v), q.x, q.y)),
    );
    let _ = bouts;
}

fn marteau(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let cy = h * 0.5;
    let (x0, x1) = (l * 0.17, l * 0.92);
    let hb = h * 0.22;
    requin(
        d,
        k,
        &p.r[0],
        &p.r[1],
        &p.r[2],
        (x0, x1),
        cy,
        hb,
        1.25,
        |_| 0.25,
    );
    // Le « marteau » : céphalofoil vu un peu de face, un œil à chaque bout.
    let hx = x1 - 1.0;
    d.ellipse(hx, cy - h * 0.02, l * 0.045, h * 0.3, 0.12, CORPS, |q| {
        if q.t > 0.3 {
            return opaque(ton(&p.r[1], 0.6, q.x, q.y));
        }
        opaque(ton(&p.r[0], 0.55 - q.v * 0.25, q.x, q.y))
    });
    d.oeil(hx - 0.5, cy - h * 0.29, 0, [0, 0, 0]);
    d.oeil(hx + 0.5, cy + h * 0.25, 0, [0, 0, 0]);
    d.trait_sur(
        (hx - 2.0, cy + h * 0.1),
        (hx + 1.0, cy + h * 0.12),
        [0x30, 0x28, 0x2c, 255],
    );
    d.queue = ((x0 - l * 0.05) as i32, cy as i32);
    d.bouche = ((x1 + l * 0.02) as i32, cy as i32);
}

fn espadon(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let b = (k * TAU).sin();
    let cy = h * 0.52;
    let (x0, x1) = (l * 0.16, l * 0.74);
    let lg = x1 - x0;
    let hb = h * 0.2;
    let c = |t: f32| cy + plier(t, b, 1.6);
    let pr = |t: f32| (PI * (0.07 + 0.86 * t)).sin().max(0.0).powf(0.6);
    let x = |t: f32| x0 + t * lg;
    let q = caudale(Caudale::Croissant, x0 + 0.5, c(0.0), l * 0.15, h * 0.48, b);
    d.poly(&q, NAGEOIRE, |q| {
        opaque(ton(&p.r[0], 0.35 + 0.3 * q.t, q.x, q.y))
    });
    // Haute dorsale en voile courte, près de la tête.
    d.poly(
        &[
            (x(0.66), c(0.66) - hb * pr(0.66) + 1.0),
            (x(0.7), c(0.7) - hb - h * 0.36),
            (x(0.74), c(0.74) - hb - h * 0.33),
            (x(0.74), c(0.74) - hb * pr(0.74) - 1.0),
            (x(0.85), c(0.85) - hb * pr(0.85) + 1.0),
        ],
        NAGEOIRE,
        |q| opaque(ton(&p.r[0], 0.3 + 0.3 * q.t, q.x, q.y)),
    );
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t),
        CORPS,
        contre(&p.r[0], &p.r[1], 0.05),
    );
    // Rostre plat (l'épée).
    d.fuseau(
        x1 - 1.0,
        l - 0.5,
        |_| c(1.0) - 0.4,
        |t| 0.9 - 0.6 * t,
        |t| 0.7 - 0.5 * t,
        CORPS,
        |q| opaque(ton(&p.r[2], 0.65 - q.v * 0.3, q.x, q.y)),
    );
    let pe = (x(0.82), c(0.82) + hb * 0.5);
    d.poly(
        &[
            (pe.0 + 1.0, pe.1 - 1.0),
            (pe.0 - l * 0.12, pe.1 + h * 0.25 + b),
            (pe.0 - l * 0.05, pe.1 + 1.5),
        ],
        NAGEOIRE,
        |q| opaque(ton(&p.r[0], 0.45, q.x, q.y)),
    );
    d.oeil(x(0.93), c(0.93) - hb * 0.2, 1, p.c[3]);
    d.queue = (x0 as i32 - 4, c(0.0) as i32);
    d.bouche = (l as i32 - 1, c(1.0) as i32);
}

fn phoque(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let b = ph.sin();
    let cy = h * 0.5;
    let (x0, x1) = (l * 0.16, l - 0.5);
    let lg = x1 - x0;
    let hb = h * 0.36;
    // Corps souple qui se courbe tout entier.
    let c = |t: f32| cy + b * h * 0.12 * (1.0 - t).powi(2) - b * h * 0.04 * t;
    let pr = |t: f32| {
        let corps = (t / 0.6).min(1.0).powf(0.7) * 0.9 + 0.1;
        let cou = 1.0 - ((t - 0.72) / 0.2).clamp(0.0, 1.0) * 0.28;
        let museau = 1.0 - ((t - 0.92) / 0.08).clamp(0.0, 1.0) * 0.5;
        corps * cou * museau
    };
    let x = |t: f32| x0 + t * lg;
    // Nageoires arrière en éventail (paire), qui battent haut-bas.
    for (dy, sombre) in [(-1.0f32, 0.15), (1.0, 0.0)] {
        let o = (x0 + 1.0, c(0.0) + dy * 0.8);
        let a = b * 0.35 + dy * 0.18;
        let pts: Vec<(f32, f32)> = [
            (0.0, -1.5),
            (-lg * 0.18, -h * 0.2),
            (-lg * 0.2, 0.0),
            (-lg * 0.16, h * 0.18),
            (0.0, 1.5),
        ]
        .iter()
        .map(|&(px, py)| tourner(o, (o.0 + px, o.1 + py * 0.6), a))
        .collect();
        d.poly(&pts, NAGEOIRE, |q| {
            opaque(ton(&p.r[2], 0.45 - sombre + 0.2 * q.t, q.x, q.y))
        });
    }
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t) * 0.95,
        CORPS,
        |q| {
            let tache = hache(q.x / 2, q.y / 2, 31) < 0.16 && q.v < 0.4;
            if q.v > 0.4 {
                return opaque(ton(&p.r[1], 0.7 - q.v * 0.2, q.x, q.y));
            }
            let lu = lum(q.v) + 0.05 - if tache { 0.3 } else { 0.0 };
            opaque(ton(&p.r[0], lu, q.x, q.y))
        },
    );
    // Nageoire avant.
    let pe = (x(0.7), c(0.7) + hb * 0.4);
    let a = 0.4 + 0.3 * (ph + 1.0).sin();
    let pts: Vec<(f32, f32)> = [
        (1.5, -1.0),
        (-lg * 0.14, 0.5),
        (-lg * 0.12, 2.0),
        (0.0, 1.5),
    ]
    .iter()
    .map(|&(px, py)| tourner(pe, (pe.0 + px, pe.1 + py), a))
    .collect();
    d.poly(&pts, NAGEOIRE, |q| opaque(ton(&p.r[2], 0.55, q.x, q.y)));
    // Grand œil noir brillant, truffe, moustaches.
    let (ex, ey) = (x(0.9), c(0.9) - hb * 0.35);
    d.oeil(ex, ey, 1, [0, 0, 0]);
    d.mettre(
        ex as i32 + 1,
        ey as i32 + 1,
        [10, 8, 14, 255],
        crate::poisson::OEIL,
    );
    d.sur(
        x1 as i32 - 1,
        (c(1.0) - 0.5) as i32,
        opaque(p.c[3]).unwrap(),
    );
    for i in 0..3 {
        let y = c(0.97) + 0.5 + i as f32;
        d.trait_(
            (x1 - 2.0, y),
            (x1 + 0.4, y + i as f32 * 0.6 - 0.6),
            [0xf0, 0xf0, 0xe8, 200],
            NAGEOIRE,
        );
    }
    d.queue = (x0 as i32 - 3, c(0.0) as i32);
    d.bouche = (x1 as i32, c(1.0) as i32);
}

fn dauphin(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let b = ph.sin();
    let cy = h * 0.5;
    let (x0, x1) = (l * 0.14, l * 0.9);
    let lg = x1 - x0;
    let hb = h * 0.24;
    // Nage de haut en bas : l'arrière du corps monte et descend.
    let c = |t: f32| cy + b * h * 0.16 * (1.0 - t / 0.7).max(0.0).powi(2);
    let pr = |t: f32| {
        let a = (t / 0.55).min(1.0).powf(0.75) * 0.88 + 0.12;
        a * (1.0 - ((t - 0.82) / 0.18).clamp(0.0, 1.0).powf(1.4) * 0.55)
    };
    let x = |t: f32| x0 + t * lg;
    // Caudale horizontale : vue de profil, une lame qui pivote.
    let o = (x0 + 1.0, c(0.0));
    let a = -b * 0.6;
    let lame: Vec<(f32, f32)> = [
        (0.0, -1.2),
        (-lg * 0.14, -h * 0.08),
        (-lg * 0.16, 0.0),
        (-lg * 0.14, h * 0.08),
        (0.0, 1.2),
    ]
    .iter()
    .map(|&(px, py)| tourner(o, (o.0 + px, o.1 + py), a))
    .collect();
    d.poly(&lame, NAGEOIRE, |q| {
        opaque(ton(&p.r[2], 0.45 + 0.2 * q.t, q.x, q.y))
    });
    // Aileron dorsal en faucille.
    d.poly(
        &[
            (x(0.42), c(0.42) - hb * pr(0.42) + 1.0),
            (x(0.47), c(0.47) - hb - h * 0.18),
            (x(0.44), c(0.44) - hb - h * 0.28),
            (x(0.56), c(0.56) - hb * pr(0.56) - 0.5),
            (x(0.62), c(0.62) - hb * pr(0.62) + 1.0),
        ],
        NAGEOIRE,
        |q| opaque(ton(&p.r[0], 0.35 + 0.3 * q.t, q.x, q.y)),
    );
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t),
        CORPS,
        |q| {
            // Dos sombre, flanc gris, ventre clair avec une limite en vague.
            let lim = 0.15 + 0.15 * (q.t * 9.0).sin();
            if q.v > lim + 0.2 {
                return opaque(ton(&p.r[1], 0.7 - q.v * 0.2, q.x, q.y));
            }
            if q.v < -0.35 {
                return opaque(ton(&p.r[2], lum(q.v) + 0.15, q.x, q.y));
            }
            opaque(ton(&p.r[0], lum(q.v) + 0.08, q.x, q.y))
        },
    );
    // Rostre (bec) et sourire.
    d.fuseau(
        x1 - 1.0,
        l - 0.5,
        |_| c(1.0) + hb * 0.25,
        |_| hb * 0.28,
        |_| hb * 0.32,
        CORPS,
        |q| opaque(ton(&p.r[if q.v > 0.0 { 1 } else { 0 }], 0.6, q.x, q.y)),
    );
    d.trait_sur(
        (x(0.92), c(0.92) + hb * 0.4),
        (l - 1.0, c(1.0) + hb * 0.25),
        [0x30, 0x34, 0x40, 255],
    );
    let pe = (x(0.72), c(0.72) + hb * 0.5);
    d.poly(
        &[
            (pe.0 + 1.0, pe.1 - 1.0),
            (pe.0 - lg * 0.12, pe.1 + h * 0.22 + b * 0.5),
            (pe.0 - lg * 0.04, pe.1 + 1.5),
        ],
        NAGEOIRE,
        |q| opaque(ton(&p.r[2], 0.5, q.x, q.y)),
    );
    d.oeil(x(0.92), c(0.92) - hb * 0.05, 0, [0, 0, 0]);
    d.queue = (x0 as i32 - 4, c(0.0) as i32);
    d.bouche = (l as i32 - 1, c(1.0) as i32);
}

fn mola(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let (cx, cy) = (l * 0.55, h * 0.5);
    let (rx, ry) = (l * 0.4, h * 0.25);
    // Dorsale et anale très hautes qui godillent (s'inclinent d'un côté puis de l'autre).
    let incl = ph.sin() * l * 0.08;
    for s in [-1.0f32, 1.0] {
        let base = (cx - rx * 0.45, cy + s * ry * 0.75);
        d.poly(
            &[
                (base.0 - rx * 0.2, base.1),
                (base.0 - rx * 0.15 + incl, cy + s * h * 0.49),
                (base.0 + rx * 0.05 + incl, cy + s * h * 0.47),
                (base.0 + rx * 0.25, base.1 - s * 1.0),
            ],
            NAGEOIRE,
            |q| {
                opaque(ton(
                    &p.r[2],
                    0.45 + 0.3 * (1.0 - (q.v * s).max(0.0)),
                    q.x,
                    q.y,
                ))
            },
        );
    }
    // Clavus : bord arrière festonné à la place de la queue.
    d.ellipse(
        cx - rx * 0.88,
        cy,
        rx * 0.2,
        ry * 0.95,
        0.0,
        NAGEOIRE,
        |q| {
            let feston = ((q.v * 5.0).fract() - 0.5).abs() < 0.2 && q.t < -0.4;
            if feston {
                return None;
            }
            opaque(ton(&p.r[2], 0.5 + q.t * 0.2, q.x, q.y))
        },
    );
    d.ellipse(cx, cy, rx, ry, 0.0, CORPS, |q| {
        // Peau rugueuse marbrée.
        let marbre = hache(q.x / 3, q.y / 3, 23) < 0.3;
        let lu = lum(q.v) * 0.75 + 0.02 + if marbre { 0.12 } else { 0.0 };
        opaque(ton(
            &p.r[if marbre { 2 } else { 0 }],
            lu + if marbre { 0.25 } else { 0.0 },
            q.x,
            q.y,
        ))
    });
    // Petite pectorale, petite bouche ronde, œil.
    let pe = (cx + rx * 0.3, cy);
    d.ellipse(
        pe.0,
        pe.1,
        l * 0.05,
        h * 0.05,
        0.4 + ph.sin() * 0.3,
        NAGEOIRE,
        |q| opaque(ton(&p.r[2], 0.6, q.x, q.y)),
    );
    d.oeil(cx + rx * 0.62, cy - ry * 0.2, 1, p.c[1]);
    d.sur((cx + rx) as i32 - 1, cy as i32 + 1, [0x30, 0x30, 0x40, 255]);
    d.queue = ((cx - rx) as i32, cy as i32);
    d.bouche = ((cx + rx) as i32, cy as i32);
    let _ = melange;
}

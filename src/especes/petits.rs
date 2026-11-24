//! Communs et peu communs : petits poissons de récif et d'eau douce, hippocampe, crabe, crevette.

use super::{
    bordee, caudale, contre, membrane, plier, v, Caudale, Espece, Mouvement, Pal, Taille, Variante,
    V_POISSON, V_VIF,
};
use crate::art::{alpha, hache, lum, opaque, ton, Dessin};
use crate::jeu::Rarete;
use crate::poisson::{CORPS, NAGEOIRE};
use std::f32::consts::{PI, TAU};

const V_CLOWN: &[Variante] = &[
    v("", 0.0, 1.0, 1.0),
    v("noir", 0.0, 0.6, 0.32),
    v("doré", 0.06, 0.9, 1.1),
    v("rose", -0.05, 0.55, 1.1),
];

pub fn especes() -> Vec<Espece> {
    let e = |cle, nom, rarete, taille, mouvement, dims, images, couleurs, variantes, art| Espece {
        cle,
        nom,
        rarete,
        taille,
        mouvement,
        dims,
        images,
        couleurs,
        variantes,
        poids: 1.0,
        art,
    };
    use Mouvement as M;
    use Rarete::{Commun as C, PeuCommun as U};
    use Taille as T;
    vec![
        e(
            "clown",
            "Poisson-clown",
            C,
            T::Petite,
            M::Nage,
            (22, 14),
            4,
            [
                [0xf0, 0x6c, 0x14],
                [0xf8, 0x8c, 0x2c],
                [0xf4, 0xf4, 0xf0],
                [0x1c, 0x16, 0x20],
            ],
            V_CLOWN,
            clown as super::Art,
        ),
        e(
            "neon",
            "Néon",
            C,
            T::Minuscule,
            M::Banc,
            (12, 6),
            4,
            [
                [0x30, 0xb8, 0xf8],
                [0xe8, 0x30, 0x3c],
                [0x7c, 0x84, 0x6c],
                [0xe0, 0xe8, 0xec],
            ],
            V_VIF,
            neon,
        ),
        e(
            "guppy",
            "Guppy",
            C,
            T::Minuscule,
            M::Banc,
            (16, 10),
            4,
            [
                [0xf8, 0x84, 0x2c],
                [0x3c, 0x7c, 0xf0],
                [0x9c, 0xa4, 0x84],
                [0xe8, 0xe4, 0xd0],
            ],
            V_VIF,
            guppy,
        ),
        e(
            "rouge",
            "Poisson rouge",
            C,
            T::Petite,
            M::Nage,
            (21, 14),
            4,
            [
                [0xf4, 0x7c, 0x18],
                [0xfc, 0xb0, 0x40],
                [0xfc, 0xe8, 0xb0],
                [0xff, 0xff, 0xff],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("blanc", 0.0, 0.12, 1.2),
                    v("noir", 0.0, 0.4, 0.3),
                    v("vermillon", -0.04, 1.1, 0.95),
                ]
            },
            rouge,
        ),
        e(
            "sardine",
            "Sardine",
            C,
            T::Petite,
            M::Banc,
            (19, 6),
            4,
            [
                [0x2c, 0x70, 0x90],
                [0x58, 0xa8, 0xb4],
                [0xd8, 0xe4, 0xec],
                [0x18, 0x30, 0x40],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("dorée", -0.4, 1.0, 1.0),
                    v("argentée", 0.0, 0.3, 1.2),
                ]
            },
            sardine,
        ),
        e(
            "gobie",
            "Gobie",
            C,
            T::Minuscule,
            M::Fond,
            (15, 6),
            4,
            [
                [0xb0, 0x94, 0x6c],
                [0x5c, 0x44, 0x34],
                [0xe8, 0xdc, 0xbc],
                [0x48, 0xc8, 0xe8],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("citron", -0.05, 2.2, 1.25),
                    v("des récifs", 0.45, 1.0, 1.0),
                ]
            },
            gobie,
        ),
        e(
            "chirurgien",
            "Chirurgien bleu",
            U,
            T::Petite,
            M::Nage,
            (24, 16),
            4,
            [
                [0x1c, 0x60, 0xe8],
                [0xf8, 0xd0, 0x20],
                [0x14, 0x14, 0x2c],
                [0x60, 0xb0, 0xff],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("violet", 0.1, 1.0, 1.0),
                    v("turquoise", -0.08, 1.0, 1.1),
                ]
            },
            chirurgien,
        ),
        e(
            "ange",
            "Poisson-ange",
            U,
            T::Petite,
            M::Nage,
            (22, 22),
            4,
            [
                [0x24, 0x40, 0xb0],
                [0xf8, 0xd8, 0x30],
                [0x10, 0x14, 0x30],
                [0xa8, 0xd8, 0xf8],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("royal", 0.12, 1.0, 1.0),
                    v("flamme", 0.45, 1.0, 1.1),
                ]
            },
            ange,
        ),
        e(
            "globe",
            "Poisson-globe",
            U,
            T::Petite,
            M::Nage,
            (19, 16),
            4,
            [
                [0xc8, 0xb0, 0x60],
                [0x4c, 0x3c, 0x24],
                [0xf4, 0xf0, 0xe0],
                [0x7c, 0xa8, 0x4c],
            ],
            V_POISSON,
            globe,
        ),
        e(
            "hippocampe",
            "Hippocampe",
            U,
            T::Petite,
            M::Debout,
            (12, 24),
            4,
            [
                [0xf0, 0xa8, 0x28],
                [0xfc, 0xe0, 0x80],
                [0x9c, 0x50, 0x18],
                [0xff, 0xff, 0xff],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("rose", -0.12, 0.8, 1.05),
                    v("noir", 0.0, 0.4, 0.4),
                    v("vert", 0.18, 0.9, 0.9),
                ]
            },
            hippocampe,
        ),
        e(
            "crabe",
            "Crabe",
            U,
            T::Petite,
            M::Marche,
            (24, 13),
            4,
            [
                [0xd8, 0x44, 0x28],
                [0xf0, 0x90, 0x58],
                [0xf8, 0xe8, 0xd8],
                [0x30, 0x10, 0x10],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("bleu", 0.58, 0.8, 0.9),
                    v("violet", 0.82, 0.7, 0.85),
                    v("des sables", 0.08, 0.5, 1.15),
                ]
            },
            crabe,
        ),
        e(
            "crevette",
            "Crevette",
            U,
            T::Petite,
            M::Fond,
            (20, 11),
            4,
            [
                [0xf0, 0x70, 0x60],
                [0xfc, 0xf0, 0xf0],
                [0xa8, 0x30, 0x30],
                [0xff, 0xff, 0xff],
            ],
            const {
                &[
                    v("", 0.0, 1.0, 1.0),
                    v("nettoyeuse", 0.0, 1.3, 0.95),
                    v("fantôme", 0.0, 0.2, 1.2),
                    v("bleue", 0.6, 0.9, 1.0),
                ]
            },
            crevette,
        ),
    ]
}

fn clown(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let b = (k * TAU).sin();
    let cy = h * 0.5;
    let (x0, x1) = (l * 0.26, l - 0.2);
    let lg = x1 - x0;
    let hb = h * 0.36;
    let c = |t: f32| cy + plier(t, b, 1.3);
    let pr = |t: f32| (PI * (0.1 + 0.82 * t)).sin().max(0.0).powf(0.55);
    let x = |t: f32| x0 + t * lg;
    let haut = |t: f32| c(t) - hb * pr(t);
    let bas = |t: f32| c(t) + hb * pr(t);
    let (or, bl, no) = (&p.r[0], &p.r[2], &p.r[3]);
    // Nageoires (derrière le corps), liseré noir.
    let q = caudale(Caudale::Ronde, x0 + 0.5, c(0.0), l * 0.27, h * 0.3, b);
    bordee(d, &q, (x0, c(0.0)), &p.r[1], no, 0.72);
    let dors = [
        (x(0.22), haut(0.22) + 1.0),
        (x(0.3), haut(0.3) - h * 0.15),
        (x(0.44), haut(0.44) - h * 0.1),
        (x(0.58), haut(0.58) - h * 0.2),
        (x(0.7), haut(0.7) - h * 0.1),
        (x(0.78), haut(0.78) + 1.0),
    ];
    bordee(d, &dors, (x(0.5), haut(0.5) + 1.0), &p.r[1], no, 0.6);
    let anale = [
        (x(0.16), bas(0.16) - 1.0),
        (x(0.28), bas(0.28) + h * 0.16),
        (x(0.42), bas(0.42) - 0.5),
    ];
    bordee(d, &anale, (x(0.3), bas(0.3) - 1.0), &p.r[1], no, 0.55);
    let pelv = [
        (x(0.58), bas(0.58) - 1.0),
        (x(0.62), bas(0.62) + h * 0.14 + b.abs() * 0.5),
        (x(0.72), bas(0.72) - 0.5),
    ];
    bordee(d, &pelv, (x(0.65), bas(0.65) - 1.0), &p.r[1], no, 0.55);
    // Corps et bandes blanches bordées de noir.
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t),
        CORPS,
        |q| {
            let bandes = [
                (0.76 - 0.06 * q.v * q.v, 1.1),
                (0.45 + 0.07 * (1.0 - q.v * q.v), 1.2),
                (0.05, 0.5),
            ];
            for (ct, w) in bandes {
                let dx = (q.t - ct).abs() * lg;
                if dx < w {
                    return opaque(ton(bl, lum(q.v) * 0.5 + 0.5, q.x, q.y));
                }
                if dx < w + 0.7 {
                    return opaque(ton(no, 0.5, q.x, q.y));
                }
            }
            opaque(ton(or, lum(q.v), q.x, q.y))
        },
    );
    // Pectorale qui bat.
    let pe = (x(0.64), c(0.64) + hb * 0.15);
    let pect = [
        (pe.0 + 1.0, pe.1),
        (pe.0 - 2.5, pe.1 + 1.0 + b),
        (pe.0 - 2.0, pe.1 + 2.5 + b * 0.5),
    ];
    bordee(d, &pect, pe, &p.r[1], no, 0.5);
    d.oeil(x(0.87), c(0.87) - hb * 0.3, 1, [0xf0, 0x80, 0x20]);
    d.sur(
        x1 as i32 - 1,
        (c(1.0) + hb * 0.2) as i32,
        [0x6c, 0x20, 0x14, 255],
    );
    d.queue = (x0 as i32 - 3, c(0.0) as i32);
    d.bouche = (x1 as i32, c(1.0) as i32);
}

fn neon(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let b = (k * TAU).sin();
    let cy = h * 0.5;
    let (x0, x1) = (l * 0.26, l - 0.2);
    let lg = x1 - x0;
    let hb = h * 0.42;
    let c = |t: f32| cy + plier(t, b, 0.8);
    let pr = |t: f32| (PI * (0.12 + 0.8 * t)).sin().max(0.0).powf(0.7);
    let q = caudale(Caudale::Fourche, x0 + 0.5, c(0.0), l * 0.26, h * 0.4, b);
    d.poly(&q, NAGEOIRE, membrane(&p.r[2], 170));
    let dors = [
        (x0 + lg * 0.45, c(0.45) - hb * 0.9),
        (x0 + lg * 0.5, c(0.5) - hb - 1.6),
        (x0 + lg * 0.62, c(0.6) - hb * 0.9),
    ];
    d.poly(&dors, NAGEOIRE, membrane(&p.r[2], 190));
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t),
        CORPS,
        |q| {
            // Ligne bleu électrique, rouge sous l'arrière, ventre argent.
            if q.v > -0.55 && q.v < 0.05 && q.t > 0.12 {
                return opaque(ton(&p.r[0], 0.75 - q.v * 0.3, q.x, q.y));
            }
            if q.v >= 0.05 && q.t < 0.58 {
                return opaque(ton(&p.r[1], lum(q.v), q.x, q.y));
            }
            if q.v >= 0.05 {
                return opaque(ton(&p.r[3], 0.6, q.x, q.y));
            }
            opaque(ton(&p.r[2], lum(q.v) * 0.8, q.x, q.y))
        },
    );
    d.oeil(x1 - 2.0, c(1.0) - 0.4, 0, [0, 0, 0]);
    d.queue = (x0 as i32 - 2, c(0.0) as i32);
    d.bouche = (x1 as i32, c(1.0) as i32);
}

fn guppy(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let b = (k * TAU).sin();
    let cy = h * 0.45;
    let (x0, x1) = (l * 0.5, l - 0.2);
    let lg = x1 - x0;
    let hb = h * 0.24;
    let c = |t: f32| cy + plier(t, b, 0.6);
    let pr = |t: f32| (PI * (0.2 + 0.72 * t)).sin().max(0.0).powf(0.6);
    // Grande queue en voile tachetée de deux couleurs.
    let q = caudale(Caudale::Voile, x0 + 0.5, c(0.0), l * 0.5, h * 0.48, b);
    d.poly(&q, NAGEOIRE, |q| {
        let tache = hache(q.x / 2, q.y / 2, 11) < 0.35;
        let r = if tache { &p.r[1] } else { &p.r[0] };
        alpha(ton(r, 0.45 + 0.4 * (1.0 - q.t), q.x, q.y), 225)
    });
    let dors = [
        (x0 + lg * 0.05, c(0.1) - hb),
        (x0 - lg * 0.15, c(0.0) - hb - 2.2 + b * 0.4),
        (x0 + lg * 0.4, c(0.4) - hb),
    ];
    d.poly(&dors, NAGEOIRE, membrane(&p.r[1], 210));
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t) * 1.1,
        CORPS,
        |q| {
            if q.t < 0.45 && hache(q.x, q.y, 5) < 0.4 {
                return opaque(ton(&p.r[0], lum(q.v), q.x, q.y));
            }
            contre(&p.r[2], &p.r[3], 0.35)(q)
        },
    );
    d.oeil(x1 - 2.0, c(1.0) - 0.5, 0, [0, 0, 0]);
    d.queue = (x0 as i32 - 4, c(0.0) as i32);
    d.bouche = (x1 as i32, c(1.0) as i32);
}

fn rouge(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let b = (k * TAU).sin();
    let cy = h * 0.5;
    let (x0, x1) = (l * 0.36, l - 0.2);
    let lg = x1 - x0;
    let hb = h * 0.34;
    let c = |t: f32| cy + plier(t, b, 0.8);
    let pr = |t: f32| (PI * (0.12 + 0.8 * t)).sin().max(0.0).powf(0.5);
    let x = |t: f32| x0 + t * lg;
    // Double queue en voile (fantail) : deux lobes décalés.
    for (dy, a) in [(-0.6, 190u8), (0.6, 230)] {
        let q = caudale(
            Caudale::Voile,
            x0 + 0.5,
            c(0.0) + dy,
            l * 0.38,
            h * 0.36,
            b * if dy < 0.0 { 0.7 } else { 1.0 },
        );
        d.poly(&q, NAGEOIRE, |q| {
            let raie = (q.y + q.x / 3).rem_euclid(3) == 0;
            let r = &p.r[if raie { 0 } else { 1 }];
            alpha(ton(r, 0.4 + 0.4 * q.t, q.x, q.y), a)
        });
    }
    let dors = [
        (x(0.25), c(0.25) - hb * pr(0.25) + 0.5),
        (x(0.34), c(0.34) - hb - h * 0.24 + b * 0.5),
        (x(0.62), c(0.62) - hb * pr(0.62) - 1.0),
        (x(0.7), c(0.7) - hb * pr(0.7) + 0.5),
    ];
    d.poly(&dors, NAGEOIRE, membrane(&p.r[1], 215));
    let anale = [
        (x(0.15), c(0.15) + hb * pr(0.15) - 0.5),
        (x(0.12), c(0.1) + hb + h * 0.2 + b),
        (x(0.42), c(0.42) + hb * pr(0.42)),
    ];
    d.poly(&anale, NAGEOIRE, membrane(&p.r[1], 200));
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t) * 1.05,
        CORPS,
        |q| {
            // Écailles : petits reflets en quinconce, ventre clair.
            let mut lu = lum(q.v);
            if (q.x + (q.y / 2) * 2).rem_euclid(4) == 0 && q.y % 2 == 0 && q.v < 0.4 {
                lu += 0.25;
            }
            if q.v > 0.45 {
                return opaque(ton(&p.r[2], lu * 0.6 + 0.3, q.x, q.y));
            }
            opaque(ton(&p.r[0], lu, q.x, q.y))
        },
    );
    let pe = (x(0.62), c(0.62) + hb * 0.4);
    d.poly(
        &[
            (pe.0 + 1.0, pe.1),
            (pe.0 - 2.0, pe.1 + 2.0 + b * 0.6),
            (pe.0 - 0.5, pe.1 + 2.5),
        ],
        NAGEOIRE,
        membrane(&p.r[1], 220),
    );
    d.oeil(x(0.84), c(0.84) - hb * 0.3, 1, [0xf0, 0x90, 0x20]);
    d.sur(
        x1 as i32 - 1,
        (c(1.0) + 0.5) as i32,
        [0x90, 0x30, 0x10, 255],
    );
    d.queue = (x0 as i32 - 4, c(0.0) as i32);
    d.bouche = (x1 as i32, c(1.0) as i32);
}

fn sardine(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let b = (k * TAU).sin();
    let cy = h * 0.5;
    let (x0, x1) = (l * 0.2, l - 0.2);
    let lg = x1 - x0;
    let hb = h * 0.42;
    let c = |t: f32| cy + plier(t, b, 0.7);
    let pr = |t: f32| (PI * (0.1 + 0.85 * t)).sin().max(0.0).powf(0.8);
    let q = caudale(Caudale::Fourche, x0 + 0.5, c(0.0), l * 0.2, h * 0.5, b);
    d.poly(&q, NAGEOIRE, membrane(&p.r[1], 200));
    d.poly(
        &[
            (x0 + lg * 0.42, c(0.42) - hb * 0.9),
            (x0 + lg * 0.5, c(0.48) - hb - 1.5),
            (x0 + lg * 0.6, c(0.6) - hb * 0.9),
        ],
        NAGEOIRE,
        membrane(&p.r[1], 200),
    );
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t),
        CORPS,
        |q| {
            if q.v < -0.1 {
                // Rangée de taches sombres le long du flanc.
                let tache = q.v > -0.45 && ((q.t * lg) as i32) % 3 == 0 && q.t > 0.3 && q.t < 0.75;
                if tache {
                    return opaque(p.c[3]);
                }
                return opaque(ton(&p.r[0], lum(q.v), q.x, q.y));
            }
            // Ventre argenté avec reflet bleu-vert.
            let r = if q.v < 0.15 { &p.r[1] } else { &p.r[2] };
            opaque(ton(r, 0.85 - q.v * 0.35, q.x, q.y))
        },
    );
    d.oeil(x1 - 2.5, c(1.0) - 0.5, 0, [0, 0, 0]);
    d.queue = (x0 as i32 - 2, c(0.0) as i32);
    d.bouche = (x1 as i32, c(1.0) as i32);
}

fn gobie(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let b = (k * TAU).sin() * 0.6;
    let cy = h * 0.6;
    let (x0, x1) = (l * 0.18, l - 0.2);
    let lg = x1 - x0;
    let hb = h * 0.34;
    let c = |t: f32| cy + plier(t, b, 0.6);
    let pr = |t: f32| (0.55 + 0.45 * t).min(1.0) * (1.0 - ((t - 0.93).max(0.0) * 9.0).powi(2));
    let q = caudale(Caudale::Ronde, x0 + 0.5, c(0.0), l * 0.18, h * 0.3, b);
    d.poly(&q, NAGEOIRE, membrane(&p.r[0], 210));
    for (a, z, hh) in [(0.3, 0.45, 1.6), (0.52, 0.8, 1.3)] {
        d.poly(
            &[
                (x0 + lg * a, c(a) - hb * pr(a) + 0.5),
                (x0 + lg * (a + 0.04), c(a) - hb * pr(a) - hh),
                (x0 + lg * z, c(z) - hb * pr(z) - hh * 0.6),
                (x0 + lg * z, c(z) - hb * pr(z) + 0.5),
            ],
            NAGEOIRE,
            membrane(&p.r[0], 210),
        );
    }
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t) * 0.9,
        CORPS,
        |q| {
            // Marbrures sombres et points bleus.
            let mx = (q.x as f32 * 0.7).floor() as i32;
            if q.v < 0.3 && hache(mx, q.y / 2, 3) < 0.28 {
                return opaque(ton(&p.r[1], lum(q.v), q.x, q.y));
            }
            if q.v < 0.2 && hache(q.x, q.y, 9) < 0.07 {
                return opaque(p.c[3]);
            }
            contre(&p.r[0], &p.r[2], 0.45)(q)
        },
    );
    // Gros yeux sur le dessus de la tête.
    d.oeil(x0 + lg * 0.86, c(0.86) - hb * 0.85, 0, [0, 0, 0]);
    d.mettre(
        (x0 + lg * 0.86) as i32,
        (c(0.86) - hb * 0.85) as i32 - 1,
        [0xf0, 0xe8, 0xc0, 255],
        crate::poisson::OEIL,
    );
    d.queue = (x0 as i32 - 2, c(0.0) as i32);
    d.bouche = (x1 as i32, c(1.0) as i32);
}

fn chirurgien(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let b = (k * TAU).sin();
    let cy = h * 0.5;
    let (x0, x1) = (l * 0.24, l - 0.4);
    let lg = x1 - x0;
    let hb = h * 0.34;
    let c = |t: f32| cy + plier(t, b, 1.0);
    let pr = |t: f32| (PI * (0.1 + 0.84 * t)).sin().max(0.0).powf(0.6);
    let x = |t: f32| x0 + t * lg;
    let no = &p.r[2];
    let q = caudale(Caudale::Croissant, x0 + 0.5, c(0.0), l * 0.24, h * 0.36, b);
    bordee(d, &q, (x0 - 2.0, c(0.0)), &p.r[1], no, 0.75);
    // Longues dorsale et anale bordées de noir.
    let dors = [
        (x(0.06), c(0.06) - hb * pr(0.06) + 0.5),
        (x(0.15), c(0.15) - hb * pr(0.15) - h * 0.14),
        (x(0.6), c(0.6) - hb * pr(0.6) - h * 0.12),
        (x(0.8), c(0.8) - hb * pr(0.8) + 0.5),
    ];
    bordee(d, &dors, (x(0.45), c(0.45)), &p.r[0], no, 0.85);
    let anale = [
        (x(0.06), c(0.06) + hb * pr(0.06) - 0.5),
        (x(0.15), c(0.15) + hb * pr(0.15) + h * 0.12),
        (x(0.5), c(0.5) + hb * pr(0.5) + h * 0.08),
        (x(0.62), c(0.62) + hb * pr(0.62) - 0.5),
    ];
    bordee(d, &anale, (x(0.4), c(0.4)), &p.r[0], no, 0.85);
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t),
        CORPS,
        |q| {
            // « Palette » noire : bande de l'œil à la queue et tache dorsale, ovale bleu entre.
            let ligne = -0.2 + (q.t - 0.15) * 0.25;
            let dans_bande = (q.v - ligne).abs() < 0.2 && q.t > 0.1 && q.t < 0.88;
            let dx = (q.t - 0.48) / 0.2;
            let dv = (q.v + 0.62) / 0.3;
            let ovale = dx * dx + dv * dv < 1.0;
            let tache = q.v < -0.45 && q.t > 0.25 && q.t < 0.72 && !ovale;
            if dans_bande || tache {
                return opaque(ton(no, 0.4, q.x, q.y));
            }
            if q.t < 0.1 {
                return opaque(ton(&p.r[1], lum(q.v), q.x, q.y));
            }
            opaque(ton(&p.r[0], lum(q.v) * 0.9 + 0.08, q.x, q.y))
        },
    );
    let pe = (x(0.7), c(0.7) + hb * 0.2);
    d.poly(
        &[
            (pe.0 + 1.0, pe.1),
            (pe.0 - 2.5, pe.1 + 1.5 + b * 0.6),
            (pe.0 - 1.0, pe.1 + 2.8),
        ],
        NAGEOIRE,
        membrane(&p.r[1], 230),
    );
    d.oeil(x(0.87), c(0.87) - hb * 0.25, 1, [0x30, 0x30, 0x60]);
    d.queue = (x0 as i32 - 4, c(0.0) as i32);
    d.bouche = (x1 as i32, c(1.0) as i32);
}

fn ange(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let b = (k * TAU).sin();
    let cy = h * 0.5;
    let (x0, x1) = (l * 0.22, l - 0.3);
    let lg = x1 - x0;
    let hb = h * 0.28;
    let c = |t: f32| cy + plier(t, b, 0.8);
    let pr = |t: f32| (PI * (0.12 + 0.8 * t)).sin().max(0.0).powf(0.5);
    let x = |t: f32| x0 + t * lg;
    let q = caudale(Caudale::Ronde, x0 + 0.5, c(0.0), l * 0.22, h * 0.24, b);
    d.poly(&q, NAGEOIRE, |q| {
        opaque(ton(&p.r[1], 0.4 + 0.3 * q.t, q.x, q.y))
    });
    // Dorsale et anale hautes qui prolongent le disque vers l'arrière.
    let dors = [
        (x(0.1), c(0.1) - hb * pr(0.1)),
        (x(0.0) - 1.5, c(0.0) - hb - h * 0.2 + b * 0.5),
        (x(0.3), c(0.3) - hb - h * 0.2),
        (x(0.62), c(0.62) - hb * pr(0.62)),
    ];
    let anale = [
        (x(0.1), c(0.1) + hb * pr(0.1)),
        (x(0.0) - 1.5, c(0.0) + hb + h * 0.18 + b * 0.5),
        (x(0.32), c(0.32) + hb + h * 0.17),
        (x(0.6), c(0.6) + hb * pr(0.6)),
    ];
    for f in [&dors, &anale] {
        d.poly(f, NAGEOIRE, |q| {
            // Bord bleu clair, rayures qui continuent.
            let r = if (q.x + q.y).rem_euclid(3) == 0 {
                &p.r[1]
            } else {
                &p.r[0]
            };
            opaque(ton(r, 0.35 + 0.3 * q.t, q.x, q.y))
        });
    }
    d.fuseau(
        x0,
        x1,
        c,
        |t| hb * pr(t),
        |t| hb * pr(t),
        CORPS,
        |q| {
            if q.t > 0.72 {
                // Masque sombre sur l'œil, front bleu pâle, gorge sombre.
                if q.t < 0.84 && q.v < 0.4 {
                    return opaque(ton(&p.r[2], 0.5, q.x, q.y));
                }
                if q.t >= 0.84 && q.v < 0.0 {
                    return opaque(ton(&p.r[3], lum(q.v), q.x, q.y));
                }
                return opaque(ton(&p.r[2], 0.6, q.x, q.y));
            }
            // Rayures jaunes en biais sur fond bleu.
            let raie = ((q.y as f32 + q.t * lg * 0.45) as i32).rem_euclid(3) == 0;
            let r = if raie { &p.r[1] } else { &p.r[0] };
            opaque(ton(r, lum(q.v) * 0.8 + 0.15, q.x, q.y))
        },
    );
    d.oeil(x(0.79), c(0.79) - hb * 0.25, 1, [0x40, 0x60, 0xc0]);
    d.queue = (x0 as i32 - 3, c(0.0) as i32);
    d.bouche = (x1 as i32, c(1.0) as i32);
}

fn globe(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let b = (k * TAU).sin();
    // Le ballon se gonfle et se dégonfle un peu.
    let gonfle = 1.0 + 0.05 * (k * TAU).cos();
    let (cx, cy) = (l * 0.58, h * 0.5);
    let (rx, ry) = (l * 0.36 * gonfle, h * 0.4 * gonfle);
    let q = caudale(Caudale::Ronde, cx - rx + 1.0, cy, l * 0.22, h * 0.18, b);
    d.poly(&q, NAGEOIRE, membrane(&p.r[3], 210));
    d.poly(
        &[
            (cx - rx * 0.5, cy - ry * 0.8),
            (cx - rx * 0.75, cy - ry - 1.5 + b * 0.5),
            (cx - rx * 0.2, cy - ry * 0.9),
        ],
        NAGEOIRE,
        membrane(&p.r[3], 200),
    );
    d.ellipse(cx, cy, rx, ry, 0.0, CORPS, |q| {
        if q.v > 0.25 {
            return opaque(ton(&p.r[2], lum(q.v) * 0.5 + 0.5, q.x, q.y));
        }
        if hache(q.x, q.y, 21) < 0.22 && q.v < 0.1 {
            return opaque(ton(&p.r[1], 0.5, q.x, q.y));
        }
        opaque(ton(&p.r[0], lum(q.v), q.x, q.y))
    });
    // Épines : petits pics qui sortent du ballon.
    for i in 0..14 {
        let a = i as f32 / 14.0 * TAU + 0.2;
        let (s, co) = a.sin_cos();
        let (ex, ey) = (cx + co * (rx + 0.8), cy + s * (ry + 0.8));
        if co > 0.85 {
            continue;
        }
        d.mettre(
            ex.floor() as i32,
            ey.floor() as i32,
            [0xe8, 0xe0, 0xb8, 255],
            NAGEOIRE,
        );
    }
    // Pectorale qui vrombit.
    let pe = (cx + rx * 0.35, cy + ry * 0.1);
    d.poly(
        &[
            (pe.0, pe.1 - 1.0),
            (pe.0 - 2.5, pe.1 - 1.5 + b * 1.2),
            (pe.0 - 2.5, pe.1 + 1.5 + b * 1.2),
            (pe.0, pe.1 + 1.0),
        ],
        NAGEOIRE,
        membrane(&p.r[3], 200),
    );
    d.oeil(cx + rx * 0.55, cy - ry * 0.3, 1, [0x30, 0xa0, 0x60]);
    // Petite bouche en bec.
    d.mettre(
        (cx + rx) as i32,
        (cy + ry * 0.1) as i32,
        [0x80, 0x60, 0x30, 255],
        CORPS,
    );
    d.mettre(
        (cx + rx) as i32,
        (cy + ry * 0.1) as i32 + 1,
        [0x50, 0x3c, 0x20, 255],
        CORPS,
    );
    d.queue = ((cx - rx) as i32 - 3, cy as i32);
    d.bouche = ((cx + rx) as i32 + 1, cy as i32);
}

fn hippocampe(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let b = (k * TAU).sin();
    let pt = |x: f32, y: f32, r: f32| (x * l, y * h, r * l);
    // Museau → tête → cou → ventre → queue enroulée.
    let corps = [
        pt(0.97, 0.2, 0.06),
        pt(0.8, 0.19, 0.08),
        pt(0.62, 0.16, 0.17),
        pt(0.45, 0.11, 0.15),
        pt(0.4, 0.24, 0.14),
        pt(0.5, 0.38, 0.2),
        pt(0.58, 0.52, 0.23),
        pt(0.54, 0.65, 0.18),
        pt(0.42, 0.77, 0.12),
        pt(0.36, 0.88, 0.09),
        pt(0.46, 0.96, 0.07),
        pt(0.62, 0.94, 0.06),
        pt(0.64, 0.86, 0.05),
        pt(0.55, 0.84, 0.04),
    ];
    // Dorsale dans le dos, qui ondule vite.
    let dy = 0.5 * b;
    d.poly(
        &[
            (l * 0.36, h * 0.44),
            (l * 0.08, h * 0.47 + dy),
            (l * 0.1, h * 0.58 - dy),
            (l * 0.4, h * 0.62),
        ],
        NAGEOIRE,
        membrane(&p.r[1], 200),
    );
    d.tube(&corps, CORPS, |q| {
        // Anneaux osseux : une ligne plus sombre régulièrement le long du corps.
        let anneau = ((q.t * 34.0).fract() < 0.22) && q.t > 0.2;
        let lu = 0.75 - q.v * 0.45 - if anneau { 0.3 } else { 0.0 };
        if q.v > 0.45 && q.t > 0.25 && q.t < 0.6 {
            return opaque(ton(&p.r[1], lu, q.x, q.y));
        }
        opaque(ton(&p.r[0], lu, q.x, q.y))
    });
    // Couronne et taches claires.
    d.mettre(
        (l * 0.46) as i32,
        (h * 0.03) as i32,
        opaque(p.c[2]).unwrap(),
        CORPS,
    );
    d.mettre(
        (l * 0.5) as i32,
        (h * 0.03) as i32 + 1,
        opaque(p.c[0]).unwrap(),
        CORPS,
    );
    for (x, y) in [(0.52, 0.44), (0.6, 0.55), (0.45, 0.3)] {
        d.sur((l * x) as i32, (h * y) as i32, opaque(p.c[3]).unwrap());
    }
    d.oeil(l * 0.6, h * 0.14, 0, [0, 0, 0]);
    d.queue = ((l * 0.5) as i32, (h * 0.95) as i32);
    d.bouche = ((l * 0.98) as i32, (h * 0.2) as i32);
}

fn crabe(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = (k * 4.0) as i32;
    let cx = l * 0.5;
    let cy = h * 0.5 + if ph % 2 == 0 { 0.0 } else { -0.4 };
    let pat = |q: &crate::art::P, base: f32| {
        let r = &p.r[0];
        opaque(ton(r, base, q.x, q.y))
    };
    // Pattes : trois de chaque côté, qui marchent en alternance.
    for i in 0..3 {
        for s in [-1.0f32, 1.0] {
            let lev = if (ph + i + if s > 0.0 { 1 } else { 0 }) % 2 == 0 {
                -1.0
            } else {
                0.0
            };
            let ax = cx + s * l * (0.12 + i as f32 * 0.08);
            let ay = cy + h * 0.12;
            let gx = cx + s * l * (0.3 + i as f32 * 0.07);
            let gy = cy - h * 0.02 + lev;
            let px = cx + s * l * (0.36 + i as f32 * 0.07);
            let py = h - 0.6;
            d.tube(&[(ax, ay, 0.8), (gx, gy, 0.7), (px, py, 0.4)], CORPS, |q| {
                pat(q, 0.45 - q.v * 0.2)
            });
        }
    }
    // Pinces levées.
    for s in [-1.0f32, 1.0] {
        let ouv = if ph < 2 { 0.0 } else { 0.8 };
        let bx = cx + s * l * 0.3;
        let by = cy - h * 0.18;
        d.tube(
            &[(cx + s * l * 0.18, cy + h * 0.05, 0.9), (bx, by + 1.0, 0.9)],
            CORPS,
            |q| pat(q, 0.55 - q.v * 0.2),
        );
        d.ellipse(
            bx + s * 0.8,
            by - 1.2,
            l * 0.08,
            h * 0.17,
            s * 0.5,
            CORPS,
            |q| opaque(ton(&p.r[0], lum(q.v) * 0.9 + 0.1, q.x, q.y)),
        );
        // Doigt mobile.
        let doigt = (bx + s * (l * 0.1 + ouv), by - h * 0.32 - ouv);
        d.tube(
            &[(bx + s * 1.5, by - 1.5, 0.7), (doigt.0, doigt.1, 0.4)],
            CORPS,
            |q| pat(q, 0.7),
        );
        d.sur(
            (bx + s * 0.8) as i32,
            (by - 2.0) as i32,
            opaque(p.c[2]).unwrap(),
        );
    }
    // Carapace bombée.
    d.ellipse(cx, cy + h * 0.06, l * 0.27, h * 0.27, 0.0, CORPS, |q| {
        if q.v > 0.55 {
            return opaque(ton(&p.r[1], 0.4, q.x, q.y));
        }
        let mut lu = lum(q.v);
        if hache(q.x, q.y, 4) < 0.12 && q.v < 0.2 {
            lu += 0.25;
        }
        opaque(ton(&p.r[0], lu, q.x, q.y))
    });
    // Yeux pédonculés.
    for s in [-1.0f32, 1.0] {
        let ex = cx + s * l * 0.08;
        d.tube(
            &[(ex, cy - h * 0.12, 0.5), (ex + s * 0.5, cy - h * 0.34, 0.5)],
            CORPS,
            |q| pat(q, 0.6),
        );
        d.oeil(ex + s * 0.5 - 0.5, cy - h * 0.42, 0, [0, 0, 0]);
    }
    d.sur(cx as i32, (cy + h * 0.16) as i32, opaque(p.c[3]).unwrap());
    d.sur(
        cx as i32 - 1,
        (cy + h * 0.16) as i32,
        opaque(p.c[3]).unwrap(),
    );
    d.queue = (cx as i32, (cy + h * 0.2) as i32);
    d.bouche = (cx as i32, (cy + h * 0.1) as i32);
}

fn crevette(d: &mut Dessin, k: f32, p: &Pal) {
    let (l, h) = (d.l as f32, d.h as f32);
    let ph = k * TAU;
    let b = ph.sin();
    // Abdomen arqué (gauche, vers le bas) puis carapace (droite) et rostre pointu.
    let corps = [
        (l * 0.14, h * 0.74, h * 0.09),
        (l * 0.2, h * 0.6, h * 0.13),
        (l * 0.3, h * 0.5, h * 0.17),
        (l * 0.44, h * 0.47, h * 0.2),
        (l * 0.6, h * 0.5, h * 0.24),
        (l * 0.74, h * 0.53, h * 0.23),
        (l * 0.84, h * 0.53, h * 0.14),
    ];
    // Longues antennes rabattues vers l'arrière, au-dessus du corps.
    let ca = [p.c[1][0], p.c[1][1], p.c[1][2], 220];
    let (tx, ty) = (l * 0.86, h * 0.45);
    d.trait_((tx, ty), (l * 0.35, h * 0.08 + b * 0.6), ca, NAGEOIRE);
    d.trait_((tx, ty + 0.5), (l * 0.05, h * 0.3 - b * 0.6), ca, NAGEOIRE);
    // Pattes marcheuses sous la carapace et pléopodes sous l'abdomen, en vague.
    for i in 0..6 {
        let x = l * (0.3 + i as f32 * 0.08);
        let onde = (ph + i as f32 * 1.1).sin();
        d.trait_(
            (x, h * 0.62),
            (x + onde * 0.8, h * 0.86),
            [p.c[2][0], p.c[2][1], p.c[2][2], 230],
            NAGEOIRE,
        );
    }
    // Éventail de la queue.
    let (qx, qy) = (corps[0].0, corps[0].1);
    d.poly(
        &[
            (qx + 1.5, qy - 1.0),
            (qx - 2.5, qy + 1.0 + b * 0.5),
            (qx - 1.5, qy + 2.5 + b * 0.5),
            (qx + 1.5, qy + 3.0),
            (qx + 2.0, qy),
        ],
        NAGEOIRE,
        membrane(&p.r[0], 220),
    );
    d.tube(&corps, CORPS, |q| {
        // Segments translucides cerclés de blanc, carapace d'une pièce.
        let seg = (q.t * 8.0).fract() < 0.2 && q.t < 0.62;
        let r = if seg { &p.r[1] } else { &p.r[0] };
        alpha(ton(r, 0.7 - q.v * 0.45, q.x, q.y), 235)
    });
    // Rostre.
    d.poly(
        &[(l * 0.8, h * 0.42), (l - 0.5, h * 0.4), (l * 0.82, h * 0.5)],
        CORPS,
        |q| opaque(ton(&p.r[0], 0.6, q.x, q.y)),
    );
    d.oeil(l * 0.82, h * 0.44, 0, [0, 0, 0]);
    d.queue = (qx as i32, qy as i32);
    d.bouche = (tx as i32, ty as i32);
}

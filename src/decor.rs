//! Décor en pixel art IA (lot 4, même recette que les créatures : `tools/pixeliser.py`) :
//! bande de sable, rochers, coraux, algues (ondulent), récif lointain. Images embarquées
//! `assets/decor/*.png`, pixels fins, alpha 0 ou 255. v0.9 : six décors à thème et les objets
//! de la boutique « Décor » (`assets/decor/<thème>/`, `assets/decor/objets/`).

use crate::toile::{melange, Rgb, Toile};
use std::sync::OnceLock;

pub struct Img {
    pub l: i32,
    pub h: i32,
    pub px: Vec<[u8; 4]>,
}

impl Img {
    fn charger(b: &[u8]) -> Img {
        let i = image::load_from_memory(b)
            .expect("décor embarqué lisible")
            .to_rgba8();
        Img {
            l: i.width() as i32,
            h: i.height() as i32,
            px: i.pixels().map(|p| p.0).collect(),
        }
    }

    pub fn px(&self, x: i32, y: i32) -> [u8; 4] {
        self.px[(y * self.l + x) as usize]
    }

    /// Première ligne opaque de chaque colonne (profil du dessus), `h` si vide.
    pub fn dessus(&self) -> Vec<i32> {
        (0..self.l)
            .map(|x| {
                (0..self.h)
                    .find(|&y| self.px(x, y)[3] > 0)
                    .unwrap_or(self.h)
            })
            .collect()
    }
}

/// Décor à thème (v0.9, Réglages) : ses propres images, composées avec les règles du lot 4
/// (enfouies dans le sable, ombre de contact, groupes avec du sable nu entre, rien devant le
/// compteur).
pub struct Theme {
    pub sable: Img,
    /// Gros éléments au fond d'un groupe (source de bulles).
    pub grands: Vec<Img>,
    /// Petits éléments devant et entre les groupes (le plus petit sert seul entre deux groupes).
    pub petits: Vec<Img>,
    pub algues: Vec<Img>,
    pub lointain: Img,
    /// Pixels vifs (fissures de lave, cheminées) qui luisent et pulsent.
    pub lueur: bool,
    /// Algues : nombre derrière (min, en plus au hasard) et devant (au plus), hauteur visible
    /// (fraction de l'eau).
    pub algues_nb: (u32, u32, u32),
    pub algues_haut: (f32, f32),
    /// Algues aussi entre les groupes (forêt de kelp).
    pub algues_entre: bool,
}

/// Animation d'un objet de décor acheté.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Anim {
    Aucune,
    /// Filet de bulles continu (coffre au trésor).
    Bulles,
    /// Bouffées de bulles toutes les quelques secondes.
    Geyser,
    /// Ondule comme une algue et luit.
    Ondule,
    /// Pixels vifs qui luisent et pulsent.
    Lueur,
}

/// Objet de la boutique « Décor » (v0.9), du moins cher au plus cher.
pub struct Objet {
    pub cle: &'static str,
    pub nom: &'static str,
    pub prix: f64,
    pub anim: Anim,
}

pub const THEMES: [&str; 6] = [
    "Récif corallien",
    "Épave & trésor",
    "Ruines englouties",
    "Abysses volcaniques",
    "Forêt de kelp",
    "Cimetière marin",
];

pub const OBJETS: [Objet; 14] = [
    Objet {
        cle: "cailloux",
        nom: "Cailloux",
        prix: 150.0,
        anim: Anim::Aucune,
    },
    Objet {
        cle: "crane",
        nom: "Crâne",
        prix: 1_500.0,
        anim: Anim::Aucune,
    },
    Objet {
        cle: "anemone",
        nom: "Anémone géante",
        prix: 12_000.0,
        anim: Anim::Aucune,
    },
    Objet {
        cle: "amphores",
        nom: "Amphores",
        prix: 90_000.0,
        anim: Anim::Aucune,
    },
    Objet {
        cle: "algue_lumineuse",
        nom: "Algue lumineuse",
        prix: 600_000.0,
        anim: Anim::Ondule,
    },
    Objet {
        cle: "ancre",
        nom: "Ancre",
        prix: 4e6,
        anim: Anim::Aucune,
    },
    Objet {
        cle: "corail_rare",
        nom: "Corail rare",
        prix: 3e7,
        anim: Anim::Lueur,
    },
    Objet {
        cle: "casque",
        nom: "Casque de scaphandrier",
        prix: 2e8,
        anim: Anim::Aucune,
    },
    Objet {
        cle: "coffre",
        nom: "Coffre au trésor",
        prix: 1.5e9,
        anim: Anim::Bulles,
    },
    Objet {
        cle: "cristaux",
        nom: "Cristaux luisants",
        prix: 1e10,
        anim: Anim::Lueur,
    },
    Objet {
        cle: "epave",
        nom: "Barque engloutie",
        prix: 8e10,
        anim: Anim::Aucune,
    },
    Objet {
        cle: "geyser",
        nom: "Geyser à bulles",
        prix: 5e11,
        anim: Anim::Geyser,
    },
    Objet {
        cle: "statue",
        nom: "Statue de la déesse",
        prix: 4e12,
        anim: Anim::Aucune,
    },
    Objet {
        cle: "chateau",
        nom: "Château englouti",
        prix: 3e13,
        anim: Anim::Aucune,
    },
];

pub fn objet(cle: &str) -> Option<&'static Objet> {
    OBJETS.iter().find(|o| o.cle == cle)
}

pub fn index_objet(cle: &str) -> Option<usize> {
    OBJETS.iter().position(|o| o.cle == cle)
}

pub struct Decor {
    pub themes: Vec<Theme>,
    /// Images des objets de la boutique, dans l'ordre de `OBJETS`.
    pub objets: Vec<Img>,
}

macro_rules! img {
    ($f:literal) => {
        Img::charger(include_bytes!(concat!("../assets/decor/", $f, ".png")))
    };
}

pub fn decor() -> &'static Decor {
    static D: OnceLock<Decor> = OnceLock::new();
    D.get_or_init(|| {
        let algues_recif = || vec![img!("algue_a"), img!("algue_b")];
        let theme = |sable, grands, petits, algues, lointain| Theme {
            sable,
            grands,
            petits,
            algues,
            lointain,
            lueur: false,
            algues_nb: (2, 3, 3),
            algues_haut: (0.12, 0.35),
            algues_entre: false,
        };
        // Cimetière : sable pâle et gris, cendré.
        let mut sable_gris = img!("sable");
        for p in sable_gris.px.iter_mut().filter(|p| p[3] > 0) {
            let g = ((p[0] as u32 * 3 + p[1] as u32 * 5 + p[2] as u32 * 2) / 10) as u8;
            let c = melange([p[0], p[1], p[2]], [g, g, g.saturating_add(6)], 0.75);
            *p = [c[0], c[1], c[2], p[3]];
        }
        let themes = vec![
            theme(
                img!("sable"),
                vec![img!("rochers_a"), img!("rochers_b")],
                vec![
                    img!("corail_branche"),
                    img!("corail_eventail"),
                    img!("corail_cerveau"),
                ],
                algues_recif(),
                img!("lointain"),
            ),
            theme(
                img!("sable"),
                vec![img!("epave/grand_a"), img!("epave/grand_b")],
                vec![
                    img!("epave/petit_a"),
                    img!("epave/petit_c"),
                    img!("epave/petit_b"),
                ],
                algues_recif(),
                img!("epave/lointain"),
            ),
            theme(
                img!("sable"),
                vec![img!("ruines/grand_a"), img!("ruines/grand_b")],
                vec![
                    img!("ruines/petit_a"),
                    img!("ruines/petit_b"),
                    img!("ruines/petit_c"),
                ],
                algues_recif(),
                img!("ruines/lointain"),
            ),
            Theme {
                lueur: true,
                // Vers géants des sources : peu nombreux, courts.
                algues_nb: (1, 1, 1),
                algues_haut: (0.08, 0.16),
                ..theme(
                    img!("volcan/sable"),
                    vec![img!("volcan/grand_a"), img!("volcan/grand_b")],
                    vec![
                        img!("volcan/petit_b"),
                        img!("volcan/petit_c"),
                        img!("volcan/petit_a"),
                    ],
                    vec![img!("volcan/algue")],
                    img!("volcan/lointain"),
                )
            },
            Theme {
                // Forêt : beaucoup de grandes algues, partout.
                algues_nb: (3, 2, 2),
                algues_haut: (0.25, 0.45),
                algues_entre: true,
                ..theme(
                    img!("sable"),
                    vec![img!("kelp/grand_a"), img!("kelp/grand_b")],
                    vec![
                        img!("kelp/petit_c"),
                        img!("kelp/petit_b"),
                        img!("kelp/petit_a"),
                    ],
                    vec![img!("kelp/algue_a"), img!("kelp/algue_b")],
                    img!("kelp/lointain"),
                )
            },
            Theme {
                algues_nb: (1, 1, 1),
                ..theme(
                    sable_gris,
                    vec![img!("cimetiere/grand_a"), img!("cimetiere/grand_b")],
                    vec![
                        img!("cimetiere/petit_b"),
                        img!("cimetiere/petit_c"),
                        img!("cimetiere/petit_a"),
                    ],
                    algues_recif(),
                    img!("cimetiere/lointain"),
                )
            },
        ];
        let objets = vec![
            img!("objets/cailloux"),
            img!("objets/crane"),
            img!("objets/anemone"),
            img!("objets/amphores"),
            img!("objets/algue_lumineuse"),
            img!("objets/ancre"),
            img!("objets/corail_rare"),
            img!("objets/casque"),
            img!("objets/coffre"),
            img!("objets/cristaux"),
            img!("objets/epave"),
            img!("objets/geyser"),
            img!("objets/statue"),
            img!("objets/chateau"),
        ];
        Decor { themes, objets }
    })
}

/// Thème du réglage (index hors limite : récif).
pub fn theme(i: usize) -> &'static Theme {
    let d = decor();
    &d.themes[i.min(d.themes.len() - 1)]
}

/// Pixels qui luisent (fissures de lave, cristaux, bouts d'algue lumineuse) : vifs et saturés.
/// Au plus ~260 par image (un sur n).
pub fn points_lumineux(img: &Img) -> Vec<(i32, i32, Rgb)> {
    let mut v = Vec::new();
    for y in 0..img.h {
        for x in 0..img.l {
            let p = img.px(x, y);
            let (_, s, val) = crate::toile::rgb_hsv([p[0], p[1], p[2]]);
            if p[3] > 0 && val > 0.72 && s > 0.45 {
                v.push((x, y, [p[0], p[1], p[2]]));
            }
        }
    }
    let pas = v.len().div_ceil(260).max(1);
    v.into_iter().step_by(pas).collect()
}

/// Couleur du décor fondue dans l'eau (`k` = part de l'eau) : décor et créatures forment une
/// seule illustration.
pub fn dans_l_eau(c: [u8; 4], eau: Rgb, k: f32) -> Rgb {
    melange([c[0], c[1], c[2]], eau, k)
}

/// Sous la ligne du sable (pixels fins, par colonne fine) ? Hors de la toile : oui.
fn sous_le_sable(sol: &[i32], x: i32, y: i32) -> bool {
    x < 0 || sol.get(x as usize).is_none_or(|&s| y >= s)
}

/// Pose une image en pixels fins, coin haut gauche (x0, y0), retournée si `miroir`. Ce qui
/// passe sous la ligne du sable `sol` est caché : le bas est enfoui.
pub fn poser(
    t: &mut Toile,
    img: &Img,
    (x0, y0): (i32, i32),
    miroir: bool,
    eau: (Rgb, f32),
    sol: &[i32],
) {
    for y in 0..img.h {
        for x in 0..img.l {
            let p = img.px(if miroir { img.l - 1 - x } else { x }, y);
            if p[3] > 0 && !sous_le_sable(sol, x0 + x, y0 + y) {
                t.poser_fin(x0 + x, y0 + y, dans_l_eau(p, eau.0, eau.1), 255);
            }
        }
    }
}

/// Ombre de contact sur le sable au pied d'un élément large de `l` pixels fins centré en `cx`.
pub fn ombre(t: &mut Toile, cx: i32, l: i32, sol: &[i32]) {
    let r = (l / 2).max(2) as f32;
    for dx in -(r as i32)..=(r as i32) {
        let x = cx + dx;
        let Some(&s) = sol.get(x.max(0) as usize) else {
            continue;
        };
        if s == i32::MAX {
            continue;
        }
        // v0.8 : ombre de contact plus marquée (la relecture la voyait à peine).
        let k = 1.0 - (dx as f32 / r).powi(2);
        // Bande sombre pleine sur 4 lignes puis fondu : visible sous chaque élément à 1080p.
        for dy in -2..9 {
            let a = (165.0 * k.sqrt() * (1.0 - (dy - 2).max(0) as f32 / 7.0)) as u8;
            if a > 0 {
                t.poser_fin(x, s + dy, [0x08, 0x0e, 0x16], a);
            }
        }
    }
}

/// Algue qui ondule : chaque ligne décalée d'une vague qui grandit vers le haut (le pied ne
/// bouge pas), `base` = bas de l'algue en pixels fins. Seules les `visible` lignes du haut
/// sortent du sable (algues de hauteurs variées sans réduire le dessin).
#[allow(clippy::too_many_arguments)]
pub fn algue(
    t: &mut Toile,
    img: &Img,
    (cx, base): (i32, i32),
    visible: i32,
    temps: f32,
    phase: f32,
    eau: (Rgb, f32),
    miroir: bool,
    sol: &[i32],
    lueur: bool,
) {
    let h = visible.clamp(1, img.h) as f32;
    // Algue lumineuse : ses pixels vifs pulsent et rayonnent un peu.
    let pouls = 0.5 + 0.5 * (temps * 1.9 + phase).sin();
    let debut = img.h - visible.clamp(1, img.h);
    for y in debut..img.h {
        let f = (img.h - 1 - y) as f32 / h;
        let dx =
            ((temps * 0.9 + phase + y as f32 * 0.045).sin() * 7.0 * f.powf(1.5)).round() as i32;
        for x in 0..img.l {
            let p = img.px(if miroir { img.l - 1 - x } else { x }, y);
            if p[3] > 0 {
                let (xx, yy) = (cx - img.l / 2 + x + dx, base - img.h + y);
                if !sous_le_sable(sol, xx, yy) {
                    t.poser_fin(xx, yy, dans_l_eau(p, eau.0, eau.1), 255);
                    if lueur {
                        let (_, s, v) = crate::toile::rgb_hsv([p[0], p[1], p[2]]);
                        if v > 0.7 && s > 0.4 {
                            let c = [p[0], p[1], p[2]];
                            t.eclairer_fin(xx, yy, c, (80.0 + 150.0 * pouls) as u8);
                            if (xx + yy).rem_euclid(2) == 0 {
                                for (dx, dy) in [(2, 0), (-2, 0), (0, 2), (0, -2)] {
                                    t.eclairer_fin(
                                        xx + dx,
                                        yy + dy,
                                        c,
                                        (25.0 + 45.0 * pouls) as u8,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

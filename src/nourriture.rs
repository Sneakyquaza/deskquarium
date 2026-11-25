//! Nourriture (v0.16) : sept nourritures achetées dans l'Écloserie, de plus en plus chères, puis
//! jetées dans l'aquarium. Elles coulent jusqu'au sable ; les poissons intéressés viennent les
//! manger et le repas donne un bonus temporaire (revenu, et chance aux éclosions pour les
//! meilleures). Données, règle du bonus et chute des morceaux : logique pure, testée.
//! Dessins IA nettoyés (`tools/nourriture_assets.sh`) : `assets/nourriture/<clé>.png` (dans
//! l'eau) et `<clé>_icone.png` (boutique).

use crate::decor::Img;
use crate::especes::Taille;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub struct Nourriture {
    /// Clé stable (images).
    pub cle: &'static str,
    pub nom: &'static str,
    pub description: &'static str,
    pub prix: f64,
    /// Revenu multiplié pendant le bonus.
    pub mult: f64,
    /// Durée du bonus d'un repas entier (secondes).
    pub duree: f64,
    /// Chances de shiny et de modificateur multipliées aux éclosions pendant le bonus.
    pub chance: f64,
    /// Morceaux par jet (une pincée de flocons s'éparpille) et bouchées par morceau.
    pub morceaux: u32,
    pub bouchees: u32,
    /// Vitesse de chute (pixels monde par seconde).
    pub chute: f32,
    /// Tailles des poissons attirés.
    pub tailles: (Taille, Taille),
}

/// Bonus au plus `PLAFOND` fois la durée d'un repas de la nourriture active.
pub const PLAFOND: f64 = 3.0;
/// Exemplaires au plus d'une nourriture en réserve.
pub const MAX_STOCK: u32 = 999;
/// Un morceau posé sur le sable disparaît après ce temps s'il n'est pas mangé (secondes).
pub const DUREE_SUR_LE_SABLE: f32 = 25.0;

/// Prix ≈ revenu « rentable » × (mult − 1) × durée : chaque nourriture devient intéressante
/// à son étape (≈ 0,5/s, 20/s, 500/s, 10 k/s, 150 k/s, 2 M/s, 30 M/s de revenu ; voir
/// `docs/equilibrage.md`).
pub const NOURRITURES: [Nourriture; 7] = [
    Nourriture {
        cle: "flocons",
        nom: "Flocons",
        description: "Une pincée de flocons qui s'éparpille. Pour les petits poissons.",
        prix: 15.0,
        mult: 1.5,
        duree: 60.0,
        chance: 1.0,
        morceaux: 6,
        bouchees: 1,
        chute: 5.0,
        tailles: (Taille::Minuscule, Taille::Moyenne),
    },
    Nourriture {
        cle: "granules",
        nom: "Granulés",
        description: "Petites billes nourrissantes qui coulent vite.",
        prix: 1_500.0,
        mult: 1.75,
        duree: 90.0,
        chance: 1.0,
        morceaux: 4,
        bouchees: 1,
        chute: 9.0,
        tailles: (Taille::Petite, Taille::Grande),
    },
    Nourriture {
        cle: "vers",
        nom: "Vers de vase",
        description: "Des vers rouges qui se tortillent. Personne n'y résiste.",
        prix: 60_000.0,
        mult: 2.0,
        duree: 120.0,
        chance: 1.0,
        morceaux: 3,
        bouchees: 1,
        chute: 7.0,
        tailles: (Taille::Petite, Taille::Grande),
    },
    Nourriture {
        cle: "crevette",
        nom: "Crevette",
        description: "Une belle crevette entière, en deux bouchées.",
        prix: 2_500_000.0,
        mult: 2.5,
        duree: 150.0,
        chance: 1.0,
        morceaux: 1,
        bouchees: 2,
        chute: 11.0,
        tailles: (Taille::Moyenne, Taille::Enorme),
    },
    Nourriture {
        cle: "calamar",
        nom: "Morceau de calamar",
        description: "Pour les grands poissons. Chance aux éclosions ×1,5.",
        prix: 50_000_000.0,
        mult: 3.0,
        duree: 180.0,
        chance: 1.5,
        morceaux: 1,
        bouchees: 3,
        chute: 13.0,
        tailles: (Taille::Moyenne, Taille::Enorme),
    },
    Nourriture {
        cle: "cage",
        nom: "Cage à requins",
        description:
            "Une cage lestée pleine de viande. Elle attire les gros prédateurs. Chance ×2.",
        prix: 1_500_000_000.0,
        mult: 4.0,
        duree: 240.0,
        chance: 2.0,
        morceaux: 1,
        bouchees: 4,
        chute: 22.0,
        tailles: (Taille::Grande, Taille::Colossale),
    },
    Nourriture {
        cle: "festin",
        nom: "Festin abyssal",
        description: "Un mets qui luit, venu des grands fonds. Tout l'aquarium accourt. Chance ×3.",
        prix: 50_000_000_000.0,
        mult: 6.0,
        duree: 300.0,
        chance: 3.0,
        morceaux: 1,
        bouchees: 5,
        chute: 4.5,
        tailles: (Taille::Minuscule, Taille::Colossale),
    },
];

/// Bonus du repas en cours : nourriture (index de `NOURRITURES`) et secondes restantes.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Festin {
    pub niveau: u8,
    pub reste: f64,
}

impl Festin {
    pub fn nourriture(&self) -> &'static Nourriture {
        &NOURRITURES[(self.niveau as usize).min(NOURRITURES.len() - 1)]
    }
}

/// Une bouchée de la nourriture `n` (`part` = part du repas entier, 0..1). Règle : un seul
/// bonus à la fois, celui de la meilleure nourriture ; la valeur (mult − 1) × secondes se
/// conserve : une nourriture moins bonne prolonge le bonus en cours à proportion, une
/// meilleure le remplace et reprend le temps restant converti. Jamais plus de `PLAFOND` repas
/// entiers de la nourriture active (pas d'explosion en empilant).
pub fn ajouter(f: Option<Festin>, n: usize, part: f64) -> Option<Festin> {
    let Some(nn) = NOURRITURES.get(n) else {
        return f;
    };
    let gain = nn.duree * part.clamp(0.0, 1.0);
    let (niveau, reste) = match f {
        None => (n, gain),
        Some(a) => {
            let an = a.nourriture();
            let k = (an.mult - 1.0) / (nn.mult - 1.0);
            if n > a.niveau as usize {
                (n, gain + a.reste * k)
            } else {
                (a.niveau as usize, a.reste + gain / k)
            }
        }
    };
    let plafond = NOURRITURES[niveau].duree * PLAFOND;
    Some(Festin {
        niveau: niveau as u8,
        reste: reste.min(plafond),
    })
}

/// Morceau de nourriture dans l'eau (pixels monde, centre).
#[derive(Clone, Debug)]
pub struct Morceau {
    pub n: usize,
    pub x: f32,
    pub y: f32,
    pub vy: f32,
    pub phase: f32,
    /// Temps passé sur le sable (négatif : il tombe encore).
    pub pose: f32,
    pub bouchees: u32,
    /// Éclat d'une bouchée (secondes restantes).
    pub croque: f32,
    /// Temps depuis le jet (les poissons la remarquent après `REMARQUEE`).
    pub age: f32,
}

/// Les poissons remarquent la nourriture après ce temps (on la voit d'abord tomber).
pub const REMARQUEE: f32 = 1.5;

impl Morceau {
    pub fn new(n: usize, x: f32, y: f32, phase: f32) -> Morceau {
        Morceau {
            n,
            x,
            y,
            vy: 0.0,
            phase,
            pose: -1.0,
            bouchees: NOURRITURES[n].bouchees,
            croque: 0.0,
            age: 0.0,
        }
    }

    pub fn nourriture(&self) -> &'static Nourriture {
        &NOURRITURES[self.n]
    }

    /// Chute : accélère jusqu'à sa vitesse, dérive en se balançant ; posé quand son bas touche
    /// `sol` (dessus du sable sous lui). `t` : horloge de l'aquarium, `demi_h` : demi-hauteur.
    pub fn avancer(&mut self, dt: f32, t: f32, sol: f32, demi_h: f32) {
        self.croque = (self.croque - dt).max(0.0);
        self.age += dt;
        if self.pose >= 0.0 {
            self.pose += dt;
            return;
        }
        let v = self.nourriture().chute;
        self.vy += (v - self.vy) * (dt * 2.5).min(1.0);
        self.y += self.vy * dt;
        // Les légers dérivent plus (flocons), les lourds presque pas (cage).
        let derive = (6.0 / v).min(1.2) * 3.0;
        self.x += (t * 1.7 + self.phase).sin() * derive * dt;
        if self.y + demi_h >= sol {
            self.y = sol - demi_h;
            self.pose = 0.0;
        }
    }

    /// Mangé entièrement, ou resté trop longtemps sur le sable.
    pub fn fini(&self) -> bool {
        self.bouchees == 0 || self.pose >= DUREE_SUR_LE_SABLE
    }
}

/// Images de la nourriture : dans l'eau et icône de boutique, dans l'ordre de `NOURRITURES`
/// (la cage est déjà assez grande : son dessin sert d'icône).
pub struct Images {
    pub eau: Vec<Img>,
    pub icones: Vec<Img>,
}

macro_rules! img {
    ($f:literal) => {
        Img::charger(include_bytes!(concat!("../assets/nourriture/", $f, ".png")))
    };
}

pub fn images() -> &'static Images {
    static I: OnceLock<Images> = OnceLock::new();
    I.get_or_init(|| Images {
        eau: vec![
            img!("flocons"),
            img!("granules"),
            img!("vers"),
            img!("crevette"),
            img!("calamar"),
            img!("cage"),
            img!("festin"),
        ],
        icones: vec![
            img!("flocons_icone"),
            img!("granules_icone"),
            img!("vers_icone"),
            img!("crevette_icone"),
            img!("calamar_icone"),
            img!("cage"),
            img!("festin_icone"),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prix_et_bonus_croissants() {
        for w in NOURRITURES.windows(2) {
            assert!(w[1].prix > w[0].prix * 10.0, "{}", w[1].cle);
            assert!(w[1].mult > w[0].mult, "{}", w[1].cle);
            assert!(w[1].duree >= w[0].duree, "{}", w[1].cle);
            assert!(w[1].chance >= w[0].chance, "{}", w[1].cle);
        }
        for n in &NOURRITURES {
            assert!(n.morceaux >= 1 && n.bouchees >= 1 && n.chute > 0.0);
            assert!(n.tailles.0 <= n.tailles.1);
        }
    }

    #[test]
    fn bonus_sans_explosion() {
        // Premier repas : sa durée.
        let f = ajouter(None, 2, 1.0).unwrap();
        assert_eq!((f.niveau, f.reste), (2, 120.0));
        // Même nourriture : prolonge, plafonné à 3 repas.
        let f = (0..10)
            .try_fold(f, |f, _| ajouter(Some(f), 2, 1.0))
            .unwrap();
        assert_eq!((f.niveau, f.reste), (2, 360.0));
        // Une moins bonne prolonge à proportion de sa valeur : flocons ×1,5 pendant 60 s =
        // 30 s de vers ×2.
        let f = ajouter(
            Some(Festin {
                niveau: 2,
                reste: 10.0,
            }),
            0,
            1.0,
        )
        .unwrap();
        assert_eq!(f.niveau, 2);
        assert!((f.reste - 40.0).abs() < 1e-9);
        // Une meilleure remplace et reprend le reste converti : 60 s de vers (×2) = 30 s de
        // crevette (×2,5) ; jamais de multiplicateurs empilés.
        let f = ajouter(
            Some(Festin {
                niveau: 2,
                reste: 45.0,
            }),
            3,
            0.5,
        )
        .unwrap();
        assert_eq!(f.niveau, 3);
        assert!((f.reste - (75.0 + 30.0)).abs() < 1e-9);
        // Morceaux : une pincée de 6 flocons mangée en entier = un repas.
        let f = (0..6).fold(None, |f, _| ajouter(f, 0, 1.0 / 6.0)).unwrap();
        assert!((f.reste - 60.0).abs() < 1e-9);
        // Nourriture inconnue : rien ne change.
        assert_eq!(ajouter(None, 99, 1.0), None);
    }

    #[test]
    fn morceau_coule_se_pose_puis_disparait() {
        let mut m = Morceau::new(0, 50.0, 10.0, 0.3);
        let (sol, dh) = (100.0, 2.0);
        let mut t = 0.0;
        let mut x_min = f32::MAX;
        let mut x_max = f32::MIN;
        while m.pose < 0.0 {
            m.avancer(1.0 / 30.0, t, sol, dh);
            t += 1.0 / 30.0;
            x_min = x_min.min(m.x);
            x_max = x_max.max(m.x);
            assert!(t < 60.0, "ne se pose jamais");
        }
        // Chute à sa vitesse (~5 px/s : 88 px en ~18 s), dérive douce, posé sur le sable.
        assert!((15.0..25.0).contains(&t), "{t}");
        assert!(x_max - x_min > 0.5 && x_max - x_min < 8.0);
        assert_eq!(m.y, sol - dh);
        assert!(!m.fini());
        let y = m.y;
        for _ in 0..(30.0 * (DUREE_SUR_LE_SABLE - 1.0)) as usize {
            m.avancer(1.0 / 30.0, t, sol, dh);
        }
        assert_eq!(m.y, y, "reste posé");
        assert!(!m.fini());
        for _ in 0..60 {
            m.avancer(1.0 / 30.0, t, sol, dh);
        }
        assert!(m.fini(), "disparaît s'il n'est pas mangé");
        // La cage coule bien plus vite.
        let mut c = Morceau::new(5, 50.0, 10.0, 0.0);
        let mut tc = 0.0;
        while c.pose < 0.0 {
            c.avancer(1.0 / 30.0, tc, sol, 20.0);
            tc += 1.0 / 30.0;
        }
        assert!(tc < 5.0, "{tc}");
        // Plusieurs bouchées pour la finir.
        assert_eq!(c.bouchees, 4);
    }
}

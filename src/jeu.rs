//! Logique du jeu idle : raretés, œufs, poissons, revenus, gains hors ligne, format des nombres.
//! Toutes les valeurs d'équilibrage sont ici (courbes documentées dans `docs/equilibrage.md`).

use crate::nourriture::{Festin, MAX_STOCK, NOURRITURES, PLAFOND};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Poissons qui nagent (et rapportent) en même temps ; au-delà, ils vont dans la réserve.
pub const CAPACITE: usize = 40;
/// Plafond des gains hors ligne (8 h).
pub const HORS_LIGNE_MAX_S: f64 = 8.0 * 3600.0;
pub const NIVEAU_MAX: u32 = 10;
/// Plafond de l'argent : évite l'infini (le JSON n'accepte pas `inf`).
const ARGENT_MAX: f64 = 1e300;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Rarete {
    Commun,
    PeuCommun,
    Rare,
    Epique,
    Legendaire,
    Mythique,
    Celeste,
    Irreel,
    /// Lot 3 : créatures disparues, seulement dans l'œuf d'ambre (au-dessus d'Irréel).
    Prehistorique,
}

impl Rarete {
    pub const TOUTES: [Rarete; 9] = [
        Rarete::Commun,
        Rarete::PeuCommun,
        Rarete::Rare,
        Rarete::Epique,
        Rarete::Legendaire,
        Rarete::Mythique,
        Rarete::Celeste,
        Rarete::Irreel,
        Rarete::Prehistorique,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn nom(self) -> &'static str {
        crate::i18n::tr(self.nom_fr())
    }

    /// Nom français (clé de traduction).
    pub fn nom_fr(self) -> &'static str {
        [
            "Commun",
            "Peu commun",
            "Rare",
            "Épique",
            "Légendaire",
            "Mythique",
            "Céleste",
            "Irréel",
            "Préhistorique",
        ][self.index()]
    }

    /// Couleur de la rareté (texte, cadres, lueurs).
    pub fn couleur(self) -> [u8; 3] {
        [
            [0xb4, 0xbc, 0xc4],
            [0x6c, 0xd8, 0x6a],
            [0x4c, 0xa0, 0xff],
            [0xb8, 0x6c, 0xff],
            [0xff, 0xc8, 0x45],
            [0xff, 0x52, 0x6e],
            [0x72, 0xf2, 0xe6],
            [0xff, 0x8c, 0xf0],
            [0xe8, 0x9a, 0x3a],
        ][self.index()]
    }

    /// Revenu par seconde d'un poisson de taille 1 au niveau 1.
    pub fn revenu_base(self) -> f64 {
        [
            1.0, 4.0, 16.0, 70.0, 320.0, 1_500.0, 7_500.0, 40_000.0, 220_000.0,
        ][self.index()]
    }
}

pub struct Oeuf {
    pub nom: &'static str,
    pub description: &'static str,
    pub prix: f64,
    /// Probabilités (en %) par rareté, dans l'ordre de [`Rarete::TOUTES`] ; somme = 100.
    pub chances: [f64; 9],
    /// Couleurs de la coquille (base, taches).
    pub coquille: [[u8; 3]; 2],
    /// v0.10 : chances de shiny et de modificateur multipliées (œufs exceptionnels).
    pub bonus_shiny: f64,
    pub bonus_modif: f64,
    /// v0.10 : œuf exceptionnel disponible par moments (heure UTC) : période, début dans la
    /// période, durée (secondes). None = toujours là.
    pub dispo: Option<(u64, u64, u64)>,
    /// v0.11 : pitié : au plus N œufs de suite sans cette rareté (ou mieux) ; le N-ième la
    /// garantit. None = l'œuf donne déjà presque toujours ses meilleures raretés.
    pub pitie: Option<(Rarete, u32)>,
}

impl Oeuf {
    /// Disponible à l'instant `maintenant` (secondes Unix) ?
    pub fn disponible(&self, maintenant: u64) -> bool {
        self.dispo
            .is_none_or(|(periode, debut, duree)| (maintenant + periode - debut) % periode < duree)
    }

    /// Œuf exceptionnel : secondes avant la fin (disponible) ou avant le retour (absent).
    pub fn attente(&self, maintenant: u64) -> Option<u64> {
        let (periode, debut, duree) = self.dispo?;
        let dans = (maintenant + periode - debut) % periode;
        Some(if dans < duree {
            duree - dans
        } else {
            periode - dans
        })
    }
}

const H: u64 = 3600;
/// Chance qu'un œuf ordinaire acheté devienne un œuf doré (meilleures chances, v0.10).
pub const CHANCE_OEUF_DORE: f64 = 0.03;

pub const OEUFS: [Oeuf; 8] = [
    Oeuf {
        nom: "Œuf de rivière",
        description: "Pondu entre deux galets. Souvent des poissons communs.",
        prix: 80.0,
        chances: [70.0, 25.0, 4.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
        coquille: [[0xd9, 0xe8, 0xc8], [0x7f, 0xa8, 0x6a]],
        bonus_shiny: 1.0,
        bonus_modif: 1.0,
        dispo: None,
        pitie: Some((Rarete::Epique, 40)),
    },
    Oeuf {
        nom: "Œuf de lagon",
        description: "Tiède et nacré, il sent le corail.",
        prix: 5_000.0,
        chances: [30.0, 42.0, 20.0, 6.5, 1.5, 0.0, 0.0, 0.0, 0.0],
        coquille: [[0xa8, 0xee, 0xf0], [0xff, 0xb0, 0xc8]],
        bonus_shiny: 1.0,
        bonus_modif: 1.0,
        dispo: None,
        pitie: Some((Rarete::Epique, 25)),
    },
    Oeuf {
        nom: "Œuf des abysses",
        description: "Remonté de très loin. Il luit faiblement dans le noir.",
        prix: 1_000_000.0,
        chances: [0.0, 20.0, 42.0, 26.0, 9.5, 2.5, 0.0, 0.0, 0.0],
        coquille: [[0x2c, 0x3c, 0x6e], [0x5c, 0xf0, 0xd8]],
        bonus_shiny: 1.0,
        bonus_modif: 1.0,
        dispo: None,
        pitie: Some((Rarete::Legendaire, 25)),
    },
    Oeuf {
        nom: "Œuf cosmique",
        description: "Tombé du ciel une nuit d'étoiles, encore tiède.",
        prix: 500_000_000.0,
        chances: [0.0, 0.0, 22.0, 36.0, 26.0, 11.0, 4.5, 0.5, 0.0],
        coquille: [[0x30, 0x1c, 0x58], [0xff, 0xe8, 0x9c]],
        bonus_shiny: 1.0,
        bonus_modif: 1.0,
        dispo: None,
        pitie: Some((Rarete::Mythique, 25)),
    },
    Oeuf {
        nom: "Œuf d'ambre",
        description: "Pris dans l'ambre depuis des millions d'années. Quelque chose bouge dedans.",
        prix: 30_000_000_000.0,
        chances: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 12.0, 8.0, 80.0],
        coquille: [[0xe8, 0x9a, 0x3a], [0x7a, 0x46, 0x1c]],
        bonus_shiny: 1.0,
        bonus_modif: 1.0,
        dispo: None,
        pitie: None,
    },
    // v0.10 : œufs exceptionnels, par moments (heure UTC).
    Oeuf {
        nom: "Œuf doré",
        description: "Deux heures toutes les huit heures. Shiny ×5, modificateurs ×4.",
        prix: 250_000.0,
        chances: [10.0, 30.0, 32.0, 18.0, 8.0, 2.0, 0.0, 0.0, 0.0],
        coquille: [[0xff, 0xcc, 0x40], [0xb0, 0x60, 0x10]],
        bonus_shiny: 5.0,
        bonus_modif: 4.0,
        dispo: Some((8 * H, 0, 2 * H)),
        pitie: Some((Rarete::Legendaire, 20)),
    },
    Oeuf {
        nom: "Œuf arc-en-ciel",
        description: "Deux heures toutes les huit heures. Modificateurs ×8, shiny ×2.",
        prix: 50_000_000.0,
        chances: [0.0, 10.0, 30.0, 32.0, 18.0, 8.0, 2.0, 0.0, 0.0],
        coquille: [[0xff, 0x9c, 0xd8], [0x7c, 0xe0, 0xff]],
        bonus_shiny: 2.0,
        bonus_modif: 8.0,
        dispo: Some((8 * H, 4 * H, 2 * H)),
        pitie: Some((Rarete::Mythique, 20)),
    },
    Oeuf {
        nom: "Œuf de l'Éclipse",
        description: "Une heure par jour, à la nuit tombée. Créatures mythiques, shiny ×10.",
        prix: 5_000_000_000.0,
        chances: [0.0, 0.0, 0.0, 10.0, 30.0, 30.0, 20.0, 8.0, 2.0],
        coquille: [[0x20, 0x18, 0x28], [0xff, 0xc0, 0x40]],
        bonus_shiny: 10.0,
        bonus_modif: 3.0,
        dispo: Some((24 * H, 22 * H, H)),
        pitie: Some((Rarete::Irreel, 20)),
    },
];

/// Chance d'un shiny à l'éclosion (tirée à part du modificateur).
pub const CHANCE_SHINY: f64 = 1.0 / 100.0;
/// Revenu d'un shiny.
pub const MULT_SHINY: f64 = 5.0;
/// Revenu en plus, pour toujours, par index complété (`Partie::index_complets`).
pub const BONUS_INDEX: f64 = 0.05;

/// Modificateur d'un poisson (au plus un, tiré à l'éclosion, à part du shiny) : aura,
/// revenu et petit changement du corps (`poisson::sprite_forme`, `scene.rs`).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Modif {
    Dore,
    Diamant,
    Electrique,
    ArcEnCiel,
    Ombre,
    Cristal,
    Galaxie,
}

impl Modif {
    pub const TOUS: [Modif; 7] = [
        Modif::Dore,
        Modif::Diamant,
        Modif::Electrique,
        Modif::ArcEnCiel,
        Modif::Ombre,
        Modif::Cristal,
        Modif::Galaxie,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn nom(self) -> &'static str {
        crate::i18n::tr(self.nom_fr())
    }

    /// Nom français (clé de traduction, et `--forme galaxie` des captures).
    pub fn nom_fr(self) -> &'static str {
        [
            "Doré",
            "Diamant",
            "Électrique",
            "Arc-en-ciel",
            "Ombre",
            "Cristal",
            "Galaxie",
        ][self.index()]
    }

    /// Chance à l'éclosion (1/20 … 1/1000).
    pub fn chance(self) -> f64 {
        1.0 / [20.0, 60.0, 120.0, 150.0, 250.0, 400.0, 1000.0][self.index()]
    }

    pub fn multiplicateur(self) -> f64 {
        [1.5, 2.5, 3.0, 4.0, 5.0, 7.0, 12.0][self.index()]
    }

    /// Couleur du badge et de l'aura.
    pub fn couleur(self) -> [u8; 3] {
        [
            [0xff, 0xcc, 0x40],
            [0xc8, 0xf4, 0xff],
            [0x7c, 0xf0, 0xff],
            [0xff, 0x7c, 0xd8],
            [0xb8, 0x98, 0xe0],
            [0xd0, 0xc0, 0xff],
            [0x9c, 0x7c, 0xff],
        ][self.index()]
    }
}

/// Forme d'un poisson : shiny et modificateur (revenu et dessin).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct Forme {
    pub shiny: bool,
    pub modif: Option<Modif>,
}

impl Forme {
    /// Tirage à l'éclosion, à partir de la graine du poisson (flux à part : le tirage des
    /// raretés et des espèces ne change pas). Shiny et modificateur sont indépendants.
    #[cfg(test)]
    pub fn tirer(graine: u64) -> Forme {
        Forme::tirer_bonus(graine, 1.0, 1.0)
    }

    /// Tirage avec chances multipliées (œufs exceptionnels, œuf doré ; v0.10). Bonus 1 = même
    /// résultat que `tirer` pour la même graine.
    pub fn tirer_bonus(graine: u64, shiny: f64, modif: f64) -> Forme {
        let mut a = Alea(graine ^ 0x5f1e_77a3_c0de_0008);
        let shiny = a.f64() < CHANCE_SHINY * shiny;
        let bonus = modif;
        let mut x = a.f64();
        let mut modif = None;
        for m in Modif::TOUS {
            x -= m.chance() * bonus;
            if x < 0.0 {
                modif = Some(m);
                break;
            }
        }
        Forme { shiny, modif }
    }

    /// Multiplicateur moyen d'un poisson éclos (shiny × modificateur, ≈ 1,17).
    pub fn esperance() -> f64 {
        let modif: f64 = 1.0
            + Modif::TOUS
                .iter()
                .map(|m| m.chance() * (m.multiplicateur() - 1.0))
                .sum::<f64>();
        (1.0 + CHANCE_SHINY * (MULT_SHINY - 1.0)) * modif
    }

    pub fn multiplicateur(self) -> f64 {
        (if self.shiny { MULT_SHINY } else { 1.0 }) * self.modif.map_or(1.0, Modif::multiplicateur)
    }
}

/// Index (Poissodex) : 0 = Normal (espèce découverte), 1..=7 = un par modificateur, 8 = Shiny.
pub const NB_INDEX: usize = Modif::TOUS.len() + 2;

pub fn nom_index(k: usize) -> &'static str {
    match k {
        0 => crate::i18n::tr("Normal"),
        k if k <= Modif::TOUS.len() => Modif::TOUS[k - 1].nom(),
        _ => crate::i18n::tr("Shiny"),
    }
}

/// Index où compte un poisson de cette forme (bits de `Partie::index`).
fn bits_index(f: Forme) -> u16 {
    1 | f.modif.map_or(0, |m| 1 << (m.index() + 1)) | if f.shiny { 1 << (NB_INDEX - 1) } else { 0 }
}

/// Générateur pseudo-aléatoire SplitMix64 (déterministe, état = un u64 sauvegardé).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Alea(pub u64);

impl Alea {
    pub fn suivant(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Flottant uniforme dans [0, 1).
    pub fn f(&mut self) -> f32 {
        (self.suivant() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Flottant uniforme dans [a, b).
    pub fn entre(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }

    /// Flottant uniforme dans [0, 1), 53 bits (petites chances).
    pub fn f64(&mut self) -> f64 {
        (self.suivant() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Entier uniforme dans [0, n).
    pub fn n(&mut self, n: u32) -> u32 {
        (((self.suivant() >> 32) * n as u64) >> 32) as u32
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Poisson {
    pub id: u64,
    /// Graine du génome : le sprite est régénéré à partir d'elle.
    pub graine: u64,
    pub rarete: Rarete,
    /// Facteur de taille 0,6..1,5 (multiplie le revenu et la longueur du sprite).
    pub taille: f32,
    pub niveau: u32,
    /// Dans l'aquarium (nage, rapporte) ou dans la réserve.
    pub actif: bool,
    /// Index de l'œuf d'origine.
    #[serde(default)]
    pub oeuf: u8,
    /// Clé de l'espèce (`especes.rs`) ; vide dans les sauvegardes du lot 1 → `Partie::migrer`.
    #[serde(default)]
    pub espece: String,
    #[serde(default)]
    pub variante: u8,
    /// v0.8 : shiny et modificateur (anciennes sauvegardes : normal, pas shiny).
    #[serde(default)]
    pub shiny: bool,
    #[serde(default)]
    pub modif: Option<Modif>,
    /// v0.16 : verrouillé (cadenas) : ni vendu par « Tout vendre », ni relâché.
    #[serde(default)]
    pub verrou: bool,
}

impl Poisson {
    pub fn forme(&self) -> Forme {
        Forme {
            shiny: self.shiny,
            modif: self.modif,
        }
    }

    pub fn revenu(&self) -> f64 {
        self.rarete.revenu_base()
            * self.taille as f64
            * multiplicateur_niveau(self.niveau)
            * self.forme().multiplicateur()
    }

    /// Prix pour passer au niveau suivant (`None` au niveau max) : 120 s × 1,9^(n−1) du revenu
    /// de base (shiny et modificateur compris : un niveau se rentabilise pareil).
    pub fn cout_nourrir(&self) -> Option<f64> {
        (self.niveau < NIVEAU_MAX).then(|| {
            self.rarete.revenu_base()
                * self.taille as f64
                * self.forme().multiplicateur()
                * 120.0
                * 1.9f64.powi(self.niveau as i32 - 1)
        })
    }

    /// Argent rendu quand on relâche le poisson (20 s de son revenu).
    pub fn prix_relache(&self) -> f64 {
        (self.revenu() * 20.0).max(1.0)
    }
}

pub fn multiplicateur_niveau(niveau: u32) -> f64 {
    1.2f64.powi(niveau.max(1) as i32 - 1)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Reglages {
    /// Opacité de la teinte de l'eau, 0 = invisible, 1 = 50 % max.
    pub teinte: f32,
    /// Nombre max de poissons dessinés (tous ceux de l'aquarium rapportent quand même).
    pub max_affiches: usize,
    /// Images par seconde du fond : 15, 24, 30 ou 60 (v0.12 : 24 par défaut).
    pub ips: u32,
    pub demarrage_windows: bool,
    pub pause_plein_ecran: bool,
    /// Windows : aquarium dans une fenêtre sans bord au fond de la pile (au lieu de derrière
    /// les icônes), si l'incrustation sur le bureau ne marche pas.
    pub mode_fenetre: bool,
    /// Windows : méthode d'incrustation (0 auto, 1 Progman + calque, 2 Progman, 3 WorkerW).
    pub incrustation: u8,
    /// Un poisson éclos qui rapporte plus que le plus faible de l'aquarium plein le remplace.
    pub equiper_auto: bool,
    /// Mode Abysses : aquarium sombre des grands fonds (choix manuel).
    pub abysses: bool,
    /// v0.17 : Abysses automatique la nuit (21 h – 7 h, heure locale) ; ignore `abysses`.
    pub abysses_auto: bool,
    /// v0.17 : il fait nuit (relu une fois par minute par l'appli, pas sauvegardé).
    #[serde(skip)]
    pub nuit: bool,
    /// v0.9 : décor à thème (index de `decor::THEMES`, 0 = récif corallien).
    pub theme: u8,
    /// v0.9 (Windows) : attraper un poisson au bureau (clic maintenu sur un poisson).
    pub attraper: bool,
    /// v0.9 : attraper seulement en tenant Alt (sécurité).
    pub attraper_alt: bool,
    /// v0.11 : sons de l'Écloserie (le fond d'écran reste muet) : volume 0..1, coupé.
    pub volume: f32,
    pub muet: bool,
    /// v0.11 : ouverture auto : s'arrête sur cette rareté ou mieux (index de `Rarete`), et sur
    /// un shiny si `auto_shiny`.
    pub auto_rarete: u8,
    pub auto_shiny: bool,
    /// v0.11 : langue de l'interface ; None = pas encore choisie (celle de Windows).
    pub langue: Option<crate::i18n::Langue>,
}

impl Reglages {
    /// Nuit pour le mode Abysses automatique : de 21 h à 7 h.
    pub fn est_nuit(heure: u32) -> bool {
        !(7..21).contains(&heure)
    }

    /// Mode Abysses affiché : la nuit en automatique, sinon le choix manuel.
    pub fn abysses_actif(&self) -> bool {
        if self.abysses_auto {
            self.nuit
        } else {
            self.abysses
        }
    }

    /// Bascule manuelle (menu) : quitte l'automatique et inverse l'état affiché.
    pub fn basculer_abysses(&mut self) {
        self.abysses = !self.abysses_actif();
        self.abysses_auto = false;
    }
}

impl Default for Reglages {
    fn default() -> Self {
        Reglages {
            teinte: 0.25,
            max_affiches: CAPACITE,
            ips: 24,
            demarrage_windows: false,
            pause_plein_ecran: true,
            mode_fenetre: false,
            incrustation: 0,
            equiper_auto: false,
            abysses: false,
            abysses_auto: true,
            nuit: false,
            theme: 0,
            attraper: true,
            attraper_alt: false,
            volume: 0.7,
            muet: false,
            auto_rarete: Rarete::Legendaire as u8,
            auto_shiny: true,
            langue: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Partie {
    pub version: u32,
    pub argent: f64,
    pub total_gagne: f64,
    /// Heure de la dernière sauvegarde (secondes Unix) pour les gains hors ligne.
    pub horodatage: u64,
    pub prochain_id: u64,
    pub alea: Alea,
    pub poissons: Vec<Poisson>,
    /// Œufs ouverts, par œuf (complété par `migrer` quand un œuf s'ajoute).
    pub oeufs_ouverts: Vec<u64>,
    /// Poissons éclos, par rareté (Poissodex ; complété par `migrer`).
    pub eclos: Vec<u64>,
    /// Poissons éclos par espèce (Poissodex, depuis la version 2).
    #[serde(default)]
    pub vus: BTreeMap<String, u64>,
    /// Index découverts par espèce (bits de `bits_index` : Normal, modificateurs, Shiny ;
    /// version 4, complété par `migrer`).
    #[serde(default)]
    pub index: BTreeMap<String, u16>,
    /// Nombre d'index complets (recompté par `recompter_index`, pas sauvegardé).
    #[serde(skip)]
    complets: usize,
    #[serde(default)]
    pub reglages: Reglages,
    /// v0.9 : objets de décor achetés (exemplaires par clé de `decor::OBJETS`).
    #[serde(default)]
    pub decor_achete: BTreeMap<String, u32>,
    /// v0.9 : objets posés dans l'aquarium.
    #[serde(default)]
    pub decor_pose: Vec<Pose>,
    /// v0.10 : le dernier œuf acheté est devenu un œuf doré (éclosion spéciale).
    #[serde(skip)]
    pub dernier_dore: bool,
    /// v0.11 : œufs ouverts de suite sans la rareté garantie, par œuf (pitié).
    #[serde(default)]
    pub pitie: Vec<u32>,
    /// v0.11 : le dernier œuf acheté a déclenché la pitié (rareté garantie).
    #[serde(skip)]
    pub derniere_pitie: bool,
    /// v0.16 : nourriture en réserve, par nourriture (ordre de `nourriture::NOURRITURES`).
    #[serde(default)]
    pub nourriture: Vec<u32>,
    /// v0.16 : bonus du repas en cours.
    #[serde(default)]
    pub festin: Option<Festin>,
}

/// Objet de décor posé (v0.9) : sur l'écran `ecran` (0 = principal, ordre de
/// `ecrans::Disposition`), à `x` = fraction de la largeur de cet écran ; la hauteur suit le
/// sable. Un écran débranché : l'objet va sur le principal (même fraction).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pose {
    pub objet: String,
    #[serde(default)]
    pub ecran: u8,
    pub x: f32,
    #[serde(default)]
    pub miroir: bool,
}

/// Version du format de sauvegarde (2 = espèces, 3 = rareté Préhistorique et œuf d'ambre,
/// 4 = shiny, modificateurs et index, 5 = décor à thème et objets de décor, 6 = pitié,
/// 7 = 24 images/s, 8 = nourriture et bonus du repas).
pub const VERSION: u32 = 8;
/// Exemplaires au plus d'un même objet de décor.
pub const MAX_EXEMPLAIRES: u32 = 8;

/// Un œuf ouvert (v0.11) : le poisson éclos, œuf devenu doré, rareté garantie par la pitié.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ouverture {
    pub id: u64,
    pub dore: bool,
    pub pitie: bool,
}

#[derive(Debug, PartialEq)]
pub enum Refus {
    PasAssez,
    Introuvable,
    NiveauMax,
    AquariumPlein,
    /// Œuf exceptionnel hors de son moment.
    Indisponible,
    /// v0.16 : créature verrouillée (cadenas).
    Verrouille,
}

impl Partie {
    /// Nouvelle partie : un petit poisson commun offert pour démarrer.
    pub fn nouvelle(maintenant: u64, graine: u64) -> Partie {
        let mut p = Partie {
            version: VERSION,
            argent: 0.0,
            total_gagne: 0.0,
            horodatage: maintenant,
            prochain_id: 1,
            alea: Alea(graine),
            poissons: Vec::new(),
            oeufs_ouverts: vec![0; OEUFS.len()],
            eclos: vec![0; Rarete::TOUTES.len()],
            vus: BTreeMap::new(),
            index: BTreeMap::new(),
            complets: 0,
            reglages: Reglages::default(),
            decor_achete: BTreeMap::new(),
            decor_pose: Vec::new(),
            dernier_dore: false,
            pitie: vec![0; OEUFS.len()],
            derniere_pitie: false,
            nourriture: vec![0; NOURRITURES.len()],
            festin: None,
        };
        let id = p.ajouter(Rarete::Commun, 1.0, 0);
        // Le poisson offert est toujours normal (départ identique pour tout le monde).
        if let Some(f) = p.poissons.iter_mut().find(|f| f.id == id) {
            (f.shiny, f.modif) = (false, None);
            p.index.insert(f.espece.clone(), 1);
        }
        p
    }

    fn ajouter(&mut self, rarete: Rarete, taille: f32, oeuf: u8) -> u64 {
        self.ajouter_bonus(rarete, taille, oeuf, (1.0, 1.0))
    }

    fn ajouter_bonus(&mut self, rarete: Rarete, taille: f32, oeuf: u8, bonus: (f64, f64)) -> u64 {
        let id = self.prochain_id;
        self.prochain_id += 1;
        let graine = self.alea.suivant();
        let actif = self.nb_actifs() < CAPACITE;
        // L'espèce vient de la graine : le tirage des raretés (et l'équilibrage) ne change pas.
        let (e, variante) = crate::especes::choisir(rarete, graine);
        let forme = Forme::tirer_bonus(graine, bonus.0, bonus.1);
        self.poissons.push(Poisson {
            id,
            graine,
            rarete,
            taille,
            niveau: 1,
            actif,
            oeuf,
            espece: e.cle.to_string(),
            variante,
            shiny: forme.shiny,
            modif: forme.modif,
            verrou: false,
        });
        self.eclos[rarete.index()] += 1;
        *self.vus.entry(e.cle.to_string()).or_default() += 1;
        let bits = self.index.entry(e.cle.to_string()).or_default();
        if *bits | bits_index(forme) != *bits {
            *bits |= bits_index(forme);
            self.recompter_index();
        }
        if !actif && self.reglages.equiper_auto {
            self.remplacer_le_plus_faible(id);
        }
        id
    }

    /// « Équiper auto » : le poisson `id` (en réserve) prend la place du plus faible de
    /// l'aquarium s'il rapporte plus.
    fn remplacer_le_plus_faible(&mut self, id: u64) {
        let Some(nouveau) = self
            .poissons
            .iter()
            .find(|p| p.id == id)
            .map(Poisson::revenu)
        else {
            return;
        };
        let faible = self
            .poissons
            .iter()
            .filter(|p| p.actif)
            .min_by(|a, b| a.revenu().total_cmp(&b.revenu()))
            .map(|p| (p.id, p.revenu()));
        if let Some((fid, frev)) = faible {
            if nouveau > frev {
                for p in self.poissons.iter_mut() {
                    if p.id == fid {
                        p.actif = false;
                    } else if p.id == id {
                        p.actif = true;
                    }
                }
            }
        }
    }

    /// « Équiper les meilleurs » : les poissons qui rapportent le plus (revenu réel : rareté,
    /// taille, niveau) remplissent l'aquarium, tous les autres vont en réserve. Renvoie le
    /// revenu par seconde avant et après.
    pub fn equiper_meilleurs(&mut self) -> (f64, f64) {
        let avant = self.revenu_par_s();
        let mut ordre: Vec<(u64, f64)> = self.poissons.iter().map(|p| (p.id, p.revenu())).collect();
        ordre.sort_by(|a, b| b.1.total_cmp(&a.1));
        let gardes: std::collections::HashSet<u64> =
            ordre.iter().take(CAPACITE).map(|o| o.0).collect();
        for p in self.poissons.iter_mut() {
            p.actif = gardes.contains(&p.id);
        }
        (avant, self.revenu_par_s())
    }

    /// Met une sauvegarde ancienne au format courant : chaque poisson sans espèce connue (lot 1,
    /// ou espèce disparue) reçoit une espèce de même rareté tirée de sa graine ; rien n'est perdu.
    pub fn migrer(&mut self) {
        // Version 3 : un œuf et une rareté de plus (les anciennes sauvegardes en ont moins).
        self.oeufs_ouverts
            .resize(OEUFS.len().max(self.oeufs_ouverts.len()), 0);
        self.eclos
            .resize(Rarete::TOUTES.len().max(self.eclos.len()), 0);
        // Version 6 : compteurs de pitié (anciennes sauvegardes : tout à zéro).
        self.pitie.resize(OEUFS.len().max(self.pitie.len()), 0);
        let mut ajout = false;
        for f in &mut self.poissons {
            let connue = crate::especes::par_cle(&f.espece).is_some_and(|e| e.rarete == f.rarete);
            if !connue {
                let (e, v) = crate::especes::choisir(f.rarete, f.graine);
                f.espece = e.cle.to_string();
                f.variante = v;
                ajout = true;
            }
        }
        // Poissodex par espèce : au moins les poissons possédés.
        if ajout || self.vus.is_empty() {
            let mut n: BTreeMap<String, u64> = BTreeMap::new();
            for f in &self.poissons {
                *n.entry(f.espece.clone()).or_default() += 1;
            }
            for (k, c) in n {
                let v = self.vus.entry(k).or_default();
                *v = (*v).max(c);
            }
        }
        // Version 4 : index. Toute espèce déjà vue compte dans l'index Normal ; les poissons
        // possédés marquent leur forme (les anciens sont normaux, pas shiny : serde par défaut).
        for (k, &n) in &self.vus {
            if n > 0 {
                *self.index.entry(k.clone()).or_default() |= 1;
            }
        }
        for f in &self.poissons {
            *self.index.entry(f.espece.clone()).or_default() |= bits_index(f.forme());
        }
        self.recompter_index();
        // Version 5 : décor. Objets inconnus retirés, jamais plus de posés que d'achetés.
        self.decor_achete
            .retain(|k, n| crate::decor::objet(k).is_some() && *n > 0);
        let mut poses: BTreeMap<String, u32> = BTreeMap::new();
        let achete = &self.decor_achete;
        self.decor_pose.retain_mut(|p| {
            let n = poses.entry(p.objet.clone()).or_default();
            *n += 1;
            p.x = if p.x.is_finite() {
                p.x.clamp(0.0, 1.0)
            } else {
                0.5
            };
            *n <= achete.get(&p.objet).copied().unwrap_or(0)
        });
        self.reglages.theme = self
            .reglages
            .theme
            .min(crate::decor::THEMES.len() as u8 - 1);
        // Version 7 (v0.12) : 24 images/s par défaut ; l'ancien défaut 30 passe à 24.
        if self.version < 7 && self.reglages.ips == 30 {
            self.reglages.ips = 24;
        }
        // Version 8 (v0.16) : réserve de nourriture (anciennes sauvegardes : vide), bonus du
        // repas relu prudemment (nourriture inconnue ou durée abîmée : plus de bonus).
        self.nourriture
            .resize(NOURRITURES.len().max(self.nourriture.len()), 0);
        self.festin = self.festin.filter(|f| {
            (f.niveau as usize) < NOURRITURES.len() && f.reste.is_finite() && f.reste > 0.0
        });
        if let Some(f) = &mut self.festin {
            f.reste = f.reste.min(f.nourriture().duree * PLAFOND);
        }
        self.version = self.version.max(VERSION);
    }

    fn recompter_index(&mut self) {
        self.complets = (0..NB_INDEX)
            .filter(|&k| {
                let (n, t) = self.completion_index(k);
                n == t
            })
            .count();
    }

    /// Une espèce est-elle découverte dans l'index `k` (0 Normal, 1..=7 modificateurs, 8 Shiny) ?
    pub fn dans_index(&self, cle: &str, k: usize) -> bool {
        self.index.get(cle).is_some_and(|b| b & (1 << k) != 0)
    }

    /// Espèces découvertes dans l'index `k`, sur le total.
    pub fn completion_index(&self, k: usize) -> (usize, usize) {
        let c = crate::especes::catalogue();
        (
            c.iter().filter(|e| self.dans_index(e.cle, k)).count(),
            c.len(),
        )
    }

    /// Index complets (toutes les espèces) : chacun rapporte `BONUS_INDEX` pour toujours.
    pub fn index_complets(&self) -> usize {
        self.complets
    }

    /// Multiplicateur de tout le revenu donné par les index complets.
    pub fn bonus_index(&self) -> f64 {
        1.0 + BONUS_INDEX * self.index_complets() as f64
    }

    pub fn nb_actifs(&self) -> usize {
        self.poissons.iter().filter(|p| p.actif).count()
    }

    /// Revenu de l'aquarium (poissons actifs, bonus des index complets et du repas compris).
    pub fn revenu_par_s(&self) -> f64 {
        self.revenu_sans_festin() * self.mult_festin()
    }

    fn revenu_sans_festin(&self) -> f64 {
        self.poissons
            .iter()
            .filter(|p| p.actif)
            .map(Poisson::revenu)
            .sum::<f64>()
            * self.bonus_index()
    }

    /// Multiplicateur du bonus du repas en cours (1 sans bonus).
    pub fn mult_festin(&self) -> f64 {
        self.festin.map_or(1.0, |f| f.nourriture().mult)
    }

    pub fn crediter(&mut self, montant: f64) {
        self.argent = (self.argent + montant).min(ARGENT_MAX);
        self.total_gagne = (self.total_gagne + montant).min(ARGENT_MAX);
    }

    pub fn avancer(&mut self, dt: f64) {
        let r = self.revenu_par_s();
        self.crediter(r * dt);
        self.user_festin(dt);
    }

    /// Le bonus du repas s'use (fini : plus de bonus).
    fn user_festin(&mut self, dt: f64) {
        if let Some(f) = &mut self.festin {
            f.reste -= dt;
            if f.reste <= 0.0 {
                self.festin = None;
            }
        }
    }

    /// Crédite les gains depuis `horodatage` (plafond 8 h) ; renvoie le montant. Le bonus du
    /// repas compte pour le temps qui lui restait.
    pub fn gains_hors_ligne(&mut self, maintenant: u64) -> f64 {
        let ecoule = maintenant.saturating_sub(self.horodatage) as f64;
        let base = self.revenu_sans_festin();
        let duree = ecoule.min(HORS_LIGNE_MAX_S);
        let bonus = self
            .festin
            .map_or(0.0, |f| (f.nourriture().mult - 1.0) * f.reste.min(duree));
        let gain = base * (duree + bonus);
        self.crediter(gain);
        self.user_festin(ecoule);
        self.horodatage = maintenant;
        gain
    }

    /// Achète `n` exemplaires de la nourriture `i` (réserve plafonnée à `MAX_STOCK`).
    pub fn acheter_nourriture(&mut self, i: usize, n: u32) -> Result<(), Refus> {
        let nn = NOURRITURES.get(i).ok_or(Refus::Introuvable)?;
        self.nourriture.resize(NOURRITURES.len(), 0);
        let n = n.min(MAX_STOCK.saturating_sub(self.nourriture[i]));
        if n == 0 {
            return Err(Refus::Introuvable);
        }
        let prix = nn.prix * n as f64;
        if self.argent < prix {
            return Err(Refus::PasAssez);
        }
        self.argent -= prix;
        self.nourriture[i] += n;
        Ok(())
    }

    /// Prend un exemplaire de la nourriture `i` dans la réserve (pour le jeter).
    pub fn prendre_nourriture(&mut self, i: usize) -> Result<(), Refus> {
        match self.nourriture.get_mut(i) {
            Some(k) if *k > 0 => {
                *k -= 1;
                Ok(())
            }
            Some(_) => Err(Refus::PasAssez),
            None => Err(Refus::Introuvable),
        }
    }

    /// Un poisson a mangé une bouchée de la nourriture `i` (`part` du repas entier).
    pub fn manger(&mut self, i: usize, part: f64) {
        self.festin = crate::nourriture::ajouter(self.festin, i, part);
    }

    pub fn tirer_rarete(alea: &mut Alea, oeuf: &Oeuf) -> Rarete {
        let mut x = alea.f() as f64 * 100.0;
        for r in Rarete::TOUTES {
            x -= oeuf.chances[r.index()];
            if x < 0.0 {
                return r;
            }
        }
        // Reste d'arrondi : la plus haute rareté possible de l'œuf.
        *Rarete::TOUTES
            .iter()
            .rev()
            .find(|r| oeuf.chances[r.index()] > 0.0)
            .unwrap_or(&Rarete::Commun)
    }

    /// Taille 0,6..1,5, les petits sont plus fréquents.
    pub fn tirer_taille(alea: &mut Alea) -> f32 {
        let t = 0.6 + 0.9 * alea.f().powf(1.5);
        (t * 100.0).round() / 100.0
    }

    /// Prix courant d'un œuf : il monte avec le nombre d'œufs de ce type déjà ouverts.
    pub fn prix_oeuf(&self, i: usize) -> f64 {
        let n = self.oeufs_ouverts[i] as f64;
        OEUFS[i].prix * (1.0 + n / 10.0).powi(2)
    }

    /// Achète et fait éclore un œuf ; renvoie l'id du poisson (aquarium s'il reste de la place,
    /// sinon réserve).
    pub fn acheter_oeuf(&mut self, i: usize) -> Result<u64, Refus> {
        self.acheter_oeuf_a(i, crate::sauve::maintenant())
    }

    /// Achat à l'instant `maintenant` (œufs exceptionnels : seulement pendant leur moment).
    /// Un œuf ordinaire devient parfois un œuf doré (`CHANCE_OEUF_DORE`) : la meilleure de
    /// deux raretés, shiny ×3, modificateurs ×2 (`dernier_dore`). Pitié (v0.11) : le N-ième
    /// œuf de suite sans la rareté garantie la donne (`derniere_pitie`).
    pub fn acheter_oeuf_a(&mut self, i: usize, maintenant: u64) -> Result<u64, Refus> {
        let oeuf = OEUFS.get(i).ok_or(Refus::Introuvable)?;
        if !oeuf.disponible(maintenant) {
            return Err(Refus::Indisponible);
        }
        let prix = self.prix_oeuf(i);
        if self.argent < prix {
            return Err(Refus::PasAssez);
        }
        self.argent -= prix;
        self.oeufs_ouverts[i] += 1;
        let mut rarete = Self::tirer_rarete(&mut self.alea, oeuf);
        let mut bonus = (oeuf.bonus_shiny, oeuf.bonus_modif);
        self.dernier_dore = oeuf.dispo.is_none() && self.alea.f64() < CHANCE_OEUF_DORE;
        if self.dernier_dore {
            rarete = rarete.max(Self::tirer_rarete(&mut self.alea, oeuf));
            bonus = (3.0, 2.0);
        }
        // v0.16 : bonus du repas (calamar, cage, festin) : chance aux éclosions.
        let chance = self.festin.map_or(1.0, |f| f.nourriture().chance);
        bonus = (bonus.0 * chance, bonus.1 * chance);
        self.derniere_pitie = false;
        self.pitie.resize(OEUFS.len().max(self.pitie.len()), 0);
        if let Some((min, n)) = oeuf.pitie {
            if rarete < min && self.pitie[i] + 1 >= n {
                rarete = Self::tirer_rarete_min(&mut self.alea, oeuf, min);
                self.derniere_pitie = true;
            }
            self.pitie[i] = if rarete >= min { 0 } else { self.pitie[i] + 1 };
        }
        let taille = Self::tirer_taille(&mut self.alea);
        Ok(self.ajouter_bonus(rarete, taille, i as u8, bonus))
    }

    /// Tirage limité aux raretés `min` ou mieux de l'œuf (mêmes proportions entre elles).
    fn tirer_rarete_min(alea: &mut Alea, oeuf: &Oeuf, min: Rarete) -> Rarete {
        let haut = |r: &Rarete| {
            if *r >= min {
                oeuf.chances[r.index()]
            } else {
                0.0
            }
        };
        let total: f64 = Rarete::TOUTES.iter().map(haut).sum();
        let mut x = alea.f64() * total;
        for r in Rarete::TOUTES.iter().filter(|r| haut(r) > 0.0) {
            x -= haut(r);
            if x < 0.0 {
                return *r;
            }
        }
        Rarete::TOUTES
            .iter()
            .rev()
            .copied()
            .find(|r| haut(r) > 0.0)
            .unwrap_or(min)
    }

    /// Œufs restants avant la rareté garantie (pitié) : (rareté, œufs), ou None.
    pub fn pitie_restante(&self, i: usize) -> Option<(Rarete, u32)> {
        let (r, n) = OEUFS.get(i)?.pitie?;
        Some((r, n.saturating_sub(self.pitie.get(i).copied().unwrap_or(0))))
    }

    /// Prix total de `n` œufs `i` achetés d'affilée (le prix monte à chaque œuf).
    pub fn prix_lot(&self, i: usize, n: u32) -> f64 {
        let o = self.oeufs_ouverts[i] as f64;
        (0..n)
            .map(|k| OEUFS[i].prix * (1.0 + (o + k as f64) / 10.0).powi(2))
            .sum()
    }

    /// Ouverture multiple (×3, ×10) : tout ou rien (il faut le prix du lot entier).
    pub fn acheter_lot(
        &mut self,
        i: usize,
        n: u32,
        maintenant: u64,
    ) -> Result<Vec<Ouverture>, Refus> {
        let oeuf = OEUFS.get(i).ok_or(Refus::Introuvable)?;
        if !oeuf.disponible(maintenant) {
            return Err(Refus::Indisponible);
        }
        if self.argent < self.prix_lot(i, n) {
            return Err(Refus::PasAssez);
        }
        let mut v = Vec::new();
        for _ in 0..n {
            // Arrondis : la somme du lot et les prix un à un peuvent différer d'un epsilon.
            let pr = self.prix_oeuf(i);
            if self.argent < pr && pr - self.argent <= pr * 1e-9 {
                self.argent = pr;
            }
            let id = self.acheter_oeuf_a(i, maintenant)?;
            v.push(Ouverture {
                id,
                dore: self.dernier_dore,
                pitie: self.derniere_pitie,
            });
        }
        Ok(v)
    }

    fn poisson_mut(&mut self, id: u64) -> Result<&mut Poisson, Refus> {
        self.poissons
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or(Refus::Introuvable)
    }

    pub fn nourrir(&mut self, id: u64) -> Result<(), Refus> {
        let argent = self.argent;
        let p = self.poisson_mut(id)?;
        let cout = p.cout_nourrir().ok_or(Refus::NiveauMax)?;
        if argent < cout {
            return Err(Refus::PasAssez);
        }
        p.niveau += 1;
        self.argent -= cout;
        Ok(())
    }

    /// Range dans la réserve (`actif = false`) ou remet dans l'aquarium.
    pub fn basculer(&mut self, id: u64) -> Result<(), Refus> {
        let plein = self.nb_actifs() >= CAPACITE;
        let p = self.poisson_mut(id)?;
        if !p.actif && plein {
            return Err(Refus::AquariumPlein);
        }
        p.actif = !p.actif;
        Ok(())
    }

    /// Prix du prochain exemplaire d'un objet de décor (×1,6 par exemplaire déjà acheté).
    pub fn prix_objet(&self, cle: &str) -> Option<f64> {
        let o = crate::decor::objet(cle)?;
        Some(o.prix * 1.6f64.powi(self.nb_achetes(cle) as i32))
    }

    pub fn nb_achetes(&self, cle: &str) -> u32 {
        self.decor_achete.get(cle).copied().unwrap_or(0)
    }

    /// Exemplaires achetés pas encore posés.
    pub fn nb_libres(&self, cle: &str) -> u32 {
        let poses = self.decor_pose.iter().filter(|p| p.objet == cle).count() as u32;
        self.nb_achetes(cle).saturating_sub(poses)
    }

    pub fn acheter_objet(&mut self, cle: &str) -> Result<(), Refus> {
        let prix = self.prix_objet(cle).ok_or(Refus::Introuvable)?;
        if self.nb_achetes(cle) >= MAX_EXEMPLAIRES {
            return Err(Refus::NiveauMax);
        }
        if self.argent < prix {
            return Err(Refus::PasAssez);
        }
        self.argent -= prix;
        *self.decor_achete.entry(cle.to_string()).or_default() += 1;
        Ok(())
    }

    /// Pose un exemplaire libre de l'objet sur l'écran `ecran`, à la fraction `x`.
    pub fn poser_objet(&mut self, cle: &str, ecran: u8, x: f32) -> Result<(), Refus> {
        if self.nb_libres(cle) == 0 {
            return Err(Refus::Introuvable);
        }
        let miroir = self.decor_pose.len() % 2 == 1;
        self.decor_pose.push(Pose {
            objet: cle.to_string(),
            ecran,
            x: x.clamp(0.0, 1.0),
            miroir,
        });
        Ok(())
    }

    pub fn relacher(&mut self, id: u64) -> Result<f64, Refus> {
        let i = self
            .poissons
            .iter()
            .position(|p| p.id == id)
            .ok_or(Refus::Introuvable)?;
        if self.poissons[i].verrou {
            return Err(Refus::Verrouille);
        }
        let prix = self.poissons.remove(i).prix_relache();
        self.argent = (self.argent + prix).min(ARGENT_MAX);
        Ok(prix)
    }

    /// v0.16 : pose ou retire le cadenas d'une créature.
    pub fn verrouiller(&mut self, id: u64) -> Result<(), Refus> {
        let p = self.poisson_mut(id)?;
        p.verrou = !p.verrou;
        Ok(())
    }

    /// v0.16 : ce que rapporterait « Tout vendre » : créatures de la réserve non verrouillées
    /// (nombre, total).
    pub fn prix_tout_vendre(&self) -> (usize, f64) {
        let v = self.poissons.iter().filter(|p| !p.actif && !p.verrou);
        (v.clone().count(), v.map(Poisson::prix_relache).sum())
    }

    /// v0.16 : vend (relâche) toute la réserve sauf les créatures verrouillées ; renvoie le
    /// nombre vendu et le total gagné.
    pub fn tout_vendre(&mut self) -> (usize, f64) {
        let (n, total) = self.prix_tout_vendre();
        self.poissons.retain(|p| p.actif || p.verrou);
        self.argent = (self.argent + total).min(ARGENT_MAX);
        (n, total)
    }
}

/// Format court français : 742, 1,23 k, 45,6 M, 1,2 Md, … puis notation 1,23e30.
pub fn court(v: f64) -> String {
    if !v.is_finite() {
        return "∞".into();
    }
    if v < 0.0 {
        return format!("-{}", court(-v));
    }
    // Anglais : point décimal et suffixes courts (k, M, B, T, Qa…).
    let en = crate::i18n::anglais();
    let virgule = |s: &str| {
        if en {
            s.to_string()
        } else {
            s.replace('.', ",")
        }
    };
    if v < 10.0 {
        let s = format!("{:.1}", (v * 10.0).floor() / 10.0);
        return virgule(s.trim_end_matches(".0"));
    }
    if v < 1000.0 {
        return format!("{}", v.floor() as u64);
    }
    const SUFFIXES: [&str; 9] = ["k", "M", "Md", "Bn", "Bd", "Tn", "Td", "Qn", "Qd"];
    const SUFFIXES_EN: [&str; 9] = ["k", "M", "B", "T", "Qa", "Qi", "Sx", "Sp", "Oc"];
    let mut e = (v.log10().floor() as i32).max(3);
    // Arrondi à 3 chiffres significatifs, en gardant le palier si l'arrondi déborde (999,96 k → 1 M).
    let mut m = v / 10f64.powi(e);
    m = (m * 100.0 + 1e-6).floor() / 100.0;
    if m >= 10.0 {
        e += 1;
        m /= 10.0;
    }
    let palier = (e / 3) as usize;
    if palier > SUFFIXES.len() {
        let s = format!("{m:.2}");
        let s = virgule(s.trim_end_matches('0').trim_end_matches('.'));
        return format!("{s}e{e}");
    }
    let chiffres = m * 10f64.powi(e - 3 * palier as i32);
    let decimales = 2 - (e - 3 * palier as i32) as usize;
    let s = format!("{:.*}", decimales, trunc(chiffres, decimales));
    let s = if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    };
    let suffixe = if en { SUFFIXES_EN } else { SUFFIXES }[palier - 1];
    format!("{} {suffixe}", virgule(&s))
}

fn trunc(v: f64, decimales: usize) -> f64 {
    let f = 10f64.powi(decimales as i32);
    ((v * f) + 1e-9).floor() / f
}

/// Durée lisible : « 3 min 20 s », « 2 h 05 », « 1 j 4 h ».
pub fn duree(s: f64) -> String {
    let s = s.max(0.0) as u64;
    let en = crate::i18n::anglais();
    if s < 60 {
        format!("{s} s")
    } else if s < 3600 {
        format!("{} min {:02} s", s / 60, s % 60)
    } else if s < 86_400 {
        if en {
            format!("{} h {:02} min", s / 3600, s / 60 % 60)
        } else {
            format!("{} h {:02}", s / 3600, s / 60 % 60)
        }
    } else {
        format!(
            "{} {} {} h",
            s / 86_400,
            if en { "d" } else { "j" },
            s / 3600 % 24
        )
    }
}

/// Jalon de la simulation d'équilibrage.
pub struct Jalon {
    pub temps_s: f64,
    pub texte: String,
}

/// Joue une partie « gloutonne » (achète l'action au meilleur gain/prix dès qu'il peut)
/// pendant `heures` de jeu et note les jalons. Sert à `--simulation` et aux tests.
pub fn simuler(heures: f64, graine: u64) -> Vec<Jalon> {
    let mut p = Partie::nouvelle(0, graine);
    let mut jalons = Vec::new();
    let mut vu_oeuf = [false; OEUFS.len()];
    let mut vu_rarete = [false; Rarete::TOUTES.len()];
    vu_rarete[0] = true;
    let mut paliers_revenu = [10.0, 100.0, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8]
        .into_iter()
        .peekable();
    let fin = heures * 3600.0;
    let mut t = 0.0;
    while t < fin {
        p.avancer(1.0);
        t += 1.0;
        // Meilleure action (gain de revenu / prix), même trop chère : on attend de pouvoir.
        let plus_faible = p
            .poissons
            .iter()
            .filter(|f| f.actif)
            .map(Poisson::revenu)
            .fold(f64::MAX, f64::min);
        let plein = p.nb_actifs() >= CAPACITE;
        let mut meilleur: Option<(f64, bool, u64)> = None;
        // Les œufs exceptionnels (par moments) ne comptent pas : le joueur glouton n'attend pas.
        for (i, o) in OEUFS.iter().enumerate().filter(|(_, o)| o.dispo.is_none()) {
            let gain: f64 = Rarete::TOUTES
                .iter()
                .map(|r| {
                    let nouveau = r.revenu_base() * 0.96 * Forme::esperance();
                    o.chances[r.index()] / 100.0
                        * if plein {
                            (nouveau - plus_faible).max(0.0)
                        } else {
                            nouveau
                        }
                })
                .sum();
            let ratio = gain / p.prix_oeuf(i);
            // Un joueur n'économise pas plus d'un jour pour un œuf (sinon il achète autre chose).
            let trop_loin = p.prix_oeuf(i) > p.argent + p.revenu_par_s() * 24.0 * 3600.0;
            if !trop_loin && meilleur.is_none_or(|m| ratio > m.0) {
                meilleur = Some((ratio, true, i as u64));
            }
        }
        for f in p.poissons.iter().filter(|f| f.actif) {
            if let Some(c) = f.cout_nourrir() {
                let gain = f.revenu() * 0.2;
                if meilleur.is_none_or(|m| gain / c > m.0) {
                    meilleur = Some((gain / c, false, f.id));
                }
            }
        }
        let Some((_, oeuf, x)) = meilleur else {
            continue;
        };
        if oeuf {
            let Ok(id) = p.acheter_oeuf(x as usize) else {
                continue;
            };
            if !vu_oeuf[x as usize] {
                vu_oeuf[x as usize] = true;
                jalons.push(Jalon {
                    temps_s: t,
                    texte: format!("1er {}", OEUFS[x as usize].nom),
                });
            }
            let nouveau = p.poissons.iter().find(|f| f.id == id).unwrap().clone();
            if !vu_rarete[nouveau.rarete.index()] {
                vu_rarete[nouveau.rarete.index()] = true;
                jalons.push(Jalon {
                    temps_s: t,
                    texte: format!("1er poisson {}", nouveau.rarete.nom()),
                });
            }
            if !nouveau.actif {
                // Aquarium plein : on garde le meilleur des deux.
                let faible = p
                    .poissons
                    .iter()
                    .filter(|f| f.actif)
                    .min_by(|a, b| a.revenu().total_cmp(&b.revenu()))
                    .map(|f| (f.id, f.revenu()))
                    .unwrap();
                if nouveau.revenu() > faible.1 {
                    let _ = p.relacher(faible.0);
                    let _ = p.basculer(id);
                } else {
                    let _ = p.relacher(id);
                }
            }
        } else {
            let _ = p.nourrir(x);
        }
        while let Some(&palier) = paliers_revenu.peek() {
            if p.revenu_par_s() < palier {
                break;
            }
            paliers_revenu.next();
            jalons.push(Jalon {
                temps_s: t,
                texte: format!("revenu {}/s", court(palier)),
            });
        }
    }
    jalons.push(Jalon {
        temps_s: t,
        texte: format!(
            "fin : {}/s, {} œufs ouverts, aquarium : {} shiny, {} modifiés ; index Normal {}/{}, {} index complets",
            court(p.revenu_par_s()),
            p.oeufs_ouverts.iter().sum::<u64>(),
            p.poissons.iter().filter(|f| f.actif && f.shiny).count(),
            p.poissons.iter().filter(|f| f.actif && f.modif.is_some()).count(),
            p.completion_index(0).0,
            p.completion_index(0).1,
            p.index_complets()
        ),
    });
    jalons
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abysses_automatiques() {
        let mut r = Reglages::default();
        assert!(r.abysses_auto);
        for (h, nuit) in [
            (20, false),
            (21, true),
            (3, true),
            (6, true),
            (7, false),
            (12, false),
        ] {
            r.nuit = Reglages::est_nuit(h);
            assert_eq!(r.abysses_actif(), nuit, "{h} h");
        }
        // Manuel : suit le choix, quelle que soit l'heure.
        r.abysses_auto = false;
        r.nuit = true;
        assert!(!r.abysses_actif());
        r.abysses = true;
        r.nuit = false;
        assert!(r.abysses_actif());
        // Menu : la nuit en automatique, les Abysses s'éteignent et l'automatique s'arrête.
        let mut r = Reglages {
            nuit: true,
            ..Reglages::default()
        };
        r.basculer_abysses();
        assert!(!r.abysses_auto && !r.abysses_actif());
        r.basculer_abysses();
        assert!(r.abysses_actif());
    }

    #[test]
    fn oeufs_exceptionnels() {
        let doré = &OEUFS[5];
        // Doré : 0 h-2 h toutes les 8 h (UTC) ; Éclipse : 22 h-23 h.
        assert!(doré.disponible(0) && doré.disponible(2 * H - 1) && !doré.disponible(2 * H));
        assert!(doré.disponible(8 * H + 60));
        assert_eq!(doré.attente(2 * H), Some(6 * H));
        assert_eq!(doré.attente(H), Some(H));
        let eclipse = &OEUFS[7];
        assert!(eclipse.disponible(22 * H) && !eclipse.disponible(23 * H));
        assert!(OEUFS[0].disponible(12345) && OEUFS[0].attente(0).is_none());
        // Chaque heure du jour : au plus un œuf exceptionnel à la fois, jamais aucun moment.
        for h in 0..24 {
            let n = OEUFS
                .iter()
                .filter(|o| o.dispo.is_some() && o.disponible(h * H))
                .count();
            assert!(n <= 1, "{h} h : {n}");
        }
        let mut p = Partie::nouvelle(0, 1);
        p.argent = 1e12;
        assert_eq!(p.acheter_oeuf_a(5, 3 * H), Err(Refus::Indisponible));
        let id = p.acheter_oeuf_a(5, 30).unwrap();
        assert_eq!(p.poissons.last().unwrap().id, id);
        assert_eq!(p.oeufs_ouverts[5], 1);
        // Bonus 1 : même forme que le tirage normal (sauvegardes, équilibrage inchangés).
        for g in 0..2000u64 {
            assert_eq!(Forme::tirer_bonus(g, 1.0, 1.0), Forme::tirer(g));
        }
        // Bonus : bien plus de shiny et de modificateurs.
        let n = 100_000u64;
        let (mut s1, mut m1, mut s5, mut m4) = (0, 0, 0, 0);
        for g in 0..n {
            let g = g.wrapping_mul(0x9e37_79b9_7f4a_7c15);
            let (a, b) = (Forme::tirer(g), Forme::tirer_bonus(g, 5.0, 4.0));
            s1 += a.shiny as u32;
            m1 += a.modif.is_some() as u32;
            s5 += b.shiny as u32;
            m4 += b.modif.is_some() as u32;
        }
        let (s1, s5) = (s1 as f64 / n as f64, s5 as f64 / n as f64);
        assert!((s5 / s1 - 5.0).abs() < 0.6, "shiny {s1} → {s5}");
        assert!(
            (m4 as f64 / m1 as f64 - 4.0).abs() < 0.3,
            "modif {m1} → {m4}"
        );
    }

    #[test]
    fn oeuf_dore_par_hasard() {
        // ~3 % des œufs ordinaires deviennent dorés, avec de meilleures raretés.
        let mut p = Partie::nouvelle(0, 7);
        p.argent = 1e300;
        let (mut dores, mut rare_d, mut rare_n, mut n_n) = (0u32, 0u32, 0u32, 0u32);
        let n = 40_000;
        for _ in 0..n {
            let id = p.acheter_oeuf_a(0, 0).unwrap();
            p.oeufs_ouverts[0] = 0;
            p.pitie[0] = 0;
            let f = p.poissons.pop().unwrap();
            assert_eq!(f.id, id);
            if p.dernier_dore {
                dores += 1;
                rare_d += (f.rarete >= Rarete::Rare) as u32;
            } else {
                n_n += 1;
                rare_n += (f.rarete >= Rarete::Rare) as u32;
            }
        }
        let taux = dores as f64 / n as f64;
        assert!((taux - CHANCE_OEUF_DORE).abs() < 0.006, "{taux}");
        assert!(rare_d as f64 / dores as f64 > 1.6 * rare_n as f64 / n_n as f64);
        // Un œuf exceptionnel n'est jamais « doré par hasard ».
        p.acheter_oeuf_a(5, 0).unwrap();
        assert!(!p.dernier_dore);
    }

    #[test]
    fn decor_prix_et_achats() {
        use crate::decor::OBJETS;
        // Du moins cher (premières minutes) au plus cher (fin de partie, après l'œuf d'ambre).
        assert!(OBJETS.windows(2).all(|w| w[0].prix < w[1].prix));
        assert!(OBJETS[0].prix <= 200.0);
        assert!(OBJETS[OBJETS.len() - 1].prix >= OEUFS[OEUFS.len() - 1].prix);
        let mut p = Partie::nouvelle(0, 1);
        assert_eq!(p.acheter_objet("cailloux"), Err(Refus::PasAssez));
        assert_eq!(p.acheter_objet("inconnu"), Err(Refus::Introuvable));
        p.argent = 1e9;
        let p0 = p.prix_objet("crane").unwrap();
        p.acheter_objet("crane").unwrap();
        assert!((p.argent - (1e9 - p0)).abs() < 1e-6);
        assert!(p.prix_objet("crane").unwrap() > p0 * 1.5);
        assert_eq!(p.nb_libres("crane"), 1);
        p.poser_objet("crane", 1, 0.4).unwrap();
        assert_eq!(p.nb_libres("crane"), 0);
        assert_eq!(p.poser_objet("crane", 0, 0.5), Err(Refus::Introuvable));
        for _ in 1..MAX_EXEMPLAIRES {
            p.acheter_objet("crane").unwrap();
        }
        assert_eq!(p.acheter_objet("crane"), Err(Refus::NiveauMax));
        // Le décor ne change pas les gains.
        let r = p.revenu_par_s();
        p.poser_objet("crane", 0, 0.9).unwrap();
        assert_eq!(p.revenu_par_s(), r);
    }

    #[test]
    fn themes_et_objets_complets() {
        use crate::decor::{decor, OBJETS, THEMES};
        let d = decor();
        assert_eq!(THEMES.len(), 6);
        assert_eq!(d.themes.len(), THEMES.len());
        assert_eq!(d.objets.len(), OBJETS.len());
        for t in &d.themes {
            assert!(t.grands.len() >= 2 && t.petits.len() >= 3 && !t.algues.is_empty());
        }
        // Clés uniques (elles sont sauvegardées).
        for (i, o) in OBJETS.iter().enumerate() {
            assert!(OBJETS[i + 1..].iter().all(|b| b.cle != o.cle), "{}", o.cle);
        }
        // Volcan : des pixels qui luisent.
        assert!(!crate::decor::points_lumineux(&d.themes[3].grands[0]).is_empty());
    }

    #[test]
    fn tables_somment_a_100() {
        for o in &OEUFS {
            let s: f64 = o.chances.iter().sum();
            assert!((s - 100.0).abs() < 1e-9, "{} : {s}", o.nom);
        }
        // Prix croissants (œufs ordinaires).
        let ordinaires: Vec<&Oeuf> = OEUFS.iter().filter(|o| o.dispo.is_none()).collect();
        assert!(ordinaires.windows(2).all(|w| w[0].prix < w[1].prix));
    }

    #[test]
    fn tirages_suivent_les_tables() {
        let mut a = Alea(42);
        for o in &OEUFS {
            let n = 200_000;
            let mut c = [0u32; Rarete::TOUTES.len()];
            for _ in 0..n {
                c[Partie::tirer_rarete(&mut a, o).index()] += 1;
            }
            for r in Rarete::TOUTES {
                let attendu = o.chances[r.index()];
                let vu = c[r.index()] as f64 * 100.0 / n as f64;
                assert!(
                    (vu - attendu).abs() < 0.35,
                    "{} {} : {vu:.2} % au lieu de {attendu} %",
                    o.nom,
                    r.nom()
                );
                if attendu == 0.0 {
                    assert_eq!(c[r.index()], 0);
                }
            }
        }
    }

    #[test]
    fn premier_oeuf_entre_1_et_2_min() {
        let p = Partie::nouvelle(0, 1);
        let t = OEUFS[0].prix / p.revenu_par_s();
        assert!((60.0..=120.0).contains(&t), "{t} s");
        let j = simuler(0.1, 7);
        assert!(j[0].texte.contains("rivière") && (60.0..=120.0).contains(&j[0].temps_s));
    }

    #[test]
    fn achat_et_capacite() {
        let mut p = Partie::nouvelle(0, 3);
        assert_eq!(p.acheter_oeuf(0), Err(Refus::PasAssez));
        p.argent = 1e9;
        for _ in 0..CAPACITE + 5 {
            p.acheter_oeuf(0).unwrap();
        }
        assert_eq!(p.nb_actifs(), CAPACITE);
        assert_eq!(p.poissons.len(), CAPACITE + 6);
        let reserve = p.poissons.iter().find(|f| !f.actif).unwrap().id;
        assert_eq!(p.basculer(reserve), Err(Refus::AquariumPlein));
        let actif = p.poissons.iter().find(|f| f.actif).unwrap().id;
        p.basculer(actif).unwrap();
        p.basculer(reserve).unwrap();
        assert_eq!(p.nb_actifs(), CAPACITE);
        let avant = p.argent;
        let prix = p.relacher(reserve).unwrap();
        assert!(prix > 0.0 && p.argent == avant + prix);
        assert_eq!(p.eclos.iter().sum::<u64>(), CAPACITE as u64 + 6);
    }

    #[test]
    fn nourrir_augmente_le_revenu() {
        let mut p = Partie::nouvelle(0, 3);
        let id = p.poissons[0].id;
        let r1 = p.revenu_par_s();
        assert_eq!(p.nourrir(id), Err(Refus::PasAssez));
        p.argent = 1e12;
        p.nourrir(id).unwrap();
        assert!((p.revenu_par_s() / r1 - 1.2).abs() < 1e-9);
        while p.nourrir(id).is_ok() {}
        assert_eq!(p.poissons[0].niveau, NIVEAU_MAX);
        assert_eq!(p.nourrir(id), Err(Refus::NiveauMax));
    }

    #[test]
    fn gains_hors_ligne_plafonnes() {
        let mut p = Partie::nouvelle(1000, 3);
        assert_eq!(p.gains_hors_ligne(1000 + 60), 60.0);
        let g = p.gains_hors_ligne(1060 + 3 * 86_400);
        assert_eq!(g, HORS_LIGNE_MAX_S);
        assert_eq!(p.horodatage, 1060 + 3 * 86_400);
        // Horloge qui recule : rien.
        assert_eq!(p.gains_hors_ligne(10), 0.0);
    }

    #[test]
    fn format_court() {
        let cas = [
            (0.0, "0"),
            (0.6, "0,6"),
            (7.25, "7,2"),
            (10.0, "10"),
            (742.9, "742"),
            (1000.0, "1 k"),
            (1234.0, "1,23 k"),
            (12_345.0, "12,3 k"),
            (123_456.0, "123 k"),
            (999_999.0, "999 k"),
            (3_400_000.0, "3,4 M"),
            (1.2e9, "1,2 Md"),
            (5e12, "5 Bn"),
            (2.5e27, "2,5 Qd"),
            (1.234e30, "1,23e30"),
            (1.5e300, "1,5e300"),
            (-1500.0, "-1,5 k"),
        ];
        for (v, attendu) in cas {
            assert_eq!(court(v), attendu, "{v}");
        }
        assert_eq!(court(f64::INFINITY), "∞");
    }

    #[test]
    fn argent_jamais_infini() {
        let mut p = Partie::nouvelle(0, 1);
        p.crediter(f64::MAX);
        p.crediter(f64::MAX);
        assert!(p.argent.is_finite() && p.total_gagne.is_finite());
    }

    #[test]
    fn equiper_les_meilleurs() {
        let mut p = Partie::nouvelle(0, 11);
        p.argent = 1e15;
        for i in 0..80 {
            let _ = p.acheter_oeuf(i % 4);
        }
        // Mélange : quelques bons poissons en réserve.
        for f in p.poissons.iter_mut().take(20) {
            f.actif = !f.actif;
        }
        let (avant, apres) = p.equiper_meilleurs();
        assert!(apres >= avant);
        assert_eq!(p.nb_actifs(), CAPACITE.min(p.poissons.len()));
        let min_actif = p
            .poissons
            .iter()
            .filter(|f| f.actif)
            .map(Poisson::revenu)
            .fold(f64::MAX, f64::min);
        assert!(p
            .poissons
            .iter()
            .filter(|f| !f.actif)
            .all(|f| f.revenu() <= min_actif));
        // Équiper auto : un poisson éclos meilleur que le plus faible le remplace.
        p.reglages.equiper_auto = true;
        let id = p.ajouter(Rarete::Irreel, 1.5, 0);
        assert!(p.poissons.iter().find(|f| f.id == id).unwrap().actif);
        assert_eq!(p.nb_actifs(), CAPACITE);
    }

    #[test]
    fn tirage_des_formes() {
        // Shiny ≈ 1/100 et modificateurs aux chances de la table, indépendants.
        let n = 2_000_000u64;
        let mut shiny = 0u64;
        let mut c = [0u64; Modif::TOUS.len()];
        let mut shiny_et_dore = 0u64;
        for g in 0..n {
            let f = Forme::tirer(g.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ 0xabc);
            shiny += f.shiny as u64;
            if let Some(m) = f.modif {
                c[m.index()] += 1;
            }
            shiny_et_dore += (f.shiny && f.modif == Some(Modif::Dore)) as u64;
        }
        let pct = |k: u64| k as f64 / n as f64;
        assert!(
            (pct(shiny) - CHANCE_SHINY).abs() < 0.0006,
            "shiny {}",
            pct(shiny)
        );
        for m in Modif::TOUS {
            let vu = pct(c[m.index()]);
            // 5 écarts-types.
            let tol = 5.0 * (m.chance() / n as f64).sqrt();
            assert!(
                (vu - m.chance()).abs() < tol,
                "{} : {vu} au lieu de {}",
                m.nom(),
                m.chance()
            );
        }
        let attendu = CHANCE_SHINY * Modif::Dore.chance();
        assert!((pct(shiny_et_dore) - attendu).abs() < 5.0 * (attendu / n as f64).sqrt());
        // Moyenne des modificateurs ≈ +13 %, shiny compris ≈ +17 %.
        let e = Forme::esperance();
        assert!((1.16..1.18).contains(&e), "{e}");
    }

    #[test]
    fn multiplicateurs_de_forme() {
        let mut p = Partie::nouvelle(0, 5);
        let f = &mut p.poissons[0];
        assert!(!f.shiny && f.modif.is_none());
        let base = f.revenu();
        let cout = f.cout_nourrir().unwrap();
        f.shiny = true;
        assert!((f.revenu() / base - 5.0).abs() < 1e-9);
        f.modif = Some(Modif::Galaxie);
        assert!((f.revenu() / base - 60.0).abs() < 1e-9);
        // Un niveau se rentabilise pareil, shiny ou non.
        assert!((f.cout_nourrir().unwrap() / cout - 60.0).abs() < 1e-9);
        f.shiny = false;
        for m in Modif::TOUS {
            f.modif = Some(m);
            assert!((f.revenu() / base - m.multiplicateur()).abs() < 1e-9);
        }
        // Le poisson offert est normal, quelle que soit la graine.
        for g in 0..500 {
            let q = Partie::nouvelle(0, g);
            assert!(!q.poissons[0].shiny && q.poissons[0].modif.is_none());
            assert_eq!(q.revenu_par_s(), 1.0);
        }
    }

    #[test]
    fn index_complets_rapportent() {
        let mut p = Partie::nouvelle(0, 8);
        let r0 = p.revenu_par_s();
        assert_eq!(p.index_complets(), 0);
        let cles: Vec<&str> = crate::especes::catalogue().iter().map(|e| e.cle).collect();
        // Toutes les espèces vues normales : index Normal complet, +5 %.
        for c in &cles {
            p.index.insert(c.to_string(), 1);
        }
        p.recompter_index();
        assert_eq!(p.index_complets(), 1);
        assert_eq!(p.completion_index(0), (cles.len(), cles.len()));
        assert!((p.revenu_par_s() / r0 - 1.05).abs() < 1e-9);
        // Il manque une espèce à l'index Shiny : pas complet ; puis complet.
        for c in &cles[1..] {
            *p.index.get_mut(*c).unwrap() |= 1 << (NB_INDEX - 1);
        }
        p.recompter_index();
        assert_eq!(p.index_complets(), 1);
        assert_eq!(p.completion_index(NB_INDEX - 1).0, cles.len() - 1);
        // L'éclosion marque l'index et recompte : le shiny manquant arrive par `ajouter`.
        *p.index.get_mut(cles[0]).unwrap() |= 1 << (NB_INDEX - 1);
        p.recompter_index();
        assert_eq!(p.index_complets(), 2);
        assert!((p.revenu_par_s() / r0 - 1.10).abs() < 1e-9);
        // Les éclosions marquent l'index de leur forme.
        let mut q = Partie::nouvelle(0, 21);
        q.argent = 1e300;
        for i in 0..3000 {
            let _ = q.acheter_oeuf(i % 5);
        }
        for f in &q.poissons {
            assert!(q.dans_index(&f.espece, 0));
            if let Some(m) = f.modif {
                assert!(q.dans_index(&f.espece, m.index() + 1));
            }
            if f.shiny {
                assert!(q.dans_index(&f.espece, NB_INDEX - 1));
            }
        }
        assert!(q.poissons.iter().any(|f| f.shiny) && q.poissons.iter().any(|f| f.modif.is_some()));
    }

    #[test]
    fn equiper_les_meilleurs_avec_formes() {
        // Un shiny ou un modifié d'une rareté plus basse peut battre un poisson plus rare :
        // « Équiper les meilleurs » suit le revenu réel.
        let mut p = Partie::nouvelle(0, 13);
        p.argent = 1e15;
        for i in 0..120 {
            let _ = p.acheter_oeuf(i % 3);
        }
        for (k, f) in p.poissons.iter_mut().enumerate() {
            f.shiny = k % 7 == 0;
            f.modif = Modif::TOUS.get(k % 11).copied();
            f.actif = k % 2 == 0;
        }
        let (avant, apres) = p.equiper_meilleurs();
        assert!(apres >= avant);
        assert_eq!(p.nb_actifs(), CAPACITE);
        let min_actif = p
            .poissons
            .iter()
            .filter(|f| f.actif)
            .map(Poisson::revenu)
            .fold(f64::MAX, f64::min);
        assert!(p
            .poissons
            .iter()
            .filter(|f| !f.actif)
            .all(|f| f.revenu() <= min_actif));
        // Somme maximale : aucun échange réserve ↔ aquarium ne rapporte plus.
        let mut tri: Vec<f64> = p.poissons.iter().map(Poisson::revenu).collect();
        tri.sort_by(|a, b| b.total_cmp(a));
        let meilleur: f64 = tri.iter().take(CAPACITE).sum::<f64>() * p.bonus_index();
        assert!((p.revenu_par_s() - meilleur).abs() <= meilleur * 1e-12);
        // Auto : un shiny commun qui rapporte plus que le plus faible le remplace.
        p.reglages.equiper_auto = true;
        let faible = min_actif;
        let id = p.ajouter(Rarete::Commun, 1.0, 0);
        let f = p.poissons.iter_mut().find(|f| f.id == id).unwrap();
        let nouveau = f.revenu();
        assert_eq!(f.actif, nouveau > faible);
    }

    #[test]
    fn pitie_garantit_la_rarete() {
        // Le N-ième œuf de suite sans la rareté garantie la donne ; le compteur repart à zéro.
        for (i, o) in OEUFS.iter().enumerate() {
            let Some((min, n)) = o.pitie else { continue };
            let mut p = Partie::nouvelle(0, 11 + i as u64);
            p.argent = 1e300;
            let quand = o.dispo.map_or(0, |(_, debut, _)| debut + 1);
            let mut sans = 0;
            for _ in 0..3000 {
                let id = p.acheter_oeuf_a(i, quand).unwrap();
                p.oeufs_ouverts[i] = 0;
                let f = p.poissons.pop().unwrap();
                assert_eq!(f.id, id);
                if f.rarete >= min {
                    // Garantie seulement au N-ième (le tirage normal peut aussi la donner).
                    assert!(!p.derniere_pitie || sans + 1 == n, "{}", o.nom);
                    sans = 0;
                } else {
                    assert!(!p.derniere_pitie);
                    sans += 1;
                }
                assert!(sans < n, "{} : {sans} œufs sans {:?}", o.nom, min);
                assert_eq!(p.pitie[i], sans);
                assert_eq!(p.pitie_restante(i), Some((min, n - sans)));
            }
        }
        assert_eq!(Partie::nouvelle(0, 1).pitie_restante(4), None);
    }

    #[test]
    fn pitie_respecte_les_proportions() {
        // Le tirage forcé garde les proportions des raretés hautes de l'œuf (cosmique :
        // Mythique 11, Céleste 4,5, Irréel 0,5 → 68,75 % / 28,1 % / 3,1 %).
        let mut a = Alea(3);
        let mut n = [0u32; 9];
        for _ in 0..100_000 {
            n[Partie::tirer_rarete_min(&mut a, &OEUFS[3], Rarete::Mythique).index()] += 1;
        }
        assert_eq!(n[..5].iter().sum::<u32>(), 0);
        assert!((n[5] as f64 / 1000.0 - 68.75).abs() < 0.8, "{n:?}");
        assert!((n[7] as f64 / 1000.0 - 3.125).abs() < 0.4, "{n:?}");
    }

    #[test]
    fn ouverture_multiple() {
        let mut p = Partie::nouvelle(0, 5);
        let lot = p.prix_lot(0, 10);
        let attendu: f64 = (0..10)
            .map(|k| 80.0 * (1.0 + k as f64 / 10.0).powi(2))
            .sum();
        assert!((lot - attendu).abs() < 1e-6);
        // Pas assez pour le lot entier : rien n'est acheté.
        p.argent = lot - 1.0;
        assert_eq!(p.acheter_lot(0, 10, 0), Err(Refus::PasAssez));
        assert_eq!(p.oeufs_ouverts[0], 0);
        assert_eq!(p.poissons.len(), 1);
        p.argent = lot;
        let v = p.acheter_lot(0, 10, 0).unwrap();
        assert_eq!(v.len(), 10);
        assert_eq!(p.oeufs_ouverts[0], 10);
        assert_eq!(p.poissons.len(), 11);
        assert!(p.argent.abs() < 1e-3);
        let mut ids: Vec<u64> = v.iter().map(|o| o.id).collect();
        ids.dedup();
        assert_eq!(ids.len(), 10);
        // Œuf exceptionnel hors de son moment : refusé.
        p.argent = 1e300;
        assert_eq!(p.acheter_lot(7, 3, 0), Err(Refus::Indisponible));
    }

    #[test]
    fn nourriture_achetee_jetee_et_bonus() {
        let mut p = Partie::nouvelle(0, 4);
        let base = p.revenu_par_s();
        assert_eq!(p.acheter_nourriture(5, 1), Err(Refus::PasAssez));
        p.argent = 1e10;
        p.acheter_nourriture(5, 2).unwrap();
        assert_eq!(p.nourriture[5], 2);
        assert!((p.argent - (1e10 - 3e9)).abs() < 1.0);
        p.prendre_nourriture(5).unwrap();
        p.prendre_nourriture(5).unwrap();
        assert_eq!(p.prendre_nourriture(5), Err(Refus::PasAssez));
        // Quatre bouchées de cage = un repas : revenu ×4 pendant 4 min.
        for _ in 0..4 {
            p.manger(5, 0.25);
        }
        assert!((p.revenu_par_s() / base - 4.0).abs() < 1e-9);
        let avant = p.argent;
        p.avancer(10.0);
        assert!((p.argent - avant - base * 40.0).abs() < 1e-6);
        assert!((p.festin.unwrap().reste - 230.0).abs() < 1e-9);
        // Hors ligne : le bonus compte pour le temps qui lui restait, puis s'arrête.
        p.horodatage = 1000;
        let gain = p.gains_hors_ligne(1000 + 3600);
        assert!(
            (gain - base * (3600.0 + 3.0 * 230.0)).abs() < 1e-6,
            "{gain}"
        );
        assert_eq!(p.festin, None);
        assert_eq!(p.revenu_par_s(), base);
        // Chance aux éclosions : la cage double les bonus de l'œuf (shiny plus fréquents).
        let shiny = |festin: bool| {
            let mut q = Partie::nouvelle(0, 11);
            q.argent = 1e300;
            let mut n = 0;
            for _ in 0..3000 {
                if festin {
                    q.festin = Some(Festin {
                        niveau: 6,
                        reste: 100.0,
                    });
                }
                let id = q.acheter_oeuf_a(0, 0).unwrap();
                n += q.poissons.iter().any(|f| f.id == id && f.shiny) as u32;
                q.poissons.retain(|f| f.id != id);
                q.oeufs_ouverts[0] = 0;
            }
            n
        };
        assert!(
            shiny(true) > shiny(false) * 2,
            "{} {}",
            shiny(true),
            shiny(false)
        );
    }

    #[test]
    fn tout_vendre_epargne_les_verrouilles() {
        let mut p = Partie::nouvelle(0, 6);
        p.argent = 1e9;
        for _ in 0..6 {
            p.acheter_oeuf(0).unwrap();
        }
        let ids: Vec<u64> = p.poissons.iter().map(|f| f.id).collect();
        for &id in &ids[1..5] {
            p.basculer(id).unwrap();
        }
        p.verrouiller(ids[2]).unwrap();
        // Vente individuelle refusée tant qu'il est verrouillé, rien ne bouge.
        let argent = p.argent;
        assert_eq!(p.relacher(ids[2]), Err(Refus::Verrouille));
        assert_eq!(p.argent, argent);
        let attendu: f64 = p
            .poissons
            .iter()
            .filter(|f| !f.actif && !f.verrou)
            .map(Poisson::prix_relache)
            .sum();
        assert_eq!(p.prix_tout_vendre(), (3, attendu));
        assert_eq!(p.tout_vendre(), (3, attendu));
        assert!((p.argent - argent - attendu).abs() < 1e-6);
        // Restent : l'aquarium entier et le verrouillé de la réserve.
        assert!(p.poissons.iter().any(|f| f.id == ids[2] && !f.actif));
        assert_eq!(p.poissons.len(), ids.len() - 3);
        assert_eq!(p.tout_vendre(), (0, 0.0));
        // Cadenas retiré : il se vend de nouveau.
        p.verrouiller(ids[2]).unwrap();
        assert!(p.relacher(ids[2]).is_ok());
    }
}

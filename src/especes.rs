//! Catalogue des espèces : nom, rareté, classe de taille, façon de bouger et variantes de
//! couleur. Le dessin de chaque espèce est une image pixel art (`assets/especes/<clé>.png`,
//! voir `poisson.rs`).

use crate::jeu::{Alea, Rarete};

/// Classe de taille : longueur typique sur la toile (~480 px de large) et place à l'écran.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Taille {
    /// ~8-12 px.
    Minuscule,
    /// ~13-22 px.
    Petite,
    /// ~23-40 px.
    Moyenne,
    /// ~40-80 px.
    Grande,
    /// Baleines, orque, grands requins : nagent au second plan.
    Enorme,
    /// Créatures irréelles qui prennent une grande part de l'écran, tout au fond.
    Colossale,
}

impl Taille {
    /// Places prises dans la limite des poissons affichés (lisibilité de l'écran).
    pub fn places(self) -> usize {
        // Lot 4 : la place à l'écran est surtout réglée par la surface des dessins (voir
        // `Aquarium::synchroniser`) ; une colossale compte double.
        match self {
            Taille::Colossale => 2,
            _ => 1,
        }
    }

    /// Profondeur 0 (devant) .. 1 (tout au fond) : teinte d'eau et ordre de dessin.
    pub fn profondeur(self) -> f32 {
        match self {
            Taille::Enorme => 0.22,
            Taille::Colossale => 0.38,
            _ => 0.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mouvement {
    /// Poisson : cibles, pauses, battement de queue.
    Nage,
    /// Petits poissons en bancs.
    Banc,
    /// Gros animaux tranquilles.
    Lent,
    /// Phoque, dauphin : nage souple et accélérations soudaines.
    Fonce,
    /// Anguilles, serpents : corps ondulant.
    Ondule,
    /// Raies, tortues : battements d'ailes et planés.
    Plane,
    /// Méduses : pulsations qui les font monter, puis elles se laissent couler.
    Pulse,
    /// Calmars : jets par à-coups.
    Jet,
    /// Gobies, crevettes, poulpes : près du sable, petits bonds.
    Fond,
    /// Hippocampe : debout, lent, se balance.
    Debout,
    /// Crabe, isopode : marche sur le sable.
    Marche,
    /// Dieux et yeux des abysses : dérive lente en suspension.
    Flotte,
    /// Kraken : posé tout en bas, tentacules qui montent.
    Abysse,
    /// Serpents colossaux : traversent l'écran d'un bord à l'autre puis disparaissent.
    Traverse,
}

/// Variante de couleur : décalage de teinte (tours), saturation et valeur des couleurs 0 et 1.
pub struct Variante {
    pub nom: &'static str,
    pub teinte: f32,
    pub sat: f32,
    pub val: f32,
}

const fn v(nom: &'static str, teinte: f32, sat: f32, val: f32) -> Variante {
    Variante {
        nom,
        teinte,
        sat,
        val,
    }
}

pub struct Espece {
    /// Identifiant stable (sauvegarde).
    pub cle: &'static str,
    pub nom: &'static str,
    pub rarete: Rarete,
    pub taille: Taille,
    pub mouvement: Mouvement,
    /// Taille du dessin en pixels fins (celle de son image).
    pub dims: (i32, i32),
    pub variantes: &'static [Variante],
    /// Poids du tirage dans sa rareté (1 = normal).
    pub poids: f32,
}

impl Espece {
    /// Nom complet d'un individu (« Requin-marteau albinos »).
    pub fn nom_variante(&self, variante: usize) -> String {
        let nom = crate::i18n::tr(self.nom);
        match self.variantes.get(variante).map(|v| v.nom) {
            // Anglais : l'adjectif avant le nom (« Albino Hammerhead shark »).
            Some(n) if !n.is_empty() && crate::i18n::anglais() => {
                format!("{} {nom}", crate::i18n::tr(n))
            }
            Some(n) if !n.is_empty() => format!("{nom} {n}"),
            _ => nom.to_string(),
        }
    }
}

/// Variantes courantes.
pub const V_POISSON: &[Variante] = &[
    v("", 0.0, 1.0, 1.0),
    v("doré", 0.06, 1.0, 1.08),
    v("azur", 0.5, 1.0, 1.0),
    v("albinos", 0.0, 0.15, 1.25),
];
pub const V_SOMBRE: &[Variante] = &[
    v("", 0.0, 1.0, 1.0),
    v("des grands fonds", 0.08, 0.8, 0.7),
    v("pâle", 0.0, 0.35, 1.2),
];
pub const V_VIF: &[Variante] = &[
    v("", 0.0, 1.0, 1.0),
    v("pourpre", 0.82, 1.0, 1.0),
    v("émeraude", 0.33, 1.0, 1.0),
    v("solaire", 0.12, 1.0, 1.1),
];
pub const V_COSMIQUE: &[Variante] = &[
    v("", 0.0, 1.0, 1.0),
    v("aurorale", 0.3, 1.0, 1.05),
    v("pourpre", 0.85, 1.0, 1.0),
];

/// Préhistorique : couleurs d'origine, fossile (pierre pâle), ambre.
pub const V_FOSSILE: &[Variante] = &[
    v("", 0.0, 1.0, 1.0),
    v("fossile", 0.0, 0.2, 1.1),
    v("d'ambre", 0.08, 1.4, 1.05),
];

pub fn catalogue() -> &'static [Espece] {
    static CAT: std::sync::OnceLock<Vec<Espece>> = std::sync::OnceLock::new();
    CAT.get_or_init(|| {
        use Mouvement as M;
        use Rarete as R;
        use Taille as T;
        let ep = |cle, nom, rarete, taille, mouvement, variantes, poids| {
            let dims = crate::poisson::dims_png(cle);
            Espece {
                cle,
                nom,
                rarete,
                taille,
                mouvement,
                dims,
                variantes,
                poids,
            }
        };
        let e = |cle, nom, rarete, taille, mouvement, variantes| {
            ep(cle, nom, rarete, taille, mouvement, variantes, 1.0)
        };
        // L'ordre compte : il fixe le tirage par graine (sauvegardes et équilibrage).
        vec![
            e(
                "clown",
                "Poisson-clown",
                R::Commun,
                T::Petite,
                M::Nage,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("noir", 0.0, 0.6, 0.32),
                        v("doré", 0.06, 0.9, 1.1),
                        v("rose", -0.05, 0.55, 1.1),
                    ]
                },
            ),
            e("neon", "Néon", R::Commun, T::Minuscule, M::Banc, V_VIF),
            e("guppy", "Guppy", R::Commun, T::Minuscule, M::Banc, V_VIF),
            e(
                "rouge",
                "Poisson rouge",
                R::Commun,
                T::Petite,
                M::Nage,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("blanc", 0.0, 0.12, 1.2),
                        v("noir", 0.0, 0.4, 0.3),
                        v("vermillon", -0.04, 1.1, 0.95),
                    ]
                },
            ),
            e(
                "sardine",
                "Sardine",
                R::Commun,
                T::Petite,
                M::Banc,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("dorée", -0.4, 1.0, 1.0),
                        v("argentée", 0.0, 0.3, 1.2),
                    ]
                },
            ),
            e(
                "gobie",
                "Gobie",
                R::Commun,
                T::Minuscule,
                M::Fond,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("citron", -0.05, 2.2, 1.25),
                        v("des récifs", 0.45, 1.0, 1.0),
                    ]
                },
            ),
            e(
                "chirurgien",
                "Chirurgien bleu",
                R::PeuCommun,
                T::Petite,
                M::Nage,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("violet", 0.1, 1.0, 1.0),
                        v("turquoise", -0.08, 1.0, 1.1),
                    ]
                },
            ),
            e(
                "ange",
                "Poisson-ange",
                R::PeuCommun,
                T::Petite,
                M::Nage,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("royal", 0.12, 1.0, 1.0),
                        v("flamme", 0.45, 1.0, 1.1),
                    ]
                },
            ),
            e(
                "globe",
                "Poisson-globe",
                R::PeuCommun,
                T::Petite,
                M::Nage,
                V_POISSON,
            ),
            e(
                "hippocampe",
                "Hippocampe",
                R::PeuCommun,
                T::Petite,
                M::Debout,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("rose", -0.12, 0.8, 1.05),
                        v("noir", 0.0, 0.4, 0.4),
                        v("vert", 0.18, 0.9, 0.9),
                    ]
                },
            ),
            e(
                "crabe",
                "Crabe",
                R::PeuCommun,
                T::Petite,
                M::Marche,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("bleu", 0.58, 0.8, 0.9),
                        v("violet", 0.82, 0.7, 0.85),
                        v("des sables", 0.08, 0.5, 1.15),
                    ]
                },
            ),
            e(
                "crevette",
                "Crevette",
                R::PeuCommun,
                T::Petite,
                M::Fond,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("nettoyeuse", 0.0, 1.3, 0.95),
                        v("fantôme", 0.0, 0.2, 1.2),
                        v("bleue", 0.6, 0.9, 1.0),
                    ]
                },
            ),
            e(
                "meduse",
                "Méduse",
                R::Rare,
                T::Moyenne,
                M::Pulse,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("lune", 0.55, 0.6, 1.0),
                        v("dorée", 0.2, 0.9, 1.0),
                        v("cristal", 0.0, 0.15, 1.1),
                    ]
                },
            ),
            e("poulpe", "Poulpe", R::Rare, T::Moyenne, M::Fond, V_VIF),
            e(
                "murene",
                "Murène",
                R::Rare,
                T::Moyenne,
                M::Ondule,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("léopard", -0.12, 1.1, 1.1),
                        v("ruban", 0.4, 1.2, 1.0),
                    ]
                },
            ),
            e(
                "anguille",
                "Anguille",
                R::Rare,
                T::Moyenne,
                M::Ondule,
                V_SOMBRE,
            ),
            e(
                "rascasse",
                "Rascasse volante",
                R::Rare,
                T::Moyenne,
                M::Lent,
                V_POISSON,
            ),
            e(
                "tortue",
                "Tortue de mer",
                R::Rare,
                T::Moyenne,
                M::Plane,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("verte", 0.15, 0.9, 0.95),
                        v("imbriquée", -0.03, 1.2, 1.1),
                    ]
                },
            ),
            e(
                "calamar",
                "Calamar",
                R::Rare,
                T::Moyenne,
                M::Jet,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("luciole", 0.5, 0.7, 1.0),
                        v("rouge", 0.0, 1.8, 0.85),
                    ]
                },
            ),
            e(
                "manta",
                "Raie manta",
                R::Epique,
                T::Grande,
                M::Plane,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("noire", 0.0, 0.6, 0.6),
                        v("des récifs", 0.45, 2.0, 1.4),
                    ]
                },
            ),
            e(
                "marteau",
                "Requin-marteau",
                R::Epique,
                T::Grande,
                M::Nage,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("bronze", -0.1, 2.0, 1.0),
                        v("albinos", 0.0, 0.1, 1.35),
                    ]
                },
            ),
            e(
                "espadon",
                "Espadon",
                R::Epique,
                T::Grande,
                M::Nage,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("pourpre", 0.12, 1.2, 1.0),
                        v("d'acier", 0.0, 0.3, 1.2),
                    ]
                },
            ),
            e(
                "phoque",
                "Phoque",
                R::Epique,
                T::Grande,
                M::Fonce,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("moine", 0.05, 1.5, 0.75),
                        v("annelé", 0.0, 0.2, 1.15),
                    ]
                },
            ),
            e(
                "dauphin",
                "Dauphin",
                R::Epique,
                T::Grande,
                M::Fonce,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("rose", -0.62, 1.6, 1.2),
                        v("tacheté", 0.0, 0.9, 0.85),
                    ]
                },
            ),
            e(
                "mola",
                "Poisson-lune",
                R::Epique,
                T::Grande,
                M::Lent,
                V_POISSON,
            ),
            e(
                "blanc",
                "Grand requin blanc",
                R::Legendaire,
                T::Enorme,
                M::Lent,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("ancien", 0.08, 1.5, 0.8),
                        v("fantôme", 0.0, 0.3, 1.3),
                    ]
                },
            ),
            e(
                "orque",
                "Orque",
                R::Legendaire,
                T::Enorme,
                M::Lent,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("blanche", 0.0, 0.0, 6.0),
                        v("des glaces", 0.55, 4.0, 2.2),
                    ]
                },
            ),
            e(
                "requin_baleine",
                "Requin-baleine",
                R::Legendaire,
                T::Enorme,
                M::Lent,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("doré", -0.45, 1.3, 1.2),
                        v("nocturne", 0.1, 0.8, 0.6),
                    ]
                },
            ),
            e(
                "narval",
                "Narval",
                R::Legendaire,
                T::Grande,
                M::Lent,
                V_SOMBRE,
            ),
            e(
                "calamar_geant",
                "Calamar géant",
                R::Legendaire,
                T::Enorme,
                M::Jet,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("des abysses", 0.75, 0.8, 0.8),
                        v("opale", 0.0, 0.25, 1.3),
                    ]
                },
            ),
            e(
                "baudroie",
                "Baudroie abyssale",
                R::Legendaire,
                T::Moyenne,
                M::Nage,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("d'encre", 0.6, 0.6, 0.6),
                        v("rouille", 0.02, 1.8, 1.4),
                    ]
                },
            ),
            e(
                "dragon_feuillu",
                "Dragon de mer feuillu",
                R::Mythique,
                T::Moyenne,
                M::Debout,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("corail", -0.1, 1.1, 1.0),
                        v("algue", 0.15, 1.0, 0.85),
                    ]
                },
            ),
            e(
                "coelacanthe",
                "Cœlacanthe",
                R::Mythique,
                T::Grande,
                M::Lent,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("brun", -0.55, 0.8, 0.8),
                        v("fossile", 0.0, 0.2, 1.1),
                    ]
                },
            ),
            e(
                "lutin",
                "Requin-lutin",
                R::Mythique,
                T::Grande,
                M::Lent,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("spectral", 0.6, 0.5, 1.1),
                        v("sanguin", 0.0, 1.6, 0.85),
                    ]
                },
            ),
            e(
                "bosse",
                "Baleine à bosse",
                R::Mythique,
                T::Enorme,
                M::Lent,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("blanche", 0.0, 0.1, 3.5),
                        v("bleue", 0.02, 1.6, 1.3),
                    ]
                },
            ),
            e(
                "isopode",
                "Isopode géant",
                R::Mythique,
                T::Moyenne,
                M::Marche,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("doré", -0.6, 1.5, 1.05),
                        v("de jais", 0.0, 0.5, 0.45),
                    ]
                },
            ),
            e(
                "hydre",
                "Hydre",
                R::Mythique,
                T::Enorme,
                M::Flotte,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("d'obsidienne", 0.4, 0.4, 0.5),
                        v("pourpre", 0.5, 1.0, 1.1),
                    ]
                },
            ),
            e(
                "cadborosaure",
                "Cadborosaure",
                R::Mythique,
                T::Enorme,
                M::Ondule,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("verdâtre", 0.15, 1.0, 0.9),
                        v("cendré", 0.0, 0.2, 1.1),
                    ]
                },
            ),
            e(
                "baleine_stellaire",
                "Baleine stellaire",
                R::Celeste,
                T::Enorme,
                M::Lent,
                V_COSMIQUE,
            ),
            e(
                "meduse_nebuleuse",
                "Méduse nébuleuse",
                R::Celeste,
                T::Grande,
                M::Pulse,
                V_COSMIQUE,
            ),
            e(
                "raie_aurore",
                "Raie aurore",
                R::Celeste,
                T::Grande,
                M::Plane,
                V_COSMIQUE,
            ),
            e(
                "carpe_comete",
                "Carpe-comète",
                R::Celeste,
                T::Moyenne,
                M::Nage,
                V_COSMIQUE,
            ),
            e(
                "kraken",
                "Kraken",
                R::Irreel,
                T::Colossale,
                M::Abysse,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("des glaces", 0.55, 0.5, 1.1),
                        v("d'encre", 0.75, 0.6, 0.6),
                    ]
                },
            ),
            e(
                "cthulhu",
                "Cthulhu",
                R::Irreel,
                T::Colossale,
                M::Flotte,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("abyssal", 0.3, 0.8, 0.8),
                        v("de corail", -0.4, 1.0, 1.2),
                    ]
                },
            ),
            e(
                "leviathan",
                "Léviathan",
                R::Irreel,
                T::Colossale,
                M::Traverse,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("de braise", -0.5, 1.4, 1.0),
                        v("d'argent", 0.0, 0.2, 1.4),
                    ]
                },
            ),
            e(
                "oeil",
                "Œil des abysses",
                R::Irreel,
                T::Colossale,
                M::Flotte,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("vert", 0.3, 1.0, 0.9),
                        v("pâle", 0.0, 0.3, 1.4),
                    ]
                },
            ),
            e(
                "bloop",
                "Le Bloop",
                R::Irreel,
                T::Colossale,
                M::Lent,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("spectral", 0.55, 2.0, 1.0),
                        v("cendré", 0.0, 0.5, 0.75),
                    ]
                },
            ),
            e(
                "grand_abyssal",
                "Le Grand Abyssal",
                R::Irreel,
                T::Colossale,
                M::Traverse,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("blême", 0.0, 0.5, 2.4),
                        v("brun", 0.05, 3.0, 1.2),
                    ]
                },
            ),
            e(
                "lusca",
                "Lusca",
                R::Irreel,
                T::Enorme,
                M::Nage,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("des trous bleus", 0.45, 1.6, 0.9),
                        v("corail", -0.55, 1.3, 1.1),
                    ]
                },
            ),
            ep(
                "jormungandr",
                "Jörmungandr",
                R::Irreel,
                T::Colossale,
                M::Traverse,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("de cendre", 0.0, 0.2, 0.9),
                        v("de cuivre", -0.3, 1.2, 1.2),
                    ]
                },
                0.35,
            ),
            // Lot 3 (ajoutées à la fin : l'ordre des espèces existantes ne bouge pas).
            e(
                "loch_ness",
                "Monstre du Loch Ness",
                R::Irreel,
                T::Enorme,
                M::Lent,
                const {
                    &[
                        v("", 0.0, 1.0, 1.0),
                        v("des brumes", 0.0, 0.3, 1.2),
                        v("des tourbières", 0.08, 1.3, 0.8),
                    ]
                },
            ),
            e(
                "megalodon",
                "Mégalodon",
                R::Prehistorique,
                T::Colossale,
                M::Lent,
                V_FOSSILE,
            ),
            e(
                "mosasaure",
                "Mosasaure",
                R::Prehistorique,
                T::Colossale,
                M::Lent,
                V_FOSSILE,
            ),
            e(
                "liopleurodon",
                "Liopleurodon",
                R::Prehistorique,
                T::Enorme,
                M::Lent,
                V_FOSSILE,
            ),
            e(
                "ichthyosaure",
                "Ichthyosaure",
                R::Prehistorique,
                T::Grande,
                M::Fonce,
                V_FOSSILE,
            ),
            e(
                "dunkleosteus",
                "Dunkleosteus",
                R::Prehistorique,
                T::Grande,
                M::Lent,
                V_FOSSILE,
            ),
            e(
                "helicoprion",
                "Hélicoprion",
                R::Prehistorique,
                T::Grande,
                M::Nage,
                V_FOSSILE,
            ),
            e(
                "anomalocaris",
                "Anomalocaris",
                R::Prehistorique,
                T::Moyenne,
                M::Plane,
                V_FOSSILE,
            ),
            e(
                "ammonite",
                "Ammonite géante",
                R::Prehistorique,
                T::Moyenne,
                M::Flotte,
                V_FOSSILE,
            ),
            e(
                "basilosaurus",
                "Basilosaurus",
                R::Prehistorique,
                T::Colossale,
                M::Traverse,
                V_FOSSILE,
            ),
            e(
                "trilobite",
                "Trilobite géant",
                R::Prehistorique,
                T::Petite,
                M::Marche,
                V_FOSSILE,
            ),
        ]
    })
}

pub fn par_cle(cle: &str) -> Option<&'static Espece> {
    catalogue().iter().find(|e| e.cle == cle)
}

pub fn de_rarete(r: Rarete) -> impl Iterator<Item = &'static Espece> {
    catalogue().iter().filter(move |e| e.rarete == r)
}

/// Espèce et variante d'un poisson, tirées de sa graine (pondérées par `poids`).
/// Sert à l'éclosion et à la migration des sauvegardes du lot 1.
pub fn choisir(r: Rarete, graine: u64) -> (&'static Espece, u8) {
    let mut a = Alea(graine ^ 0x5bec_1e5a_u64.rotate_left(17));
    let liste: Vec<&Espece> = de_rarete(r).collect();
    let total: f32 = liste.iter().map(|e| e.poids).sum();
    let mut x = a.f() * total;
    let mut e = liste[liste.len() - 1];
    for &c in &liste {
        x -= c.poids;
        if x < 0.0 {
            e = c;
            break;
        }
    }
    // La variante de base sort une fois sur deux.
    let var = if a.f() < 0.5 {
        0
    } else {
        1 + a.n(e.variantes.len().max(2) as u32 - 1) as u8
    };
    (e, var.min(e.variantes.len() as u8 - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_coherent() {
        let c = catalogue();
        assert!(c.len() >= 40, "{} espèces", c.len());
        for r in Rarete::TOUTES {
            assert!(
                de_rarete(r).count() >= 3,
                "{} : moins de 3 espèces",
                r.nom()
            );
        }
        let mut cles: Vec<&str> = c.iter().map(|e| e.cle).collect();
        let mut noms: Vec<&str> = c.iter().map(|e| e.nom).collect();
        cles.sort();
        noms.sort();
        cles.dedup();
        noms.dedup();
        assert_eq!(cles.len(), c.len(), "clés en double");
        assert_eq!(noms.len(), c.len(), "noms en double");
        for e in c {
            assert!(!e.variantes.is_empty() && e.poids > 0.0, "{}", e.cle);
        }
    }

    #[test]
    fn tailles_relatives() {
        // Échelle « de jeu » : dessins en pleine finesse (jamais réduits), hiérarchie tassée mais
        // lisible (un poisson-clown fait environ un quart d'un requin).
        let lg = |k| {
            let d = par_cle(k).unwrap().dims;
            d.0.max(d.1)
        };
        for petit in [
            "clown",
            "neon",
            "gobie",
            "sardine",
            "crabe",
            "hippocampe",
            "crevette",
        ] {
            assert!(
                lg(petit) * 5 / 2 <= lg("marteau"),
                "{petit} : {}",
                lg(petit)
            );
        }
        assert!(lg("marteau") < lg("requin_baleine") && lg("requin_baleine") < lg("bosse"));
        assert!(lg("clown") < lg("tortue") && lg("tortue") < lg("dauphin"));
        // v0.8 : néon et guppy plus petits que le clown, koï plus petite que le requin blanc.
        assert!(lg("neon") < lg("clown") * 4 / 5 && lg("guppy") < lg("clown") * 4 / 5);
        assert!(lg("carpe_comete") < lg("blanc") * 2 / 3);
    }

    #[test]
    fn tirage_par_graine() {
        for r in Rarete::TOUTES {
            let mut vues = std::collections::HashSet::new();
            for g in 0..2000u64 {
                let (e, v) = choisir(r, g.wrapping_mul(0x9e37_79b9_7f4a_7c15));
                assert_eq!(e.rarete, r);
                assert!((v as usize) < e.variantes.len());
                assert_eq!(
                    choisir(r, g.wrapping_mul(0x9e37_79b9_7f4a_7c15)).0.cle,
                    e.cle
                );
                vues.insert(e.cle);
            }
            // Toutes les espèces d'une rareté finissent par sortir.
            assert_eq!(vues.len(), de_rarete(r).count(), "{}", r.nom());
        }
    }
}

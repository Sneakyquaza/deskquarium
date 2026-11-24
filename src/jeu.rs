//! Logique du jeu idle : raretés, œufs, poissons, revenus, gains hors ligne, format des nombres.
//! Toutes les valeurs d'équilibrage sont ici (courbes documentées dans `docs/equilibrage.md`).

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
}

pub const OEUFS: [Oeuf; 5] = [
    Oeuf {
        nom: "Œuf de rivière",
        description: "Pondu entre deux galets. Souvent des poissons communs.",
        prix: 80.0,
        chances: [70.0, 25.0, 4.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
        coquille: [[0xd9, 0xe8, 0xc8], [0x7f, 0xa8, 0x6a]],
    },
    Oeuf {
        nom: "Œuf de lagon",
        description: "Tiède et nacré, il sent le corail.",
        prix: 5_000.0,
        chances: [30.0, 42.0, 20.0, 6.5, 1.5, 0.0, 0.0, 0.0, 0.0],
        coquille: [[0xa8, 0xee, 0xf0], [0xff, 0xb0, 0xc8]],
    },
    Oeuf {
        nom: "Œuf des abysses",
        description: "Remonté de très loin. Il luit faiblement dans le noir.",
        prix: 1_000_000.0,
        chances: [0.0, 20.0, 42.0, 26.0, 9.5, 2.5, 0.0, 0.0, 0.0],
        coquille: [[0x2c, 0x3c, 0x6e], [0x5c, 0xf0, 0xd8]],
    },
    Oeuf {
        nom: "Œuf cosmique",
        description: "Tombé du ciel une nuit d'étoiles, encore tiède.",
        prix: 500_000_000.0,
        chances: [0.0, 0.0, 22.0, 36.0, 26.0, 11.0, 4.5, 0.5, 0.0],
        coquille: [[0x30, 0x1c, 0x58], [0xff, 0xe8, 0x9c]],
    },
    Oeuf {
        nom: "Œuf d'ambre",
        description: "Pris dans l'ambre depuis des millions d'années. Quelque chose bouge dedans.",
        prix: 30_000_000_000.0,
        chances: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 12.0, 8.0, 80.0],
        coquille: [[0xe8, 0x9a, 0x3a], [0x7a, 0x46, 0x1c]],
    },
];

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
}

impl Poisson {
    pub fn revenu(&self) -> f64 {
        self.rarete.revenu_base() * self.taille as f64 * multiplicateur_niveau(self.niveau)
    }

    /// Prix pour passer au niveau suivant (`None` au niveau max).
    pub fn cout_nourrir(&self) -> Option<f64> {
        (self.niveau < NIVEAU_MAX).then(|| {
            self.rarete.revenu_base()
                * self.taille as f64
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
    /// Images par seconde du fond : 30 ou 60.
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
    /// Mode Abysses : aquarium sombre des grands fonds.
    pub abysses: bool,
}

impl Default for Reglages {
    fn default() -> Self {
        Reglages {
            teinte: 0.25,
            max_affiches: CAPACITE,
            ips: 30,
            demarrage_windows: false,
            pause_plein_ecran: true,
            mode_fenetre: false,
            incrustation: 0,
            equiper_auto: false,
            abysses: false,
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
    #[serde(default)]
    pub reglages: Reglages,
}

/// Version du format de sauvegarde (2 = espèces, 3 = rareté Préhistorique et œuf d'ambre).
pub const VERSION: u32 = 3;

#[derive(Debug, PartialEq)]
pub enum Refus {
    PasAssez,
    Introuvable,
    NiveauMax,
    AquariumPlein,
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
            reglages: Reglages::default(),
        };
        p.ajouter(Rarete::Commun, 1.0, 0);
        p
    }

    fn ajouter(&mut self, rarete: Rarete, taille: f32, oeuf: u8) -> u64 {
        let id = self.prochain_id;
        self.prochain_id += 1;
        let graine = self.alea.suivant();
        let actif = self.nb_actifs() < CAPACITE;
        // L'espèce vient de la graine : le tirage des raretés (et l'équilibrage) ne change pas.
        let (e, variante) = crate::especes::choisir(rarete, graine);
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
        });
        self.eclos[rarete.index()] += 1;
        *self.vus.entry(e.cle.to_string()).or_default() += 1;
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
        self.version = self.version.max(VERSION);
    }

    pub fn nb_actifs(&self) -> usize {
        self.poissons.iter().filter(|p| p.actif).count()
    }

    pub fn revenu_par_s(&self) -> f64 {
        self.poissons
            .iter()
            .filter(|p| p.actif)
            .map(Poisson::revenu)
            .sum()
    }

    pub fn crediter(&mut self, montant: f64) {
        self.argent = (self.argent + montant).min(ARGENT_MAX);
        self.total_gagne = (self.total_gagne + montant).min(ARGENT_MAX);
    }

    pub fn avancer(&mut self, dt: f64) {
        let r = self.revenu_par_s();
        self.crediter(r * dt);
    }

    /// Crédite les gains depuis `horodatage` (plafond 8 h) ; renvoie le montant.
    pub fn gains_hors_ligne(&mut self, maintenant: u64) -> f64 {
        let ecoule = maintenant.saturating_sub(self.horodatage) as f64;
        let gain = self.revenu_par_s() * ecoule.min(HORS_LIGNE_MAX_S);
        self.crediter(gain);
        self.horodatage = maintenant;
        gain
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
        let oeuf = OEUFS.get(i).ok_or(Refus::Introuvable)?;
        let prix = self.prix_oeuf(i);
        if self.argent < prix {
            return Err(Refus::PasAssez);
        }
        self.argent -= prix;
        self.oeufs_ouverts[i] += 1;
        let rarete = Self::tirer_rarete(&mut self.alea, oeuf);
        let taille = Self::tirer_taille(&mut self.alea);
        Ok(self.ajouter(rarete, taille, i as u8))
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

    pub fn relacher(&mut self, id: u64) -> Result<f64, Refus> {
        let i = self
            .poissons
            .iter()
            .position(|p| p.id == id)
            .ok_or(Refus::Introuvable)?;
        let prix = self.poissons.remove(i).prix_relache();
        self.argent = (self.argent + prix).min(ARGENT_MAX);
        Ok(prix)
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
    if v < 10.0 {
        let s = format!("{:.1}", (v * 10.0).floor() / 10.0);
        return s.trim_end_matches(".0").replace('.', ",");
    }
    if v < 1000.0 {
        return format!("{}", v.floor() as u64);
    }
    const SUFFIXES: [&str; 9] = ["k", "M", "Md", "Bn", "Bd", "Tn", "Td", "Qn", "Qd"];
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
        let s = s
            .trim_end_matches('0')
            .trim_end_matches('.')
            .replace('.', ",");
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
    format!("{} {}", s.replace('.', ","), SUFFIXES[palier - 1])
}

fn trunc(v: f64, decimales: usize) -> f64 {
    let f = 10f64.powi(decimales as i32);
    ((v * f) + 1e-9).floor() / f
}

/// Durée lisible : « 3 min 20 s », « 2 h 05 », « 1 j 4 h ».
pub fn duree(s: f64) -> String {
    let s = s.max(0.0) as u64;
    if s < 60 {
        format!("{s} s")
    } else if s < 3600 {
        format!("{} min {:02} s", s / 60, s % 60)
    } else if s < 86_400 {
        format!("{} h {:02}", s / 3600, s / 60 % 60)
    } else {
        format!("{} j {} h", s / 86_400, s / 3600 % 24)
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
        for (i, o) in OEUFS.iter().enumerate() {
            let gain: f64 = Rarete::TOUTES
                .iter()
                .map(|r| {
                    let nouveau = r.revenu_base() * 0.96;
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
            "fin : {}/s, {} œufs ouverts",
            court(p.revenu_par_s()),
            p.oeufs_ouverts.iter().sum::<u64>()
        ),
    });
    jalons
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_somment_a_100() {
        for o in &OEUFS {
            let s: f64 = o.chances.iter().sum();
            assert!((s - 100.0).abs() < 1e-9, "{} : {s}", o.nom);
        }
        // Prix croissants.
        assert!(OEUFS.windows(2).all(|w| w[0].prix < w[1].prix));
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
}

//! Déroulé d'une éclosion (v0.11), logique pure et testée : la lumière des fêlures monte par
//! PALIERS (blanc → bleu → violet → or → arc-en-ciel) selon le VRAI résultat, avec un temps
//! fort à chaque palier (pause, secousse, saut de couleur) ; shiny et modificateur ajoutent
//! chacun un temps fort ; puis la scène propre à l'œuf, l'éclatement et la révélation.

use crate::jeu::Rarete;
use crate::toile::{hsv, Rgb};

/// Nombre de paliers de lumière (blanc, bleu, violet, or, arc-en-ciel).
pub const NB_PALIERS: usize = 5;

/// Palier final atteint pour une rareté : le dernier palier = la rareté.
pub fn palier(r: Rarete) -> usize {
    match r {
        Rarete::Commun | Rarete::PeuCommun => 0,
        Rarete::Rare => 1,
        Rarete::Epique => 2,
        Rarete::Legendaire | Rarete::Mythique => 3,
        Rarete::Celeste | Rarete::Irreel | Rarete::Prehistorique => 4,
    }
}

/// Couleur de la lumière d'un palier (arc-en-ciel : la teinte tourne avec `t`).
pub fn couleur_palier(k: usize, t: f32) -> Rgb {
    match k {
        0 => [0xf0, 0xf4, 0xff],
        1 => [0x52, 0xa8, 0xff],
        2 => [0xbc, 0x6c, 0xff],
        3 => [0xff, 0xc8, 0x45],
        _ => hsv(t * 0.9, 0.8, 1.0),
    }
}

/// Lumière « arc-en-ciel » (palier 4) pour les fêlures et le liseré : chaque pixel prend sa
/// teinte (jamais une seule couleur froide qui se lirait comme une baisse).
pub const ARC_EN_CIEL: Rgb = [1, 2, 3];

/// Lumière des fêlures d'un palier : arc-en-ciel au palier 4.
pub fn lumiere_palier(k: usize, t: f32) -> Rgb {
    if k >= 4 {
        ARC_EN_CIEL
    } else {
        couleur_palier(k, t)
    }
}

/// Temps fort en plus des paliers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Extra {
    Shiny,
    Modif,
}

/// Instants (secondes depuis le début de l'éclosion).
#[derive(Clone, Debug, PartialEq)]
pub struct Deroule {
    /// Coups qui fêlent l'œuf (lumière blanche).
    pub coups: [f32; 3],
    /// Début de chaque saut de palier (1..=palier) : la pause commence ici, le saut a lieu
    /// `PAUSE` plus tard.
    pub paliers: Vec<f32>,
    pub extras: Vec<(f32, Extra)>,
    /// Scène propre à l'œuf : début et fin (= éclatement).
    pub scene: (f32, f32),
    /// Le palier final, la scène renforcée (haute rareté, shiny, modificateur).
    pub palier: usize,
    pub fort: bool,
}

/// Arrivée et premières fêlures.
pub const ENTREE: f32 = 1.9;
/// Durée d'un temps fort de palier, et la pause qui le précède.
pub const TEMPS_PALIER: f32 = 0.6;
pub const PAUSE: f32 = 0.2;
pub const TEMPS_EXTRA: f32 = 0.66;
/// Révélation : texte et boutons après ce délai.
pub const REVELATION: f32 = 0.9;

impl Deroule {
    pub fn nouveau(r: Rarete, shiny: bool, modif: bool, dore: bool) -> Deroule {
        let p = palier(r);
        let fort = p >= 3 || shiny || modif;
        // L'œuf doré se transforme pendant l'entrée : un peu plus long.
        let mut t = ENTREE + if dore { 0.4 } else { 0.0 };
        let coups = [0.85, 1.25, 1.62];
        let mut paliers = Vec::new();
        for _ in 1..=p {
            paliers.push(t);
            t += TEMPS_PALIER;
        }
        let mut extras = Vec::new();
        if shiny {
            extras.push((t, Extra::Shiny));
            t += TEMPS_EXTRA;
        }
        if modif {
            extras.push((t, Extra::Modif));
            t += TEMPS_EXTRA;
        }
        let scene = 1.5 + if fort { 0.5 } else { 0.0 };
        Deroule {
            coups,
            paliers,
            extras,
            scene: (t, t + scene),
            palier: p,
            fort,
        }
    }

    /// Instant de l'éclatement.
    pub fn eclat(&self) -> f32 {
        self.scene.1
    }

    /// Palier de lumière atteint à l'instant `t` (le saut a lieu après la pause).
    pub fn palier_a(&self, t: f32) -> usize {
        self.paliers.iter().filter(|&&d| t >= d + PAUSE).count()
    }

    /// Dans la pause d'un temps fort (palier ou extra) : l'œuf se fige, la lumière baisse.
    pub fn en_pause(&self, t: f32) -> bool {
        self.paliers
            .iter()
            .chain(self.extras.iter().map(|(d, _)| d))
            .any(|&d| (d..d + PAUSE).contains(&t))
    }

    /// Durée totale avant que le texte de la révélation s'affiche.
    pub fn total(&self) -> f32 {
        self.eclat() + REVELATION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paliers_selon_la_rarete() {
        // Le palier final reflète la vraie rareté, sans jamais redescendre.
        let attendu = [0, 0, 1, 2, 3, 3, 4, 4, 4];
        for (r, &p) in Rarete::TOUTES.iter().zip(&attendu) {
            assert_eq!(palier(*r), p, "{r:?}");
        }
        for w in Rarete::TOUTES.windows(2) {
            assert!(palier(w[0]) <= palier(w[1]));
        }
        for r in Rarete::TOUTES {
            let d = Deroule::nouveau(r, false, false, false);
            assert_eq!(d.paliers.len(), palier(r));
            assert_eq!(d.palier_a(d.eclat()), palier(r));
            assert_eq!(d.palier_a(0.0), 0);
            // La lumière ne fait que monter.
            let mut avant = 0;
            for k in 0..200 {
                let p = d.palier_a(k as f32 * d.eclat() / 200.0);
                assert!(p >= avant);
                avant = p;
            }
        }
    }

    #[test]
    fn temps_forts_et_durees() {
        // Commun : 5 s environ en tout ; tout en haut (shiny + modificateur) : 8 à 10 s.
        let c = Deroule::nouveau(Rarete::Commun, false, false, false);
        assert!((3.0..5.5).contains(&c.total()), "{}", c.total());
        assert!(!c.fort);
        let m = Deroule::nouveau(Rarete::Prehistorique, true, true, false);
        assert!(m.fort);
        assert_eq!(m.extras.len(), 2);
        assert!((7.5..10.5).contains(&m.total()), "{}", m.total());
        // Les temps forts s'enchaînent sans se chevaucher, et tous avant la scène de l'œuf.
        let mut debuts: Vec<f32> = m.paliers.clone();
        debuts.extend(m.extras.iter().map(|e| e.0));
        for w in debuts.windows(2) {
            assert!(w[1] - w[0] >= TEMPS_EXTRA.min(TEMPS_PALIER) - 1e-4);
        }
        assert!(debuts.iter().all(|&d| d >= ENTREE && d < m.scene.0));
        assert!(m.en_pause(m.paliers[0] + 0.05) && !m.en_pause(m.paliers[0] + PAUSE + 0.05));
        // Plus rare = plus long (la montée fait partie de la récompense).
        let mut avant = 0.0;
        for r in Rarete::TOUTES {
            let d = Deroule::nouveau(r, false, false, false).total();
            assert!(d >= avant);
            avant = d;
        }
    }
}

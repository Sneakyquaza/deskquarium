//! Attraper un poisson au bureau (v0.9) : règle commune au crochet souris de Windows
//! (`windows.rs`, WH_MOUSE_LL) et à la fenêtre de test. Logique pure, sans Windows, testée.
//!
//! - Appui gauche sur un poisson (bureau vide, réglage actif, Alt si demandé) : l'appui est
//!   AVALÉ (le bureau ne commence pas de rectangle de sélection) et le jeu est prévenu.
//! - Relâché avant `DELAI_MS` sans bouger : clic bref → fiche du poisson, et le clic est
//!   REJOUÉ au bureau (appui + relâché injectés) : le bureau le reçoit quand même.
//! - Bougé de plus de `TOLERANCE` px avant `DELAI_MS` : ce n'était pas pour nous → l'appui est
//!   rejoué au point de départ (le rectangle de sélection part de là) et tout passe.
//! - Tenu `DELAI_MS` : le poisson est attrapé ; les mouvements passent (le curseur bouge, le
//!   bureau n'a pas reçu d'appui donc pas de rectangle) ; le relâché est avalé → lâché.

/// Appui maintenu avant d'attraper (ms), comme `scene::DELAI_PRISE`.
pub const DELAI_MS: u32 = 200;
/// Mouvement toléré pendant l'appui avant de rendre la main au bureau (pixels physiques).
pub const TOLERANCE: i32 = 5;

/// Ce que le crochet doit faire de l'évènement.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Decision {
    Laisser,
    Avaler,
}

/// À injecter au bureau après coup (hors du crochet), et à dire au jeu.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Suite {
    Rien,
    /// Le jeu : appui sur ce poisson (il sera attrapé après le délai).
    Appui(u64),
    /// Le jeu : abandon ; le bureau : rejouer l'appui en (x, y) puis revenir au curseur.
    RendreAppui(i32, i32),
    /// Le jeu : fiche du poisson ; le bureau : rejouer un clic complet.
    Clic(u64),
    /// Le jeu : lâcher le poisson tenu.
    Lache,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Etat {
    #[default]
    Libre,
    /// Appui avalé sur un poisson : instant (ms), point, poisson.
    Appui(u32, i32, i32, u64),
    /// Poisson tenu (le relâché sera avalé).
    Tenu,
}

#[derive(Debug, Default)]
pub struct Pince {
    etat: Etat,
}

impl Pince {
    /// Appui gauche en (x, y) à `t` ms ; `poisson` = poisson attrapable sous le curseur (bureau
    /// vide, réglage actif, Alt tenu si demandé), sinon None.
    pub fn appui(&mut self, t: u32, x: i32, y: i32, poisson: Option<u64>) -> (Decision, Suite) {
        // Un état resté coincé (relâché perdu) ne survit pas à un nouvel appui.
        let avant = std::mem::take(&mut self.etat);
        let suite_avant = if avant == Etat::Tenu {
            Suite::Lache
        } else {
            Suite::Rien
        };
        match poisson {
            Some(id) => {
                self.etat = Etat::Appui(t, x, y, id);
                (Decision::Avaler, Suite::Appui(id))
            }
            None => (Decision::Laisser, suite_avant),
        }
    }

    /// Mouvement en (x, y) à `t` ms.
    pub fn mouvement(&mut self, t: u32, x: i32, y: i32) -> (Decision, Suite) {
        if let Etat::Appui(t0, x0, y0, _) = self.etat {
            let loin = (x - x0).abs() > TOLERANCE || (y - y0).abs() > TOLERANCE;
            if t.wrapping_sub(t0) >= DELAI_MS {
                self.etat = Etat::Tenu;
            } else if loin {
                self.etat = Etat::Libre;
                return (Decision::Laisser, Suite::RendreAppui(x0, y0));
            }
        }
        // Les mouvements passent toujours (le curseur doit bouger).
        (Decision::Laisser, Suite::Rien)
    }

    /// Relâché gauche à `t` ms.
    pub fn relache(&mut self, t: u32) -> (Decision, Suite) {
        match std::mem::take(&mut self.etat) {
            Etat::Libre => (Decision::Laisser, Suite::Rien),
            Etat::Appui(t0, _, _, id) if t.wrapping_sub(t0) < DELAI_MS => {
                (Decision::Avaler, Suite::Clic(id))
            }
            Etat::Appui(..) | Etat::Tenu => (Decision::Avaler, Suite::Lache),
        }
    }

    /// Un poisson est-il en main (pour le jeu) ?
    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn occupe(&self) -> bool {
        self.etat != Etat::Libre
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hors_poisson_tout_passe() {
        let mut p = Pince::default();
        assert_eq!(p.appui(0, 10, 10, None), (Decision::Laisser, Suite::Rien));
        assert_eq!(p.mouvement(50, 40, 40), (Decision::Laisser, Suite::Rien));
        assert_eq!(p.relache(80), (Decision::Laisser, Suite::Rien));
        assert!(!p.occupe());
    }

    #[test]
    fn clic_bref_donne_la_fiche_et_rend_le_clic() {
        let mut p = Pince::default();
        assert_eq!(
            p.appui(1000, 10, 10, Some(7)),
            (Decision::Avaler, Suite::Appui(7))
        );
        assert_eq!(p.mouvement(1050, 12, 11).0, Decision::Laisser);
        assert_eq!(p.relache(1120), (Decision::Avaler, Suite::Clic(7)));
        assert!(!p.occupe());
    }

    #[test]
    fn glisser_tout_de_suite_rend_la_selection_au_bureau() {
        let mut p = Pince::default();
        p.appui(0, 100, 100, Some(3));
        assert_eq!(
            p.mouvement(60, 120, 100),
            (Decision::Laisser, Suite::RendreAppui(100, 100))
        );
        // Le relâché est celui du bureau (rejoué), il passe.
        assert_eq!(p.relache(300), (Decision::Laisser, Suite::Rien));
    }

    #[test]
    fn appui_tenu_attrape_puis_lache() {
        let mut p = Pince::default();
        p.appui(0, 100, 100, Some(3));
        assert_eq!(p.mouvement(150, 101, 100).1, Suite::Rien);
        // Après le délai, on peut bouger loin : le poisson suit, rien ne passe au bureau.
        assert_eq!(p.mouvement(260, 300, 200), (Decision::Laisser, Suite::Rien));
        assert!(p.occupe());
        assert_eq!(p.relache(900), (Decision::Avaler, Suite::Lache));
        // Tenu sans bouger puis relâché après le délai : lâché aussi (pas de clic).
        p.appui(2000, 5, 5, Some(4));
        assert_eq!(p.relache(2300), (Decision::Avaler, Suite::Lache));
    }

    #[test]
    fn relache_perdu_ne_coince_rien() {
        let mut p = Pince::default();
        p.appui(0, 1, 1, Some(1));
        p.mouvement(500, 50, 50);
        // Nouvel appui ailleurs (relâché jamais vu) : le poisson est lâché, l'appui passe.
        assert_eq!(p.appui(5000, 9, 9, None), (Decision::Laisser, Suite::Lache));
        assert!(!p.occupe());
        // Horloge qui reboucle (GetTickCount) : pas de faux délai.
        p.appui(u32::MAX - 50, 0, 0, Some(2));
        assert_eq!(p.relache(40).1, Suite::Clic(2));
    }
}

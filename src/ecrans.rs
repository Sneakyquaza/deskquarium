//! Disposition des écrans (multi-écrans) : rectangles des moniteurs en pixels physiques dans
//! le bureau virtuel (lus sous Windows par `windows::disposition`, simulés sous Linux par
//! `--ecrans`). L'aquarium couvre la boîte englobante ; on ne nage que sur les écrans (les
//! trous entre écrans de tailles différentes ne sont pas de l'eau).

/// Un moniteur : rectangle en pixels physiques (bureau virtuel, origine quelconque).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ecran {
    pub x: i32,
    pub y: i32,
    pub l: i32,
    pub h: i32,
    pub principal: bool,
    /// Zone de travail (sans la barre des tâches) : x, y, l, h ; None = tout l'écran.
    pub travail: Option<(i32, i32, i32, i32)>,
}

impl Ecran {
    #[cfg(test)]
    fn contient(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.l && y >= self.y && y < self.y + self.h
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Disposition {
    pub ecrans: Vec<Ecran>,
}

impl Disposition {
    /// Un seul écran de `l`x`h` (fenêtre de test, capture).
    pub fn simple(l: u32, h: u32) -> Disposition {
        Disposition {
            ecrans: vec![Ecran {
                x: 0,
                y: 0,
                l: l as i32,
                h: h as i32,
                principal: true,
                travail: None,
            }],
        }
    }

    /// « 1920x1080+0+0,1920x1080+1920+0 » (le premier est le principal) : disposition
    /// simulée pour les tests et captures sous Linux ; « /48 » à la fin d'un écran = barre des
    /// tâches de 48 px en bas.
    pub fn depuis_texte(t: &str) -> Result<Disposition, String> {
        let mut ecrans = Vec::new();
        for (i, morceau) in t.split(',').filter(|m| !m.trim().is_empty()).enumerate() {
            let err = || format!("--ecrans : « {morceau} » (attendu LxH+X+Y[/barre])");
            let (morceau_, barre) = match morceau.trim().split_once('/') {
                Some((m, b)) => (m, Some(b.trim().parse::<i32>().map_err(|_| err())?)),
                None => (morceau.trim(), None),
            };
            let (taille, pos) = morceau_.split_once('+').ok_or_else(err)?;
            let (l, h) = taille.split_once('x').ok_or_else(err)?;
            // Positions négatives : « +-1920+0 ».
            let (x, y) = pos.split_once('+').ok_or_else(err)?;
            let n = |s: &str| s.trim().parse::<i32>().map_err(|_| err());
            let e = Ecran {
                x: n(x)?,
                y: n(y)?,
                l: n(l)?,
                h: n(h)?,
                principal: i == 0,
                travail: None,
            };
            if e.l < 64 || e.h < 64 {
                return Err(err());
            }
            let travail = barre.map(|b| (e.x, e.y, e.l, e.h - b));
            ecrans.push(Ecran { travail, ..e });
        }
        Disposition::nouvelle(ecrans).ok_or_else(|| "--ecrans : aucun écran".to_string())
    }

    /// Disposition valide : au moins un écran, exactement un principal (le premier sinon).
    pub fn nouvelle(mut ecrans: Vec<Ecran>) -> Option<Disposition> {
        ecrans.retain(|e| e.l > 0 && e.h > 0);
        if ecrans.is_empty() {
            return None;
        }
        let p = ecrans.iter().position(|e| e.principal).unwrap_or(0);
        for (i, e) in ecrans.iter_mut().enumerate() {
            e.principal = i == p;
        }
        Some(Disposition { ecrans })
    }

    /// Boîte englobante (x, y, l, h) : la fenêtre de l'aquarium.
    pub fn boite(&self) -> (i32, i32, i32, i32) {
        let x0 = self.ecrans.iter().map(|e| e.x).min().unwrap_or(0);
        let y0 = self.ecrans.iter().map(|e| e.y).min().unwrap_or(0);
        let x1 = self.ecrans.iter().map(|e| e.x + e.l).max().unwrap_or(0);
        let y1 = self.ecrans.iter().map(|e| e.y + e.h).max().unwrap_or(0);
        (x0, y0, x1 - x0, y1 - y0)
    }

    pub fn principal(&self) -> &Ecran {
        self.ecrans
            .iter()
            .find(|e| e.principal)
            .unwrap_or(&self.ecrans[0])
    }

    /// Un point du bureau virtuel est-il sur un écran (pas dans un trou) ? (Tests : l'aquarium
    /// lit les trous dans ses zones, voir `scene::Bassin`.)
    #[cfg(test)]
    pub fn sur_un_ecran(&self, x: i32, y: i32) -> bool {
        self.ecrans.iter().any(|e| e.contient(x, y))
    }

    /// Paires d'écrans qui se touchent par un bord (un poisson passe de l'un à l'autre).
    #[cfg(test)]
    pub fn voisins(&self) -> Vec<(usize, usize)> {
        let mut v = Vec::new();
        for (i, a) in self.ecrans.iter().enumerate() {
            for (j, b) in self.ecrans.iter().enumerate().skip(i + 1) {
                let commun = |a0: i32, a1: i32, b0: i32, b1: i32| a1.min(b1) - a0.max(b0) > 0;
                let cote = (a.x + a.l == b.x || b.x + b.l == a.x)
                    && commun(a.y, a.y + a.h, b.y, b.y + b.h);
                let dessus = (a.y + a.h == b.y || b.y + b.h == a.y)
                    && commun(a.x, a.x + a.l, b.x, b.x + b.l);
                if cote || dessus {
                    v.push((i, j));
                }
            }
        }
        v
    }

    /// Rectangles des écrans en pixels monde de la toile (échelle `e`, origine = boîte) :
    /// (x0, y0, x1, y1), bord exclu. Le principal en premier.
    pub fn zones(&self, e: u32) -> Vec<(i32, i32, i32, i32)> {
        let (bx, by, _, _) = self.boite();
        let e = e as i32;
        let mut z: Vec<_> = self
            .ecrans
            .iter()
            .map(|c| {
                (
                    (c.x - bx) / e,
                    (c.y - by) / e,
                    (c.x - bx + c.l + e - 1) / e,
                    (c.y - by + c.h + e - 1) / e,
                )
            })
            .collect();
        let p = self.ecrans.iter().position(|c| c.principal).unwrap_or(0);
        z.swap(0, p);
        z
    }

    /// Zones de travail (sans barre des tâches) en pixels monde, même ordre que `zones`.
    pub fn travail(&self, e: u32) -> Vec<(i32, i32, i32, i32)> {
        let (bx, by, _, _) = self.boite();
        let e = e as i32;
        let mut z: Vec<_> = self
            .ecrans
            .iter()
            .map(|c| {
                let (x, y, l, h) = c.travail.unwrap_or((c.x, c.y, c.l, c.h));
                (
                    (x - bx) / e,
                    (y - by) / e,
                    (x - bx + l) / e,
                    (y - by + h) / e,
                )
            })
            .collect();
        let p = self.ecrans.iter().position(|c| c.principal).unwrap_or(0);
        z.swap(0, p);
        z
    }

    /// Échelle entière toile → écran : celle de l'écran principal (pas de la boîte entière).
    pub fn echelle(&self) -> u32 {
        crate::scene::echelle_pour(self.principal().l as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deux_ecrans_cote_a_cote() {
        let d = Disposition::depuis_texte("1920x1080+0+0,1920x1080+1920+0").unwrap();
        assert_eq!(d.boite(), (0, 0, 3840, 1080));
        assert_eq!(d.voisins(), vec![(0, 1)]);
        assert!(d.sur_un_ecran(1919, 500) && d.sur_un_ecran(1920, 500));
        assert!(!d.sur_un_ecran(3840, 500));
        assert_eq!(d.echelle(), 4);
        assert_eq!(d.zones(4), vec![(0, 0, 480, 270), (480, 0, 960, 270)]);
        assert_eq!(d.principal().x, 0);
    }

    #[test]
    fn tailles_differentes_et_trous() {
        // Principal 1920x1080 à droite d'un 2560x1440 décalé vers le haut (coordonnées
        // négatives, comme Windows quand le principal n'est pas le plus à gauche).
        let d = Disposition::depuis_texte("1920x1080+0+0,2560x1440+-2560+-200").unwrap();
        assert_eq!(d.boite(), (-2560, -200, 4480, 1440));
        assert_eq!(d.voisins(), vec![(0, 1)]);
        // Trous : au-dessus et au-dessous du principal, plus court que son voisin.
        assert!(!d.sur_un_ecran(100, -100));
        assert!(!d.sur_un_ecran(100, 1200));
        assert!(d.sur_un_ecran(-100, 1200));
        // Le principal vient en premier, à droite dans la toile : son coin bas gauche porte
        // le compteur.
        let z = d.zones(4);
        assert_eq!(z[0], (640, 50, 1120, 320));
        assert_eq!(z[1], (0, 0, 640, 360));
        assert!(!d.ecrans[1].principal && d.principal().l == 1920);
    }

    #[test]
    fn barre_des_taches() {
        let d = Disposition::depuis_texte("1920x1080+0+0/48,1920x1080+1920+0").unwrap();
        assert_eq!(d.travail(4)[0], (0, 0, 480, 258));
        assert_eq!(d.travail(4)[1], (480, 0, 960, 270));
        assert!(Disposition::depuis_texte("1920x1080+0+0/x").is_err());
    }

    #[test]
    fn ecrans_empiles_et_sans_contact() {
        let d = Disposition::depuis_texte("1920x1080+0+0,1920x1080+0+1080").unwrap();
        assert_eq!(d.voisins(), vec![(0, 1)]);
        // Coins qui se touchent seulement : pas voisins (aucun bord commun).
        let d = Disposition::depuis_texte("1920x1080+0+0,1920x1080+1920+1080").unwrap();
        assert!(d.voisins().is_empty());
        assert!(Disposition::depuis_texte("").is_err());
        assert!(Disposition::depuis_texte("1920x1080").is_err());
        assert!(Disposition::nouvelle(vec![]).is_none());
    }
}

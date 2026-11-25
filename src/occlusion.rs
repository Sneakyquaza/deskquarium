//! v0.12 : un écran est-il entièrement recouvert par des fenêtres ? Logique pure (testée sous
//! Linux) ; la liste des fenêtres visibles vient de `windows.rs`.

/// Boîte (gauche, haut, droite, bas), droite et bas exclus, coordonnées du bureau virtuel.
pub type Boite = (i32, i32, i32, i32);

/// L'union des `fenetres` couvre-t-elle tout `ecran` ? Bandes verticales entre les bords des
/// fenêtres ; dans chaque bande, les intervalles en hauteur des fenêtres qui la traversent
/// doivent couvrir l'écran de haut en bas.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn recouvert(ecran: Boite, fenetres: &[Boite]) -> bool {
    let (ex0, ey0, ex1, ey1) = ecran;
    if ex1 <= ex0 || ey1 <= ey0 {
        return false;
    }
    // Fenêtres coupées à l'écran (celles qui ne le touchent pas sont oubliées).
    let f: Vec<Boite> = fenetres
        .iter()
        .map(|&(x0, y0, x1, y1)| (x0.max(ex0), y0.max(ey0), x1.min(ex1), y1.min(ey1)))
        .filter(|&(x0, y0, x1, y1)| x1 > x0 && y1 > y0)
        .collect();
    if f.is_empty() {
        return false;
    }
    if f.contains(&ecran) {
        return true;
    }
    let mut xs: Vec<i32> = f
        .iter()
        .flat_map(|b| [b.0, b.2])
        .chain([ex0, ex1])
        .collect();
    xs.sort_unstable();
    xs.dedup();
    let mut intervalles: Vec<(i32, i32)> = Vec::new();
    for bande in xs.windows(2) {
        let (a, b) = (bande[0], bande[1]);
        intervalles.clear();
        intervalles.extend(
            f.iter()
                .filter(|w| w.0 <= a && w.2 >= b)
                .map(|w| (w.1, w.3)),
        );
        intervalles.sort_unstable();
        let mut haut = ey0;
        for &(y0, y1) in &intervalles {
            if y0 > haut {
                break;
            }
            haut = haut.max(y1);
        }
        if haut < ey1 {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeu::Alea;

    const E: Boite = (0, 0, 1920, 1080);

    #[test]
    fn cas_simples() {
        assert!(!recouvert(E, &[]));
        // Fenêtre agrandie (bords invisibles au-delà de l'écran) : recouvert.
        assert!(recouvert(E, &[(-8, -8, 1928, 1088)]));
        // Zone de travail + barre des tâches.
        assert!(recouvert(E, &[(0, 0, 1920, 1032), (0, 1032, 1920, 1080)]));
        // Deux moitiés côte à côte (ancrage).
        assert!(recouvert(E, &[(0, 0, 960, 1080), (960, 0, 1920, 1080)]));
        // Un pixel de bureau entre les deux : visible.
        assert!(!recouvert(E, &[(0, 0, 960, 1080), (961, 0, 1920, 1080)]));
        // Une ligne en bas : visible.
        assert!(!recouvert(E, &[(0, 0, 1920, 1079)]));
        // Fenêtre de l'autre écran seulement : visible.
        assert!(!recouvert(E, &[(1920, 0, 3840, 1080)]));
        // Écran de droite recouvert par une fenêtre qui déborde de l'écran de gauche.
        assert!(recouvert((1920, 0, 3840, 1080), &[(1000, -10, 3900, 1100)]));
        // Quatre quarts qui se chevauchent.
        assert!(recouvert(
            E,
            &[
                (0, 0, 1000, 600),
                (900, 0, 1920, 600),
                (0, 500, 1000, 1080),
                (900, 500, 1920, 1080)
            ]
        ));
    }

    /// Comparé pixel par pixel sur un petit écran, fenêtres au hasard.
    #[test]
    fn identique_au_calcul_pixel_par_pixel() {
        let mut a = Alea(3);
        let ecran = (2, 1, 30, 20);
        for _ in 0..3000 {
            let n = (a.f() * 6.0) as usize;
            let mut f: Vec<Boite> = (0..n)
                .map(|_| {
                    let x0 = (a.f() * 34.0) as i32 - 2;
                    let y0 = (a.f() * 24.0) as i32 - 2;
                    (
                        x0,
                        y0,
                        x0 + 1 + (a.f() * 30.0) as i32,
                        y0 + 1 + (a.f() * 22.0) as i32,
                    )
                })
                .collect();
            if a.f() < 0.5 {
                // Écran découpé en colonnes (souvent recouvert), une parfois raccourcie.
                let mut x = ecran.0;
                while x < ecran.2 {
                    let x1 = (x + 1 + (a.f() * 10.0) as i32).min(ecran.2);
                    let bas = if a.f() < 0.1 { ecran.3 - 1 } else { ecran.3 };
                    f.push((x, ecran.1, x1, bas));
                    x = x1;
                }
            }
            let brut = (ecran.1..ecran.3).all(|y| {
                (ecran.0..ecran.2)
                    .all(|x| f.iter().any(|w| x >= w.0 && x < w.2 && y >= w.1 && y < w.3))
            });
            assert_eq!(recouvert(ecran, &f), brut, "{f:?}");
        }
    }
}

//! Sauvegarde JSON : `%APPDATA%\aquarium\partie.json` (Linux : `~/.local/share/aquarium`).
//! Écriture atomique (fichier temporaire puis renommage) ; la version précédente reste en `.bak`.
//! Fichier abîmé : on reprend le `.bak` ; les deux abîmés : le fichier est mis de côté
//! (`partie.abime-<horodatage>.json`), jamais écrasé, et une partie neuve commence.

use crate::jeu::Partie;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn maintenant() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Dossier de sauvegarde (`AQUARIUM_DOSSIER` le remplace, pour les tests et captures).
pub fn dossier() -> PathBuf {
    if let Some(d) = std::env::var_os("AQUARIUM_DOSSIER") {
        return PathBuf::from(d);
    }
    if cfg!(windows) {
        if let Some(a) = std::env::var_os("APPDATA") {
            return PathBuf::from(a).join("aquarium");
        }
    }
    if let Some(x) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(x).join("aquarium");
    }
    let maison = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .unwrap_or_else(|| ".".into());
    PathBuf::from(maison).join(".local/share/aquarium")
}

pub fn fichier(d: &Path) -> PathBuf {
    d.join("partie.json")
}

/// Charge la partie ; renvoie aussi un message à montrer si quelque chose a été réparé.
pub fn charger(d: &Path) -> (Partie, Option<String>) {
    let f = fichier(d);
    let lire = |p: &Path| -> Result<Partie, String> {
        let octets = std::fs::read(p).map_err(|e| e.to_string())?;
        let mut p = serde_json::from_slice::<Partie>(&octets).map_err(|e| e.to_string())?;
        p.migrer();
        Ok(p)
    };
    match lire(&f) {
        Ok(p) => (p, None),
        Err(_) if !f.exists() => {
            // Première fois (ou fichier principal disparu) : le `.bak` s'il existe.
            match lire(&f.with_extension("bak")) {
                Ok(p) => (
                    p,
                    Some("Sauvegarde reprise depuis la copie de secours.".into()),
                ),
                Err(_) => (Partie::nouvelle(maintenant(), maintenant() ^ 0x5eed), None),
            }
        }
        Err(e) => match lire(&f.with_extension("bak")) {
            Ok(p) => (
                p,
                Some(format!(
                    "Sauvegarde abîmée ({e}) : copie de secours reprise."
                )),
            ),
            Err(_) => {
                let cote = d.join(format!("partie.abime-{}.json", maintenant()));
                let _ = std::fs::copy(&f, &cote);
                (
                    Partie::nouvelle(maintenant(), maintenant() ^ 0x5eed),
                    Some(format!(
                        "Sauvegarde illisible, mise de côté dans {} ; nouvelle partie.",
                        cote.display()
                    )),
                )
            }
        },
    }
}

pub fn ecrire(d: &Path, p: &Partie) -> io::Result<()> {
    std::fs::create_dir_all(d)?;
    let json = serde_json::to_vec_pretty(p).map_err(io::Error::other)?;
    ecrire_atomique(&fichier(d), &json)
}

/// Fichier temporaire synchronisé, copie de l'ancien en `.bak`, puis renommage.
pub fn ecrire_atomique(chemin: &Path, donnees: &[u8]) -> io::Result<()> {
    let tmp = chemin.with_extension("tmp");
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(donnees)?;
        f.sync_all()?;
    }
    // Le `.bak` ne reçoit qu'une version lisible : un fichier abîmé ne remplace pas la bonne copie.
    if std::fs::read(chemin)
        .ok()
        .is_some_and(|o| serde_json::from_slice::<Partie>(&o).is_ok())
    {
        let _ = std::fs::copy(chemin, chemin.with_extension("bak"));
    }
    // Windows : renommer sur un fichier ouvert ailleurs (antivirus) peut échouer un instant.
    let mut r = std::fs::rename(&tmp, chemin);
    for _ in 0..5 {
        if r.is_ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
        r = std::fs::rename(&tmp, chemin);
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dossier_test(nom: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("aquarium-test-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn aller_retour_et_bak() {
        let d = dossier_test("ar");
        let mut p = Partie::nouvelle(100, 9);
        p.argent = 1234.5;
        ecrire(&d, &p).unwrap();
        p.argent = 99.0;
        ecrire(&d, &p).unwrap();
        let (q, msg) = charger(&d);
        assert!(msg.is_none());
        assert_eq!(q.argent, 99.0);
        assert_eq!(q.poissons.len(), 1);
        assert_eq!(q.poissons[0].graine, p.poissons[0].graine);
        let bak: Partie =
            serde_json::from_slice(&std::fs::read(fichier(&d).with_extension("bak")).unwrap())
                .unwrap();
        assert_eq!(bak.argent, 1234.5);
    }

    #[test]
    fn sauvegarde_corrompue_reprend_le_bak() {
        let d = dossier_test("bak");
        let mut p = Partie::nouvelle(100, 9);
        p.argent = 50.0;
        ecrire(&d, &p).unwrap();
        p.argent = 60.0;
        ecrire(&d, &p).unwrap();
        std::fs::write(fichier(&d), b"{\"version\": 1, \"argent\": tronqu").unwrap();
        let (q, msg) = charger(&d);
        assert_eq!(q.argent, 50.0);
        assert!(msg.unwrap().contains("secours"));
        // L'écriture suivante ne recopie pas le fichier abîmé dans le `.bak`.
        ecrire(&d, &q).unwrap();
        std::fs::write(fichier(&d), b"\0\0").unwrap();
        assert_eq!(charger(&d).0.argent, 50.0);
    }

    #[test]
    fn tout_abime_met_de_cote() {
        let d = dossier_test("abime");
        std::fs::write(fichier(&d), b"pas du json").unwrap();
        let (q, msg) = charger(&d);
        assert_eq!(q.poissons.len(), 1);
        assert!(msg.unwrap().contains("mise de côté"));
        let cote = std::fs::read_dir(&d)
            .unwrap()
            .filter_map(|e| e.ok())
            .any(|e| e.file_name().to_string_lossy().starts_with("partie.abime-"));
        assert!(cote);
        // Le fichier abîmé d'origine est toujours là (pas de perte).
        assert_eq!(std::fs::read(fichier(&d)).unwrap(), b"pas du json");
    }

    #[test]
    fn sauvegarde_lot1_migree_sans_perte() {
        // Format exact du lot 1 (version 1, sans espèce ni `vus`).
        let d = dossier_test("lot1");
        let mut json = String::from(
            r#"{"version":1,"argent":123456.5,"total_gagne":9e9,"horodatage":1700000000,
            "prochain_id":40,"alea":77,"poissons":["#,
        );
        let n = 24;
        for i in 0..n {
            let r = crate::jeu::Rarete::TOUTES[i % 8];
            json += &format!(
                r#"{}{{"id":{},"graine":{},"rarete":"{r:?}","taille":1.1,"niveau":3,"actif":{},"oeuf":2}}"#,
                if i == 0 { "" } else { "," },
                i + 1,
                1000 + i * 7919,
                i % 3 != 0
            );
        }
        json += r#"],"oeufs_ouverts":[10,5,2,1],"eclos":[3,3,3,3,3,3,3,3],
            "reglages":{"teinte":0.3,"max_affiches":40,"ips":30,"demarrage_windows":false,"pause_plein_ecran":true}}"#;
        std::fs::write(fichier(&d), &json).unwrap();
        let (q, msg) = charger(&d);
        assert!(msg.is_none(), "{msg:?}");
        assert_eq!(q.version, crate::jeu::VERSION);
        assert_eq!(q.argent, 123456.5);
        assert_eq!(q.poissons.len(), n);
        for (i, f) in q.poissons.iter().enumerate() {
            let e = crate::especes::par_cle(&f.espece).expect("espèce connue");
            assert_eq!(e.rarete, f.rarete, "même rareté");
            assert!((f.variante as usize) < e.variantes.len());
            assert_eq!(f.niveau, 3);
            assert_eq!(f.actif, i % 3 != 0);
        }
        assert_eq!(q.vus.values().sum::<u64>(), n as u64);
        // Version 3 : l'œuf d'ambre et la rareté Préhistorique s'ajoutent à zéro.
        assert_eq!(q.oeufs_ouverts, vec![10, 5, 2, 1, 0]);
        assert_eq!(q.eclos, vec![3, 3, 3, 3, 3, 3, 3, 3, 0]);
        assert_eq!(q.prix_oeuf(4), crate::jeu::OEUFS[4].prix);
        // Déterministe et stable : relire puis réécrire ne change rien.
        ecrire(&d, &q).unwrap();
        let (r, _) = charger(&d);
        for (a, b) in q.poissons.iter().zip(&r.poissons) {
            assert_eq!((&a.espece, a.variante), (&b.espece, b.variante));
        }
        assert_eq!(r.vus, q.vus);
    }
}

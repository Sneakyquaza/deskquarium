//! Mise à jour depuis les GitHub Releases du dépôt public `Sneakyquaza/aquarium` : lecture de
//! la dernière version, téléchargement vérifié (sha256) à côté de l'exécutable, échange des
//! fichiers puis relance après la fermeture propre (fond d'écran remis). Tout le réseau tourne
//! sur d'autres fils ; jamais d'installation sans un clic. Modèle : `gbemu/gb-app/src/maj.rs`.

use serde::Deserialize;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const URL_RELEASE: &str = "https://api.github.com/repos/Sneakyquaza/aquarium/releases/latest";
pub const PAGE_RELEASES: &str = "https://github.com/Sneakyquaza/aquarium/releases";
const AGENT: &str = "aquarium-maj";
const NOM_EXE: &str = "aquarium.exe";
/// Exécutable plus gros : refusé (release fausse).
const TAILLE_MAX: u64 = 256 << 20;
/// Hôtes autorisés (https seulement) : l'API, la page des releases et ses fichiers.
const HOTES: [&str; 4] = [
    "api.github.com",
    "github.com",
    "objects.githubusercontent.com",
    "release-assets.githubusercontent.com",
];

/// Dernière version publiée.
#[derive(Clone, Debug, PartialEq)]
pub struct Annonce {
    pub version: String,
    pub url: String,
    pub url_sha256: String,
    pub taille: u64,
    pub notes: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Etat {
    Rien,
    Recherche,
    AJour,
    /// Recherche échouée (message lisible).
    Erreur(String),
    Dispo(Annonce),
    /// Version, octets reçus, total.
    Telechargement(String, u64, u64),
    /// Nouvel exécutable vérifié, prêt à remplacer l'actuel.
    Pret(PathBuf),
    /// Fichiers échangés : relance à la fermeture.
    Installation,
    /// Téléchargement ou installation échoués : rien n'a été changé.
    Echec(String),
}

pub struct Maj {
    etat: Arc<Mutex<Etat>>,
    /// Exécutable lu au lancement (après un renommage, `current_exe` suivrait le `.ancien`).
    exe: Option<PathBuf>,
    reveil: Arc<dyn Fn() + Send + Sync>,
}

impl Maj {
    pub fn new(reveil: impl Fn() + Send + Sync + 'static) -> Maj {
        Maj {
            etat: Arc::new(Mutex::new(Etat::Rien)),
            exe: std::env::current_exe().ok(),
            reveil: Arc::new(reveil),
        }
    }

    pub fn etat(&self) -> Etat {
        self.verrou().clone()
    }

    fn verrou(&self) -> std::sync::MutexGuard<'_, Etat> {
        self.etat.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn mettre(&self, e: Etat) {
        *self.verrou() = e;
    }

    /// Version disponible (badge de l'Écloserie, infobulle de l'icône).
    pub fn disponible(&self) -> Option<String> {
        match self.etat() {
            Etat::Dispo(a) => Some(a.version),
            _ => None,
        }
    }

    /// Lit la dernière release sur un autre fil. `silencieux` (lancement, toutes les 6 h) :
    /// un échec ne laisse aucune trace.
    pub fn chercher(&self, silencieux: bool) {
        {
            let mut e = self.verrou();
            if matches!(
                *e,
                Etat::Recherche
                    | Etat::Telechargement(..)
                    | Etat::Pret(_)
                    | Etat::Installation
                    | Etat::Dispo(_)
            ) && (silencieux || !matches!(*e, Etat::Dispo(_)))
            {
                return;
            }
            *e = Etat::Recherche;
        }
        let (etat, r) = (Arc::clone(&self.etat), Arc::clone(&self.reveil));
        std::thread::spawn(move || {
            let res = match lire_release() {
                Ok(a) if plus_recente(&a.version, VERSION) => Etat::Dispo(a),
                Ok(_) => Etat::AJour,
                Err(_) if silencieux => Etat::Rien,
                Err(m) => Etat::Erreur(m),
            };
            *etat.lock().unwrap_or_else(|e| e.into_inner()) = res;
            r();
        });
    }

    /// Télécharge la version annoncée dans `<exe>.nouveau` (vérifiée), puis état `Pret`.
    pub fn telecharger(&self) {
        let Etat::Dispo(a) = self.etat() else { return };
        let Some(exe) = self.exe.clone() else {
            return self.mettre(Etat::Echec("Emplacement de l'appli introuvable".into()));
        };
        self.mettre(Etat::Telechargement(a.version.clone(), 0, a.taille));
        let (etat, r) = (Arc::clone(&self.etat), Arc::clone(&self.reveil));
        std::thread::spawn(move || {
            let nouveau = suffixe(&exe, ".nouveau");
            let mut dernier = 0;
            let res = lire_empreinte(&agent(10), &a.url_sha256).and_then(|sha| {
                telecharger_vers(&agent(20), &a, &sha, &nouveau, |faits| {
                    // Interface réveillée à chaque pour cent reçu, pas à chaque bloc.
                    if faits - dernier > a.taille / 100 {
                        dernier = faits;
                        *etat.lock().unwrap_or_else(|e| e.into_inner()) =
                            Etat::Telechargement(a.version.clone(), faits, a.taille);
                        r();
                    }
                })
            });
            let fin = match res {
                Ok(()) => Etat::Pret(nouveau),
                Err(m) => {
                    let _ = std::fs::remove_file(&nouveau);
                    Etat::Echec(m)
                }
            };
            *etat.lock().unwrap_or_else(|e| e.into_inner()) = fin;
            r();
        });
    }

    /// Échange les fichiers (`aquarium.exe` → `.ancien`, `.nouveau` → `aquarium.exe`) et rend
    /// l'exécutable à relancer après la fermeture propre. Échec : rien n'a changé, message.
    pub fn installer(&self) -> Option<PathBuf> {
        let nouveau = {
            let mut e = self.verrou();
            let Etat::Pret(p) = &*e else { return None };
            let p = p.clone();
            *e = Etat::Installation;
            p
        };
        let res = self
            .exe
            .clone()
            .ok_or_else(|| "Emplacement de l'appli introuvable".to_string())
            .and_then(|exe| {
                remplacer(&exe, &nouveau)
                    .map(|()| exe)
                    .map_err(|_| "Fichier de l'appli verrouillé : rien n'a été changé".into())
            });
        match res {
            Ok(exe) => Some(exe),
            Err(m) => {
                let _ = std::fs::remove_file(&nouveau);
                self.mettre(Etat::Echec(m));
                None
            }
        }
    }
}

/// Relance la nouvelle version (après la fermeture : fond d'écran remis, partie sauvée).
/// Lancement impossible : l'ancienne version reprend sa place.
pub fn relancer(exe: &Path) {
    let args = std::env::args_os().skip(1).filter(|a| a != "--apres-maj");
    if std::process::Command::new(exe)
        .args(args)
        .arg("--apres-maj")
        .spawn()
        .is_err()
    {
        let nouveau = suffixe(exe, ".nouveau");
        restaurer(exe, &nouveau);
        let _ = std::fs::remove_file(&nouveau);
    }
}

/// `chemin` + `fin` (`aquarium.exe` → `aquarium.exe.ancien`).
fn suffixe(chemin: &Path, fin: &str) -> PathBuf {
    let mut s = chemin.as_os_str().to_owned();
    s.push(fin);
    PathBuf::from(s)
}

/// Agent HTTP sans redirection automatique (chaque saut est vérifié par `url_permise`) ;
/// `lecture` = attente maximale entre deux blocs reçus (s).
fn agent(lecture: u64) -> ureq::Agent {
    ureq::AgentBuilder::new()
        .redirects(0)
        .timeout_connect(Duration::from_secs(8))
        .timeout_read(Duration::from_secs(lecture))
        .user_agent(AGENT)
        .build()
}

/// https seulement, vers GitHub (API, page des releases, fichiers des releases).
pub fn url_permise(url: &str) -> bool {
    let Some(reste) = url.strip_prefix("https://") else {
        return false;
    };
    let hote = reste.split(['/', '?', '#']).next().unwrap_or("");
    // Pas d'identifiants ni de port : « github.com@pirate.fr » ou « github.com:444 » refusés.
    HOTES.contains(&hote)
}

/// GET qui suit au plus 5 redirections, chacune vérifiée par `url_permise`.
fn obtenir(agent: &ureq::Agent, url: &str) -> Result<ureq::Response, String> {
    let mut url = url.to_string();
    for _ in 0..6 {
        if !url_permise(&url) {
            return Err("Lien de téléchargement refusé (hors de GitHub)".into());
        }
        let rep = agent
            .get(&url)
            .set(
                "Accept",
                "application/vnd.github+json, application/octet-stream",
            )
            .call()
            .map_err(|e| match e {
                ureq::Error::Status(403 | 429, _) => {
                    "GitHub limite les demandes pour l'instant : réessaie dans une heure"
                        .to_string()
                }
                ureq::Error::Status(404, _) => "Aucune version publiée pour l'instant".to_string(),
                ureq::Error::Status(code, _) => format!("Réponse de GitHub : erreur {code}"),
                ureq::Error::Transport(_) => {
                    "GitHub injoignable : vérifie la connexion à Internet".to_string()
                }
            })?;
        if (300..400).contains(&rep.status()) {
            url = rep
                .header("location")
                .ok_or("Réponse de GitHub illisible")?
                .to_string();
            continue;
        }
        return Ok(rep);
    }
    Err("Trop de redirections : réessaie plus tard".into())
}

fn lire_release() -> Result<Annonce, String> {
    let rep = obtenir(&agent(10), URL_RELEASE)?;
    let mut d = Vec::new();
    rep.into_reader()
        .take(1 << 20)
        .read_to_end(&mut d)
        .map_err(|_| "Réponse de GitHub interrompue".to_string())?;
    annonce_depuis_json(&d)
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    assets: Vec<Fichier>,
}

#[derive(Deserialize)]
struct Fichier {
    name: String,
    browser_download_url: String,
    size: u64,
}

/// Release GitHub (JSON de `releases/latest`) → annonce : `aquarium.exe` et son `.sha256`.
pub fn annonce_depuis_json(d: &[u8]) -> Result<Annonce, String> {
    let r: Release =
        serde_json::from_slice(d).map_err(|_| "Réponse de GitHub illisible".to_string())?;
    let trouver = |nom: &str| r.assets.iter().find(|a| a.name == nom);
    let (Some(exe), Some(sha)) = (trouver(NOM_EXE), trouver(&format!("{NOM_EXE}.sha256"))) else {
        return Err("La dernière version ne contient pas l'exécutable Windows".into());
    };
    if !url_permise(&exe.browser_download_url) || !url_permise(&sha.browser_download_url) {
        return Err("Lien de téléchargement refusé (hors de GitHub)".into());
    }
    Ok(Annonce {
        version: r.tag_name.trim_start_matches('v').to_string(),
        url: exe.browser_download_url.clone(),
        url_sha256: sha.browser_download_url.clone(),
        taille: exe.size,
        notes: r.body.unwrap_or_default(),
    })
}

/// Fichier `.sha256` (« <64 hex>  aquarium.exe » ou les 64 caractères seuls).
pub fn empreinte_depuis_texte(t: &str) -> Result<String, String> {
    let h = t.split_whitespace().next().unwrap_or("");
    if h.len() == 64 && h.bytes().all(|c| c.is_ascii_hexdigit()) {
        Ok(h.to_ascii_lowercase())
    } else {
        Err("Empreinte de la nouvelle version illisible : rien n'a été changé".into())
    }
}

fn lire_empreinte(agent: &ureq::Agent, url: &str) -> Result<String, String> {
    let mut t = String::new();
    obtenir(agent, url)?
        .into_reader()
        .take(4096)
        .read_to_string(&mut t)
        .map_err(|_| "Téléchargement interrompu : réessaie".to_string())?;
    empreinte_depuis_texte(&t)
}

/// SHA-256 en hexadécimal minuscule.
#[cfg(test)]
fn sha256_hex(d: &[u8]) -> String {
    hex(ring::digest::digest(&ring::digest::SHA256, d).as_ref())
}

fn hex(o: &[u8]) -> String {
    o.iter().map(|o| format!("{o:02x}")).collect()
}

/// Écrit le fichier annoncé dans `dest` en vérifiant taille et empreinte `sha` ; `progres`
/// reçoit les octets déjà reçus.
fn telecharger_vers(
    agent: &ureq::Agent,
    a: &Annonce,
    sha: &str,
    dest: &Path,
    progres: impl FnMut(u64),
) -> Result<(), String> {
    if a.taille == 0 || a.taille > TAILLE_MAX {
        return Err("Version publiée invalide".into());
    }
    let rep = obtenir(agent, &a.url)?;
    ecrire_verifie(rep.into_reader(), a.taille, sha, dest, progres)
}

/// Copie `lecteur` dans `dest` (taille `taille` exacte, empreinte `sha`).
fn ecrire_verifie(
    lecteur: impl Read,
    taille: u64,
    sha: &str,
    dest: &Path,
    mut progres: impl FnMut(u64),
) -> Result<(), String> {
    let mut f = std::fs::File::create(dest).map_err(|_| {
        format!("Dossier de l'appli protégé : télécharge la nouvelle version sur {PAGE_RELEASES}")
    })?;
    let mut lecteur = lecteur.take(taille + 1);
    let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
    let mut tampon = vec![0u8; 64 << 10];
    let mut faits = 0u64;
    loop {
        let n = lecteur
            .read(&mut tampon)
            .map_err(|_| "Téléchargement interrompu : réessaie".to_string())?;
        if n == 0 {
            break;
        }
        ctx.update(&tampon[..n]);
        f.write_all(&tampon[..n]).map_err(|_| {
            "Écriture impossible (disque plein ?) : rien n'a été changé".to_string()
        })?;
        faits += n as u64;
        progres(faits);
    }
    if faits != taille || !hex(ctx.finish().as_ref()).eq_ignore_ascii_case(sha) {
        return Err("Fichier reçu abîmé (empreinte différente) : rien n'a été changé".into());
    }
    f.sync_all()
        .map_err(|_| "Écriture impossible : rien n'a été changé".to_string())
}

/// `exe` → `exe.ancien`, `nouveau` → `exe` (Windows permet de renommer l'exe en cours).
/// Échec du second renommage : l'exe d'origine est remis.
fn remplacer(exe: &Path, nouveau: &Path) -> std::io::Result<()> {
    let ancien = suffixe(exe, ".ancien");
    let _ = std::fs::remove_file(&ancien);
    std::fs::rename(exe, &ancien)?;
    if let Err(e) = std::fs::rename(nouveau, exe) {
        let _ = std::fs::rename(&ancien, exe);
        return Err(e);
    }
    Ok(())
}

/// Annule [`remplacer`] : la nouvelle version retourne en `nouveau`, l'ancienne reprend sa place.
fn restaurer(exe: &Path, nouveau: &Path) {
    if std::fs::rename(exe, nouveau).is_ok() {
        let _ = std::fs::rename(suffixe(exe, ".ancien"), exe);
    }
}

/// Lancement : supprime en fond l'ancienne version (et un téléchargement interrompu). Juste
/// après une mise à jour, l'ancien processus peut encore tenir son fichier : quelques essais.
pub fn nettoyer() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    std::thread::spawn(move || {
        let _ = std::fs::remove_file(suffixe(&exe, ".nouveau"));
        let ancien = suffixe(&exe, ".ancien");
        for _ in 0..60 {
            if !ancien.exists() || std::fs::remove_file(&ancien).is_ok() {
                return;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    });
}

/// `a` plus récente que `b` (« 0.10.0 » > « 0.9.3 », préfixe « v » et suffixe « -… » ignorés).
pub fn plus_recente(a: &str, b: &str) -> bool {
    let nums = |s: &str| -> Vec<u64> {
        let s = s.trim().trim_start_matches('v');
        let s = s.split(['-', '+']).next().unwrap_or("");
        s.split('.').map(|p| p.parse().unwrap_or(0)).collect()
    };
    let (mut x, mut y) = (nums(a), nums(b));
    let n = x.len().max(y.len());
    x.resize(n, 0);
    y.resize(n, 0);
    x > y
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dossier(nom: &str) -> PathBuf {
        let d =
            std::env::temp_dir().join(format!("aquarium-test-maj-{nom}-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&d);
        d
    }

    #[test]
    fn comparaison_des_versions() {
        assert!(plus_recente("v0.8.0", "0.7.0"));
        assert!(plus_recente("0.10.0", "0.9.0"));
        assert!(plus_recente("v1.0", "0.99.9"));
        assert!(plus_recente("0.8.1", "0.8"));
        assert!(!plus_recente("v0.8.0", "0.8.0"));
        assert!(!plus_recente("0.7.9", "0.8.0"));
        assert!(!plus_recente("0.8.0-beta", "0.8.0"));
        assert!(!plus_recente("n'importe quoi", "0.1.0"));
    }

    #[test]
    fn lecture_d_une_release() {
        let json = br#"{"url":"https://api.github.com/repos/Sneakyquaza/aquarium/releases/1",
            "tag_name":"v0.8.0","name":"v0.8.0","body":"Shiny, modificateurs, index.",
            "assets":[
              {"name":"aquarium.exe.sha256","size":80,"browser_download_url":
               "https://github.com/Sneakyquaza/aquarium/releases/download/v0.8.0/aquarium.exe.sha256"},
              {"name":"aquarium.exe","size":12345678,"browser_download_url":
               "https://github.com/Sneakyquaza/aquarium/releases/download/v0.8.0/aquarium.exe"}]}"#;
        let a = annonce_depuis_json(json).unwrap();
        assert_eq!(a.version, "0.8.0");
        assert_eq!(a.taille, 12_345_678);
        assert!(a.url.ends_with("/aquarium.exe") && a.url_sha256.ends_with(".sha256"));
        assert_eq!(a.notes, "Shiny, modificateurs, index.");
        // Sans l'exécutable, ou un lien hors de GitHub : refusé.
        let sans = br#"{"tag_name":"v0.8.0","body":null,"assets":[]}"#;
        assert!(annonce_depuis_json(sans).is_err());
        let pirate = String::from_utf8_lossy(json).replace(
            "https://github.com/Sneakyquaza/aquarium/releases/download/v0.8.0/aquarium.exe\"",
            "https://pirate.example/aquarium.exe\"",
        );
        assert!(annonce_depuis_json(pirate.as_bytes()).is_err());
        assert!(annonce_depuis_json(b"<html>").is_err());
    }

    #[test]
    fn liste_des_liens_permis() {
        for ok in [
            URL_RELEASE,
            "https://github.com/Sneakyquaza/aquarium/releases/download/v0.8.0/aquarium.exe",
            "https://objects.githubusercontent.com/github-production-release-asset/1?x=y",
            "https://release-assets.githubusercontent.com/github-production-release-asset/1",
        ] {
            assert!(url_permise(ok), "{ok}");
        }
        for non in [
            "http://github.com/Sneakyquaza/aquarium/releases/download/v0.8.0/aquarium.exe",
            "https://github.com.pirate.fr/aquarium.exe",
            "https://github.com@pirate.fr/aquarium.exe",
            "https://github.com:444/aquarium.exe",
            "https://evilgithub.com/aquarium.exe",
            "ftp://github.com/x",
            "file:///C:/aquarium.exe",
        ] {
            assert!(!url_permise(non), "{non}");
        }
    }

    #[test]
    fn empreinte_verifiee() {
        let d = dossier("hash");
        let corps = b"nouvelle version".to_vec();
        let vrai = sha256_hex(&corps);
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            empreinte_depuis_texte(&format!("{}  aquarium.exe\n", vrai.to_uppercase())).unwrap(),
            vrai
        );
        assert!(empreinte_depuis_texte("pas une empreinte").is_err());
        let dest = d.join("aquarium.exe.nouveau");
        let mut vus = 0;
        ecrire_verifie(&corps[..], corps.len() as u64, &vrai, &dest, |n| vus = n).unwrap();
        assert_eq!(std::fs::read(&dest).unwrap(), corps);
        assert_eq!(vus, corps.len() as u64);
        // Empreinte fausse, fichier tronqué : refusés.
        let faux = sha256_hex(b"autre");
        let e = ecrire_verifie(&corps[..], corps.len() as u64, &faux, &dest, |_| {}).unwrap_err();
        assert!(e.contains("empreinte"), "{e}");
        assert!(ecrire_verifie(&corps[..4], corps.len() as u64, &vrai, &dest, |_| {}).is_err());
        // Dossier non inscriptible : renvoi vers la page des releases.
        let e =
            ecrire_verifie(&corps[..], 16, &vrai, &d.join("non/inscriptible"), |_| {}).unwrap_err();
        assert!(e.contains(PAGE_RELEASES), "{e}");
    }

    #[test]
    fn remplacement_et_retour() {
        let d = dossier("remplacer");
        let exe = d.join("aquarium.exe");
        let nouveau = suffixe(&exe, ".nouveau");
        std::fs::write(&exe, b"ancien").unwrap();
        std::fs::write(&nouveau, b"nouveau").unwrap();
        remplacer(&exe, &nouveau).unwrap();
        assert_eq!(std::fs::read(&exe).unwrap(), b"nouveau");
        assert_eq!(std::fs::read(suffixe(&exe, ".ancien")).unwrap(), b"ancien");
        restaurer(&exe, &nouveau);
        assert_eq!(std::fs::read(&exe).unwrap(), b"ancien");
        // Nouveau fichier absent : l'exe d'origine reste en place.
        assert!(remplacer(&exe, &d.join("absent")).is_err());
        assert_eq!(std::fs::read(&exe).unwrap(), b"ancien");
    }
}

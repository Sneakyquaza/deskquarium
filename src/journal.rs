//! Journal de diagnostic : `journal.log` dans le dossier de sauvegarde (%APPDATA%\Deskquarium).
//! Incrustation sur le bureau, fenêtres trouvées, première image, pauses : ce qui permet de
//! comprendre à distance pourquoi rien ne s'affiche.

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

static VERROU: Mutex<()> = Mutex::new(());
/// v0.18 : instant du lancement du processus (jalons du démarrage).
static DEPART: OnceLock<Instant> = OnceLock::new();
/// `--demarrage` : jalons aussi affichés en console (mesure sous Linux).
pub static CONSOLE: AtomicBool = AtomicBool::new(false);

/// Note l'instant du lancement (tout début de `main`).
pub fn depart() {
    DEPART.get_or_init(Instant::now);
}

/// Millisecondes depuis le lancement.
pub fn ms() -> f64 {
    DEPART.get_or_init(Instant::now).elapsed().as_secs_f64() * 1000.0
}

/// v0.18 : jalon du démarrage ou de la fermeture (« démarrage : <étape> à N ms » : temps
/// depuis le lancement du processus).
pub fn jalon(etape: &str) {
    let m = format!("{etape} à {:.0} ms", ms());
    if CONSOLE.load(Ordering::Relaxed) {
        println!("{m}");
    }
    ecrire(&m);
}

pub fn chemin() -> PathBuf {
    crate::sauve::dossier().join("journal.log")
}

/// Ajoute une ligne horodatée (secondes Unix). Jamais d'erreur : le journal est facultatif.
pub fn ecrire(m: &str) {
    let _v = VERROU.lock();
    let f = chemin();
    if let Some(d) = f.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    // Au-delà de 512 Ko, l'ancien journal est gardé à côté et on repart à zéro.
    if std::fs::metadata(&f).is_ok_and(|m| m.len() > 512 * 1024) {
        let _ = std::fs::rename(&f, f.with_extension("ancien.log"));
    }
    if let Ok(mut o) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&f)
    {
        let _ = writeln!(o, "[{}] {m}", crate::sauve::maintenant());
    }
}

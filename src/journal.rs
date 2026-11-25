//! Journal de diagnostic : `journal.log` dans le dossier de sauvegarde (%APPDATA%\Deskquarium).
//! Incrustation sur le bureau, fenêtres trouvées, première image, pauses : ce qui permet de
//! comprendre à distance pourquoi rien ne s'affiche.

use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

static VERROU: Mutex<()> = Mutex::new(());

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

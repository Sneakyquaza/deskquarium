//! Langue de l'interface (v0.11) : français ou anglais. Le texte français du code sert de clé :
//! `tr("Œufs")` renvoie « Eggs » en anglais. Toutes les traductions sont dans `TABLE`
//! (test : chaque clé utilisée dans le code y est, et aucun texte d'interface n'y échappe).
//! `trf` remplace les `{}` dans l'ordre. La langue s'applique tout de suite (atomique).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg_attr(test, allow(unused_imports))]
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Langue {
    Fr,
    En,
}

static LANGUE: AtomicU8 = AtomicU8::new(0);

// Tests : langue propre à chaque fil (les tests tournent en parallèle).
#[cfg(test)]
thread_local! {
    static LANGUE_TEST: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
}

fn lire() -> u8 {
    #[cfg(test)]
    {
        let _ = &LANGUE;
        LANGUE_TEST.with(|c| c.get())
    }
    #[cfg(not(test))]
    {
        LANGUE.load(Ordering::Relaxed)
    }
}

/// Langue en cours.
pub fn langue() -> Langue {
    if lire() == 1 {
        Langue::En
    } else {
        Langue::Fr
    }
}

pub fn regler(l: Langue) {
    let v = (l == Langue::En) as u8;
    #[cfg(test)]
    LANGUE_TEST.with(|c| c.set(v));
    #[cfg(not(test))]
    LANGUE.store(v, Ordering::Relaxed);
}

pub fn anglais() -> bool {
    langue() == Langue::En
}

/// Langue de Windows (GetUserDefaultUILanguage : français → Français, sinon anglais) ;
/// ailleurs, la variable LANG.
pub fn langue_systeme() -> Langue {
    static L: OnceLock<Langue> = OnceLock::new();
    *L.get_or_init(langue_systeme_lue)
}

fn langue_systeme_lue() -> Langue {
    // DESKQUARIUM_LANGUE, ou l'ancien nom AQUARIUM_LANGUE.
    if let Ok(v) = std::env::var("DESKQUARIUM_LANGUE").or_else(|_| std::env::var("AQUARIUM_LANGUE"))
    {
        return if v.starts_with("en") {
            Langue::En
        } else {
            Langue::Fr
        };
    }
    #[cfg(windows)]
    {
        // Identifiant de langue : les 10 bits bas = langue principale (0x0C = français).
        let id = unsafe { windows_sys::Win32::Globalization::GetUserDefaultUILanguage() };
        if id & 0x3ff == 0x0c {
            Langue::Fr
        } else {
            Langue::En
        }
    }
    #[cfg(not(windows))]
    {
        match std::env::var("LANG") {
            Ok(v) if v.starts_with("fr") => Langue::Fr,
            Ok(v) if !v.is_empty() && v != "C" && v != "C.UTF-8" && v != "POSIX" => Langue::En,
            _ => Langue::Fr,
        }
    }
}

fn table() -> &'static HashMap<&'static str, &'static str> {
    static T: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    T.get_or_init(|| TABLE.iter().copied().collect())
}

/// Texte d'interface dans la langue en cours (clé = le texte français).
pub fn tr(fr: &'static str) -> &'static str {
    if anglais() {
        table().get(fr).copied().unwrap_or(fr)
    } else {
        fr
    }
}

/// `tr` puis chaque `{}` remplacé par l'argument suivant.
pub fn trf(fr: &'static str, args: &[&str]) -> String {
    let mut s = String::new();
    let mut a = args.iter();
    let mut reste = tr(fr);
    while let Some(i) = reste.find("{}") {
        s.push_str(&reste[..i]);
        s.push_str(a.next().copied().unwrap_or(""));
        reste = &reste[i + 2..];
    }
    s.push_str(reste);
    s
}

include!("i18n_table.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_complete_sans_doublon() {
        let mut vu = std::collections::HashSet::new();
        for (fr, en) in TABLE {
            assert!(vu.insert(*fr), "clé en double : {fr}");
            assert!(!en.trim().is_empty(), "pas d'anglais pour {fr}");
            assert_eq!(fr.matches("{}").count(), en.matches("{}").count(), "{fr}");
        }
    }

    /// Littéraux de `tr("…")` / `trf("…"` dans les sources.
    fn cles_du_code() -> Vec<(String, String)> {
        let mut v = Vec::new();
        for f in sources().into_iter().filter(|f| !f.ends_with("i18n.rs")) {
            let s = std::fs::read_to_string(&f).unwrap();
            for motif in ["tr(\"", "trf(\""] {
                for (i, _) in s.match_indices(motif) {
                    // Pas un autre identifiant qui finit par « tr ».
                    let avant = s[..i].chars().last().unwrap_or(' ');
                    if avant.is_alphanumeric() || avant == '_' {
                        continue;
                    }
                    let debut = i + motif.len();
                    let fin = litteral_fin(&s[debut..]);
                    v.push((f.clone(), s[debut..debut + fin].replace("\\\"", "\"")));
                }
            }
        }
        v
    }

    fn litteral_fin(s: &str) -> usize {
        let b = s.as_bytes();
        let mut i = 0;
        while i < b.len() {
            if b[i] == b'\\' {
                i += 2;
                continue;
            }
            if b[i] == b'"' {
                return i;
            }
            i += 1;
        }
        i
    }

    fn sources() -> Vec<String> {
        let mut v = Vec::new();
        let mut pile = vec![std::path::PathBuf::from("src")];
        while let Some(d) = pile.pop() {
            for e in std::fs::read_dir(d).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    pile.push(p);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    v.push(p.display().to_string());
                }
            }
        }
        v
    }

    #[test]
    fn chaque_cle_du_code_est_traduite() {
        let t = table();
        let manque: Vec<_> = cles_du_code()
            .into_iter()
            .filter(|(_, k)| !t.contains_key(k.as_str()))
            .collect();
        assert!(manque.is_empty(), "sans traduction : {manque:#?}");
    }

    #[test]
    fn aucun_texte_d_interface_n_echappe() {
        // Dans les fichiers d'interface, un texte affiché passe par tr/trf : on cherche les
        // littéraux avec des mots donnés directement aux widgets egui.
        let motifs = [
            "RichText::new(\"",
            ".label(\"",
            "Button::new(\"",
            ".small_button(\"",
            ".heading(\"",
            "on_hover_text(\"",
            "selected_text(\"",
            "RichText::new(format!(\"",
            ".checkbox(&mut r.", // suivi d'un littéral sur la même ligne
        ];
        let fichiers = [
            "src/ecloserie.rs",
            "src/ecloserie/decor_ui.rs",
            "src/appli.rs",
        ];
        let mut fautes = Vec::new();
        for f in fichiers {
            let s = std::fs::read_to_string(f).unwrap();
            for (n, l) in s.lines().enumerate() {
                for m in motifs {
                    if let Some(i) = l.find(m) {
                        let reste = &l[i + m.len()..];
                        let lit = if m.ends_with('.') {
                            reste.split('"').nth(1).unwrap_or("")
                        } else {
                            &reste[..litteral_fin(reste)]
                        };
                        if lit.chars().filter(|c| c.is_alphabetic()).count() >= 2
                            && !l.contains("tr(")
                            && !["OK", "Langue · Language"].contains(&lit)
                        {
                            fautes.push(format!("{f}:{} {}", n + 1, l.trim()));
                        }
                    }
                }
            }
        }
        assert!(
            fautes.is_empty(),
            "textes non traduits :\n{}",
            fautes.join("\n")
        );
    }

    #[test]
    fn anglais_et_francais() {
        regler(Langue::En);
        assert_eq!(tr("Œufs"), "Eggs");
        assert_eq!(
            trf("Encore {}", &["3 min"]),
            "{} more".replace("{}", "3 min")
        );
        regler(Langue::Fr);
        assert_eq!(tr("Œufs"), "Œufs");
        assert_eq!(trf("Encore {}", &["3 min"]), "Encore 3 min");
    }
}

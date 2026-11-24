// Pas de console sous Windows en version finale.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod appli;
mod art;
mod capture;
mod ecloserie;
mod especes;
mod fond;
mod jeu;
mod poisson;
mod sauve;
mod scene;
mod toile;
#[cfg(windows)]
mod windows;

use std::path::PathBuf;

const AIDE: &str = "aquarium : fond d'écran aquarium pixel art + idle game

Sans option : sous Windows, l'aquarium se place derrière les icônes du bureau
(icône près de l'horloge : Ouvrir l'Écloserie, Pause, Quitter).

Options (tests) :
  --fenetre                       fenêtre normale (aquarium + Écloserie)
  --capture X.png --secondes N    rendu sans écran après N secondes
  --video X.mp4 --secondes N      vidéo 30 i/s (ffmpeg, AQUARIUM_FFMPEG)
  --planche X.png                 toutes les espèces, une ligne par rareté, x4
  --planche-geantes X.png         créatures énormes et colossales à taille réelle, x2
  --demo                          aquarium pré-rempli (raretés et espèces variées)
  --taille 1920x1080              taille de la capture / vidéo
  --fond image.png                fond d'écran sous l'aquarium (sinon fond factice)
  --captures-ecloserie DOSSIER    (avec --fenetre) captures de chaque onglet puis quitte
  --simulation                    jalons de l'équilibrage (docs/equilibrage.md)";

fn main() {
    if let Err(e) = lancer() {
        eprintln!("aquarium : {e}");
        #[cfg(windows)]
        windows::alerte(&e);
        std::process::exit(1);
    }
}

fn lancer() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let valeur = |nom: &str| -> Option<String> {
        args.iter()
            .position(|a| a == nom)
            .and_then(|i| args.get(i + 1).cloned())
    };
    let a = |nom: &str| args.iter().any(|x| x == nom);
    if a("--aide") || a("--help") || a("-h") {
        println!("{AIDE}");
        return Ok(());
    }
    if a("--simulation") {
        for graine in [1, 2, 3] {
            println!("partie {graine}");
            for j in jeu::simuler(120.0, graine) {
                println!("{:>12}  {}", jeu::duree(j.temps_s), j.texte);
            }
        }
        return Ok(());
    }
    let (largeur, hauteur) = match valeur("--taille") {
        Some(t) => {
            let (l, h) = t.split_once('x').ok_or("--taille attend LxH")?;
            (
                l.parse().map_err(|_| "--taille : largeur")?,
                h.parse().map_err(|_| "--taille : hauteur")?,
            )
        }
        None => (1920, 1080),
    };
    let o = capture::Options {
        largeur,
        hauteur,
        secondes: valeur("--secondes")
            .map(|s| s.parse().map_err(|_| "--secondes : nombre"))
            .transpose()?
            .unwrap_or(5.0),
        demo: a("--demo"),
        fond: valeur("--fond"),
    };
    if let Some(f) = valeur("--capture") {
        return capture::capture(&o, &PathBuf::from(f));
    }
    if let Some(f) = valeur("--video") {
        return capture::video(&o, &PathBuf::from(f));
    }
    if let Some(f) = valeur("--planche") {
        return capture::planche(&PathBuf::from(f), o.secondes.min(3.0));
    }
    if let Some(f) = valeur("--planche-geantes") {
        return capture::planche_geantes(&PathBuf::from(f), o.secondes.min(3.0));
    }
    let fenetre = a("--fenetre") || !cfg!(windows);
    appli::lancer(appli::Options {
        fenetre,
        demo: a("--demo"),
        fond: o.fond,
        captures: valeur("--captures-ecloserie").map(PathBuf::from),
    })
}

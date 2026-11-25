// Pas de console sous Windows en version finale.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod appli;
mod bench;
#[cfg(windows)]
mod calque;
mod capture;
mod compo;
#[cfg(windows)]
mod crochet;
mod decor;
mod ecloserie;
mod eclosion;
mod ecrans;
mod especes;
mod fond;
mod i18n;
mod jeu;
mod journal;
mod maj;
#[cfg(windows)]
mod moteur;
mod nourriture;
mod occlusion;
mod penche;
mod poisson;
mod sauve;
mod scene;
mod son;
mod souris;
mod toile;
#[cfg(windows)]
mod windows;

use std::path::PathBuf;

const AIDE: &str = "deskquarium : fond d'écran aquarium pixel art + idle game

Sans option : sous Windows, l'aquarium se place derrière les icônes du bureau
(icône près de l'horloge : Ouvrir l'Écloserie, Pause, Quitter).

Options (tests) :
  --fenetre                       fenêtre normale (aquarium + Écloserie)
  --capture X.png --secondes N    rendu sans écran après N secondes
  --video X.mp4 --secondes N      vidéo 30 i/s (ffmpeg, AQUARIUM_FFMPEG)
  --planche X.png                 toutes les espèces, une ligne par rareté, x4
  --planche-geantes X.png         créatures énormes et colossales à taille réelle, x2
  --alignement X.png              toutes les espèces à leur taille en 1920x1080 (tailles relatives)
  --planche-images X.png cle,cle  images d'animation des espèces données, x4
  --planche-pente X.png cle,cle   espèces penchées de −30° à 30°, dans les deux sens, x4
  --planche-formes X.png cle,cle [--modifs]  normal et shiny (ou les 7 modificateurs), x2
  --planche-demi-tour X.png cle   demi-tour image par image (0,1 s), x4
  --demo                          aquarium pré-rempli (raretés et espèces variées)
  --taille 1920x1080              taille de la capture / vidéo
  --ecrans 1920x1080+0+0,...      écrans simulés (capture / vidéo), le premier est le principal
  --abysses                       mode Abysses (capture / vidéo)
  --theme N                      décor à thème 0-5 (capture / vidéo)
  --planche-souris X.png         curiosité, fuite, poisson attrapé, fiche (avec --demo)
  --eclosion X.mp4 --oeuf N --espece cle [--forme shiny,galaxie] [--dore]
                                 éclosion en vidéo (60 i/s) + planche X.png
  --multi X.mp4 --oeuf N [--nombre 10] [--graine G]   ouverture ×N (v0.11)
  --repos-oeufs X.mp4 [--secondes 8]   les 8 œufs au repos (v0.11)
  --planche-oeufs X.png          les œufs et leur animation au repos (x3)
  --objets cle,cle               objets de décor posés (capture / vidéo)
  --nourrir N@S[@X[@Y]],…        jette la nourriture N (0-6) à la seconde S, en X (fraction de
                                 l'écran principal) ou au hasard, d'en haut ou en Y comme un
                                 clic (capture / vidéo)
  --fond image.png                fond d'écran sous l'aquarium (sinon fond factice)
  --captures-ecloserie DOSSIER    (avec --fenetre) captures de chaque onglet puis quitte
  --audit DOSSIER [cle,cle]       planches d'audit des sprites (formes, nage, demi-tour)
  --bench [--secondes 60] [--ips 24] [--ancien]  coût du fond d'écran (3840x1080, 40 poissons)
  --simulation                    jalons de l'équilibrage (docs/equilibrage.md)";

fn main() {
    if let Err(e) = lancer() {
        eprintln!("deskquarium : {e}");
        #[cfg(windows)]
        windows::alerte(&e);
        std::process::exit(1);
    }
}

fn lancer() -> Result<(), String> {
    // Captures et outils : langue du système (DESKQUARIUM_LANGUE ou AQUARIUM_LANGUE=en pour l'anglais).
    i18n::regler(i18n::langue_systeme());
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
            for j in jeu::simuler(600.0, graine) {
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
        ecrans: valeur("--ecrans"),
        abysses: a("--abysses"),
        theme: valeur("--theme")
            .map(|s| s.parse().map_err(|_| "--theme : 0 à 5"))
            .transpose()?,
        objets: valeur("--objets")
            .map(|s| {
                s.split(',')
                    .filter(|c| !c.is_empty())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default(),
        nourrir: valeur("--nourrir")
            .map(|s| {
                s.split(',')
                    .filter_map(|j| {
                        let mut p = j.split('@');
                        Some((
                            p.next()?.parse().ok()?,
                            p.next()?.parse().ok()?,
                            p.next().and_then(|x| x.parse().ok()),
                            p.next().and_then(|y| y.parse().ok()),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default(),
    };
    if a("--bench") {
        // --bench [--secondes 60] [--ips 24] [--ancien]
        let ips = valeur("--ips").and_then(|v| v.parse().ok()).unwrap_or(24);
        let s = valeur("--secondes")
            .and_then(|v| v.parse().ok())
            .unwrap_or(60.0);
        return bench::bench(s, ips, a("--ancien"));
    }
    if let Some(f) = valeur("--capture") {
        return capture::capture(&o, &PathBuf::from(f));
    }
    if let Some(f) = valeur("--multi") {
        // --multi X.mp4 --oeuf N [--nombre 10] [--graine G]
        let oeuf = valeur("--oeuf").and_then(|v| v.parse().ok()).unwrap_or(0);
        let n = valeur("--nombre")
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);
        let g = valeur("--graine").and_then(|v| v.parse().ok()).unwrap_or(1);
        return capture::multi(&PathBuf::from(f), oeuf, n, g);
    }
    if let Some(f) = valeur("--repos-oeufs") {
        let s = valeur("--secondes")
            .and_then(|v| v.parse().ok())
            .unwrap_or(8.0);
        return capture::repos_oeufs(&PathBuf::from(f), s);
    }
    if let Some(f) = valeur("--planche-oeufs") {
        return capture::planche_oeufs(&PathBuf::from(f));
    }
    if let Some(f) = valeur("--eclosion") {
        // --eclosion X.mp4 --oeuf N --espece cle [--forme shiny,galaxie] [--dore]
        let oeuf = valeur("--oeuf").and_then(|v| v.parse().ok()).unwrap_or(0);
        let cle = valeur("--espece").unwrap_or_else(|| "clown".into());
        let fo = valeur("--forme").unwrap_or_default();
        let forme = jeu::Forme {
            shiny: fo.contains("shiny"),
            modif: jeu::Modif::TOUS
                .into_iter()
                .find(|m| fo.contains(&m.nom_fr().to_lowercase())),
        };
        return capture::eclosion(&PathBuf::from(f), oeuf, &cle, forme, a("--dore"));
    }
    if let Some(f) = valeur("--planche-souris") {
        return capture::planche_souris(&o, &PathBuf::from(f));
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
    if let Some(f) = valeur("--alignement") {
        return capture::alignement(&PathBuf::from(f), o.secondes.min(3.0));
    }
    if let Some(f) = valeur("--planche-formes") {
        let cles = valeur(&f).unwrap_or_default();
        let cles: Vec<&str> = cles.split(',').filter(|c| !c.is_empty()).collect();
        return capture::planche_formes(
            &PathBuf::from(&f),
            &cles,
            a("--modifs"),
            o.secondes.min(3.0),
        );
    }
    if let Some(f) = valeur("--planche-demi-tour") {
        let cle = valeur(&f).unwrap_or_default();
        return capture::planche_demi_tour(&PathBuf::from(&f), &cle);
    }
    if let Some(f) = valeur("--audit") {
        let cles = valeur(&f).unwrap_or_default();
        let cles: Vec<&str> = cles.split(',').filter(|c| !c.is_empty()).collect();
        return capture::audit(&PathBuf::from(&f), &cles);
    }
    if let Some(f) = valeur("--planche-pente") {
        let cles = valeur(&f).unwrap_or_default();
        let cles: Vec<&str> = cles.split(',').filter(|c| !c.is_empty()).collect();
        return capture::planche_pente(&PathBuf::from(&f), &cles, 4);
    }
    if let Some(f) = valeur("--planche-images") {
        let cles = valeur(&f).unwrap_or_default();
        let cles: Vec<&str> = cles.split(',').filter(|c| !c.is_empty()).collect();
        return capture::planche_images(&PathBuf::from(&f), &cles, 4);
    }
    let fenetre = a("--fenetre") || !cfg!(windows);
    appli::lancer(appli::Options {
        fenetre,
        demo: a("--demo"),
        fond: o.fond,
        captures: valeur("--captures-ecloserie").map(PathBuf::from),
    })
}

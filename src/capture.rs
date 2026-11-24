//! Rendu sans écran (Linux) : capture PNG, vidéo via ffmpeg, planche des raretés, mesures.

use crate::fond;
use crate::jeu::{Alea, Partie, Rarete, CAPACITE};
use crate::scene::{echelle_pour, sprite_de, Aquarium, Nageur};
use crate::toile::Toile;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

/// Aquarium pré-rempli : au moins un poisson de chaque rareté.
pub fn partie_demo() -> Partie {
    let mut p = Partie::nouvelle(crate::sauve::maintenant(), 2024);
    p.argent = 1e15;
    for _ in 0..22 {
        let _ = p.acheter_oeuf(0);
    }
    for i in 1..4 {
        for _ in 0..4 {
            let _ = p.acheter_oeuf(i);
        }
    }
    let mut a = Alea(99);
    for r in Rarete::TOUTES {
        if !p.poissons.iter().any(|f| f.rarete == r && f.actif) {
            // Remplace le plus faible pour garder la limite.
            if p.nb_actifs() >= CAPACITE {
                let id = p
                    .poissons
                    .iter()
                    .filter(|f| f.actif)
                    .min_by(|x, y| x.revenu().total_cmp(&y.revenu()))
                    .unwrap()
                    .id;
                let _ = p.basculer(id);
            }
            let id = p.prochain_id;
            p.prochain_id += 1;
            p.poissons.push(crate::jeu::Poisson {
                id,
                graine: a.suivant(),
                rarete: r,
                taille: 1.2,
                niveau: 3,
                actif: true,
                oeuf: 3,
            });
            p.eclos[r.index()] += 1;
        }
    }
    p.argent = 1_234_567.0;
    p
}

pub struct Options {
    pub largeur: u32,
    pub hauteur: u32,
    pub secondes: f32,
    pub demo: bool,
    pub fond: Option<String>,
}

fn preparer(o: &Options) -> Result<(Partie, Aquarium, image::RgbaImage, usize), String> {
    let partie = if o.demo {
        partie_demo()
    } else {
        crate::sauve::charger(&crate::sauve::dossier()).0
    };
    let e = echelle_pour(o.largeur) as usize;
    let mut aq = Aquarium::new(
        (o.largeur as usize).div_ceil(e),
        (o.hauteur as usize).div_ceil(e),
        1,
    );
    aq.synchroniser(&partie);
    let fond = match &o.fond {
        Some(f) => {
            let img = image::open(f).map_err(|e| format!("fond {f} : {e}"))?;
            fond::couvrir(&img.to_rgba8(), o.largeur, o.hauteur)
        }
        None => fond::factice(o.largeur, o.hauteur),
    };
    Ok((partie, aq, fond, e))
}

pub fn capture(o: &Options, sortie: &Path) -> Result<(), String> {
    let (mut partie, mut aq, fond, e) = preparer(o)?;
    let dt = 1.0 / 30.0;
    let n = (o.secondes / dt) as usize;
    let debut = Instant::now();
    for _ in 0..n.max(1) {
        partie.avancer(dt as f64);
        aq.avancer(dt);
        aq.dessiner(&partie);
    }
    let par_image = debut.elapsed().as_secs_f64() * 1000.0 / n.max(1) as f64;
    println!(
        "toile {}x{} (x{e}), {} poissons : {par_image:.2} ms par image (simulation + dessin)",
        aq.toile.l,
        aq.toile.h,
        aq.nageurs.len()
    );
    let img = fond::composer(&fond, &aq.toile, e);
    creer_dossier(sortie)?;
    img.save(sortie).map_err(|e| e.to_string())
}

pub fn video(o: &Options, sortie: &Path) -> Result<(), String> {
    let (mut partie, mut aq, fond, e) = preparer(o)?;
    creer_dossier(sortie)?;
    let ffmpeg = std::env::var("AQUARIUM_FFMPEG").unwrap_or_else(|_| "ffmpeg".into());
    let mut enfant = std::process::Command::new(&ffmpeg)
        .args([
            "-y",
            "-loglevel",
            "error",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-s",
        ])
        .arg(format!("{}x{}", o.largeur, o.hauteur))
        .args([
            "-r", "30", "-i", "-", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "18",
        ])
        .arg(sortie)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("{ffmpeg} : {e} (AQUARIUM_FFMPEG pour donner le chemin)"))?;
    let mut entree = enfant.stdin.take().ok_or("ffmpeg sans entrée")?;
    let dt = 1.0 / 30.0;
    // Une seconde d'avance : bulles et particules déjà en place.
    for _ in 0..30 {
        aq.avancer(dt);
    }
    for _ in 0..(o.secondes * 30.0) as usize {
        partie.avancer(dt as f64);
        aq.avancer(dt);
        aq.dessiner(&partie);
        let img = fond::composer(&fond, &aq.toile, e);
        entree.write_all(img.as_raw()).map_err(|e| e.to_string())?;
    }
    drop(entree);
    let s = enfant.wait().map_err(|e| e.to_string())?;
    s.success()
        .then_some(())
        .ok_or_else(|| format!("ffmpeg : {s}"))
}

/// Planche des 8 raretés (un poisson chacune, immobiles, effets actifs), agrandie x4.
pub fn planche(sortie: &Path, secondes: f32) -> Result<(), String> {
    let (cl, ch) = (120, 64);
    let mut aq = Aquarium::new(cl * 4, ch * 2, 3);
    aq.avec_compteur = false;
    aq.decor = false;
    let mut a = Alea(5);
    for (i, r) in Rarete::TOUTES.iter().enumerate() {
        let graine = 1000 + i as u64 * 17;
        let s = Arc::new(sprite_de(graine, *r, 1.3));
        let x = (i % 4 * cl + cl / 2) as f32;
        let y = (i / 4 * ch + ch / 2 - 4) as f32;
        let mut n = Nageur::new(i as u64 + 1, *r, s, x, y, &mut a);
        n.fixe = true;
        n.gauche = false;
        aq.nageurs.push(n);
    }
    let partie = Partie::nouvelle(0, 1);
    let mut p2 = partie.clone();
    p2.reglages.teinte = 0.0;
    for _ in 0..(secondes * 30.0) as usize {
        aq.avancer(1.0 / 30.0);
    }
    aq.dessiner(&p2);
    let mut t = Toile::new(cl * 4, ch * 2);
    t.rect(
        0,
        0,
        (cl * 4) as i32,
        (ch * 2) as i32,
        [0x12, 0x22, 0x3a],
        255,
    );
    for i in 0..8 {
        let (x, y) = ((i % 4 * cl) as i32, (i / 4 * ch) as i32);
        if (i % 4 + i / 4) % 2 == 0 {
            t.rect(x, y, cl as i32, ch as i32, [0x18, 0x2c, 0x48], 255);
        }
    }
    t.superposer(&aq.toile);
    for (i, r) in Rarete::TOUTES.iter().enumerate() {
        let (x, y) = ((i % 4 * cl) as i32 + 4, (i / 4 * ch) as i32 + ch as i32 - 8);
        t.texte(x, y, r.nom(), r.couleur(), 255, Some([0, 0, 0]));
    }
    creer_dossier(sortie)?;
    image::save_buffer(
        sortie,
        &t.agrandie_rgba(4),
        (t.l * 4) as u32,
        (t.h * 4) as u32,
        image::ColorType::Rgba8,
    )
    .map_err(|e| e.to_string())
}

fn creer_dossier(f: &Path) -> Result<(), String> {
    if let Some(d) = f.parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    Ok(())
}

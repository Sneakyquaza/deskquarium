//! Rendu sans écran (Linux) : capture PNG, vidéo via ffmpeg, planche des raretés, mesures.

use crate::ecrans::Disposition;
use crate::especes::{Espece, Taille};
use crate::fond;
use crate::jeu::{Alea, Partie, Rarete, CAPACITE};
use crate::scene::{Aquarium, Nageur};
use crate::toile::Toile;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

/// Aquarium pré-rempli : au moins un poisson de chaque rareté, et un mélange de créatures
/// remarquables (raie manta, requins, murène, méduse, kraken, léviathan…).
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
    let voulues = [
        "kraken",
        "leviathan",
        "manta",
        "marteau",
        "blanc",
        "murene",
        "meduse",
        "tortue",
        "baudroie",
        "hippocampe",
        "crabe",
        "carpe_comete",
        "dauphin",
        "ichthyosaure",
        "trilobite",
        "anguille",
    ];
    for cle in voulues {
        let Some(e) = crate::especes::par_cle(cle) else {
            continue;
        };
        if p.poissons.iter().any(|f| f.espece == cle && f.actif) {
            continue;
        }
        // Remplace le plus faible pour garder la limite.
        if p.nb_actifs() >= CAPACITE {
            let id = p
                .poissons
                .iter()
                .filter(|f| f.actif && crate::scene::espece_de(f).taille < Taille::Enorme)
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
            rarete: e.rarete,
            taille: 1.2,
            niveau: 3,
            actif: true,
            oeuf: 3,
            espece: cle.to_string(),
            variante: 0,
        });
        p.eclos[e.rarete.index()] += 1;
        *p.vus.entry(cle.to_string()).or_default() += 1;
    }
    // Les autres créatures colossales vont dans la réserve (deux à l'écran au plus).
    let gardees: Vec<u64> = p
        .poissons
        .iter()
        .filter(|f| {
            f.actif
                && crate::scene::espece_de(f).taille == Taille::Colossale
                && f.espece != "kraken"
                && f.espece != "leviathan"
        })
        .map(|f| f.id)
        .collect();
    for id in gardees {
        let _ = p.basculer(id);
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
    /// Écrans simulés (« 1920x1080+0+0,1920x1080+1920+0 ») : remplace `largeur`x`hauteur`.
    pub ecrans: Option<String>,
    /// Mode Abysses.
    pub abysses: bool,
}

type Prepare = (
    Partie,
    Aquarium,
    image::RgbaImage,
    usize,
    (u32, u32),
    Disposition,
);

/// Barres des tâches simulées (`--ecrans …/48`) dessinées par-dessus l'image, comme Windows.
fn barres(img: &mut image::RgbaImage, d: &Disposition) {
    let (bx, by, _, _) = d.boite();
    for e in &d.ecrans {
        let Some((x, y, l, h)) = e.travail else {
            continue;
        };
        for py in (y + h)..(e.y + e.h) {
            for px in x..x + l {
                let (u, v) = ((px - bx) as u32, (py - by) as u32);
                if u < img.width() && v < img.height() {
                    img.put_pixel(u, v, image::Rgba([0x1c, 0x20, 0x2a, 255]));
                }
            }
        }
    }
}

fn preparer(o: &Options) -> Result<Prepare, String> {
    let mut partie = if o.demo {
        partie_demo()
    } else {
        crate::sauve::charger(&crate::sauve::dossier()).0
    };
    partie.reglages.abysses |= o.abysses;
    let d = match &o.ecrans {
        Some(t) => Disposition::depuis_texte(t)?,
        None => Disposition::simple(o.largeur, o.hauteur),
    };
    let (bx, by, bl, bh) = d.boite();
    let (lp, hp) = (bl as u32, bh as u32);
    let e = d.echelle() as usize;
    let mut aq = Aquarium::avec_zones(
        (lp as usize).div_ceil(e),
        (hp as usize).div_ceil(e),
        1,
        &d.zones(e as u32),
        &d.travail(e as u32),
    );
    aq.synchroniser(&partie);
    let image_fond = match &o.fond {
        Some(f) => Some(
            image::open(f)
                .map_err(|e| format!("fond {f} : {e}"))?
                .to_rgba8(),
        ),
        None => None,
    };
    // Chaque écran son fond (le factice retourné un écran sur deux), trous en noir.
    let mut fond = image::RgbaImage::from_pixel(lp, hp, image::Rgba([0, 0, 0, 255]));
    for (i, c) in d.ecrans.iter().enumerate() {
        let (l, h) = (c.l as u32, c.h as u32);
        let mut img = match &image_fond {
            Some(f) => fond::couvrir(f, l, h),
            None => fond::factice(l, h),
        };
        if i % 2 == 1 && image_fond.is_none() {
            img = image::imageops::flip_horizontal(&img);
        }
        image::imageops::replace(&mut fond, &img, (c.x - bx) as i64, (c.y - by) as i64);
    }
    Ok((partie, aq, fond, e, (lp, hp), d))
}

pub fn capture(o: &Options, sortie: &Path) -> Result<(), String> {
    let (mut partie, mut aq, fond, e, _, d) = preparer(o)?;
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
    let mut img = fond::composer(&fond, &aq.toile, e);
    barres(&mut img, &d);
    creer_dossier(sortie)?;
    img.save(sortie).map_err(|e| e.to_string())
}

pub fn video(o: &Options, sortie: &Path) -> Result<(), String> {
    let (mut partie, mut aq, fond, e, (lp, hp), d) = preparer(o)?;
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
        .arg(format!("{lp}x{hp}"))
        .args([
            "-r", "30", "-i", "-", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "24",
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
        let mut img = fond::composer(&fond, &aq.toile, e);
        barres(&mut img, &d);
        entree.write_all(img.as_raw()).map_err(|e| e.to_string())?;
    }
    drop(entree);
    let s = enfant.wait().map_err(|e| e.to_string())?;
    s.success()
        .then_some(())
        .ok_or_else(|| format!("ffmpeg : {s}"))
}

/// Planche des espèces agrandie x4 : une ligne par rareté, chaque espèce (variante de base)
/// immobile avec ses effets, son nom dessous. Les très grandes sont réduites pour tenir dans la
/// case (elles sont à taille réelle sur `planche_geantes`).
pub fn planche(sortie: &Path, secondes: f32) -> Result<(), String> {
    let (cl, ch, gauche) = (124usize, 90usize, 56usize);
    let lignes: Vec<Vec<&Espece>> = Rarete::TOUTES
        .iter()
        .map(|r| crate::especes::de_rarete(*r).collect())
        .collect();
    let n_col = lignes.iter().map(Vec::len).max().unwrap_or(1);
    let (lt, htt) = (gauche + cl * n_col, ch * lignes.len());
    let mut aq = Aquarium::new(lt, htt, 3);
    aq.avec_compteur = false;
    aq.decor = false;
    let mut a = Alea(5);
    let mut id = 1;
    for (j, ligne) in lignes.iter().enumerate() {
        for (i, e) in ligne.iter().enumerate() {
            // Case en pixels monde, sprite en pixels fins.
            let (f, m) = (
                crate::toile::FIN as f32,
                2.0 * crate::poisson::MARGE as f32 + 40.0, // animées : débordent du dessin
            );
            let k = ((cl as f32 - 6.0) * f - m) / e.dims.0 as f32;
            let k = k
                .min(((ch as f32 - 16.0) * f - m) / e.dims.1 as f32)
                .min(1.0);
            let s = Arc::new(crate::poisson::sprite(e, 0, k));
            let x = (gauche + i * cl + cl / 2) as f32;
            let y = (j * ch + (ch - 10) / 2) as f32;
            let mut n = Nageur::new(id, e.rarete, s, x, y, &mut a);
            id += 1;
            n.fixe = true;
            n.gauche = false;
            aq.nageurs.push(n);
        }
    }
    let mut t = rendre_planche(&mut aq, secondes, n_col, lignes.len(), (cl, ch, gauche));
    for (j, ligne) in lignes.iter().enumerate() {
        let r = Rarete::TOUTES[j];
        let y = (j * ch) as i32;
        t.texte(
            4,
            y + ch as i32 / 2 - 3,
            r.nom(),
            r.couleur(),
            255,
            Some([0, 0, 0]),
        );
        for (i, e) in ligne.iter().enumerate() {
            let x = (gauche + i * cl) as i32;
            let nom = e.nom;
            let w = crate::toile::largeur_texte(nom);
            t.texte(
                x + (cl as i32 - w) / 2,
                y + ch as i32 - 9,
                nom,
                [0xe8, 0xf0, 0xf4],
                255,
                Some([0, 0, 0]),
            );
        }
    }
    enregistrer(&t, 4, sortie)
}

/// Créatures énormes et colossales à leur vraie taille, agrandies x2.
pub fn planche_geantes(sortie: &Path, secondes: f32) -> Result<(), String> {
    let geantes: Vec<&Espece> = crate::especes::catalogue()
        .iter()
        .filter(|e| e.taille >= Taille::Enorme)
        .collect();
    planche_a_taille(sortie, secondes, &geantes, 900, 2)
}

/// Toutes les espèces côte à côte, de la plus petite à la plus grande, à leur taille dans
/// l'aquarium en 1920x1080 (toile x4), avec une règle de 100 pixels écran : tailles relatives.
pub fn alignement(sortie: &Path, secondes: f32) -> Result<(), String> {
    let mut toutes: Vec<&Espece> = crate::especes::catalogue().iter().collect();
    toutes.sort_by_key(|e| e.dims.0.max(e.dims.1));
    planche_a_taille(sortie, secondes, &toutes, 480, 4)
}

/// Espèces à leur vraie taille rangées en lignes de `largeur` pixels monde, agrandies x`e`.
fn planche_a_taille(
    sortie: &Path,
    secondes: f32,
    geantes: &[&Espece],
    largeur: usize,
    e: usize,
) -> Result<(), String> {
    let mut places = Vec::new();
    let (mut x, mut y, mut hl) = (0, 0, 0);
    for e in geantes {
        let f = crate::toile::FIN;
        let nom = crate::toile::largeur_texte(&format!("{} - {}", e.nom, e.rarete.nom()));
        let w = (e.dims.0 as usize / f + 16).max(nom as usize + 6);
        let h = e.dims.1 as usize / f + 22;
        if x + w > largeur {
            x = 0;
            y += hl;
            hl = 0;
        }
        places.push((x, y, w, h));
        x += w;
        hl = hl.max(h);
    }
    let ht = y + hl;
    let mut aq = Aquarium::new(largeur, ht, 3);
    aq.avec_compteur = false;
    aq.decor = false;
    let mut a = Alea(8);
    for (i, (e, &(x, y, w, h))) in geantes.iter().zip(&places).enumerate() {
        let s = Arc::new(crate::poisson::sprite(e, 0, 1.0));
        let mut n = Nageur::new(
            i as u64 + 1,
            e.rarete,
            s,
            (x + w / 2) as f32,
            (y + (h - 10) / 2) as f32,
            &mut a,
        );
        n.fixe = true;
        n.gauche = false;
        aq.nageurs.push(n);
    }
    let mut t = Toile::new(largeur, ht);
    t.rect(0, 0, largeur as i32, ht as i32, [0x12, 0x22, 0x3a], 255);
    for _ in 0..(secondes * 30.0) as usize {
        aq.avancer(1.0 / 30.0);
    }
    let mut p2 = Partie::nouvelle(0, 1);
    p2.reglages.teinte = 0.0;
    aq.dessiner(&p2);
    t.superposer(&aq.toile);
    // Règle : 100 pixels écran.
    let r = 100 / e as i32;
    t.rect(2, 2, r, 1, [0xff, 0xff, 0xff], 255);
    t.rect(2, 1, 1, 3, [0xff, 0xff, 0xff], 255);
    t.rect(1 + r, 1, 1, 3, [0xff, 0xff, 0xff], 255);
    t.texte(r + 5, 1, "100 PX", [0xff, 0xff, 0xff], 255, None);
    for (esp, &(x, y, w, h)) in geantes.iter().zip(&places) {
        let e = esp;
        let nom = format!("{} - {}", e.nom, e.rarete.nom());
        let tw = crate::toile::largeur_texte(&nom);
        t.texte(
            (x + w / 2) as i32 - tw / 2,
            (y + h) as i32 - 9,
            &nom,
            e.rarete.couleur(),
            255,
            Some([0, 0, 0]),
        );
    }
    enregistrer(&t, e, sortie)
}

fn rendre_planche(
    aq: &mut Aquarium,
    secondes: f32,
    n_col: usize,
    n_lig: usize,
    (cl, ch, gauche): (usize, usize, usize),
) -> Toile {
    let mut p2 = Partie::nouvelle(0, 1);
    p2.reglages.teinte = 0.0;
    for _ in 0..(secondes * 30.0) as usize {
        aq.avancer(1.0 / 30.0);
    }
    aq.dessiner(&p2);
    let (lt, ht) = (gauche + cl * n_col, ch * n_lig);
    let mut t = Toile::new(lt, ht);
    t.rect(0, 0, lt as i32, ht as i32, [0x12, 0x22, 0x3a], 255);
    for j in 0..n_lig {
        for i in 0..n_col {
            if (i + j) % 2 == 0 {
                let (x, y) = ((gauche + i * cl) as i32, (j * ch) as i32);
                t.rect(x, y, cl as i32, ch as i32, [0x18, 0x2c, 0x48], 255);
            }
        }
    }
    t.superposer(&aq.toile);
    t
}

/// Images d'animation des espèces `cles` (variante de base, sans effets), une ligne par
/// espèce, agrandies x`e` : pour relire la déformation image par image.
pub fn planche_images(sortie: &Path, cles: &[&str], e: usize) -> Result<(), String> {
    let mut lignes = Vec::new();
    for cle in cles {
        let esp = crate::especes::par_cle(cle).ok_or(format!("espèce inconnue : {cle}"))?;
        lignes.push((esp.nom, crate::poisson::sprite(esp, 0, 1.0)));
    }
    let n_col = lignes
        .iter()
        .map(|l| l.1.images.len())
        .max()
        .unwrap_or(1)
        .min(8);
    // Cases en pixels monde ; le sprite y est posé au pixel fin.
    let f = crate::toile::FIN as i32;
    let case = |s: &crate::poisson::Sprite| ((s.l + f - 1) / f + 1, (s.h + f - 1) / f + 1);
    let (mut lt, mut ht) = (0usize, 0usize);
    for (_, s) in &lignes {
        let (cl, ch) = case(s);
        lt = lt.max(n_col * cl as usize);
        ht += s.images.len().div_ceil(n_col) * ch as usize + 9;
    }
    let mut t = Toile::new(lt, ht);
    t.rect(0, 0, lt as i32, ht as i32, [0x12, 0x22, 0x3a], 255);
    let mut y0 = 0i32;
    for (nom, s) in &lignes {
        let (cl, ch) = case(s);
        t.texte(2, y0 + 2, nom, [0xe8, 0xf0, 0xf4], 255, Some([0, 0, 0]));
        y0 += 9;
        for (k, im) in s.images.iter().enumerate() {
            let (cx, cy) = ((k % n_col) as i32 * cl, y0 + (k / n_col) as i32 * ch);
            if (k + k / n_col) % 2 == 0 {
                t.rect(cx, cy, cl, ch, [0x18, 0x2c, 0x48], 255);
            }
            for y in 0..s.h {
                for x in 0..s.l {
                    let p = im.px((y * s.l + x) as usize);
                    if p[3] > 0 {
                        t.poser_fin(cx * f + x, cy * f + y, [p[0], p[1], p[2]], p[3]);
                    }
                }
            }
        }
        y0 += s.images.len().div_ceil(n_col) as i32 * ch;
    }
    enregistrer(&t, e, sortie)
}

fn enregistrer(t: &Toile, e: usize, sortie: &Path) -> Result<(), String> {
    creer_dossier(sortie)?;
    image::save_buffer(
        sortie,
        &t.agrandie_rgba(e),
        (t.l * e) as u32,
        (t.h * e) as u32,
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

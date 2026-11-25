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
            shiny: false,
            modif: None,
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
    // v0.8 : quelques shiny et modificateurs à l'écran.
    use crate::jeu::Modif;
    for (cle, shiny, modif) in [
        ("blanc", true, None),
        ("manta", false, Some(Modif::Galaxie)),
        ("tortue", false, Some(Modif::Dore)),
        ("meduse", false, Some(Modif::Cristal)),
        ("hippocampe", false, Some(Modif::ArcEnCiel)),
        ("dauphin", false, Some(Modif::Electrique)),
        ("murene", false, Some(Modif::Ombre)),
        ("carpe_comete", true, Some(Modif::Diamant)),
    ] {
        if let Some(f) = p.poissons.iter_mut().find(|f| f.espece == cle && f.actif) {
            (f.shiny, f.modif) = (shiny, modif);
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
    /// Écrans simulés (« 1920x1080+0+0,1920x1080+1920+0 ») : remplace `largeur`x`hauteur`.
    pub ecrans: Option<String>,
    /// Mode Abysses.
    pub abysses: bool,
    /// v0.9 : décor à thème (index de `decor::THEMES`).
    pub theme: Option<u8>,
    /// v0.9 : objets de décor posés (clés), répartis sur l'écran principal.
    pub objets: Vec<String>,
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
    if let Some(t) = o.theme {
        partie.reglages.theme = t;
    }
    for (i, cle) in o.objets.iter().enumerate() {
        *partie.decor_achete.entry(cle.clone()).or_default() += 1;
        let x = (i as f32 + 0.5) / o.objets.len() as f32;
        partie
            .poser_objet(cle, 0, x)
            .map_err(|_| format!("objet de décor inconnu : {cle}"))?;
    }
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
    // 30 s d'avance : bulles, particules et nageurs déjà répartis (comme un fond allumé).
    for _ in 0..30 * 30 {
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

/// Planche des formes : une ligne par espèce ; colonnes Normal + Shiny (`modifs` faux) ou
/// Normal + les 7 modificateurs (`modifs` vrai), animées `secondes` (auras et particules).
pub fn planche_formes(
    sortie: &Path,
    cles: &[&str],
    modifs: bool,
    secondes: f32,
) -> Result<(), String> {
    use crate::jeu::{Forme, Modif};
    let colonnes: Vec<(String, Forme)> = if modifs {
        std::iter::once(("Normal".to_string(), Forme::default()))
            .chain(Modif::TOUS.iter().map(|&m| {
                (
                    m.nom().to_string(),
                    Forme {
                        shiny: false,
                        modif: Some(m),
                    },
                )
            }))
            .collect()
    } else {
        vec![
            ("Normal".to_string(), Forme::default()),
            (
                "Shiny".to_string(),
                Forme {
                    shiny: true,
                    modif: None,
                },
            ),
        ]
    };
    let especes: Vec<&Espece> = cles
        .iter()
        .map(|c| crate::especes::par_cle(c).ok_or(format!("espèce inconnue : {c}")))
        .collect::<Result<_, _>>()?;
    let lmax = especes.iter().map(|e| e.dims.0).max().unwrap_or(40);
    let hmax = especes.iter().map(|e| e.dims.1).max().unwrap_or(30);
    let f = crate::toile::FIN;
    // Cases à la taille réelle du plus grand dessin (jamais réduit) + place pour l'aura.
    let (cl, ch, gauche) = (lmax as usize / f + 30, hmax as usize / f + 34, 0usize);
    let (lt, ht) = (cl * colonnes.len(), ch * especes.len());
    let mut aq = Aquarium::new(lt, ht, 3);
    aq.avec_compteur = false;
    aq.decor = false;
    let mut a = Alea(5);
    let mut id = 1;
    for (j, e) in especes.iter().enumerate() {
        for (i, (_, forme)) in colonnes.iter().enumerate() {
            let s = Arc::new(crate::poisson::sprite_forme(e, 0, 1.0, *forme));
            let x = (i * cl + cl / 2) as f32;
            let y = (j * ch + ch / 2) as f32;
            let mut n = Nageur::new(id, e.rarete, s, x, y, &mut a);
            id += 1;
            n.fixe = true;
            n.gauche = false;
            n.forme = *forme;
            aq.nageurs.push(n);
        }
    }
    // Noms des colonnes en haut de la première ligne, noms des espèces en bas de chaque ligne.
    let mut t = rendre_planche(
        &mut aq,
        secondes,
        colonnes.len(),
        especes.len(),
        (cl, ch, gauche),
    );
    for (i, (nom, _)) in colonnes.iter().enumerate() {
        let w = crate::toile::largeur_texte(nom);
        t.texte(
            (i * cl) as i32 + (cl as i32 - w) / 2,
            2,
            nom,
            [0xff, 0xea, 0xa8],
            255,
            None,
        );
    }
    for (j, e) in especes.iter().enumerate() {
        t.texte(
            2,
            (j * ch + ch - 8) as i32,
            e.nom,
            [0xe8, 0xf0, 0xf4],
            255,
            Some([0, 0, 0]),
        );
    }
    enregistrer(&t, 2, sortie)
}

/// Demi-tour d'une espèce image par image (une case toutes les 0,1 s), x4 : la créature nage
/// vers la droite puis repart vers la gauche.
pub fn planche_demi_tour(sortie: &Path, cle: &str) -> Result<(), String> {
    let e = crate::especes::par_cle(cle).ok_or(format!("espèce inconnue : {cle}"))?;
    let s = Arc::new(crate::poisson::sprite(e, 0, 1.0));
    let f = crate::toile::FIN as f32;
    let (cw, chh) = ((s.lw() * 1.5) as i32 + 8, (s.hw() * 1.4) as i32 + 8);
    let (l, h) = (cw as usize * 6, chh as usize + 40);
    let mut aq = Aquarium::new(l, h, 3);
    aq.avec_compteur = false;
    aq.decor = false;
    let mut a = Alea(9);
    let mut n = Nageur::new(1, e.rarete, s, l as f32 * 0.4, h as f32 / 2.0, &mut a);
    n.gauche = false;
    n.forcer_cible((l as f32 - 4.0, h as f32 / 2.0));
    aq.nageurs.push(n);
    let p = Partie::nouvelle(0, 1);
    let mut cases: Vec<Toile> = Vec::new();
    let long = aq.nageurs[0].corps_long();
    // Repli : les 6 images d'avant le demi-tour puis la suite (le repli dure ~5 images). Corps
    // long (jamais replié) : une image sur 6 pendant 8 s (il sort de l'écran et revient).
    let mut vu_tour = false;
    for k in 0..1200 {
        if k == 45 {
            aq.nageurs[0].forcer_cible((4.0, h as f32 / 2.0));
        }
        aq.avancer(1.0 / 30.0);
        if k < 45 || cases.len() >= 40 || (long && k % 6 != 0) {
            continue;
        }
        vu_tour |= aq.nageurs[0].en_demi_tour();
        aq.dessiner(&p);
        let cx = aq.nageurs[0].x;
        let mut c = Toile::new(cw as usize, chh as usize);
        c.rect(0, 0, cw, chh, [0x18, 0x2c, 0x48], 255);
        let (x0, y0) = (
            ((cx - cw as f32 / 2.0) * f) as i32,
            ((h as f32 - chh as f32) / 2.0 * f) as i32,
        );
        for y in 0..c.hf() as i32 {
            for x in 0..c.lf() as i32 {
                let (sx, sy) = (x0 + x, y0 + y);
                if sx >= 0
                    && sy >= 0
                    && (sx as usize) < aq.toile.lf()
                    && (sy as usize) < aq.toile.hf()
                {
                    let px = aq.toile.px[sy as usize * aq.toile.lf() + sx as usize];
                    if px[3] > 0 {
                        let d = crate::toile::demultiplier(px);
                        c.poser_fin(x, y, [d[0], d[1], d[2]], d[3]);
                    }
                }
            }
        }
        cases.push(c);
        if !long && !vu_tour && cases.len() > 6 {
            cases.remove(0);
        }
    }
    let n_col = 10;
    let lignes = cases.len().div_ceil(n_col);
    let mut t = Toile::new(cw as usize * n_col + n_col, (chh as usize + 8) * lignes);
    t.rect(0, 0, t.l as i32, t.h as i32, [0x0a, 0x14, 0x24], 255);
    for (i, c) in cases.iter().enumerate() {
        let (x, y) = (
            (i % n_col) as i32 * (cw + 1),
            (i / n_col) as i32 * (chh + 8),
        );
        for yy in 0..c.hf() as i32 {
            for xx in 0..c.lf() as i32 {
                let d = crate::toile::demultiplier(c.px[yy as usize * c.lf() + xx as usize]);
                t.poser_fin(x * 2 + xx, (y + 7) * 2 + yy, [d[0], d[1], d[2]], 255);
            }
        }
        t.texte(
            x + 2,
            y + 1,
            &format!("{}", i + 1),
            [0xff, 0xea, 0xa8],
            255,
            None,
        );
    }
    enregistrer(&t, 4, sortie)
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

/// Planche de la souris (v0.9) : curiosité (curseur immobile), fuite (coup rapide), poisson
/// attrapé qui suit le curseur puis lâché, fiche après un clic. Recadrages à taille écran, le
/// curseur dessiné en flèche blanche.
pub fn planche_souris(o: &Options, sortie: &Path) -> Result<(), String> {
    let (mut partie, mut aq, fond, e, _, _) = preparer(o)?;
    let dt = 1.0 / 30.0;
    let mut cases: Vec<(String, image::RgbaImage)> = Vec::new();
    fn pas(aq: &mut Aquarium, partie: &mut Partie, c: Option<(f32, f32)>, dt: f32) {
        partie.avancer(dt as f64);
        aq.curseur(c, dt);
        aq.avancer(dt);
    }
    for _ in 0..30 * 30 {
        pas(&mut aq, &mut partie, None, dt);
    }
    let (lw, hw) = (aq.toile.l as f32, aq.toile.h as f32);
    // Poisson moyen ou petit le plus près du centre.
    let cible = |aq: &Aquarium| {
        aq.nageurs
            .iter()
            .filter(|n| n.sprite.taille <= Taille::Moyenne && n.sprite.taille.profondeur() == 0.0)
            .min_by(|a, b| {
                let d = |n: &Nageur| (n.x - lw * 0.45).hypot(n.y - hw * 0.45);
                d(a).total_cmp(&d(b))
            })
            .map(|n| (n.id, n.x, n.y))
    };
    let mut cliche = |aq: &mut Aquarium, partie: &Partie, nom: &str, c: (f32, f32)| {
        aq.dessiner(partie);
        let img = fond::composer(&fond, &aq.toile, e);
        let (cl, ch) = (170 * e as i32, 100 * e as i32);
        let x0 = ((c.0 * e as f32) as i32 - cl / 2).clamp(0, img.width() as i32 - cl);
        let y0 = ((c.1 * e as f32) as i32 - ch / 2).clamp(0, img.height() as i32 - ch);
        let mut r =
            image::imageops::crop_imm(&img, x0 as u32, y0 as u32, cl as u32, ch as u32).to_image();
        // Flèche du curseur (pointe en haut à gauche), contour noir.
        let (px, py) = ((c.0 * e as f32) as i32 - x0, (c.1 * e as f32) as i32 - y0);
        for y in 0..22 {
            for x in 0..=y.min(14) {
                let bord = x == 0 || x == y.min(14) || y == 21;
                let (u, v) = (px + x, py + y);
                if u >= 0 && v >= 0 && (u as u32) < r.width() && (v as u32) < r.height() {
                    let c = if bord {
                        [0, 0, 0, 255]
                    } else {
                        [255, 255, 255, 255]
                    };
                    r.put_pixel(u as u32, v as u32, image::Rgba(c));
                }
            }
        }
        cases.push((nom.to_string(), r));
    };
    // Curiosité et fuite : là où il y a le plus de petits poissons ensemble.
    let petits: Vec<(f32, f32)> = aq
        .nageurs
        .iter()
        .filter(|n| n.sprite.taille <= Taille::Moyenne && n.sprite.taille.profondeur() == 0.0)
        .map(|n| (n.x, n.y))
        .collect();
    let Some(&(x, y)) = petits.iter().max_by_key(|a| {
        petits
            .iter()
            .filter(|b| (a.0 - b.0).hypot(a.1 - b.1) < 70.0)
            .count()
    }) else {
        return Err("aucun poisson à montrer".into());
    };
    // 1. Curseur immobile près d'un poisson : les curieux s'approchent.
    let c = (x + 30.0, y - 6.0);
    cliche(&mut aq, &partie, "1 CURSEUR IMMOBILE 0 S", c);
    for k in 0..90 {
        pas(&mut aq, &mut partie, Some(c), dt);
        if k == 44 {
            cliche(&mut aq, &partie, "2 CURIEUX 1,5 S", c);
        }
    }
    cliche(&mut aq, &partie, "3 CURIEUX 3 S", c);
    // 2. Coup de souris rapide à travers eux : ils fuient.
    for k in 0..9 {
        let cc = (c.0 - 70.0 + k as f32 * 17.5, c.1);
        pas(&mut aq, &mut partie, Some(cc), dt);
    }
    let c2 = (c.0 + 70.0, c.1);
    cliche(&mut aq, &partie, "4 COUP RAPIDE : FUITE", c);
    for _ in 0..15 {
        pas(&mut aq, &mut partie, Some(c2), dt);
    }
    cliche(&mut aq, &partie, "5 FUITE 0,5 S", c);
    for _ in 0..60 {
        pas(&mut aq, &mut partie, None, dt);
    }
    // 3. Appui maintenu sur un poisson : attrapé, il suit le curseur en gigotant.
    let Some((id, x, y)) = cible(&aq) else {
        return Err("aucun poisson à attraper".into());
    };
    aq.appuyer(id);
    let mut c = (x, y);
    for _ in 0..8 {
        pas(&mut aq, &mut partie, Some(c), dt);
    }
    cliche(&mut aq, &partie, "6 ATTRAPÉ APRÈS 0,25 S", c);
    for k in 0..30 {
        c = (x + k as f32 * 2.5, y - k as f32 * 1.2);
        pas(&mut aq, &mut partie, Some(c), dt);
        if k == 14 {
            cliche(&mut aq, &partie, "7 TENU, IL SUIT", c);
        }
    }
    cliche(&mut aq, &partie, "8 TENU 1 S", c);
    aq.lacher();
    for _ in 0..30 {
        pas(&mut aq, &mut partie, Some(c), dt);
    }
    cliche(&mut aq, &partie, "9 LÂCHÉ, IL REPART", c);
    // 4. Clic bref : la fiche du poisson.
    if let Some((id, x, y)) = cible(&aq) {
        aq.cliquer(id);
        for _ in 0..10 {
            pas(&mut aq, &mut partie, Some((x, y)), dt);
        }
        // Recadré un peu au-dessus du poisson : la fiche est au-dessus de lui.
        let n = aq
            .nageurs
            .iter()
            .find(|n| n.id == id)
            .map_or((x, y), |n| (n.x, n.y - 20.0));
        cliche(&mut aq, &partie, "10 CLIC : FICHE", n);
    }
    // Planche : 3 colonnes, titre au-dessus de chaque case.
    let (cl, ch) = (cases[0].1.width(), cases[0].1.height());
    let bande = 12 * e as u32;
    let n_col = 3;
    let lignes = cases.len().div_ceil(n_col) as u32;
    let mut out = image::RgbaImage::from_pixel(
        cl * n_col as u32 + 8 * (n_col as u32 - 1),
        (ch + bande + 8) * lignes,
        image::Rgba([0x0a, 0x14, 0x24, 255]),
    );
    for (i, (nom, r)) in cases.iter().enumerate() {
        let (x, y) = (
            (i % n_col) as u32 * (cl + 8),
            (i / n_col) as u32 * (ch + bande + 8),
        );
        let mut t = Toile::new((cl as usize).div_ceil(e), 12);
        t.texte(2, 3, nom, [0xff, 0xea, 0xa8], 255, None);
        let fond_titre =
            image::RgbaImage::from_pixel(cl, bande, image::Rgba([0x0a, 0x14, 0x24, 255]));
        let titre = fond::composer(&fond_titre, &t, e);
        image::imageops::replace(&mut out, &titre, x as i64, y as i64);
        image::imageops::replace(&mut out, r, x as i64, (y + bande) as i64);
    }
    creer_dossier(sortie)?;
    out.save(sortie).map_err(|e| e.to_string())
}

/// Éclosion (v0.10) sans écran : vidéo mp4 (60 i/s, ×3) et planche d'images (toutes les
/// 0,1 s) d'un œuf `oeuf` qui donne l'espèce `cle` (forme imposée, œuf doré si `dore`).
pub fn eclosion(
    sortie: &Path,
    oeuf: usize,
    cle: &str,
    forme: crate::jeu::Forme,
    dore: bool,
) -> Result<(), String> {
    let e = crate::especes::par_cle(cle).ok_or(format!("espèce inconnue : {cle}"))?;
    let f = crate::jeu::Poisson {
        id: 77,
        graine: 77,
        rarete: e.rarete,
        taille: 1.0,
        niveau: 1,
        actif: true,
        oeuf: oeuf as u8,
        espece: cle.to_string(),
        variante: 0,
        shiny: forme.shiny,
        modif: forme.modif,
    };
    let mut ecl = crate::eclosion::Eclosion::nouvelle(
        &f,
        oeuf.min(crate::jeu::OEUFS.len() - 1),
        0.0,
        dore,
        false,
    );
    let duree = ecl.suspense() + 2.6;
    println!(
        "temps forts : {:?} ; éclatement {:.2}",
        ecl.temps_forts(),
        ecl.suspense()
    );
    let k = 3usize;
    let (l, h) = (ecl.toile.lf() * k, ecl.toile.hf() * k);
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
        .arg(format!("{l}x{h}"))
        .args([
            "-r", "60", "-i", "-", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "18",
        ])
        .arg(sortie.with_extension("mp4"))
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("{ffmpeg} : {e} (AQUARIUM_FFMPEG pour donner le chemin)"))?;
    let mut entree = enfant.stdin.take().ok_or("ffmpeg sans entrée")?;
    let mut cases = Vec::new();
    let mut sons = Vec::new();
    let n = (duree * 60.0) as usize;
    for i in 0..n {
        let t = i as f64 / 60.0;
        ecl.dessiner(t);
        sons.extend(ecl.sons.drain(..).map(|(s, h)| (t, s, h)));
        let img = ecl.toile.agrandie_rgba(k * crate::toile::FIN);
        entree.write_all(&img).map_err(|e| e.to_string())?;
        if i % 6 == 0 {
            cases.push((t, ecl.toile.clone()));
        }
    }
    drop(entree);
    let s = enfant.wait().map_err(|e| e.to_string())?;
    if !s.success() {
        return Err(format!("ffmpeg : {s}"));
    }
    ajouter_son(&ffmpeg, &sortie.with_extension("mp4"), &sons, duree)?;
    // Planche : 8 colonnes, temps au-dessus de chaque image.
    let (cl, ch) = (128i32, 96i32);
    let n_col = 8;
    let lignes = cases.len().div_ceil(n_col) as i32;
    let mut p = Toile::new(
        ((cl + 2) * n_col as i32) as usize,
        ((ch + 9) * lignes) as usize,
    );
    p.rect(0, 0, p.l as i32, p.h as i32, [0x06, 0x0c, 0x14], 255);
    for (i, (t, c)) in cases.iter().enumerate() {
        let (x, y) = ((i % n_col) as i32 * (cl + 2), (i / n_col) as i32 * (ch + 9));
        p.texte(
            x + 1,
            y + 1,
            &format!("{t:.1}"),
            [0xff, 0xea, 0xa8],
            255,
            None,
        );
        for yy in 0..c.hf() as i32 {
            for xx in 0..c.lf() as i32 {
                let d = crate::toile::demultiplier(c.px[(yy * c.lf() as i32 + xx) as usize]);
                p.poser_fin(x * 2 + xx, (y + 8) * 2 + yy, [d[0], d[1], d[2]], 255);
            }
        }
    }
    enregistrer(&p, 2, &sortie.with_extension("png"))
}

/// Met la piste son (sons de l'Écloserie mixés hors ligne, `son::rendre_wav`) sur la vidéo.
pub fn ajouter_son(
    ffmpeg: &str,
    video: &Path,
    sons: &[(f64, crate::son::Son, f32)],
    duree: f64,
) -> Result<(), String> {
    let wav = video.with_extension("wav");
    crate::son::rendre_wav(sons, duree, 0.8, &wav).map_err(|e| e.to_string())?;
    let tmp = video.with_extension("son.mp4");
    let s = std::process::Command::new(ffmpeg)
        .args(["-y", "-loglevel", "error", "-i"])
        .arg(video)
        .arg("-i")
        .arg(&wav)
        .args(["-c:v", "copy", "-c:a", "aac", "-b:a", "128k", "-shortest"])
        .arg(&tmp)
        .status()
        .map_err(|e| e.to_string())?;
    if !s.success() {
        return Err(format!("ffmpeg (son) : {s}"));
    }
    std::fs::rename(&tmp, video).map_err(|e| e.to_string())?;
    std::fs::rename(&wav, video.with_extension("son.wav")).map_err(|e| e.to_string())
}

/// Planche des œufs (v0.10) : chaque œuf (une ligne) à 6 instants de son animation au repos.
pub fn planche_oeufs(sortie: &Path) -> Result<(), String> {
    let n = crate::jeu::OEUFS.len();
    let (cl, ch) = (
        crate::eclosion::VIGNETTE.0 as i32,
        crate::eclosion::VIGNETTE.1 as i32,
    );
    let mut p = Toile::new((cl * 6 + 70) as usize, ((ch + 2) * n as i32) as usize);
    p.rect(0, 0, p.l as i32, p.h as i32, [0x10, 0x32, 0x4a], 255);
    for i in 0..n {
        let y = i as i32 * (ch + 2);
        p.texte(
            2,
            y + ch / 2,
            crate::jeu::OEUFS[i].nom,
            [0xe8, 0xf0, 0xf4],
            255,
            None,
        );
        for k in 0..6 {
            let v = crate::eclosion::vignette(i, k as f32 * 0.45);
            let x = 70 + k * cl;
            for yy in 0..v.hf() as i32 {
                for xx in 0..v.lf() as i32 {
                    let d = crate::toile::demultiplier(v.px[(yy * v.lf() as i32 + xx) as usize]);
                    if d[3] > 0 {
                        p.poser_fin(x * 2 + xx, y * 2 + yy, [d[0], d[1], d[2]], d[3]);
                    }
                }
            }
        }
    }
    enregistrer(&p, 6, sortie)
}

/// Vidéo mp4 60 i/s (RGBA brut → ffmpeg H.264) de `n` images de `l` x `h` pixels, chaque
/// image donnée par `image(i)` ; renvoie le chemin d'ffmpeg (pour le son).
fn filmer(
    sortie: &Path,
    (l, h): (usize, usize),
    n: usize,
    mut image: impl FnMut(usize) -> Vec<u8>,
) -> Result<String, String> {
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
        .arg(format!("{l}x{h}"))
        .args([
            "-r", "60", "-i", "-", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "18",
        ])
        .arg(sortie.with_extension("mp4"))
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("{ffmpeg} : {e} (AQUARIUM_FFMPEG pour donner le chemin)"))?;
    let mut entree = enfant.stdin.take().ok_or("ffmpeg sans entrée")?;
    for i in 0..n {
        entree.write_all(&image(i)).map_err(|e| e.to_string())?;
    }
    drop(entree);
    let s = enfant.wait().map_err(|e| e.to_string())?;
    if !s.success() {
        return Err(format!("ffmpeg : {s}"));
    }
    Ok(ffmpeg)
}

/// Ouverture ×`n` (v0.11) sans écran : une vraie partie achète un lot de l'œuf `oeuf`
/// (graine `graine`), la grille est filmée à 60 i/s avec ses sons.
pub fn multi(sortie: &Path, oeuf: usize, n: u32, graine: u64) -> Result<(), String> {
    let mut p = Partie::nouvelle(0, graine);
    p.argent = 1e300;
    let quand = crate::jeu::OEUFS
        .get(oeuf)
        .ok_or("œuf inconnu")?
        .dispo
        .map_or(0, |(_, debut, _)| debut + 1);
    let v = p
        .acheter_lot(oeuf, n, quand)
        .map_err(|e| format!("{e:?}"))?;
    let poissons: Vec<_> = v
        .iter()
        .filter_map(|o| {
            let f = p.poissons.iter().find(|f| f.id == o.id)?.clone();
            Some((f, o.dore, o.pitie))
        })
        .collect();
    for (f, _, _) in &poissons {
        println!(
            "{} {:?} shiny {} {:?}",
            f.espece, f.rarete, f.shiny, f.modif
        );
    }
    let fin = crate::eclosion::multi::Multi::nouvelle(&poissons, oeuf, 0.0).fin() as f64 + 1.6;
    let mut m = crate::eclosion::multi::Multi::nouvelle(&poissons, oeuf, 0.0);
    let k = 3usize;
    let (l, h) = (m.toile.lf() * k, m.toile.hf() * k);
    let nb = (fin * 60.0) as usize;
    let mut sons = Vec::new();
    let ffmpeg = filmer(sortie, (l, h), nb, |i| {
        let t = i as f64 / 60.0;
        m.dessiner(t);
        sons.extend(m.sons.drain(..).map(|(s, hh)| (t, s, hh)));
        m.toile.agrandie_rgba(k * crate::toile::FIN)
    })?;
    ajouter_son(&ffmpeg, &sortie.with_extension("mp4"), &sons, fin)
}

/// Les 8 œufs au repos côte à côte (v0.11), `secondes` de vidéo à 60 i/s (×3).
pub fn repos_oeufs(sortie: &Path, secondes: f64) -> Result<(), String> {
    let n = crate::jeu::OEUFS.len();
    let (vl, vh) = crate::eclosion::VIGNETTE;
    let (cols, k) = (4usize, 3usize);
    let mut t = Toile::new(vl * cols, vh * n.div_ceil(cols));
    let (l, h) = (t.lf() * k, t.hf() * k);
    let nb = (secondes * 60.0) as usize;
    let ffmpeg = filmer(sortie, (l, h), nb, |i| {
        let temps = i as f32 / 60.0;
        t.rect(0, 0, t.l as i32, t.h as i32, [0x10, 0x32, 0x4a], 255);
        for o in 0..n {
            let v = crate::eclosion::vignette(o, temps);
            let (x, y) = (((o % cols) * vl * 2) as i32, ((o / cols) * vh * 2) as i32);
            for yy in 0..v.hf() as i32 {
                for xx in 0..v.lf() as i32 {
                    let d = crate::toile::demultiplier(v.px[(yy * v.lf() as i32 + xx) as usize]);
                    if d[3] > 0 {
                        t.poser_fin(x + xx, y + yy, [d[0], d[1], d[2]], d[3]);
                    }
                }
            }
        }
        t.agrandie_rgba(k * crate::toile::FIN)
    })?;
    let _ = ffmpeg;
    Ok(())
}

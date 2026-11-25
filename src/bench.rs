//! `--bench` (v0.12) : chaîne du fond d'écran sans écran sur deux écrans 1920x1080 simulés
//! (3840x1080) avec 40 poissons : simulation, dessin de la toile et envoi à l'écran (copie
//! GDI simulée sous Linux), moyenne et p95 en ms par image, mémoire en pointe.
//! « Ancien » = ce que faisait la v0.11 à chaque image (texture egui, toile convertie en BGRA,
//! tampon plein écran, fond recopié, toile agrandie et mêlée sur tout l'écran, copie entière).

use crate::capture::{preparer, Options};
use std::time::Instant;

/// Temps processeur du fil (ms) : la mesure ne compte pas le temps où d'autres programmes
/// ont la main (Linux) ; ailleurs, l'horloge murale.
fn cpu_ms() -> f64 {
    #[cfg(target_os = "linux")]
    {
        #[repr(C)]
        struct Timespec {
            s: i64,
            ns: i64,
        }
        unsafe extern "C" {
            fn clock_gettime(horloge: i32, t: *mut Timespec) -> i32;
        }
        let mut t = Timespec { s: 0, ns: 0 };
        // 3 = CLOCK_THREAD_CPUTIME_ID.
        if unsafe { clock_gettime(3, &mut t) } == 0 {
            return t.s as f64 * 1000.0 + t.ns as f64 / 1e6;
        }
    }
    static DEBUT: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    DEBUT.get_or_init(Instant::now).elapsed().as_secs_f64() * 1000.0
}

/// Durées d'une étape (ms par image).
#[derive(Default)]
struct Mesure(Vec<f64>);

impl Mesure {
    fn ajouter(&mut self, t: f64) -> f64 {
        let n = cpu_ms();
        self.0.push(n - t);
        n
    }

    fn texte(&self) -> String {
        let mut v = self.0.clone();
        v.sort_by(f64::total_cmp);
        let moy = v.iter().sum::<f64>() / v.len().max(1) as f64;
        let p95 = v.get(v.len() * 95 / 100).copied().unwrap_or(0.0);
        format!("moyenne {moy:6.2} ms, p95 {p95:6.2} ms")
    }

    fn moyenne(&self) -> f64 {
        self.0.iter().sum::<f64>() / self.0.len().max(1) as f64
    }
}

/// Mémoire en pointe et actuelle (Linux : /proc/self/status), en Mo. La mémoire libérée est
/// d'abord rendue au système (glibc la garde sinon), comme le fait le tas de Windows.
pub fn memoire() -> Option<(f64, f64)> {
    #[cfg(target_os = "linux")]
    {
        unsafe extern "C" {
            fn malloc_trim(garde: usize) -> i32;
        }
        unsafe { malloc_trim(0) };
    }
    let s = std::fs::read_to_string("/proc/self/status").ok()?;
    let lire = |cle: &str| -> Option<f64> {
        let l = s.lines().find(|l| l.starts_with(cle))?;
        let kb: f64 = l.split_whitespace().nth(1)?.parse().ok()?;
        Some(kb / 1024.0)
    };
    Some((lire("VmHWM:")?, lire("VmRSS:")?))
}

pub fn bench(secondes: f32, ips: u32, ancien: bool) -> Result<(), String> {
    let o = Options {
        largeur: 3840,
        hauteur: 1080,
        secondes,
        demo: true,
        fond: None,
        ecrans: Some("1920x1080+0+0/48,1920x1080+1920+0/48".into()),
        abysses: false,
        theme: None,
        objets: Vec::new(),
        nourrir: Vec::new(),
    };
    let debut = Instant::now();
    let (mut partie, mut aq, fond, e, (lp, hp), _) = preparer(&o)?;
    // 40 poissons à l'écran (aquarium plein).
    partie.argent = 1e30;
    while partie.nb_actifs() < crate::jeu::CAPACITE {
        if partie.acheter_oeuf(1).is_err() {
            break;
        }
    }
    aq.synchroniser(&partie);
    println!(
        "bench : {lp}x{hp} (toile {}x{} x{e}), {} poissons, {} dessinés, {ips} i/s, {secondes} s, \
         préparation {:.0} ms",
        aq.toile.l,
        aq.toile.h,
        partie.nb_actifs(),
        aq.nageurs.len(),
        debut.elapsed().as_secs_f64() * 1000.0
    );
    let (o, ns) = aq.octets_sprites();
    println!("sprites    : {ns} cuits, {:.1} Mo", o as f64 / 1e6);
    // Fond d'écran comme le garde le calque (BGRA) ; l'image RGBA n'est gardée que pour
    // l'ancienne chaîne.
    let fond_u32: Vec<u32> = fond
        .pixels()
        .map(|c| 0xff00_0000 | (c[0] as u32) << 16 | (c[1] as u32) << 8 | c[2] as u32)
        .collect();
    let fond_bgra: Vec<u8> = if ancien {
        fond.pixels()
            .flat_map(|c| [c[2], c[1], c[0], 255])
            .collect()
    } else {
        Vec::new()
    };
    drop(fond);
    let dt = 1.0 / ips.max(1) as f32;
    let n = (secondes / dt) as usize;
    let (mut sim, mut des, mut env) = (Mesure::default(), Mesure::default(), Mesure::default());
    let mut env_ancien = Mesure::default();
    let mut compo = crate::compo::Compo::default();
    let (mut morceau, mut copie) = (Vec::new(), Vec::new());
    let (mut aire, mut nb_rect) = (0f64, 0usize);
    let mut empreinte = 0xcbf2_9ce4_8422_2325u64;
    let (l, h) = (lp as usize, hp as usize);
    let mut ecran_ancien: Vec<u8> = Vec::new();
    let mut toile_bgra: Vec<u8> = Vec::new();
    for i in 0..n {
        let t = cpu_ms();
        partie.avancer(dt as f64);
        aq.synchroniser(&partie);
        aq.avancer(dt);
        let t = sim.ajouter(t);
        aq.dessiner(&partie);
        let t = des.ajouter(t);
        // v0.12 : cases changées seulement, composées puis copiées (copie GDI simulée).
        compo.recevoir(&aq.toile, (e / crate::toile::FIN).max(1));
        for r in compo.rectangles(lp as i32, hp as i32, &[]) {
            compo.composer(&fond_u32, l, h, r, &mut morceau);
            copie.clear();
            copie.extend_from_slice(&morceau);
            std::hint::black_box(&copie);
            aire += (r.l * r.h) as f64;
            nb_rect += 1;
        }
        let t = env.ajouter(t);
        if ancien {
            // v0.11 : texture egui de la toile (téléversement GL non compté).
            let img = crate::ecloserie::image_couleur(&aq.toile);
            std::hint::black_box(&img);
            drop(img);
            // presenter : toile en BGRA.
            toile_bgra.clear();
            toile_bgra.extend(aq.toile.px.iter().flat_map(|c| [c[2], c[1], c[0], c[3]]));
            // peindre : tampon plein écran neuf, fond recopié, toile agrandie mêlée, copie.
            let mut tampon = vec![0u8; l * h * 4];
            tampon.copy_from_slice(&fond_bgra);
            let k = (e / crate::toile::FIN).max(1);
            let tl = aq.toile.lf();
            for y in 0..h.min(aq.toile.hf() * k) {
                let ty = y / k;
                for x in 0..l.min(tl * k) {
                    let s = &toile_bgra[(ty * tl + x / k) * 4..][..4];
                    let d = &mut tampon[(y * l + x) * 4..][..4];
                    let inv = 255 - s[3] as u16;
                    for c in 0..3 {
                        d[c] = (s[c] as u16 + (d[c] as u16 * inv + 127) / 255).min(255) as u8;
                    }
                }
            }
            ecran_ancien.clear();
            ecran_ancien.extend_from_slice(&tampon);
            std::hint::black_box(&ecran_ancien);
            env_ancien.ajouter(t);
        }
        if i % 24 == 0 {
            for p in aq.toile.px.iter().step_by(97) {
                empreinte =
                    (empreinte ^ u32::from_le_bytes(*p) as u64).wrapping_mul(0x100_0000_01b3);
            }
        }
    }
    // Composition de tout l'écran (fenêtre découverte, fond changé) : par bandes de 64 lignes.
    let t = cpu_ms();
    for _ in 0..10 {
        let mut y = 0;
        while y < h as i32 {
            let hh = (h as i32 - y).min(64);
            let r = crate::compo::Rect {
                x: 0,
                y,
                l: l as i32,
                h: hh,
            };
            compo.composer(&fond_u32, l, h, r, &mut morceau);
            y += hh;
        }
    }
    let complete = (cpu_ms() - t) / 10.0;
    println!("simulation : {}", sim.texte());
    println!("dessin     : {}", des.texte());
    println!(
        "envoi      : {} ({:.0} % de l'écran, {:.0} rectangles par image ; écran entier {complete:.2} ms)",
        env.texte(),
        aire * 100.0 / (n.max(1) as f64 * l as f64 * h as f64),
        nb_rect as f64 / n.max(1) as f64
    );
    if ancien {
        println!("envoi v0.11 (simulé) : {}", env_ancien.texte());
    }
    let total = sim.moyenne() + des.moyenne() + env.moyenne();
    println!(
        "total      : {total:.2} ms/image = {:.1} % d'un cœur à {ips} i/s",
        total * ips as f64 / 10.0
    );
    if let Some((pointe, rss)) = memoire() {
        println!(
            "mémoire    : pointe {pointe:.1} Mo (préparation comprise), en marche {rss:.1} Mo \
             (dont fond d'écran {:.1} Mo, toiles {:.1} Mo)",
            fond_u32.len() as f64 * 4.0 / 1048576.0,
            (aq.toile.px.len() * 2 + compo.px.len()) as f64 * 4.0 / 1048576.0
        );
    }
    println!("empreinte de la toile : {empreinte:016x}");
    Ok(())
}

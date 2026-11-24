//! Application eframe : la fenêtre racine dessine l'aquarium (sous Windows, elle est
//! incrustée derrière les icônes du bureau) ; l'Écloserie est une seconde fenêtre (viewport
//! immédiat) du même processus, qui partage la partie en direct.

use crate::ecloserie::{installer_style, Ecloserie, Onglet};
use crate::ecrans::Disposition;
use crate::jeu::{court, Partie};
use crate::scene::Aquarium;
use crate::{fond, sauve};
use eframe::egui::{
    self, Color32, TextureHandle, TextureOptions, Ui, Vec2, ViewportBuilder, ViewportId,
};
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct Options {
    pub fenetre: bool,
    pub demo: bool,
    pub fond: Option<String>,
    pub captures: Option<PathBuf>,
}

/// Commandes venues de l'icône de notification ou du raccourci global.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(not(windows), allow(dead_code))]
pub enum Commande {
    Ouvrir,
    Pause,
    Quitter,
    /// Bascule incrustation derrière les icônes / fenêtre au fond (réglage `mode_fenetre`).
    ModeFenetre,
    /// Méthode d'incrustation suivante (auto, Progman + calque, Progman, WorkerW).
    AutreMethode,
    /// L'Explorateur a redémarré : réincruster.
    Rattacher,
    /// Mire de couleurs 5 s sur le calque du fond.
    Tester,
}

struct Appli {
    partie: Partie,
    aquarium: Option<Aquarium>,
    echelle: usize,
    toile_tex: Option<TextureHandle>,
    fond_img: Option<image::RgbaImage>,
    fond_tex: Option<TextureHandle>,
    fond_chemin: Option<String>,
    dossier: PathBuf,
    sauve_le: Instant,
    tick: Instant,
    pause: bool,
    plein_ecran: bool,
    verif_plein_ecran: Instant,
    /// Écrans de Windows (relus toutes les 3 s : branchés, débranchés, déplacés).
    #[cfg_attr(not(windows), allow(dead_code))]
    disposition: Option<Disposition>,
    #[cfg_attr(not(windows), allow(dead_code))]
    verif_ecrans: Instant,
    message_depart: Option<String>,
    ecloserie: Ecloserie,
    ecloserie_ouverte: bool,
    #[cfg_attr(not(windows), allow(dead_code))]
    fenetre: bool,
    demo: bool,
    captures: Option<CapturesAuto>,
    #[cfg(windows)]
    systeme: Option<crate::windows::Systeme>,
    demarrage_applique: Option<bool>,
    /// Mode fenêtre et méthode d'incrustation appliqués (Windows).
    #[cfg_attr(not(windows), allow(dead_code))]
    affichage_applique: Option<(bool, u8)>,
    /// Première image dessinée (journal).
    premiere_image: bool,
    /// Mesure : images dessinées depuis `mesure_depuis` (affichées si AQUARIUM_IPS est défini).
    mesure: (u32, Instant, Duration),
}

/// Captures automatiques de chaque onglet (tests sous Xvfb).
/// eframe/glow ne sait capturer que la fenêtre racine : dans ce mode, la racine affiche
/// l'Écloserie seule (même code d'interface) ; l'aquarium se capture avec `--capture`.
struct CapturesAuto {
    dossier: PathBuf,
    etape: usize,
}

pub fn lancer(o: Options) -> Result<(), String> {
    let dossier = sauve::dossier();
    let (mut partie, mut message) = if o.demo {
        (crate::capture::partie_demo(), None)
    } else {
        sauve::charger(&dossier)
    };
    let gain = partie.gains_hors_ligne(sauve::maintenant());
    // Sur le fond : seulement les gains (la petite police n'a que majuscules et chiffres).
    let mut message_fond = None;
    if gain >= 1.0 {
        let m = format!("Pendant ton absence : +{}", court(gain));
        message_fond = Some(m.clone());
        message = Some(match message {
            Some(avant) => format!("{avant} {m}"),
            None => m,
        });
    }
    let icone = std::sync::Arc::new(icone());
    let viewport = if o.fenetre {
        ViewportBuilder::default()
            .with_title("Aquarium")
            .with_app_id("aquarium")
            .with_inner_size(if o.captures.is_some() {
                [960.0, 720.0]
            } else {
                [1280.0, 720.0]
            })
            .with_icon(icone)
    } else {
        // Fenêtre sans bord, hors barre des tâches : elle est ensuite incrustée dans WorkerW.
        ViewportBuilder::default()
            .with_title("Aquarium (fond)")
            .with_decorations(false)
            .with_taskbar(false)
            // Ne montre rien : le calque GDI (calque.rs) dessine l'aquarium.
            .with_inner_size([1.0, 1.0])
            .with_position([0.0, 0.0])
            .with_icon(icone)
    };
    let options = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    let mut ecloserie = Ecloserie::new(dossier.display().to_string());
    ecloserie.bandeau = message.clone();
    let appli = Appli {
        partie,
        aquarium: None,
        echelle: 4,
        toile_tex: None,
        fond_img: None,
        fond_tex: None,
        fond_chemin: o.fond,
        dossier,
        sauve_le: Instant::now(),
        tick: Instant::now(),
        pause: false,
        plein_ecran: false,
        verif_plein_ecran: Instant::now(),
        disposition: None,
        verif_ecrans: Instant::now(),
        message_depart: message_fond,
        ecloserie,
        // Sous Windows l'Écloserie s'ouvre depuis l'icône ; en fenêtre de test, tout de suite.
        ecloserie_ouverte: o.fenetre
            && o.captures.is_none()
            && std::env::var_os("AQUARIUM_IPS").is_none(),
        fenetre: o.fenetre,
        demo: o.demo,
        captures: o.captures.map(|dossier| CapturesAuto { dossier, etape: 0 }),
        #[cfg(windows)]
        systeme: None,
        demarrage_applique: None,
        affichage_applique: None,
        premiere_image: false,
        mesure: (0, Instant::now(), Duration::ZERO),
    };
    eframe::run_native(
        "aquarium",
        options,
        Box::new(move |cc| {
            installer_style(&cc.egui_ctx);
            Ok(Box::new(appli))
        }),
    )
    .map_err(|e| {
        // OpenGL indisponible, fenêtre impossible… : le journal le garde.
        crate::journal::ecrire(&format!("fenêtre / OpenGL : {e}"));
        format!("fenêtre : {e}")
    })
}

/// Icône 32x32 (fenêtres) : un poisson commun dessiné par le jeu.
pub fn icone_rgba() -> Vec<u8> {
    let e = crate::especes::par_cle("clown").expect("poisson-clown au catalogue");
    // Réduit pour tenir dans 32x32 (marges du sprite comprises).
    let s = crate::poisson::sprite(e, 0, 26.0 / e.dims.0 as f32);
    let mut out = vec![0u8; 32 * 32 * 4];
    let (ox, oy) = ((32 - s.l) / 2, (32 - s.h) / 2);
    for y in 0..s.h {
        for x in 0..s.l {
            let p = s.images[0].px((y * s.l + x) as usize);
            let (xx, yy) = (x + ox, y + oy);
            if (0..32).contains(&xx) && (0..32).contains(&yy) {
                out[((yy * 32 + xx) * 4) as usize..][..4].copy_from_slice(&p);
            }
        }
    }
    out
}

fn icone() -> egui::IconData {
    egui::IconData {
        rgba: icone_rgba(),
        width: 32,
        height: 32,
    }
}

impl Appli {
    fn sauver(&mut self) {
        // Démo et captures automatiques : la vraie sauvegarde n'est jamais touchée.
        if self.demo || self.captures.is_some() {
            return;
        }
        self.partie.horodatage = sauve::maintenant();
        if let Err(e) = sauve::ecrire(&self.dossier, &self.partie) {
            self.ecloserie.bandeau = Some(format!("Sauvegarde impossible : {e}"));
        }
        self.sauve_le = Instant::now();
    }

    fn commande(&mut self, ctx: &egui::Context, c: Commande) {
        match c {
            Commande::Ouvrir => {
                self.ecloserie_ouverte = true;
                ctx.send_viewport_cmd_to(ecloserie_id(), egui::ViewportCommand::Focus);
            }
            Commande::Pause => self.pause = !self.pause,
            Commande::Quitter => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Commande::ModeFenetre => {
                let r = &mut self.partie.reglages;
                r.mode_fenetre = !r.mode_fenetre;
            }
            Commande::AutreMethode => {
                let r = &mut self.partie.reglages;
                r.mode_fenetre = false;
                r.incrustation = (r.incrustation + 1) % 4;
            }
            Commande::Rattacher => self.affichage_applique = None,
            Commande::Tester =>
            {
                #[cfg(windows)]
                if let Some(s) = &self.systeme {
                    s.tester();
                }
            }
        }
    }

    /// Écrans couverts par la fenêtre : sous Windows ceux du bureau virtuel (la fenêtre
    /// recouvre leur boîte englobante), sinon un seul écran de la taille de la fenêtre.
    fn disposition(&mut self, lp: u32, hp: u32) -> Disposition {
        #[cfg(windows)]
        if !self.fenetre {
            if self.disposition.is_none() {
                self.disposition = self.systeme.as_ref().and_then(|s| s.disposition());
            }
            if let Some(d) = &self.disposition {
                return d.clone();
            }
        }
        Disposition::simple(lp, hp)
    }

    /// Toile à la taille de la fenêtre (pixels physiques / échelle entière de l'écran principal).
    fn preparer(&mut self, ui: &Ui) -> (f32, f32) {
        let ppp = ui.ctx().pixels_per_point();
        let taille = ui.ctx().content_rect().size() * ppp;
        let (lp, hp) = (
            taille.x.round().max(64.0) as u32,
            taille.y.round().max(64.0) as u32,
        );
        let disp = self.disposition(lp, hp);
        // Windows : la toile suit le calque du fond (la boîte des écrans), pas la fenêtre eframe.
        #[cfg(windows)]
        let (lp, hp) = if self.systeme.is_some() {
            let (_, _, l, h) = disp.boite();
            (l as u32, h as u32)
        } else {
            (lp, hp)
        };
        let e = disp.echelle() as usize;
        let (l, h) = ((lp as usize).div_ceil(e), (hp as usize).div_ceil(e));
        let refaire = self
            .aquarium
            .as_ref()
            .is_none_or(|a| a.toile.l != l || a.toile.h != h || self.echelle != e);
        if refaire {
            let mut aq = Aquarium::avec_zones(l, h, 1, &disp.zones(e as u32));
            if let Some(m) = &self.message_depart {
                aq.message = Some((m.clone(), 15.0));
            }
            aq.synchroniser(&self.partie);
            self.aquarium = Some(aq);
            self.echelle = e;
            self.fond_tex = None;
            if self.fond_img.is_none() {
                let img = self.charger_fond(&disp, lp, hp);
                #[cfg(windows)]
                if let Some(s) = &self.systeme {
                    s.fond(&img);
                }
                self.fond_img = Some(img);
            }
        }
        (lp as f32, hp as f32)
    }

    fn charger_fond(&self, d: &Disposition, l: u32, h: u32) -> image::RgbaImage {
        #[cfg(windows)]
        if !self.fenetre {
            if let Some(img) = crate::windows::fond_bureau(d) {
                return img;
            }
        }
        let _ = d;
        match &self.fond_chemin {
            Some(f) => image::open(f)
                .map(|i| fond::couvrir(&i.to_rgba8(), l, h))
                .unwrap_or_else(|_| fond::factice(l, h)),
            None => fond::factice(l, h),
        }
    }

    /// Étapes à heure fixe (secondes depuis le lancement) : l'éclosion dure quelques secondes.
    fn captures_auto(&mut self, ui: &Ui) {
        let Some(c) = &mut self.captures else {
            return;
        };
        let t = ui.input(|i| i.time);
        let etapes: [(f64, &str); 8] = [
            (1.0, "ecloserie-oeufs"),
            (1.5, "achat"),
            (3.3, "ecloserie-eclosion"),
            (6.5, "ecloserie-revelation"),
            (7.5, "ecloserie-aquarium"),
            (8.5, "ecloserie-poissodex"),
            (9.5, "ecloserie-reglages"),
            (10.5, "fin"),
        ];
        let Some(&(_, nom)) = etapes.get(c.etape) else {
            return;
        };
        if t < etapes[c.etape].0 {
            return;
        }
        c.etape += 1;
        let dossier = c.dossier.clone();
        match nom {
            "achat" => {
                // Œuf d'ambre (le dernier) : montre une éclosion Préhistorique (le plus souvent).
                self.partie.argent = self.partie.argent.max(1e12);
                self.ecloserie
                    .acheter(ui, &mut self.partie, crate::jeu::OEUFS.len() - 1);
            }
            "fin" => ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close),
            _ => {
                // La capture prend l'image suivante : l'onglet change d'abord.
                self.ecloserie.onglet = match nom {
                    "ecloserie-aquarium" => Onglet::Aquarium,
                    "ecloserie-poissodex" => Onglet::Poissodex,
                    "ecloserie-reglages" => Onglet::Reglages,
                    _ => self.ecloserie.onglet,
                };
                ui.ctx()
                    .send_viewport_cmd(capture_vers(dossier.join(format!("{nom}.png"))));
            }
        }
    }
}

fn capture_vers(f: PathBuf) -> egui::ViewportCommand {
    egui::ViewportCommand::Screenshot(egui::ScreenshotCallback::new(move |img| {
        if let Some(d) = f.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        let rgba: Vec<u8> = img
            .pixels
            .iter()
            .flat_map(|c| c.to_srgba_unmultiplied())
            .collect();
        let r = image::save_buffer(
            &f,
            &rgba,
            img.width() as u32,
            img.height() as u32,
            image::ColorType::Rgba8,
        );
        println!("capture {} : {:?}", f.display(), r.err());
    }))
}

fn ecloserie_id() -> ViewportId {
    ViewportId::from_hash_of("ecloserie")
}

impl eframe::App for Appli {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        #[cfg(windows)]
        if !self.fenetre && self.systeme.is_none() {
            let r = &self.partie.reglages;
            let voulu = (r.mode_fenetre, r.incrustation);
            match crate::windows::Systeme::demarrer(frame, &ctx, voulu.0, voulu.1) {
                Ok(s) => {
                    self.systeme = Some(s);
                    self.affichage_applique = Some(voulu);
                }
                Err(e) => {
                    crate::journal::ecrire(&format!("démarrage impossible : {e}"));
                    crate::windows::alerte(&e);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return;
                }
            }
        }
        // Mode fenêtre ou méthode changés (menu, Réglages), Explorateur relancé : réappliquer.
        #[cfg(windows)]
        {
            let r = &self.partie.reglages;
            let voulu = (r.mode_fenetre, r.incrustation);
            if self.affichage_applique != Some(voulu) {
                if let Some(s) = &mut self.systeme {
                    crate::journal::ecrire(&format!(
                        "affichage : mode fenêtre {}, méthode {}",
                        voulu.0, voulu.1
                    ));
                    if let Some(e) = s.appliquer(voulu.0, voulu.1) {
                        self.ecloserie.bandeau = Some(e);
                    }
                    self.disposition = None;
                    self.fond_img = None;
                    self.aquarium = None;
                    self.fond_tex = None;
                    self.toile_tex = None;
                }
                self.affichage_applique = Some(voulu);
            }
        }
        let _ = frame;
        #[cfg(windows)]
        let commandes: Vec<Commande> = self
            .systeme
            .as_ref()
            .map(|s| s.commandes())
            .unwrap_or_default();
        #[cfg(not(windows))]
        let commandes: Vec<Commande> = Vec::new();
        for c in commandes {
            self.commande(&ctx, c);
        }
        // Fenêtre de test : E ouvre l'Écloserie, P met en pause.
        if ctx.input(|i| i.key_pressed(egui::Key::E)) {
            self.commande(&ctx, Commande::Ouvrir);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::P)) {
            self.commande(&ctx, Commande::Pause);
        }
        if self.captures.is_some() {
            self.ecloserie.ui(ui, &mut self.partie);
            self.captures_auto(ui);
            ctx.request_repaint();
            return;
        }
        let maintenant = Instant::now();
        let dt = maintenant.duration_since(self.tick).as_secs_f64();
        if self.mesure.1.elapsed() > Duration::from_secs(5) {
            if std::env::var_os("AQUARIUM_IPS").is_some() {
                let (n, d, cpu) = self.mesure;
                println!(
                    "{:.1} images/s, toile {:.2} ms/image",
                    n as f64 / d.elapsed().as_secs_f64(),
                    cpu.as_secs_f64() * 1000.0 / n.max(1) as f64
                );
            }
            self.mesure = (0, Instant::now(), Duration::ZERO);
        }
        self.mesure.0 += 1;
        self.tick = maintenant;
        // L'argent continue même en pause.
        self.partie.avancer(dt);
        if self.verif_plein_ecran.elapsed() > Duration::from_secs(1) {
            self.verif_plein_ecran = Instant::now();
            #[cfg(windows)]
            {
                let raison = crate::windows::plein_ecran()
                    .filter(|_| self.partie.reglages.pause_plein_ecran && !self.fenetre);
                if raison.is_some() != self.plein_ecran {
                    crate::journal::ecrire(&match &raison {
                        Some(r) => format!("pause plein écran : {r}"),
                        None => "fin de la pause plein écran".to_string(),
                    });
                }
                self.plein_ecran = raison.is_some();
            }
        }
        #[cfg(windows)]
        if !self.fenetre && self.verif_ecrans.elapsed() > Duration::from_secs(3) {
            self.verif_ecrans = Instant::now();
            let d = self.systeme.as_ref().and_then(|s| s.disposition());
            if d.is_some() && d != self.disposition {
                // Écran branché, débranché ou déplacé : nouvelle place, nouveaux fonds.
                self.disposition = d;
                if let Some(s) = &self.systeme {
                    s.recaler();
                }
                self.aquarium = None;
                self.fond_img = None;
                self.fond_tex = None;
            }
        }
        let fige = self.pause || self.plein_ecran;
        #[cfg(windows)]
        if let Some(s) = &self.systeme {
            s.signaler_pause(self.pause);
        }
        let (lp, hp) = self.preparer(ui);
        let e = self.echelle;
        if let Some(aq) = &mut self.aquarium {
            aq.synchroniser(&self.partie);
            if !fige || self.toile_tex.is_none() {
                let t0 = Instant::now();
                aq.avancer(dt as f32);
                aq.dessiner(&self.partie);
                self.mesure.2 += t0.elapsed();
                let img = crate::ecloserie::image_couleur(&aq.toile);
                match &mut self.toile_tex {
                    Some(t) => t.set(img, TextureOptions::NEAREST),
                    None => {
                        self.toile_tex =
                            Some(ctx.load_texture("toile", img, TextureOptions::NEAREST))
                    }
                }
                #[cfg(windows)]
                if let Some(s) = &self.systeme {
                    s.presenter(&aq.toile, e);
                }
                if !self.premiere_image {
                    self.premiere_image = true;
                    crate::journal::ecrire(&format!(
                        "première image : fenêtre {lp}x{hp} px, toile {}x{} x{e}, {} pixels par \
                         point, {} poissons",
                        aq.toile.l,
                        aq.toile.h,
                        ctx.pixels_per_point(),
                        aq.nageurs.len()
                    ));
                }
            }
        }
        // Windows : le calque GDI montre l'aquarium, la fenêtre eframe (cachée) ne dessine rien.
        #[cfg(windows)]
        let calque = self.systeme.is_some();
        #[cfg(not(windows))]
        let calque = false;
        if self.fond_tex.is_none() && !calque {
            if let Some(img) = &self.fond_img {
                let ci = egui::ColorImage::from_rgba_unmultiplied(
                    [img.width() as usize, img.height() as usize],
                    img.as_raw(),
                );
                self.fond_tex = Some(ctx.load_texture("fond", ci, TextureOptions::LINEAR));
            }
        }
        // Fond puis toile, en pixels physiques entiers (échelle entière).
        let ppp = ctx.pixels_per_point();
        let p = ui.painter();
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        if let (Some(t), false) = (&self.fond_tex, calque) {
            p.image(
                t.id(),
                egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(lp, hp) / ppp),
                uv,
                Color32::WHITE,
            );
        }
        if let (Some(t), Some(aq), false) = (&self.toile_tex, &self.aquarium, calque) {
            let taille = Vec2::new((aq.toile.l * e) as f32, (aq.toile.h * e) as f32) / ppp;
            p.image(
                t.id(),
                egui::Rect::from_min_size(egui::Pos2::ZERO, taille),
                uv,
                Color32::WHITE,
            );
        }

        if self.ecloserie_ouverte {
            let mut fermer = false;
            let builder = ViewportBuilder::default()
                .with_title("Écloserie")
                .with_app_id("aquarium-ecloserie")
                .with_inner_size([960.0, 720.0])
                .with_min_inner_size([640.0, 520.0])
                .with_icon(std::sync::Arc::new(icone()));
            ctx.show_viewport_immediate(ecloserie_id(), builder, |ui, _| {
                self.ecloserie.ui(ui, &mut self.partie);
                if ui.input(|i| i.viewport().close_requested()) {
                    fermer = true;
                }
            });
            if fermer {
                self.ecloserie_ouverte = false;
                self.sauver();
            }
        }
        // Lancement au démarrage : appliqué quand le réglage change.
        if self.demarrage_applique != Some(self.partie.reglages.demarrage_windows) {
            #[cfg(windows)]
            if !self.fenetre {
                if let Err(e) = crate::windows::demarrage(self.partie.reglages.demarrage_windows) {
                    self.ecloserie.bandeau = Some(e);
                }
            }
            self.demarrage_applique = Some(self.partie.reglages.demarrage_windows);
        }
        if self.sauve_le.elapsed() > Duration::from_secs(30) {
            self.sauver();
        }
        let ips = self.partie.reglages.ips.clamp(15, 60) as f32;
        if fige {
            ctx.request_repaint_after(Duration::from_millis(500));
        } else {
            ctx.request_repaint_after(Duration::from_secs_f32(1.0 / ips));
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.sauver();
        #[cfg(windows)]
        if let Some(s) = self.systeme.take() {
            s.arreter();
        }
    }
}

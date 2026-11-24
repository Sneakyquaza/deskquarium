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
    /// Bascule le mode Abysses (réglage sauvegardé).
    Abysses,
    /// Recherche une mise à jour (ouvre les Réglages de l'Écloserie).
    ChercherMaj,
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
    /// Dernière recherche automatique de mise à jour (au lancement puis toutes les 6 h).
    maj_le: Option<Instant>,
    /// Version annoncée dans l'infobulle de l'icône.
    #[cfg_attr(not(windows), allow(dead_code))]
    maj_annoncee: Option<String>,
    /// Coin haut gauche de la toile dans le bureau virtuel (pixels physiques) : curseur.
    #[cfg_attr(not(windows), allow(dead_code))]
    origine: (i32, i32),
    /// Fenêtre de test : même règle d'appui que le crochet Windows (`souris.rs`).
    pince: crate::souris::Pince,
    /// Windows : crochet souris (attraper les poissons), voulu selon le réglage.
    #[cfg(windows)]
    crochet: Option<crate::crochet::Crochet>,
    #[cfg(windows)]
    crochet_voulu: Option<bool>,
    /// Windows : dernière recherche d'icône sous le curseur (point, instant, icône ?).
    #[cfg(windows)]
    icone_vue: Option<((i32, i32), Instant, Option<bool>)>,
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
    // Langue : AQUARIUM_LANGUE (captures) l'impose, sinon celle choisie, sinon celle de Windows.
    if std::env::var_os("AQUARIUM_LANGUE").is_some() {
        partie.reglages.langue = Some(crate::i18n::langue_systeme());
    }
    crate::i18n::regler(
        partie
            .reglages
            .langue
            .unwrap_or_else(crate::i18n::langue_systeme),
    );
    let gain = partie.gains_hors_ligne(sauve::maintenant());
    // Sur le fond : seulement les gains (la petite police n'a que majuscules et chiffres).
    let mut message_fond = None;
    if gain >= 1.0 {
        let m = crate::i18n::trf("Pendant ton absence : +{}", &[&court(gain)]);
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
        maj_le: None,
        maj_annoncee: None,
        origine: (0, 0),
        pince: crate::souris::Pince::default(),
        #[cfg(windows)]
        crochet: None,
        #[cfg(windows)]
        crochet_voulu: None,
        #[cfg(windows)]
        icone_vue: None,
    };
    // Ancienne version laissée par une mise à jour : supprimée.
    crate::maj::nettoyer();
    eframe::run_native(
        "aquarium",
        options,
        Box::new(move |cc| {
            installer_style(&cc.egui_ctx);
            let mut appli = appli;
            let ctx = cc.egui_ctx.clone();
            appli.ecloserie.maj = Some(crate::maj::Maj::new(move || ctx.request_repaint()));
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
            self.ecloserie.bandeau = Some(crate::i18n::trf(
                "Sauvegarde impossible : {}",
                &[&e.to_string()],
            ));
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
            Commande::ChercherMaj => {
                self.ecloserie_ouverte = true;
                self.ecloserie.onglet = Onglet::Reglages;
                if let Some(m) = &self.ecloserie.maj {
                    m.chercher(false);
                }
                ctx.send_viewport_cmd_to(ecloserie_id(), egui::ViewportCommand::Focus);
            }
            Commande::Abysses => {
                let r = &mut self.partie.reglages;
                r.abysses = !r.abysses;
                self.sauver();
            }
            Commande::Tester =>
            {
                #[cfg(windows)]
                if let Some(s) = &self.systeme {
                    s.tester();
                }
            }
        }
    }

    /// Mises à jour : recherche discrète au lancement puis toutes les 6 h (jamais en démo ni
    /// en captures), infobulle de l'icône, installation (après le clic « Installer ») puis
    /// fermeture propre et relance.
    fn mises_a_jour(&mut self, ctx: &egui::Context) {
        let Some(m) = &self.ecloserie.maj else {
            return;
        };
        if !self.demo
            && self.captures.is_none()
            && self
                .maj_le
                .is_none_or(|t| t.elapsed() > Duration::from_secs(6 * 3600))
        {
            self.maj_le = Some(Instant::now());
            m.chercher(true);
        }
        #[cfg(windows)]
        {
            let dispo = m.disponible();
            if dispo != self.maj_annoncee {
                if let Some(s) = &self.systeme {
                    s.infobulle(&match &dispo {
                        Some(v) => crate::i18n::trf(
                            "Aquarium : mise à jour v{} disponible",
                            &[&v.to_string()],
                        ),
                        None => String::new(),
                    });
                }
                self.maj_annoncee = dispo;
            }
        }
        if matches!(m.etat(), crate::maj::Etat::Pret(_)) {
            if let Some(exe) = m.installer() {
                self.ecloserie.relancer = Some(exe);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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
        let (bx, by, _, _) = disp.boite();
        self.origine = (bx, by);
        self.ecloserie.disposition = Some((disp.clone(), e as u32));
        let (l, h) = ((lp as usize).div_ceil(e), (hp as usize).div_ceil(e));
        let refaire = self
            .aquarium
            .as_ref()
            .is_none_or(|a| a.toile.l != l || a.toile.h != h || self.echelle != e);
        if refaire {
            let mut aq =
                Aquarium::avec_zones(l, h, 1, &disp.zones(e as u32), &disp.travail(e as u32));
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

    /// Souris (v0.9) : curseur sur le bureau (curiosité, fuite) et poisson attrapé. Windows :
    /// position par GetCursorPos (le fond ne reçoit aucun clic) et crochet souris ; fenêtre de
    /// test : la souris d'egui, même règle (`souris.rs`).
    fn souris(&mut self, ctx: &egui::Context, dt: f32, fige: bool) {
        use crate::souris::Suite;
        let Some(aq) = &mut self.aquarium else {
            return;
        };
        let e = self.echelle as f32;
        let r = &self.partie.reglages;
        let attraper = r.attraper && !fige;
        let mut suites: Vec<Suite> = Vec::new();
        #[allow(unused_mut)]
        let mut pos: Option<(f32, f32)> = None;
        #[allow(unused_mut)]
        let mut fait = false;
        #[cfg(windows)]
        if let Some(s) = &self.systeme {
            fait = true;
            // Crochet installé seulement si le réglage est actif (retiré sinon).
            if self.crochet_voulu != Some(r.attraper) {
                self.crochet_voulu = Some(r.attraper);
                self.crochet = None;
                if r.attraper {
                    self.crochet = crate::crochet::Crochet::demarrer();
                }
            }
            let cur = crate::crochet::curseur_bureau(s.calque_hwnd());
            pos = cur.map(|(pt, _)| {
                (
                    (pt.x - self.origine.0) as f32 / e,
                    (pt.y - self.origine.1) as f32 / e,
                )
            });
            // Poisson attrapable sous le curseur, sur le bureau vide : publié au crochet.
            let mut arme = None;
            if let (true, Some((pt, liste)), Some(p)) = (attraper, cur, pos) {
                if let Some(id) = aq.poisson_sous(p, true).filter(|_| !aq.en_prise()) {
                    // Icône sous le curseur : 10 recherches par seconde au plus.
                    let icone = match self.icone_vue {
                        Some((q, t, v))
                            if (q.0 - pt.x).abs() <= 3
                                && (q.1 - pt.y).abs() <= 3
                                && t.elapsed() < Duration::from_millis(100) =>
                        {
                            v
                        }
                        _ => {
                            let v = crate::crochet::icone_sous(liste, pt);
                            self.icone_vue = Some(((pt.x, pt.y), Instant::now(), v));
                            v
                        }
                    };
                    if icone == Some(false) {
                        arme = Some((pt.x, pt.y, id));
                    }
                }
            }
            crate::crochet::armer(arme, r.attraper_alt);
            if let Some(c) = &self.crochet {
                suites = c.suites();
            }
        }
        if !fait {
            // Fenêtre de test : pixels physiques de la fenêtre.
            let ppp = ctx.pixels_per_point();
            let (survol, t, evts) = ctx.input(|i| {
                let evts: Vec<(egui::Pos2, bool)> = i
                    .events
                    .iter()
                    .filter_map(|ev| match ev {
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed,
                            ..
                        } => Some((*pos, *pressed)),
                        _ => None,
                    })
                    .collect();
                (i.pointer.hover_pos(), (i.time * 1000.0) as u32, evts)
            });
            pos = survol.map(|p| (p.x * ppp / e, p.y * ppp / e));
            let px = |p: egui::Pos2| ((p.x * ppp) as i32, (p.y * ppp) as i32);
            if let Some(p) = survol {
                let (x, y) = px(p);
                suites.push(self.pince.mouvement(t, x, y).1);
            }
            for (p, appui) in evts {
                let (x, y) = px(p);
                suites.push(if appui {
                    let alt_ok = !r.attraper_alt || ctx.input(|i| i.modifiers.alt);
                    let id = aq
                        .poisson_sous((p.x * ppp / e, p.y * ppp / e), true)
                        .filter(|_| attraper && alt_ok);
                    self.pince.appui(t, x, y, id).1
                } else {
                    self.pince.relache(t).1
                });
            }
        }
        for s in suites {
            match s {
                Suite::Appui(id) => aq.appuyer(id),
                Suite::RendreAppui(..) => aq.annuler(),
                Suite::Clic(id) => aq.cliquer(id),
                Suite::Lache => aq.lacher(),
                Suite::Rien => {}
            }
        }
        if !fige {
            aq.curseur(pos, dt);
        }
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
        let etapes: [(f64, &str); 13] = [
            (1.0, "ecloserie-oeufs"),
            (1.5, "achat"),
            (3.3, "ecloserie-eclosion"),
            (6.5, "ecloserie-revelation"),
            (7.5, "ecloserie-aquarium"),
            (8.5, "ecloserie-poissodex"),
            (9.5, "ecloserie-index-dore"),
            (10.5, "ecloserie-index-shiny"),
            (11.5, "ecloserie-reglages"),
            (12.0, "decor-achats"),
            (12.8, "ecloserie-decor-boutique"),
            (13.8, "ecloserie-decor-editeur"),
            (14.8, "fin"),
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
                // Œuf d'ambre (toujours disponible) : montre une éclosion Préhistorique (le plus souvent).
                self.partie.argent = self.partie.argent.max(1e12);
                // AQUARIUM_OEUF=0 : œuf commun (petites espèces à l'éclosion).
                let oeuf = std::env::var("AQUARIUM_OEUF")
                    .ok()
                    .and_then(|v| v.parse::<usize>().ok())
                    .filter(|&i| i < crate::jeu::OEUFS.len())
                    .unwrap_or(4);
                self.ecloserie.acheter(ui, &mut self.partie, oeuf);
                // AQUARIUM_FORME=shiny / galaxie… : force la forme du poisson éclos (captures).
                if let Ok(v) = std::env::var("AQUARIUM_FORME") {
                    let forme = crate::jeu::Forme {
                        shiny: v.contains("shiny"),
                        modif: crate::jeu::Modif::TOUS
                            .into_iter()
                            .find(|m| v.contains(&m.nom_fr().to_lowercase())),
                    };
                    self.ecloserie.forcer_forme(&mut self.partie, forme);
                }
            }
            "decor-achats" => {
                // Quelques objets achetés et posés (boutique et éditeur montrent un vrai décor).
                self.partie.argent = 1e15;
                for cle in [
                    "coffre",
                    "algue_lumineuse",
                    "statue",
                    "geyser",
                    "crane",
                    "cristaux",
                ] {
                    let _ = self.partie.acheter_objet(cle);
                }
                self.partie.decor_pose.clear();
                for (i, cle) in ["coffre", "algue_lumineuse", "statue", "geyser", "cristaux"]
                    .iter()
                    .enumerate()
                {
                    let _ = self.partie.poser_objet(cle, 0, 0.12 + i as f32 * 0.2);
                }
                self.partie.argent = 2.4e10;
            }
            "fin" => ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close),
            _ => {
                // La capture prend l'image suivante : l'onglet change d'abord.
                self.ecloserie.onglet = match nom {
                    "ecloserie-aquarium" => Onglet::Aquarium,
                    "ecloserie-poissodex" | "ecloserie-index-dore" | "ecloserie-index-shiny" => {
                        self.ecloserie.index = match nom {
                            "ecloserie-index-dore" => 1,
                            "ecloserie-index-shiny" => crate::jeu::NB_INDEX - 1,
                            _ => 0,
                        };
                        Onglet::Poissodex
                    }
                    "ecloserie-reglages" => Onglet::Reglages,
                    "ecloserie-decor-boutique" | "ecloserie-decor-editeur" => {
                        self.ecloserie.decor.editeur = nom == "ecloserie-decor-editeur";
                        Onglet::Decor
                    }
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
        // Langue choisie (sinon celle de Windows) : appliquée tout de suite partout.
        crate::i18n::regler(
            self.partie
                .reglages
                .langue
                .unwrap_or_else(crate::i18n::langue_systeme),
        );
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
            s.signaler_abysses(self.partie.reglages.abysses);
            s.signaler_pause(self.pause);
        }
        let (lp, hp) = self.preparer(ui);
        let e = self.echelle;
        self.souris(&ctx, dt as f32, fige);
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
                .with_title(crate::i18n::tr("Écloserie"))
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
        // Grandes éclosions : annoncées sur le fond d'écran, près du compteur (v0.11).
        if let Some(aq) = &mut self.aquarium {
            aq.annonces.append(&mut self.ecloserie.annonces);
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
        self.mises_a_jour(&ctx);
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
        // Mise à jour installée : la nouvelle version démarre une fois le fond d'écran remis.
        if let Some(exe) = self.ecloserie.relancer.take() {
            crate::maj::relancer(&exe);
        }
    }
}

//! Fenêtre « Écloserie » (egui) : œufs et éclosion, mon aquarium, Poissodex, réglages.
//! Style : verre d'aquarium biseauté en pixels, polices pixel OFL (Silkscreen pour les titres,
//! Pixelify Sans pour le texte), images pixel art agrandies en entier au plus proche.

use crate::especes::{de_rarete, Espece};
use crate::i18n::{tr, trf};
use crate::jeu::{
    court, duree, nom_index, Forme, Modif, Partie, Poisson, Rarete, Refus, BONUS_INDEX, CAPACITE,
    MULT_SHINY, NB_INDEX, OEUFS,
};
use crate::scene::{nom_de, sprite_ajuste, Aquarium};
use crate::toile::{melange, Rgb, Toile};
use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, Margin, RichText, Stroke, TextureHandle,
    TextureOptions, Ui, Vec2,
};
use std::collections::HashMap;
use std::sync::Arc;

mod decor_ui;
mod nourriture_ui;

/// Jetons de couleur.
pub const EAU: Color32 = Color32::from_rgb(0x10, 0x32, 0x4a);
const VERRE: Color32 = Color32::from_rgb(0x16, 0x45, 0x60);
const VERRE_FONCE: Color32 = Color32::from_rgb(0x0d, 0x29, 0x3b);
const BISEAU_CLAIR: Color32 = Color32::from_rgb(0x4a, 0x9c, 0xb4);
const BISEAU_SOMBRE: Color32 = Color32::from_rgb(0x07, 0x1a, 0x28);
const ECUME: Color32 = Color32::from_rgb(0xe8, 0xf6, 0xf2);
const BRUME: Color32 = Color32::from_rgb(0x8f, 0xbc, 0xc4);
const OR: Color32 = Color32::from_rgb(0xff, 0xd3, 0x6e);
const SABLE: Color32 = Color32::from_rgb(0xf2, 0xc1, 0x4e);
const ENCRE: Color32 = Color32::from_rgb(0x2a, 0x1a, 0x08);
const ALERTE: Color32 = Color32::from_rgb(0xe8, 0x5d, 0x75);
/// Badge shiny (rose doré).
const SHINY: Color32 = Color32::from_rgb(0xff, 0xb8, 0xe8);

fn c32(c: Rgb) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

fn titre(taille: f32) -> FontId {
    FontId::new(taille, FontFamily::Name("titre".into()))
}

/// Polices embarquées et visuels egui.
pub fn installer_style(ctx: &egui::Context) {
    let mut polices = egui::FontDefinitions::empty();
    polices.font_data.insert(
        "pixelify".into(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/polices/PixelifySans-Regular.ttf"
        ))),
    );
    polices.font_data.insert(
        "silkscreen".into(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/polices/Silkscreen-Regular.ttf"
        ))),
    );
    polices.families.insert(
        FontFamily::Proportional,
        vec!["pixelify".into(), "silkscreen".into()],
    );
    polices.families.insert(
        FontFamily::Monospace,
        vec!["pixelify".into(), "silkscreen".into()],
    );
    polices.families.insert(
        FontFamily::Name("titre".into()),
        vec!["silkscreen".into(), "pixelify".into()],
    );
    ctx.set_fonts(polices);
    let mut style = (*ctx.global_style()).clone();
    style.text_styles = [
        (egui::TextStyle::Small, FontId::proportional(13.0)),
        (egui::TextStyle::Body, FontId::proportional(16.0)),
        (egui::TextStyle::Button, FontId::proportional(16.0)),
        (egui::TextStyle::Heading, titre(22.0)),
        (egui::TextStyle::Monospace, FontId::proportional(16.0)),
    ]
    .into();
    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(10.0, 5.0);
    style.spacing.slider_width = 260.0;
    let v = &mut style.visuals;
    *v = egui::Visuals::dark();
    v.override_text_color = Some(ECUME);
    v.panel_fill = EAU;
    v.window_fill = EAU;
    v.extreme_bg_color = VERRE_FONCE;
    v.faint_bg_color = VERRE;
    v.selection.bg_fill = Color32::from_rgb(0x2c, 0x78, 0x90);
    v.selection.stroke = Stroke::new(2.0, ECUME);
    v.slider_trailing_fill = true;
    v.handle_shape = egui::style::HandleShape::Rect { aspect_ratio: 0.6 };
    for (w, fond) in [
        (&mut v.widgets.inactive, Color32::from_rgb(0x1e, 0x58, 0x74)),
        (&mut v.widgets.hovered, Color32::from_rgb(0x27, 0x6c, 0x8a)),
        (&mut v.widgets.active, Color32::from_rgb(0x30, 0x80, 0xa0)),
        (&mut v.widgets.open, Color32::from_rgb(0x27, 0x6c, 0x8a)),
        (&mut v.widgets.noninteractive, VERRE),
    ] {
        w.bg_fill = fond;
        w.weak_bg_fill = fond;
        w.corner_radius = CornerRadius::ZERO;
        w.expansion = 0.0;
        w.fg_stroke = Stroke::new(1.5, ECUME);
    }
    v.widgets.inactive.bg_stroke = Stroke::new(2.0, BISEAU_SOMBRE);
    v.widgets.hovered.bg_stroke = Stroke::new(2.0, BISEAU_CLAIR);
    v.widgets.active.bg_stroke = Stroke::new(2.0, ECUME);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BISEAU_SOMBRE);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, BRUME);
    v.window_corner_radius = CornerRadius::ZERO;
    v.menu_corner_radius = CornerRadius::ZERO;
    ctx.set_theme(egui::Theme::Dark);
    ctx.set_global_style(style);
}

#[derive(Clone, Copy, PartialEq)]
pub enum Onglet {
    Oeufs,
    Aquarium,
    Poissodex,
    Decor,
    Nourriture,
    Reglages,
}

use crate::eclosion::multi::Multi;
use crate::eclosion::Eclosion;
use crate::scene::Annonce;
use crate::son::{self, Son};

pub struct Ecloserie {
    pub onglet: Onglet,
    eclosion: Option<Eclosion>,
    vignettes: HashMap<u64, TextureHandle>,
    vignettes_dex: HashMap<String, TextureHandle>,
    /// Index affiché (0 Normal, 1..=7 modificateurs, 8 Shiny).
    pub index: usize,
    oeufs: Vec<TextureHandle>,
    confirmer: Option<u64>,
    pub reserve: bool,
    /// v0.16 : « Tout vendre » de la réserve en attente de confirmation.
    confirmer_tout: bool,
    /// Message du haut (gains hors ligne, sauvegarde réparée…).
    pub bandeau: Option<String>,
    info: Option<(String, f64)>,
    pub dossier: String,
    /// Le lancement au démarrage n'existe que sous Windows.
    pub windows: bool,
    /// Mises à jour (GitHub Releases), créé au lancement de l'appli.
    pub maj: Option<crate::maj::Maj>,
    /// Nouvelle version installée : à relancer à la fermeture.
    pub relancer: Option<std::path::PathBuf>,
    /// Écrans de l'aquarium et échelle (éditeur du décor), donnés par l'appli.
    pub disposition: Option<(crate::ecrans::Disposition, u32)>,
    /// Onglet Décor (boutique, éditeur).
    pub decor: decor_ui::EtatDecor,
    /// v0.11 : ouverture ×3/×10 en cours.
    multi: Option<Multi>,
    /// v0.11 : grandes éclosions à annoncer sur le fond d'écran (l'appli les vide), et
    /// celles qui attendent la fin de l'animation (pas de surprise gâchée).
    pub annonces: Vec<Annonce>,
    a_annoncer: Vec<Annonce>,
    /// v0.11 : ouverture auto (même œuf, même lot) : lot en cours, arrêt (message), instant
    /// de la dernière révélation.
    auto: bool,
    lot: u32,
    auto_arret: Option<String>,
    revele_le: Option<f64>,
    /// Langue choisie au premier lancement (appliquée à la partie après l'en-tête).
    langue_choisie: Option<crate::i18n::Langue>,
    /// v0.16 : nourriture à jeter au hasard (l'appli la jette), nourriture qui attend un clic
    /// sur le bureau (et depuis quand), icônes de la boutique.
    pub jets: Vec<usize>,
    pub viser: Option<(usize, std::time::Instant)>,
    icones_nourriture: Vec<TextureHandle>,
}

impl Ecloserie {
    pub fn new(dossier: String) -> Ecloserie {
        Ecloserie {
            onglet: Onglet::Oeufs,
            eclosion: None,
            vignettes: HashMap::new(),
            vignettes_dex: HashMap::new(),
            index: 0,
            oeufs: Vec::new(),
            confirmer: None,
            reserve: false,
            confirmer_tout: false,
            bandeau: None,
            info: None,
            dossier,
            windows: cfg!(windows),
            maj: None,
            relancer: None,
            disposition: None,
            decor: Default::default(),
            multi: None,
            annonces: Vec::new(),
            a_annoncer: Vec::new(),
            auto: false,
            lot: 1,
            auto_arret: None,
            revele_le: None,
            langue_choisie: None,
            jets: Vec::new(),
            viser: None,
            icones_nourriture: Vec::new(),
        }
    }

    #[cfg_attr(not(windows), allow(dead_code))]
    /// Fenêtre fermée (v0.12, Windows) : textures, animations en cours et images oubliées
    /// (le contexte egui est détruit avec la fenêtre) ; restent l'onglet, les messages, les
    /// mises à jour et les annonces à faire (celles qui attendaient la fin d'une éclosion
    /// partent tout de suite).
    pub fn liberer(&mut self) {
        let mut neuf = Ecloserie::new(std::mem::take(&mut self.dossier));
        neuf.onglet = self.onglet;
        neuf.index = self.index;
        neuf.bandeau = self.bandeau.take();
        neuf.windows = self.windows;
        neuf.maj = self.maj.take();
        neuf.relancer = self.relancer.take();
        neuf.disposition = self.disposition.take();
        neuf.lot = self.lot;
        neuf.langue_choisie = self.langue_choisie;
        neuf.jets = std::mem::take(&mut self.jets);
        neuf.viser = self.viser;
        neuf.annonces = std::mem::take(&mut self.annonces);
        neuf.annonces.append(&mut self.a_annoncer);
        *self = neuf;
        son::liberer();
    }

    /// Lance une éclosion (pour les captures automatiques).
    pub fn acheter(&mut self, ui: &Ui, p: &mut Partie, oeuf: usize) {
        self.acheter_lot(ui, p, oeuf, 1);
    }

    /// Ouvre `n` œufs (×1 : éclosion seule, ×3/×10 : grille de cartes).
    pub fn acheter_lot(&mut self, ui: &Ui, p: &mut Partie, oeuf: usize, n: u32) {
        let maintenant = ui.input(|i| i.time);
        let res = if n <= 1 {
            p.acheter_oeuf(oeuf).map(|id| {
                vec![crate::jeu::Ouverture {
                    id,
                    dore: p.dernier_dore,
                    pitie: p.derniere_pitie,
                }]
            })
        } else {
            p.acheter_lot(oeuf, n, crate::sauve::maintenant())
        };
        match res {
            Ok(v) => {
                son::jouer(Son::Piece, 1.0);
                self.lot = n.max(1);
                self.revele_le = None;
                let poissons: Vec<(Poisson, bool, bool)> = v
                    .iter()
                    .filter_map(|o| {
                        let f = p.poissons.iter().find(|f| f.id == o.id)?.clone();
                        Some((f, o.dore, o.pitie))
                    })
                    .collect();
                for (f, _, _) in &poissons {
                    if let Some(a) = annonce(f) {
                        self.a_annoncer.push(a);
                    }
                }
                if n <= 1 {
                    if let Some((f, dore, pitie)) = poissons.first() {
                        self.eclosion =
                            Some(Eclosion::nouvelle(f, oeuf, maintenant, *dore, *pitie));
                    }
                } else {
                    self.multi = Some(Multi::nouvelle(&poissons, oeuf, maintenant));
                }
            }
            Err(Refus::Indisponible) => {
                self.auto = false;
                self.info = Some((
                    tr("Cet œuf exceptionnel n'est pas là en ce moment.").into(),
                    maintenant,
                ))
            }
            Err(Refus::PasAssez) => {
                if self.auto {
                    self.auto = false;
                    self.auto_arret =
                        Some(tr("Ouverture auto arrêtée : plus assez d'argent.").into());
                }
                self.info = Some((tr("Pas assez d'argent pour ces œufs.").into(), maintenant))
            }
            Err(_) => {}
        }
    }

    /// Les annonces en attente partent vers le fond d'écran (fin de l'animation).
    fn liberer_annonces(&mut self) {
        self.annonces.append(&mut self.a_annoncer);
    }

    /// Captures : impose la forme du poisson de l'éclosion en cours.
    pub fn forcer_forme(&mut self, p: &mut Partie, forme: Forme) {
        let Some(e) = self.eclosion.as_mut() else {
            return;
        };
        if let Some(f) = p.poissons.iter_mut().find(|f| f.id == e.id) {
            (f.shiny, f.modif) = (forme.shiny, forme.modif);
            e.changer_poisson(f);
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, p: &mut Partie) {
        let maintenant = ui.input(|i| i.time);
        // Sons : seulement l'Écloserie (le fond d'écran reste muet).
        son::regler(p.reglages.volume, p.reglages.muet);
        for (s, h) in self
            .eclosion
            .iter_mut()
            .flat_map(|e| e.sons.drain(..))
            .chain(self.multi.iter_mut().flat_map(|m| m.sons.drain(..)))
        {
            son::jouer(s, h);
        }
        // Œufs animés au repos (onglet Œufs) : vignettes redessinées à chaque image.
        if self.onglet == Onglet::Oeufs && self.eclosion.is_none() && self.multi.is_none()
            || self.oeufs.is_empty()
        {
            for i in 0..OEUFS.len() {
                let img = image_couleur(&crate::eclosion::vignette(i, maintenant as f32));
                match self.oeufs.get_mut(i) {
                    Some(t) => t.set(img, TextureOptions::NEAREST),
                    None => self.oeufs.push(ui.ctx().load_texture(
                        format!("oeuf{i}"),
                        img,
                        TextureOptions::NEAREST,
                    )),
                }
            }
            if self.onglet == Onglet::Oeufs {
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(33));
            }
        }
        egui::Panel::top("entete")
            .frame(
                egui::Frame::new()
                    .fill(VERRE_FONCE)
                    .inner_margin(Margin::symmetric(18, 12)),
            )
            .show(ui, |ui| self.entete(ui, p));
        if let Some(l) = self.langue_choisie.take() {
            p.reglages.langue = Some(l);
        }
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(EAU)
                    .inner_margin(Margin::symmetric(18, 14)),
            )
            .show(ui, |ui| {
                if let Some((m, t)) = &self.info {
                    if maintenant - t < 3.0 {
                        ui.label(RichText::new(m).color(ALERTE));
                    } else {
                        self.info = None;
                    }
                }
                match self.onglet {
                    Onglet::Oeufs => self.onglet_oeufs(ui, p, maintenant),
                    Onglet::Aquarium => self.onglet_aquarium(ui, p),
                    Onglet::Poissodex => self.onglet_dex(ui, p),
                    Onglet::Decor => self.onglet_decor(ui, p),
                    Onglet::Nourriture => self.onglet_nourriture(ui, p),
                    Onglet::Reglages => self.onglet_reglages(ui, p),
                }
            });
    }

    fn entete(&mut self, ui: &mut Ui, p: &Partie) {
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(Vec2::splat(30.0), egui::Sense::hover());
            piece_ui(ui, r);
            ui.label(RichText::new(court(p.argent)).font(titre(30.0)).color(OR));
            ui.add_space(6.0);
            ui.label(
                RichText::new(trf(
                    "+{} par seconde",
                    &[&(court(p.revenu_par_s())).to_string()],
                ))
                .font(titre(14.0))
                .color(BRUME),
            );
            // v0.16 : bonus du repas en cours.
            if let Some(f) = p.festin {
                ui.label(
                    RichText::new(trf(
                        "Repas ×{} · {}",
                        &[&decimale(f.nourriture().mult.to_string()), &duree(f.reste)],
                    ))
                    .font(titre(14.0))
                    .color(OR),
                );
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(trf(
                        "{} / {} dans l'aquarium",
                        &[&p.nb_actifs().to_string(), &CAPACITE.to_string()],
                    ))
                    .font(titre(13.0))
                    .color(BRUME),
                );
                // Mise à jour disponible : pastille discrète qui mène aux Réglages.
                if let Some(v) = self.maj.as_ref().and_then(|m| m.disponible()) {
                    let b = egui::Button::new(
                        RichText::new(trf("Mise à jour v{}", &[&v.to_string()]))
                            .font(titre(12.0))
                            .color(ENCRE),
                    )
                    .fill(OR);
                    if ui.add(b).clicked() {
                        self.onglet = Onglet::Reglages;
                    }
                }
            });
        });
        // Premier lancement : choix de la langue (proposée : celle de Windows).
        if p.reglages.langue.is_none() {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Langue · Language")
                        .color(OR)
                        .font(titre(14.0)),
                );
                for (l, nom) in [
                    (crate::i18n::Langue::Fr, "Français"),
                    (crate::i18n::Langue::En, "English"),
                ] {
                    if ui.button(nom).clicked() {
                        crate::i18n::regler(l);
                        self.langue_choisie = Some(l);
                    }
                }
            });
        }
        if let Some(b) = self.bandeau.clone() {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(b).color(OR));
                if ui.small_button("OK").clicked() {
                    self.bandeau = None;
                }
            });
        }
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            for (o, nom) in [
                (Onglet::Oeufs, tr("Œufs")),
                (Onglet::Aquarium, tr("Mon aquarium")),
                (Onglet::Poissodex, tr("Index")),
                (Onglet::Decor, tr("Décor")),
                (Onglet::Nourriture, tr("Nourriture")),
                (Onglet::Reglages, tr("Réglages")),
            ] {
                let actif = self.onglet == o;
                let b = egui::Button::new(RichText::new(nom).font(titre(15.0)).color(if actif {
                    ENCRE
                } else {
                    ECUME
                }))
                .fill(if actif { SABLE } else { VERRE })
                .stroke(Stroke::new(2.0, if actif { ENCRE } else { BISEAU_SOMBRE }))
                .min_size(Vec2::new(0.0, 34.0));
                if ui.add(b).clicked() {
                    son::jouer(Son::Clic, 1.0);
                    self.onglet = o;
                    self.confirmer = None;
                }
            }
        });
    }

    fn onglet_oeufs(&mut self, ui: &mut Ui, p: &mut Partie, maintenant: f64) {
        if self.eclosion.is_some() {
            self.vue_eclosion(ui, p, maintenant);
            return;
        }
        if self.multi.is_some() {
            self.vue_multi(ui, p, maintenant);
            return;
        }
        let colonnes = if ui.available_width() > 1000.0 {
            5
        } else if ui.available_width() > 620.0 {
            3
        } else {
            2
        };
        let mut achat = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Ordre : œufs exceptionnels présents d'abord, puis les ordinaires, puis les absents.
            let horloge0 = crate::sauve::maintenant();
            let mut ordre: Vec<usize> = (0..OEUFS.len()).collect();
            ordre.sort_by_key(|&i| {
                let o = &OEUFS[i];
                match (o.dispo.is_some(), o.disponible(horloge0)) {
                    (true, true) => 0,
                    (false, _) => 1,
                    _ => 2,
                }
            });
            for debut in (0..OEUFS.len()).step_by(colonnes) {
                ui.columns(colonnes, |cols| {
                    for (k, ui) in cols.iter_mut().enumerate() {
                        let Some(&i) = ordre.get(debut + k) else { continue };
                        let o = &OEUFS[i];
                        biseau(o.coquille[1]).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_min_height(420.0);
                            // Colonnes justifiées : on revient à un texte aligné à gauche.
                            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                            ui.vertical_centered(|ui| {
                                if o.dispo.is_some() {
                                    let (c, t) = if o.disponible(crate::sauve::maintenant()) {
                                        (OR, tr("LIMITÉ · EN CE MOMENT"))
                                    } else {
                                        (BRUME, tr("LIMITÉ"))
                                    };
                                    ui.label(RichText::new(t).font(titre(13.0)).color(c));
                                }
                                ui.add(egui::Image::from_texture((self.oeufs[i].id(), Vec2::new(196.0, 224.0))));
                                ui.label(RichText::new(tr(o.nom)).font(titre(15.0)));
                            });
                            ui.label(RichText::new(tr(o.description)).color(BRUME).size(14.0));
                            barre_chances(ui, &o.chances);
                            for r in Rarete::TOUTES {
                                let c = o.chances[r.index()];
                                if c > 0.0 {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(r.nom()).color(c32(r.couleur())).size(14.0));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(RichText::new(format!("{} %", court_pct(c))).font(titre(13.0)));
                                        });
                                    });
                                }
                            }
                            ui.add_space(6.0);
                            let prix = p.prix_oeuf(i);
                            // Œuf exceptionnel : là encore… / revient dans…
                            let horloge = crate::sauve::maintenant();
                            let dispo = o.disponible(horloge);
                            if let Some(s) = o.attente(horloge) {
                                let (texte, c) = if dispo {
                                    (trf("Encore là {}", &[&(duree(s as f64)).to_string()]), OR)
                                } else {
                                    (trf("Revient dans {}", &[&(duree(s as f64)).to_string()]), BRUME)
                                };
                                ui.vertical_centered(|ui| ui.label(RichText::new(texte).font(titre(13.0)).color(c)));
                            }
                            let assez = p.argent >= prix && dispo;
                            let texte = RichText::new(trf("Ouvrir · {}", &[&(court(prix)).to_string()]))
                                .font(titre(14.0))
                                .color(if assez { ENCRE } else { BRUME });
                            let b = egui::Button::new(texte)
                                .fill(if assez { SABLE } else { VERRE_FONCE })
                                .stroke(Stroke::new(2.0, if assez { ENCRE } else { BISEAU_SOMBRE }))
                                .min_size(Vec2::new(ui.available_width(), 36.0));
                            if ui.add_enabled(assez, b).clicked() {
                                achat = Some((i, 1));
                            }
                            // Ouvertures multiples ×3 et ×10 (prix du lot entier).
                            ui.horizontal(|ui| {
                                let w = (ui.available_width() - 8.0) / 2.0;
                                for n in [3u32, 10] {
                                    let lot = p.prix_lot(i, n);
                                    let ok = p.argent >= lot && dispo;
                                    let b = egui::Button::new(
                                        RichText::new(format!("×{n} · {}", court(lot)))
                                            .font(titre(12.0))
                                            .color(if ok { ENCRE } else { BRUME }),
                                    )
                                    .fill(if ok { OR } else { VERRE_FONCE })
                                    .stroke(Stroke::new(2.0, if ok { ENCRE } else { BISEAU_SOMBRE }))
                                    .min_size(Vec2::new(w, 30.0));
                                    if ui.add_enabled(ok, b).clicked() {
                                        achat = Some((i, n));
                                    }
                                }
                            });
                            // Pitié : compteur avant la rareté garantie.
                            if let Some((r, reste)) = p.pitie_restante(i) {
                                let txt = if reste <= 1 {
                                    trf("{} ou mieux GARANTI au prochain !", &[r.nom()])
                                } else {
                                    trf("{} ou mieux garanti dans {} œufs", &[r.nom(), &reste.to_string()])
                                };
                                ui.vertical_centered(|ui| {
                                    ui.label(RichText::new(txt).color(c32(r.couleur())).size(14.0))
                                });
                            }
                            if !assez && dispo {
                                let r = p.revenu_par_s();
                                let t = if r > 0.0 { duree((prix - p.argent) / r) } else { tr("jamais").into() };
                                ui.vertical_centered(|ui| ui.label(RichText::new(trf("Encore {}", &[&t.to_string()])).color(BRUME).size(14.0)));
                            }
                            });
                        });
                    }
                });
                ui.add_space(8.0);
            }
            ui.add_space(8.0);
            ui.label(
                RichText::new(tr("Le prix d'un œuf monte un peu à chaque ouverture. Aquarium plein : le poisson part en réserve. Parfois, un œuf ordinaire devient un œuf doré (meilleures chances) ! Malchance : après une longue série sans, la rareté indiquée est garantie."))
                    .color(BRUME)
                    .size(14.0),
            );
        });
        if let Some((i, n)) = achat {
            self.auto_arret = None;
            self.acheter_lot(ui, p, i, n);
        }
    }

    fn vue_eclosion(&mut self, ui: &mut Ui, p: &mut Partie, maintenant: f64) {
        let Some(e) = self.eclosion.as_mut() else {
            return;
        };
        let t = maintenant - e.debut;
        e.dessiner(t);
        let img = image_couleur(&e.toile);
        match &mut e.texture {
            Some(tex) => tex.set(img, TextureOptions::NEAREST),
            None => {
                e.texture = Some(
                    ui.ctx()
                        .load_texture("eclosion", img, TextureOptions::NEAREST),
                )
            }
        }
        ui.ctx().request_repaint();
        let echelle = (ui.available_width() / 128.0)
            .min((ui.available_height() - 200.0) / 96.0)
            .floor()
            .clamp(2.0, 5.0);
        let revele = e.revele(maintenant);
        let (oeuf, id) = (e.oeuf, e.id);
        ui.vertical_centered(|ui| {
            let tex = e.texture.as_ref().expect("texture d'éclosion");
            let r = ui.add(
                egui::Image::from_texture((tex.id(), Vec2::new(128.0, 96.0) * echelle))
                    .sense(egui::Sense::click()),
            );
            if !revele {
                if r.clicked() {
                    e.passer(maintenant);
                }
                ui.label(
                    RichText::new(trf(
                        "{} éclot…",
                        &[(if e.dore {
                            tr("Œuf doré")
                        } else {
                            tr(OEUFS[e.oeuf].nom)
                        })],
                    ))
                    .font(titre(18.0))
                    .color(BRUME),
                );
                ui.label(
                    RichText::new(tr("Clique pour passer"))
                        .color(BRUME)
                        .size(14.0),
                );
                return;
            }
            let Some(f) = p.poissons.iter().find(|f| f.id == e.id) else {
                return;
            };
            let nom = nom_de(f);
            if f.shiny {
                ui.label(texte_shiny(maintenant, 30.0));
            }
            ui.label(texte_rarete(e.rarete, maintenant, 34.0));
            ui.label(RichText::new(nom).size(22.0));
            if let Some(m) = f.modif {
                ui.label(
                    RichText::new(trf(
                        "{} · revenu ×{}",
                        &[m.nom(), &(court_pct(m.multiplicateur())).to_string()],
                    ))
                    .font(titre(16.0))
                    .color(c32(m.couleur())),
                );
            }
            ui.label(
                RichText::new(trf(
                    "Rapporte {}/s · taille {} · {}",
                    &[
                        &(court(f.revenu())).to_string(),
                        &(taille(f.taille)).to_string(),
                        (if f.actif {
                            tr("nage déjà dans ton aquarium")
                        } else {
                            tr("rangé dans la réserve (aquarium plein)")
                        }),
                    ],
                ))
                .color(BRUME)
                .font(titre(13.0)),
            );
        });
        self.suite(ui, p, oeuf, &[id], revele, maintenant);
    }

    /// Ouverture ×3/×10 : la grille, puis le résumé.
    fn vue_multi(&mut self, ui: &mut Ui, p: &mut Partie, maintenant: f64) {
        let Some(m) = self.multi.as_mut() else {
            return;
        };
        m.dessiner(maintenant - m.debut);
        let img = image_couleur(&m.toile);
        match &mut m.texture {
            Some(tex) => tex.set(img, TextureOptions::NEAREST),
            None => m.texture = Some(ui.ctx().load_texture("multi", img, TextureOptions::NEAREST)),
        }
        ui.ctx().request_repaint();
        let (ml, mh) = (
            crate::eclosion::multi::ML as f32,
            crate::eclosion::multi::MH as f32,
        );
        let echelle = (ui.available_width() / ml)
            .min((ui.available_height() - 170.0) / mh)
            .floor()
            .clamp(2.0, 5.0);
        let revele = m.revele(maintenant);
        let (oeuf, ids) = (m.oeuf, m.ids());
        ui.vertical_centered(|ui| {
            let tex = m.texture.as_ref().expect("texture de la grille");
            let r = ui.add(
                egui::Image::from_texture((tex.id(), Vec2::new(ml, mh) * echelle))
                    .sense(egui::Sense::click()),
            );
            if !revele {
                if r.clicked() {
                    m.passer(maintenant);
                }
                ui.label(
                    RichText::new(trf(
                        "{} ×{} éclosent…",
                        &[tr(OEUFS[oeuf].nom), &ids.len().to_string()],
                    ))
                    .font(titre(18.0))
                    .color(BRUME),
                );
                ui.label(
                    RichText::new(tr("Clique pour passer"))
                        .color(BRUME)
                        .size(14.0),
                );
                return;
            }
            // Résumé : la meilleure, puis le revenu ajouté.
            let fs: Vec<&Poisson> = ids
                .iter()
                .filter_map(|id| p.poissons.iter().find(|f| f.id == *id))
                .collect();
            if let Some(b) = fs
                .iter()
                .max_by_key(|f| (f.rarete, f.shiny, f.modif.is_some()))
            {
                ui.label(texte_rarete(b.rarete, maintenant, 28.0));
                ui.label(
                    RichText::new(trf("Meilleure : {}", &[&(nom_de(b)).to_string()])).size(20.0),
                );
            }
            let gain: f64 = fs.iter().map(|f| f.revenu()).sum();
            let shiny = fs.iter().filter(|f| f.shiny).count();
            let modif = fs.iter().filter(|f| f.modif.is_some()).count();
            ui.label(
                RichText::new(trf(
                    "+{}/s en tout · {} shiny · {} modifié{}",
                    &[
                        &(court(gain)).to_string(),
                        &shiny.to_string(),
                        &modif.to_string(),
                        (if modif > 1 { "s" } else { "" }),
                    ],
                ))
                .color(BRUME)
                .font(titre(13.0)),
            );
        });
        self.suite(ui, p, oeuf, &ids, revele, maintenant);
    }

    /// Sous une éclosion révélée : Encore (même lot), Retour, Ouverture auto (avec sa
    /// condition d'arrêt). L'auto relance seul 1 s après la révélation.
    fn suite(
        &mut self,
        ui: &mut Ui,
        p: &mut Partie,
        oeuf: usize,
        ids: &[u64],
        revele: bool,
        maintenant: f64,
    ) {
        let mut fini = false;
        let mut encore = false;
        if revele && self.revele_le.is_none() {
            self.revele_le = Some(maintenant);
            self.liberer_annonces();
            let r = &p.reglages;
            let arret = ids
                .iter()
                .filter_map(|id| p.poissons.iter().find(|f| f.id == *id))
                .filter(|f| f.rarete.index() >= r.auto_rarete as usize || (r.auto_shiny && f.shiny))
                .max_by_key(|f| (f.rarete, f.shiny));
            if let (true, Some(f)) = (self.auto, arret) {
                self.auto = false;
                self.auto_arret = Some(trf(
                    "Ouverture auto arrêtée : {}{} !",
                    &[f.rarete.nom(), (if f.shiny { tr(" shiny") } else { "" })],
                ));
            }
        }
        if self.auto && revele && self.revele_le.is_some_and(|t| maintenant - t > 1.0) {
            encore = true;
        }
        ui.add_space(6.0);
        if let Some(m) = &self.auto_arret {
            ui.vertical_centered(|ui| ui.label(RichText::new(m).color(OR).font(titre(14.0))));
        }
        let lot = self.lot;
        let prix = if lot <= 1 {
            p.prix_oeuf(oeuf)
        } else {
            p.prix_lot(oeuf, lot)
        };
        if revele {
            ui.horizontal(|ui| {
                let w = 220.0;
                ui.add_space((ui.available_width() - 2.0 * w - 8.0).max(0.0) / 2.0);
                let assez = p.argent >= prix;
                let texte = if lot <= 1 {
                    trf("Encore un · {}", &[&(court(prix)).to_string()])
                } else {
                    trf(
                        "Encore ×{} · {}",
                        &[&lot.to_string(), &(court(prix)).to_string()],
                    )
                };
                let b = egui::Button::new(RichText::new(texte).font(titre(14.0)).color(if assez {
                    ENCRE
                } else {
                    BRUME
                }))
                .fill(if assez { SABLE } else { VERRE_FONCE })
                .min_size(Vec2::new(w, 38.0));
                if ui.add_enabled(assez, b).clicked() {
                    encore = true;
                    self.auto_arret = None;
                }
                if ui
                    .add(
                        egui::Button::new(RichText::new(tr("Retour aux œufs")).font(titre(14.0)))
                            .min_size(Vec2::new(w, 38.0)),
                    )
                    .clicked()
                {
                    fini = true;
                    self.auto = false;
                }
            });
        }
        // Ouverture auto et sa condition d'arrêt.
        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 520.0).max(0.0) / 2.0);
            let avant = self.auto;
            ui.checkbox(
                &mut self.auto,
                RichText::new(tr("Ouverture auto")).font(titre(13.0)),
            );
            if self.auto && !avant {
                self.auto_arret = None;
                if revele {
                    self.revele_le = Some(maintenant - 1.0);
                }
            }
            let r = &mut p.reglages;
            ui.label(RichText::new(tr("s'arrête sur")).color(BRUME).size(14.0));
            egui::ComboBox::from_id_salt("auto_rarete")
                .selected_text(Rarete::TOUTES[(r.auto_rarete as usize).min(8)].nom())
                .show_ui(ui, |ui| {
                    for (k, rr) in Rarete::TOUTES.iter().enumerate().skip(2) {
                        ui.selectable_value(&mut r.auto_rarete, k as u8, format!("{}+", rr.nom()));
                    }
                });
            ui.checkbox(&mut r.auto_shiny, tr("ou un shiny"));
        });
        if fini || encore {
            self.liberer_annonces();
            self.eclosion = None;
            self.multi = None;
        }
        if encore {
            self.acheter_lot(ui, p, oeuf, lot);
        }
    }

    fn vignette(&mut self, ctx: &egui::Context, f: &Poisson) -> TextureHandle {
        self.vignettes
            .entry(f.graine)
            .or_insert_with(|| {
                let s = sprite_ajuste(f, 108, 48);
                ctx.load_texture(
                    format!("p{}", f.graine),
                    image_sprite(&s, None),
                    TextureOptions::NEAREST,
                )
            })
            .clone()
    }

    fn onglet_aquarium(&mut self, ui: &mut Ui, p: &mut Partie) {
        let n_reserve = p.poissons.len() - p.nb_actifs();
        ui.horizontal(|ui| {
            if ui
                .selectable_label(
                    !self.reserve,
                    RichText::new(trf(
                        "Dans l'aquarium ({}/{})",
                        &[&p.nb_actifs().to_string(), &CAPACITE.to_string()],
                    ))
                    .font(titre(14.0)),
                )
                .clicked()
            {
                self.reserve = false;
            }
            if ui
                .selectable_label(
                    self.reserve,
                    RichText::new(trf("Réserve ({})", &[&n_reserve.to_string()])).font(titre(14.0)),
                )
                .clicked()
            {
                self.reserve = true;
            }
        });
        // « Équiper les meilleurs » : les plus rentables dans l'aquarium, le reste en réserve.
        ui.horizontal(|ui| {
            let b = egui::Button::new(
                RichText::new(tr("Équiper les meilleurs"))
                    .font(titre(14.0))
                    .color(ENCRE),
            )
            .fill(SABLE)
            .min_size(Vec2::new(220.0, 34.0));
            if ui.add(b).clicked() {
                let (avant, apres) = p.equiper_meilleurs();
                let d = apres - avant;
                self.bandeau = Some(trf(
                    "Meilleurs équipés : {}/s ({}{}/s)",
                    &[
                        &(court(apres)).to_string(),
                        (if d >= 0.0 { "+" } else { "-" }),
                        &(court(d.abs())).to_string(),
                    ],
                ));
            }
            ui.checkbox(
                &mut p.reglages.equiper_auto,
                RichText::new(tr(
                    "Auto : un nouveau poisson meilleur remplace le plus faible",
                ))
                .size(13.0),
            );
        });
        ui.label(
            RichText::new(if self.reserve {
                tr("Les poissons de la réserve ne rapportent rien. Sors-les quand il y a de la place.")
            } else {
                tr("Nourrir fait monter le niveau : +20 % de revenu par niveau, jusqu'au niveau 10.")
            })
            .color(BRUME)
            .size(14.0),
        );
        // v0.16 : « Tout vendre » la réserve (les créatures verrouillées restent), confirmé.
        if self.reserve {
            let (n, total) = p.prix_tout_vendre();
            ui.horizontal(|ui| {
                if !self.confirmer_tout {
                    let b = egui::Button::new(
                        RichText::new(trf(
                            "Tout vendre ({} · +{})",
                            &[&n.to_string(), &court(total)],
                        ))
                        .font(titre(13.0)),
                    )
                    .min_size(Vec2::new(0.0, 30.0));
                    if ui
                        .add_enabled(n > 0, b)
                        .on_hover_text(tr("Les créatures verrouillées (cadenas) sont gardées."))
                        .clicked()
                    {
                        self.confirmer_tout = true;
                    }
                } else {
                    ui.label(
                        RichText::new(trf(
                            "Vendre {} créatures pour +{} ?",
                            &[&n.to_string(), &court(total)],
                        ))
                        .font(titre(13.0))
                        .color(OR),
                    );
                    let b = egui::Button::new(
                        RichText::new(tr("Confirmer"))
                            .font(titre(13.0))
                            .color(ENCRE),
                    )
                    .fill(ALERTE);
                    if ui.add(b).clicked() {
                        let (n, total) = p.tout_vendre();
                        son::jouer(Son::Piece, 1.0);
                        self.bandeau = Some(trf(
                            "{} créatures vendues : +{}",
                            &[&n.to_string(), &court(total)],
                        ));
                        self.confirmer_tout = false;
                    }
                    if ui.button(tr("Annuler")).clicked() {
                        self.confirmer_tout = false;
                    }
                }
            });
        } else {
            self.confirmer_tout = false;
        }
        ui.add_space(4.0);
        let mut liste: Vec<Poisson> = p
            .poissons
            .iter()
            .filter(|f| f.actif != self.reserve)
            .cloned()
            .collect();
        liste.sort_by(|a, b| b.revenu().total_cmp(&a.revenu()));
        if liste.is_empty() {
            ui.add_space(30.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(if self.reserve {
                        tr("Réserve vide.")
                    } else {
                        tr("Aquarium vide : ouvre un œuf.")
                    })
                    .color(BRUME),
                );
            });
            return;
        }
        let mut action: Option<(u64, u8)> = None;
        let hauteur = 64.0;
        egui::ScrollArea::vertical().show_rows(ui, hauteur, liste.len(), |ui, lignes| {
            for f in &liste[lignes] {
                let tex = self.vignette(ui.ctx(), f);
                let fond = if f.rarete >= Rarete::Epique {
                    VERRE
                } else {
                    VERRE_FONCE
                };
                egui::Frame::new()
                    .fill(fond)
                    .stroke(Stroke::new(1.0, BISEAU_SOMBRE))
                    .inner_margin(Margin::symmetric(10, 4))
                    .show(ui, |ui| {
                        ui.set_min_height(hauteur - 14.0);
                        ui.horizontal(|ui| {
                            // Liseré de rareté.
                            let (r, _) =
                                ui.allocate_exact_size(Vec2::new(4.0, 44.0), egui::Sense::hover());
                            ui.painter().rect_filled(r, 0.0, c32(f.rarete.couleur()));
                            let s = tex.size_vec2() * 2.0;
                            let (r, _) = ui
                                .allocate_exact_size(Vec2::new(110.0, 48.0), egui::Sense::hover());
                            let s = if s.x > r.width() { tex.size_vec2() } else { s };
                            ui.painter().image(
                                tex.id(),
                                egui::Rect::from_center_size(r.center().round(), s),
                                egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                ),
                                Color32::WHITE,
                            );
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 2.0;
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(nom_de(f)).size(17.0));
                                    badges(ui, f);
                                });
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(f.rarete.nom())
                                            .color(c32(f.rarete.couleur()))
                                            .size(14.0),
                                    );
                                    ui.label(
                                        RichText::new(trf(
                                            "niv. {} · taille {} · {}/s",
                                            &[
                                                &f.niveau.to_string(),
                                                &(taille(f.taille)).to_string(),
                                                &(court(f.revenu())).to_string(),
                                            ],
                                        ))
                                        .color(BRUME)
                                        .font(titre(12.0)),
                                    );
                                });
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    // v0.16 : cadenas (un clic le pose ou le retire).
                                    let (r, rep) = ui.allocate_exact_size(
                                        Vec2::new(28.0, 32.0),
                                        egui::Sense::click(),
                                    );
                                    cadenas(ui, r, f.verrou, rep.hovered());
                                    let rep = rep.on_hover_text(if f.verrou {
                                        tr("Verrouillé : ni vendu ni relâché. Clic pour ouvrir.")
                                    } else {
                                        tr("Verrouiller (protège de « Tout vendre »).")
                                    });
                                    if rep.clicked() {
                                        action = Some((f.id, 3));
                                    }
                                    let confirme = self.confirmer == Some(f.id);
                                    let t = if confirme {
                                        trf(
                                            "Relâcher ? +{}",
                                            &[&(court(f.prix_relache())).to_string()],
                                        )
                                    } else {
                                        trf(
                                            "Relâcher +{}",
                                            &[&(court(f.prix_relache())).to_string()],
                                        )
                                    };
                                    let b = egui::Button::new(
                                        RichText::new(t).font(titre(12.0)).color(if confirme {
                                            ENCRE
                                        } else {
                                            ECUME
                                        }),
                                    )
                                    .fill(if confirme { ALERTE } else { VERRE_FONCE });
                                    if ui
                                        .add_enabled(!f.verrou, b)
                                        .on_disabled_hover_text(tr("Verrouillé"))
                                        .clicked()
                                    {
                                        // Peu commun et plus : il faut confirmer.
                                        if confirme || f.rarete == Rarete::Commun {
                                            action = Some((f.id, 2));
                                        } else {
                                            self.confirmer = Some(f.id);
                                        }
                                    }
                                    let plein = p.nb_actifs() >= CAPACITE;
                                    let (t, ok) = if f.actif {
                                        (tr("Ranger"), true)
                                    } else {
                                        (tr("Sortir"), !plein)
                                    };
                                    if ui
                                        .add_enabled(
                                            ok,
                                            egui::Button::new(RichText::new(t).font(titre(12.0))),
                                        )
                                        .on_disabled_hover_text(tr("Aquarium plein"))
                                        .clicked()
                                    {
                                        action = Some((f.id, 1));
                                    }
                                    if let Some(c) = f.cout_nourrir() {
                                        let assez = p.argent >= c;
                                        let b = egui::Button::new(
                                            RichText::new(trf(
                                                "Nourrir · {}",
                                                &[&(court(c)).to_string()],
                                            ))
                                            .font(titre(12.0))
                                            .color(if assez { ENCRE } else { BRUME }),
                                        )
                                        .fill(if assez { SABLE } else { VERRE_FONCE });
                                        if ui.add_enabled(assez, b).clicked() {
                                            action = Some((f.id, 0));
                                        }
                                    } else {
                                        ui.label(RichText::new(tr("Niveau max")).color(OR));
                                    }
                                },
                            );
                        });
                    });
            }
        });
        if let Some((id, a)) = action {
            let _ = match a {
                0 => p.nourrir(id),
                1 => p.basculer(id),
                3 => p.verrouiller(id),
                _ => p.relacher(id).map(|_| ()),
            };
            self.confirmer = None;
        }
    }

    /// Index façon Roblox : un onglet par catégorie (Normal, chaque modificateur, Shiny),
    /// toutes les espèces listées (couleur si découverte, silhouette sinon), % de complétion.
    fn onglet_dex(&mut self, ui: &mut Ui, p: &Partie) {
        ui.horizontal_wrapped(|ui| {
            for k in 0..NB_INDEX {
                let (n, t) = p.completion_index(k);
                let actif = self.index == k;
                let couleur = couleur_index(k);
                let texte = format!("{} {}%", nom_index(k), n * 100 / t.max(1));
                let b = egui::Button::new(RichText::new(texte).font(titre(13.0)).color(if actif {
                    ENCRE
                } else {
                    c32(couleur)
                }))
                .fill(if actif { c32(couleur) } else { VERRE })
                .stroke(Stroke::new(
                    2.0,
                    if n == t {
                        OR
                    } else if actif {
                        ENCRE
                    } else {
                        BISEAU_SOMBRE
                    },
                ))
                .min_size(Vec2::new(0.0, 30.0));
                if ui.add(b).clicked() {
                    self.index = k;
                }
            }
        });
        let k = self.index.min(NB_INDEX - 1);
        let (n, total) = p.completion_index(k);
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(trf(
                    "Index {} : {}/{} ({} %)",
                    &[
                        (nom_index(k)),
                        &n.to_string(),
                        &total.to_string(),
                        &(n * 100 / total.max(1)).to_string(),
                    ],
                ))
                .font(titre(16.0))
                .color(c32(couleur_index(k))),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(trf(
                        "{} index complets : +{} % de revenu",
                        &[
                            &p.index_complets().to_string(),
                            &(court_pct(p.index_complets() as f64 * BONUS_INDEX * 100.0))
                                .to_string(),
                        ],
                    ))
                    .font(titre(12.0))
                    .color(OR),
                );
            });
        });
        let regle = match k {
            0 => tr("Chaque espèce éclose (sous n'importe quelle forme) compte.").to_string(),
            k if k < NB_INDEX - 1 => {
                let m = Modif::TOUS[k - 1];
                trf(
                    "Espèces écloses avec le modificateur {} (1 chance sur {}, revenu ×{}).",
                    &[
                        m.nom(),
                        &((1.0 / m.chance()).round()).to_string(),
                        &(court_pct(m.multiplicateur())).to_string(),
                    ],
                )
            }
            _ => trf(
                "Espèces écloses shiny (1 chance sur {}, revenu ×{}).",
                &[
                    &((1.0 / crate::jeu::CHANCE_SHINY).round()).to_string(),
                    &(court_pct(MULT_SHINY)).to_string(),
                ],
            ),
        };
        ui.label(
            RichText::new(trf(
                "{} Index complet : +{} % de revenu pour toujours.",
                &[
                    &regle.to_string(),
                    &(court_pct(BONUS_INDEX * 100.0)).to_string(),
                ],
            ))
            // Chiffres en Silkscreen (dans Pixelify Sans, le 5 ressemble à un S).
            .color(BRUME)
            .font(titre(11.0)),
        );
        ui.add_space(6.0);
        egui::ScrollArea::vertical().show(ui, |ui| {
            for r in Rarete::TOUTES {
                let especes: Vec<&Espece> = de_rarete(r).collect();
                let n = especes.iter().filter(|e| p.dans_index(e.cle, k)).count();
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(r.nom())
                            .font(titre(15.0))
                            .color(c32(r.couleur())),
                    );
                    ui.label(
                        RichText::new(format!("{n}/{}", especes.len()))
                            .color(BRUME)
                            .font(titre(12.0)),
                    );
                });
                ui.horizontal_wrapped(|ui| {
                    for e in especes {
                        self.carte_espece(ui, p, e, k);
                    }
                });
                ui.add_space(8.0);
            }
        });
    }

    /// Case d'une espèce dans l'index `k` : dessin de cette forme (silhouette si pas encore
    /// découverte), nom, nombre d'éclos.
    fn carte_espece(&mut self, ui: &mut Ui, p: &Partie, e: &'static Espece, k: usize) {
        let n = p.vus.get(e.cle).copied().unwrap_or(0);
        let vu = p.dans_index(e.cle, k);
        // 122 px : 7 cases par ligne dans la fenêtre de 960 px.
        let (rect, _) = ui.allocate_exact_size(Vec2::new(122.0, 96.0), egui::Sense::hover());
        let peintre = ui.painter();
        let bord = if vu {
            e.rarete.couleur()
        } else {
            [0x24, 0x40, 0x54]
        };
        peintre.rect_filled(rect, 0.0, VERRE_FONCE);
        peintre.rect_stroke(
            rect.shrink(0.5),
            0.0,
            Stroke::new(1.0, c32(bord)),
            egui::StrokeKind::Inside,
        );
        let cle = format!("dex/{}/{k}/{vu}", e.cle);
        let tex = self
            .vignettes_dex
            .entry(cle.clone())
            .or_insert_with(|| {
                let m = 2.0 * crate::poisson::MARGE as f32;
                let z = ((108.0 - m) / e.dims.0 as f32)
                    .min((54.0 - m) / e.dims.1 as f32)
                    .min(1.0);
                let forme = forme_index(k);
                let s = crate::poisson::sprite_forme(e, 0, z, forme);
                let ombre = (!vu).then_some([0x05, 0x14, 0x20]);
                ui.ctx()
                    .load_texture(&cle, image_sprite(&s, ombre), TextureOptions::NEAREST)
            })
            .clone();
        let zone =
            egui::Rect::from_min_size(rect.min + Vec2::new(4.0, 4.0), Vec2::new(114.0, 58.0));
        let taille = tex.size_vec2();
        let z = (zone.width() / taille.x)
            .min(zone.height() / taille.y)
            .floor()
            .clamp(1.0, 3.0);
        peintre.image(
            tex.id(),
            egui::Rect::from_center_size(zone.center().round(), taille * z),
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        let (nom, c) = if vu {
            (tr(e.nom).to_string(), Color32::WHITE)
        } else {
            ("???".to_string(), BRUME)
        };
        peintre.text(
            egui::pos2(rect.center().x, rect.top() + 70.0),
            egui::Align2::CENTER_CENTER,
            nom,
            FontId::proportional(13.0),
            c,
        );
        let bas = if !vu {
            tr("À découvrir").into()
        } else if k == 0 {
            trf("{} éclos", &[&n.to_string()])
        } else {
            tr("Découvert").into()
        };
        peintre.text(
            egui::pos2(rect.center().x, rect.top() + 86.0),
            egui::Align2::CENTER_CENTER,
            bas,
            titre(10.0),
            BRUME,
        );
    }

    fn onglet_reglages(&mut self, ui: &mut Ui, p: &mut Partie) {
        let r = &mut p.reglages;
        let largeur = ui.available_width().min(640.0);
        let ligne = |ui: &mut Ui, titre_: &str, aide: &str, contenu: &mut dyn FnMut(&mut Ui)| {
            biseau([0x24, 0x40, 0x54]).show(ui, |ui| {
                ui.set_width(largeur);
                ui.label(RichText::new(titre_).font(titre(15.0)));
                ui.label(RichText::new(aide).color(BRUME).size(14.0));
                contenu(ui);
            });
            ui.add_space(6.0);
        };
        let maj = self.maj.as_ref();
        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Some(m) = maj {
                ligne(
                    ui,
                    tr("Mises à jour"),
                    &trf("Version actuelle : {}. L'appli se met à jour elle-même (rien à dézipper).", &[(crate::maj::VERSION)]),
                    &mut |ui| section_maj(ui, m),
                );
            }
            ligne(
                ui,
                tr("Langue"),
                tr("La langue de l'interface (les noms des créatures aussi)."),
                &mut |ui| {
                    let mut l = r.langue.unwrap_or_else(crate::i18n::langue_systeme);
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut l, crate::i18n::Langue::Fr, "Français");
                        ui.radio_value(&mut l, crate::i18n::Langue::En, "English");
                    });
                    if Some(l) != r.langue
                        && (r.langue.is_some() || l != crate::i18n::langue_systeme())
                    {
                        r.langue = Some(l);
                        crate::i18n::regler(l);
                    }
                },
            );
            ligne(
                ui,
                tr("Sons"),
                tr("Seulement dans l'Écloserie (éclosions, cartes, achats) : le fond d'écran reste toujours silencieux."),
                &mut |ui| {
                    let mut pct = (r.volume * 100.0).round();
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(
                                !r.muet,
                                egui::Slider::new(&mut pct, 0.0..=100.0)
                                    .suffix(" %")
                                    .step_by(5.0)
                                    .fixed_decimals(0),
                            )
                            .changed()
                        {
                            r.volume = pct / 100.0;
                        }
                        ui.checkbox(&mut r.muet, tr("Couper le son"));
                    });
                },
            );
            ligne(
                ui,
                tr("Décor"),
                tr("Le décor du fond de l'aquarium (les objets achetés restent posés)."),
                &mut |ui| {
                    for (i, nom) in crate::decor::THEMES.iter().enumerate() {
                        ui.radio_value(&mut r.theme, i as u8, tr(nom));
                    }
                },
            );
            ligne(
                ui,
                tr("Poissons et souris"),
                "Garde le clic appuyé un instant sur un poisson (bureau vide, pas sur une icône) \
                 pour l'attraper et le déplacer ; un clic bref montre sa fiche.",
                &mut |ui| {
                    ui.checkbox(&mut r.attraper, tr("Attraper les poissons"));
                    ui.add_enabled(
                        r.attraper,
                        egui::Checkbox::new(&mut r.attraper_alt, tr("Seulement avec Alt")),
                    );
                },
            );
            ligne(
                ui,
                tr("Teinte de l'eau"),
                tr("0 % : ton fond d'écran reste intact derrière les poissons."),
                &mut |ui| {
                    let mut pct = (r.teinte * 100.0).round();
                    if ui
                        .add(
                            egui::Slider::new(&mut pct, 0.0..=100.0)
                                .suffix(" %")
                                .step_by(5.0)
                                .fixed_decimals(0),
                        )
                        .changed()
                    {
                        r.teinte = pct / 100.0;
                    }
                },
            );
            ligne(
                ui,
                tr("Poissons dessinés"),
                tr("Les plus rentables sont dessinés ; tous rapportent quand même."),
                &mut |ui| {
                    ui.add(egui::Slider::new(&mut r.max_affiches, 5..=CAPACITE));
                },
            );
            ligne(
                ui,
                tr("Fluidité"),
                tr("24 images par seconde suffisent et ménagent le processeur."),
                &mut |ui| {
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut r.ips, 15, "15 i/s");
                        ui.radio_value(&mut r.ips, 24, "24 i/s");
                        ui.radio_value(&mut r.ips, 30, "30 i/s");
                        ui.radio_value(&mut r.ips, 60, "60 i/s");
                    });
                },
            );
            let windows = self.windows;
            ligne(
                ui,
                tr("Démarrage"),
                if windows {
                    tr("Deskquarium se lance tout seul à l'ouverture de ta session.")
                } else {
                    tr("Disponible seulement sous Windows.")
                },
                &mut |ui| {
                    ui.add_enabled(
                        windows,
                        egui::Checkbox::new(
                            &mut r.demarrage_windows,
                            tr("Lancer au démarrage de Windows"),
                        ),
                    );
                },
            );
            ligne(
                ui,
                tr("Jeux et vidéos"),
                tr("Animation en pause tant qu'une appli est en plein écran (l'argent continue)."),
                &mut |ui| {
                    ui.checkbox(&mut r.pause_plein_ecran, tr("Pause en plein écran"));
                },
            );
            ligne(
                ui,
                tr("Affichage"),
                tr("Si l'aquarium n'apparaît pas derrière les icônes : fenêtre sans bord au fond."),
                &mut |ui| {
                    ui.checkbox(&mut r.mode_fenetre, tr("Mode fenêtre"));
                },
            );
            ligne(
                ui,
                tr("Mode Abysses"),
                tr("Aquarium des grands fonds : eau sombre, neige marine, poissons qui luisent."),
                &mut |ui| {
                    let avant = r.abysses_actif();
                    if ui
                        .checkbox(&mut r.abysses_auto, tr("Automatique la nuit (21 h – 7 h)"))
                        .changed()
                        && !r.abysses_auto
                    {
                        // Sortie de l'automatique : garde l'état affiché, pas de saut.
                        r.abysses = avant;
                    }
                    // Automatique : la coche manuelle montre l'état du moment, sans effet.
                    let mut actif = r.abysses_actif();
                    let c = if r.abysses_auto { &mut actif } else { &mut r.abysses };
                    ui.add_enabled(!r.abysses_auto, egui::Checkbox::new(c, tr("Mode Abysses")));
                },
            );
            ui.label(
                RichText::new(trf("Sauvegarde : {}", &[&self.dossier.to_string()]))
                    .color(BRUME)
                    .size(13.0),
            );
            ui.label(
                RichText::new(tr("Raccourci : Ctrl + Alt + A ouvre l'Écloserie."))
                    .color(BRUME)
                    .size(13.0),
            );
        });
    }
}

/// Section « Mises à jour » des Réglages : recherche, notes de version, installation avec
/// barre de progression ; messages d'erreur clairs (rien n'est changé en cas d'échec).
fn section_maj(ui: &mut Ui, m: &crate::maj::Maj) {
    use crate::maj::Etat;
    let chercher = |ui: &mut Ui, texte: &str| {
        if ui
            .add(egui::Button::new(RichText::new(texte).font(titre(13.0))))
            .clicked()
        {
            m.chercher(false);
        }
    };
    match m.etat() {
        Etat::Rien => chercher(ui, tr("Rechercher une mise à jour")),
        Etat::Recherche => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(RichText::new(tr("Recherche en cours…")).color(BRUME));
            });
        }
        Etat::AJour => {
            ui.label(RichText::new(tr("Tu as la dernière version.")).color(OR));
            chercher(ui, tr("Rechercher à nouveau"));
        }
        Etat::Erreur(e) => {
            ui.label(RichText::new(e).color(ALERTE));
            chercher(ui, tr("Réessayer"));
        }
        Etat::Dispo(a) => {
            ui.label(
                RichText::new(trf(
                    "Nouvelle version {} disponible",
                    &[&a.version.to_string()],
                ))
                .font(titre(14.0))
                .color(OR),
            );
            if !a.notes.trim().is_empty() {
                egui::ScrollArea::vertical()
                    .id_salt("notes_maj")
                    .max_height(140.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new(a.notes.trim()).size(14.0));
                    });
            }
            let b = egui::Button::new(
                RichText::new(trf("Installer v{}", &[&a.version.to_string()]))
                    .font(titre(14.0))
                    .color(ENCRE),
            )
            .fill(SABLE)
            .min_size(Vec2::new(220.0, 34.0));
            if ui.add(b).clicked() {
                m.telecharger();
            }
            ui.label(
                RichText::new(tr("L'appli redémarre toute seule après l'installation."))
                    .color(BRUME)
                    .size(13.0),
            );
        }
        Etat::Telechargement(v, n, t) => {
            ui.label(
                RichText::new(trf("Téléchargement de la version {}…", &[&v.to_string()]))
                    .color(BRUME),
            );
            ui.add(
                egui::ProgressBar::new(n as f32 / t.max(1) as f32)
                    .show_percentage()
                    .desired_width(ui.available_width()),
            );
        }
        Etat::Pret(_) | Etat::Installation => {
            ui.label(RichText::new(tr("Installation, redémarrage…")).color(OR));
        }
        Etat::Echec(e) => {
            ui.label(RichText::new(e).color(ALERTE));
            chercher(ui, tr("Réessayer"));
        }
    }
}

/// Cadre « verre » : biseau clair en haut à gauche, sombre en bas à droite, liseré coloré.
fn biseau(liseré: Rgb) -> egui::Frame {
    egui::Frame::new()
        .fill(VERRE)
        .stroke(Stroke::new(
            2.0,
            c32(melange(liseré, [0x16, 0x45, 0x60], 0.35)),
        ))
        .inner_margin(Margin::same(12))
        .shadow(egui::Shadow {
            offset: [4, 4],
            blur: 0,
            spread: 0,
            color: BISEAU_SOMBRE,
        })
}

/// Forme dessinée dans l'index `k`.
fn forme_index(k: usize) -> Forme {
    Forme {
        shiny: k == NB_INDEX - 1,
        modif: (1..NB_INDEX - 1).contains(&k).then(|| Modif::TOUS[k - 1]),
    }
}

fn couleur_index(k: usize) -> Rgb {
    match k {
        0 => [0xe8, 0xf6, 0xf2],
        k if k < NB_INDEX - 1 => Modif::TOUS[k - 1].couleur(),
        _ => [0xff, 0xb8, 0xe8],
    }
}

/// Badges shiny et modificateur (liste de « Mon aquarium »).
fn badges(ui: &mut Ui, f: &Poisson) {
    if f.shiny {
        ui.label(RichText::new(tr("SHINY")).font(titre(12.0)).color(SHINY))
            .on_hover_text(trf(
                "Shiny : revenu ×{}",
                &[&(court_pct(MULT_SHINY)).to_string()],
            ));
    }
    if let Some(m) = f.modif {
        ui.label(
            RichText::new(m.nom().to_uppercase())
                .font(titre(12.0))
                .color(c32(m.couleur())),
        )
        .on_hover_text(trf(
            "{} : revenu ×{}",
            &[m.nom(), &(court_pct(m.multiplicateur())).to_string()],
        ));
    }
}

/// « SHINY ! » en lettres qui scintillent (révélation).
fn texte_shiny(t: f64, taille: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    for (i, ch) in tr("SHINY !").chars().enumerate() {
        let k = 0.5 + 0.5 * (t as f32 * 6.0 - i as f32 * 0.9).sin();
        let c = c32(melange([0xff, 0x9c, 0xdc], [0xff, 0xf6, 0xc0], k));
        job.append(
            &ch.to_string(),
            0.0,
            egui::TextFormat::simple(titre(taille), c),
        );
    }
    job
}

fn taille(t: f32) -> String {
    decimale(format!("{t:.2}"))
}

fn court_pct(c: f64) -> String {
    if c.fract() == 0.0 {
        format!("{c:.0}")
    } else {
        decimale(format!("{c:.1}"))
    }
}

/// Barre des chances : un segment par rareté, largeur ∝ probabilité (minimum visible).
fn barre_chances(ui: &mut Ui, chances: &[f64; 9]) {
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 10.0), egui::Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, 0.0, BISEAU_SOMBRE);
    let total: f64 = chances
        .iter()
        .map(|&c| if c > 0.0 { c.max(2.0) } else { 0.0 })
        .sum();
    let mut x = rect.left();
    for r in Rarete::TOUTES {
        let c = chances[r.index()];
        if c <= 0.0 {
            continue;
        }
        let w = (c.max(2.0) / total) as f32 * rect.width();
        p.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(x.round(), rect.top()),
                Vec2::new(w.round().max(2.0), rect.height()),
            ),
            0.0,
            c32(r.couleur()),
        );
        x += w;
    }
}

/// Nom de la rareté en grand ; Irréel : lettres arc-en-ciel qui défilent ; Préhistorique :
/// ambre qui luit par vagues.
fn texte_rarete(r: Rarete, t: f64, taille: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    for (i, ch) in r.nom().chars().enumerate() {
        let c = if r == Rarete::Irreel {
            c32(crate::toile::hsv(
                i as f32 * 0.1 - t as f32 * 0.4,
                0.55,
                1.0,
            ))
        } else if r == Rarete::Prehistorique {
            // Ambre qui luit par vagues lentes, lettre après lettre.
            let k = (0.5 + 0.5 * (t as f32 * 2.0 - i as f32 * 0.5).sin()) * 0.6;
            c32(crate::toile::melange(r.couleur(), [0xff, 0xf0, 0xc0], k))
        } else {
            c32(r.couleur())
        };
        job.append(
            &ch.to_string(),
            0.0,
            egui::TextFormat::simple(titre(taille), c),
        );
    }
    job
}

/// v0.16 : cadenas en gros pixels (fermé : or ; ouvert : anse levée, pâle).
fn cadenas(ui: &Ui, r: egui::Rect, ferme: bool, survol: bool) {
    let k = 3.0;
    let p = ui.painter();
    let c = if ferme {
        OR
    } else if survol {
        ECUME
    } else {
        BRUME
    };
    let o = r.center() - Vec2::new(4.0 * k, 5.0 * k);
    let px = |x: f32, y: f32, w: f32, h: f32, c: Color32| {
        p.rect_filled(
            egui::Rect::from_min_size(o + Vec2::new(x * k, y * k), Vec2::new(w * k, h * k)),
            0.0,
            c,
        );
    };
    // Anse : deux montants et le haut ; ouverte, elle est levée et le montant droit décroché.
    let lev = if ferme { 0.0 } else { -2.0 };
    px(2.0, lev, 4.0, 1.0, c);
    px(1.0, lev + 1.0, 1.0, 3.0, c);
    px(6.0, lev + 1.0, 1.0, if ferme { 3.0 } else { 1.0 }, c);
    // Corps, trou de serrure.
    px(0.0, 4.0, 8.0, 6.0, c);
    px(3.0, 6.0, 2.0, 2.0, EAU);
}

fn piece_ui(ui: &Ui, r: egui::Rect) {
    let p = ui.painter();
    let u = (r.width() / 7.0).floor();
    let motif = [
        "..ddd..", ".dhhgd.", "dhgggsd", "dhgggsd", "dgggssd", ".dsssd.", "..ddd..",
    ];
    for (j, l) in motif.iter().enumerate() {
        for (i, b) in l.bytes().enumerate() {
            let c = match b {
                b'd' => Color32::from_rgb(0x8a, 0x56, 0x10),
                b'h' => Color32::from_rgb(0xff, 0xf4, 0xb8),
                b'g' => OR,
                b's' => Color32::from_rgb(0xd8, 0x98, 0x2c),
                _ => continue,
            };
            p.rect_filled(
                egui::Rect::from_min_size(
                    r.min + Vec2::new(i as f32 * u, j as f32 * u),
                    Vec2::splat(u),
                ),
                0.0,
                c,
            );
        }
    }
}

/// Image 0 d'un sprite (un texel par pixel du sprite) ; `ombre` = silhouette unie de cette
/// couleur, tirée de l'alpha du sprite.
fn image_sprite(s: &crate::poisson::Sprite, ombre: Option<Rgb>) -> egui::ColorImage {
    let im = &s.images[0];
    let px: Vec<u8> = (0..im.idx.len())
        .map(|i| im.px(i))
        .flat_map(|p| {
            let c = ombre.unwrap_or([p[0], p[1], p[2]]);
            [c[0], c[1], c[2], p[3]]
        })
        .collect();
    egui::ColorImage::from_rgba_unmultiplied([s.l as usize, s.h as usize], &px)
}

pub fn image_couleur(t: &Toile) -> egui::ColorImage {
    egui::ColorImage::from_rgba_premultiplied([t.lf(), t.hf()], t.px.as_flattened())
}

/// Annonce sur le fond d'écran pour une grande éclosion : Légendaire ou mieux, shiny, ou
/// modificateur rare (revenu ×3 ou plus).
fn annonce(f: &Poisson) -> Option<Annonce> {
    let rare_modif = f.modif.filter(|m| m.multiplicateur() >= 3.0);
    if f.rarete < Rarete::Legendaire && !f.shiny && rare_modif.is_none() {
        return None;
    }
    let mut texte = f.rarete.nom().to_uppercase();
    if f.shiny {
        texte = trf("SHINY {}", &[&texte.to_string()]);
    }
    if let Some(m) = rare_modif {
        texte = format!("{texte} {}", m.nom().to_uppercase());
    }
    Some(Annonce {
        texte: format!("{texte} ! {}", nom_de(f)),
        couleur: rare_modif.map_or(f.rarete.couleur(), |m| m.couleur()),
        id: f.id,
        shiny: f.shiny,
        reste: crate::scene::DUREE_ANNONCE,
    })
}

/// Virgule décimale en français, point en anglais.
fn decimale(s: String) -> String {
    if crate::i18n::anglais() {
        s
    } else {
        s.replace('.', ",")
    }
}

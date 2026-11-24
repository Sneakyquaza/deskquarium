//! Fenêtre « Écloserie » (egui) : œufs et éclosion, mon aquarium, Poissodex, réglages.
//! Style : verre d'aquarium biseauté en pixels, polices pixel OFL (Silkscreen pour les titres,
//! Pixelify Sans pour le texte), images pixel art agrandies en entier au plus proche.

use crate::especes::{de_rarete, Espece};
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
    Reglages,
}

use crate::eclosion::Eclosion;

pub struct Ecloserie {
    pub onglet: Onglet,
    eclosion: Option<Eclosion>,
    vignettes: HashMap<u64, TextureHandle>,
    vignettes_dex: HashMap<String, TextureHandle>,
    /// Index affiché (0 Normal, 1..=7 modificateurs, 8 Shiny).
    pub index: usize,
    oeufs: Vec<TextureHandle>,
    confirmer: Option<u64>,
    reserve: bool,
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
            bandeau: None,
            info: None,
            dossier,
            windows: cfg!(windows),
            maj: None,
            relancer: None,
            disposition: None,
            decor: Default::default(),
        }
    }

    /// Lance une éclosion (pour les captures automatiques).
    pub fn acheter(&mut self, ui: &Ui, p: &mut Partie, oeuf: usize) {
        match p.acheter_oeuf(oeuf) {
            Ok(id) => {
                let f = p
                    .poissons
                    .iter()
                    .find(|f| f.id == id)
                    .cloned()
                    .expect("poisson éclos");
                self.eclosion = Some(Eclosion::nouvelle(
                    &f,
                    oeuf,
                    ui.input(|i| i.time),
                    p.dernier_dore,
                ));
            }
            Err(Refus::Indisponible) => {
                self.info = Some((
                    "Cet œuf exceptionnel n'est pas là en ce moment.".into(),
                    ui.input(|i| i.time),
                ))
            }
            Err(Refus::PasAssez) => {
                self.info = Some((
                    "Pas assez d'argent pour cet œuf.".into(),
                    ui.input(|i| i.time),
                ))
            }
            Err(_) => {}
        }
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
        // Œufs animés au repos (onglet Œufs) : vignettes redessinées à chaque image.
        if self.onglet == Onglet::Oeufs && self.eclosion.is_none() || self.oeufs.is_empty() {
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
                RichText::new(format!("+{} par seconde", court(p.revenu_par_s())))
                    .font(titre(14.0))
                    .color(BRUME),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{} / {CAPACITE} dans l'aquarium", p.nb_actifs()))
                        .font(titre(13.0))
                        .color(BRUME),
                );
                // Mise à jour disponible : pastille discrète qui mène aux Réglages.
                if let Some(v) = self.maj.as_ref().and_then(|m| m.disponible()) {
                    let b = egui::Button::new(
                        RichText::new(format!("Mise à jour v{v}"))
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
                (Onglet::Oeufs, "Œufs"),
                (Onglet::Aquarium, "Mon aquarium"),
                (Onglet::Poissodex, "Index"),
                (Onglet::Decor, "Décor"),
                (Onglet::Reglages, "Réglages"),
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
                                        (OR, "LIMITÉ · EN CE MOMENT")
                                    } else {
                                        (BRUME, "LIMITÉ")
                                    };
                                    ui.label(RichText::new(t).font(titre(13.0)).color(c));
                                }
                                ui.add(egui::Image::from_texture((self.oeufs[i].id(), Vec2::new(192.0, 208.0))));
                                ui.label(RichText::new(o.nom).font(titre(15.0)));
                            });
                            ui.label(RichText::new(o.description).color(BRUME).size(14.0));
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
                                    (format!("Encore là {}", duree(s as f64)), OR)
                                } else {
                                    (format!("Revient dans {}", duree(s as f64)), BRUME)
                                };
                                ui.vertical_centered(|ui| ui.label(RichText::new(texte).font(titre(13.0)).color(c)));
                            }
                            let assez = p.argent >= prix && dispo;
                            let texte = RichText::new(format!("Ouvrir · {}", court(prix)))
                                .font(titre(14.0))
                                .color(if assez { ENCRE } else { BRUME });
                            let b = egui::Button::new(texte)
                                .fill(if assez { SABLE } else { VERRE_FONCE })
                                .stroke(Stroke::new(2.0, if assez { ENCRE } else { BISEAU_SOMBRE }))
                                .min_size(Vec2::new(ui.available_width(), 36.0));
                            if ui.add_enabled(assez, b).clicked() {
                                achat = Some(i);
                            }
                            if !assez && dispo {
                                let r = p.revenu_par_s();
                                let t = if r > 0.0 { duree((prix - p.argent) / r) } else { "jamais".into() };
                                ui.vertical_centered(|ui| ui.label(RichText::new(format!("Encore {t}")).color(BRUME).size(14.0)));
                            }
                            });
                        });
                    }
                });
                ui.add_space(8.0);
            }
            ui.add_space(8.0);
            ui.label(
                RichText::new("Le prix d'un œuf monte un peu à chaque ouverture. Aquarium plein : le poisson part en réserve. Parfois, un œuf ordinaire devient un œuf doré (meilleures chances) !")
                    .color(BRUME)
                    .size(14.0),
            );
        });
        if let Some(i) = achat {
            self.acheter(ui, p, i);
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
            .min((ui.available_height() - 170.0) / 96.0)
            .floor()
            .clamp(2.0, 5.0);
        let mut fini = false;
        let mut encore = None;
        ui.vertical_centered(|ui| {
            let tex = e.texture.as_ref().expect("texture d'éclosion");
            let r = ui.add(
                egui::Image::from_texture((tex.id(), Vec2::new(128.0, 96.0) * echelle))
                    .sense(egui::Sense::click()),
            );
            if !e.revele(maintenant) {
                if r.clicked() {
                    e.passer(maintenant);
                }
                ui.label(
                    RichText::new(format!(
                        "{} éclot…",
                        if e.dore {
                            "Œuf doré"
                        } else {
                            OEUFS[e.oeuf].nom
                        }
                    ))
                    .font(titre(18.0))
                    .color(BRUME),
                );
                ui.label(RichText::new("Clique pour passer").color(BRUME).size(14.0));
                return;
            }
            let Some(f) = p.poissons.iter().find(|f| f.id == e.id) else {
                fini = true;
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
                    RichText::new(format!(
                        "{} · revenu ×{}",
                        m.nom(),
                        court_pct(m.multiplicateur())
                    ))
                    .font(titre(16.0))
                    .color(c32(m.couleur())),
                );
            }
            ui.label(
                RichText::new(format!(
                    "Rapporte {}/s · taille {} · {}",
                    court(f.revenu()),
                    taille(f.taille),
                    if f.actif {
                        "nage déjà dans ton aquarium"
                    } else {
                        "rangé dans la réserve (aquarium plein)"
                    }
                ))
                .color(BRUME)
                .font(titre(13.0)),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let prix = p.prix_oeuf(e.oeuf);
                let w = 220.0;
                ui.add_space((ui.available_width() - 2.0 * w - 8.0).max(0.0) / 2.0);
                let assez = p.argent >= prix;
                let b = egui::Button::new(
                    RichText::new(format!("Encore un · {}", court(prix)))
                        .font(titre(14.0))
                        .color(if assez { ENCRE } else { BRUME }),
                )
                .fill(if assez { SABLE } else { VERRE_FONCE })
                .min_size(Vec2::new(w, 38.0));
                if ui.add_enabled(assez, b).clicked() {
                    encore = Some(e.oeuf);
                }
                if ui
                    .add(
                        egui::Button::new(RichText::new("Retour aux œufs").font(titre(14.0)))
                            .min_size(Vec2::new(w, 38.0)),
                    )
                    .clicked()
                {
                    fini = true;
                }
            });
        });
        if fini || encore.is_some() {
            self.eclosion = None;
        }
        if let Some(i) = encore {
            self.acheter(ui, p, i);
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
                    RichText::new(format!("Dans l'aquarium ({}/{CAPACITE})", p.nb_actifs()))
                        .font(titre(14.0)),
                )
                .clicked()
            {
                self.reserve = false;
            }
            if ui
                .selectable_label(
                    self.reserve,
                    RichText::new(format!("Réserve ({n_reserve})")).font(titre(14.0)),
                )
                .clicked()
            {
                self.reserve = true;
            }
        });
        // « Équiper les meilleurs » : les plus rentables dans l'aquarium, le reste en réserve.
        ui.horizontal(|ui| {
            let b = egui::Button::new(
                RichText::new("Équiper les meilleurs")
                    .font(titre(14.0))
                    .color(ENCRE),
            )
            .fill(SABLE)
            .min_size(Vec2::new(220.0, 34.0));
            if ui.add(b).clicked() {
                let (avant, apres) = p.equiper_meilleurs();
                let d = apres - avant;
                self.bandeau = Some(format!(
                    "Meilleurs équipés : {}/s ({}{}/s)",
                    court(apres),
                    if d >= 0.0 { "+" } else { "-" },
                    court(d.abs())
                ));
            }
            ui.checkbox(
                &mut p.reglages.equiper_auto,
                RichText::new("Auto : un nouveau poisson meilleur remplace le plus faible")
                    .size(13.0),
            );
        });
        ui.label(
            RichText::new(if self.reserve {
                "Les poissons de la réserve ne rapportent rien. Sors-les quand il y a de la place."
            } else {
                "Nourrir fait monter le niveau : +20 % de revenu par niveau, jusqu'au niveau 10."
            })
            .color(BRUME)
            .size(14.0),
        );
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
                        "Réserve vide."
                    } else {
                        "Aquarium vide : ouvre un œuf."
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
                                        RichText::new(format!(
                                            "niv. {} · taille {} · {}/s",
                                            f.niveau,
                                            taille(f.taille),
                                            court(f.revenu())
                                        ))
                                        .color(BRUME)
                                        .font(titre(12.0)),
                                    );
                                });
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let confirme = self.confirmer == Some(f.id);
                                    let t = if confirme {
                                        format!("Relâcher ? +{}", court(f.prix_relache()))
                                    } else {
                                        format!("Relâcher +{}", court(f.prix_relache()))
                                    };
                                    let b = egui::Button::new(
                                        RichText::new(t).font(titre(12.0)).color(if confirme {
                                            ENCRE
                                        } else {
                                            ECUME
                                        }),
                                    )
                                    .fill(if confirme { ALERTE } else { VERRE_FONCE });
                                    if ui.add(b).clicked() {
                                        // Peu commun et plus : il faut confirmer.
                                        if confirme || f.rarete == Rarete::Commun {
                                            action = Some((f.id, 2));
                                        } else {
                                            self.confirmer = Some(f.id);
                                        }
                                    }
                                    let plein = p.nb_actifs() >= CAPACITE;
                                    let (t, ok) = if f.actif {
                                        ("Ranger", true)
                                    } else {
                                        ("Sortir", !plein)
                                    };
                                    if ui
                                        .add_enabled(
                                            ok,
                                            egui::Button::new(RichText::new(t).font(titre(12.0))),
                                        )
                                        .on_disabled_hover_text("Aquarium plein")
                                        .clicked()
                                    {
                                        action = Some((f.id, 1));
                                    }
                                    if let Some(c) = f.cout_nourrir() {
                                        let assez = p.argent >= c;
                                        let b = egui::Button::new(
                                            RichText::new(format!("Nourrir · {}", court(c)))
                                                .font(titre(12.0))
                                                .color(if assez { ENCRE } else { BRUME }),
                                        )
                                        .fill(if assez { SABLE } else { VERRE_FONCE });
                                        if ui.add_enabled(assez, b).clicked() {
                                            action = Some((f.id, 0));
                                        }
                                    } else {
                                        ui.label(RichText::new("Niveau max").color(OR));
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
                RichText::new(format!(
                    "Index {} : {n}/{total} ({} %)",
                    nom_index(k),
                    n * 100 / total.max(1)
                ))
                .font(titre(16.0))
                .color(c32(couleur_index(k))),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!(
                        "{} index complets : +{} % de revenu",
                        p.index_complets(),
                        court_pct(p.index_complets() as f64 * BONUS_INDEX * 100.0)
                    ))
                    .font(titre(12.0))
                    .color(OR),
                );
            });
        });
        let regle = match k {
            0 => "Chaque espèce éclose (sous n'importe quelle forme) compte.".to_string(),
            k if k < NB_INDEX - 1 => {
                let m = Modif::TOUS[k - 1];
                format!(
                    "Espèces écloses avec le modificateur {} (1 chance sur {}, revenu ×{}).",
                    m.nom(),
                    (1.0 / m.chance()).round(),
                    court_pct(m.multiplicateur())
                )
            }
            _ => format!(
                "Espèces écloses shiny (1 chance sur {}, revenu ×{}).",
                (1.0 / crate::jeu::CHANCE_SHINY).round(),
                court_pct(MULT_SHINY)
            ),
        };
        ui.label(
            RichText::new(format!(
                "{regle} Index complet : +{} % de revenu pour toujours.",
                court_pct(BONUS_INDEX * 100.0)
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
            (e.nom.to_string(), Color32::WHITE)
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
            "À découvrir".into()
        } else if k == 0 {
            format!("{n} éclos")
        } else {
            "Découvert".into()
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
                    "Mises à jour",
                    &format!(
                        "Version actuelle : {}. L'appli se met à jour elle-même (rien à dézipper).",
                        crate::maj::VERSION
                    ),
                    &mut |ui| section_maj(ui, m),
                );
            }
            ligne(
                ui,
                "Décor",
                "Le décor du fond de l'aquarium (les objets achetés restent posés).",
                &mut |ui| {
                    for (i, nom) in crate::decor::THEMES.iter().enumerate() {
                        ui.radio_value(&mut r.theme, i as u8, *nom);
                    }
                },
            );
            ligne(
                ui,
                "Poissons et souris",
                "Garde le clic appuyé un instant sur un poisson (bureau vide, pas sur une icône) \
                 pour l'attraper et le déplacer ; un clic bref montre sa fiche.",
                &mut |ui| {
                    ui.checkbox(&mut r.attraper, "Attraper les poissons");
                    ui.add_enabled(
                        r.attraper,
                        egui::Checkbox::new(&mut r.attraper_alt, "Seulement avec Alt"),
                    );
                },
            );
            ligne(
                ui,
                "Teinte de l'eau",
                "0 % : ton fond d'écran reste intact derrière les poissons.",
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
                "Poissons dessinés",
                "Les plus rentables sont dessinés ; tous rapportent quand même.",
                &mut |ui| {
                    ui.add(egui::Slider::new(&mut r.max_affiches, 5..=CAPACITE));
                },
            );
            ligne(
                ui,
                "Fluidité",
                "30 images par seconde suffisent et ménagent le processeur.",
                &mut |ui| {
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut r.ips, 30, "30 i/s");
                        ui.radio_value(&mut r.ips, 60, "60 i/s");
                    });
                },
            );
            let windows = self.windows;
            ligne(
                ui,
                "Démarrage",
                if windows {
                    "L'aquarium se lance tout seul à l'ouverture de ta session."
                } else {
                    "Disponible seulement sous Windows."
                },
                &mut |ui| {
                    ui.add_enabled(
                        windows,
                        egui::Checkbox::new(
                            &mut r.demarrage_windows,
                            "Lancer au démarrage de Windows",
                        ),
                    );
                },
            );
            ligne(
                ui,
                "Jeux et vidéos",
                "Animation en pause tant qu'une appli est en plein écran (l'argent continue).",
                &mut |ui| {
                    ui.checkbox(&mut r.pause_plein_ecran, "Pause en plein écran");
                },
            );
            ligne(
                ui,
                "Affichage",
                "Si l'aquarium n'apparaît pas derrière les icônes : fenêtre sans bord au fond.",
                &mut |ui| {
                    ui.checkbox(&mut r.mode_fenetre, "Mode fenêtre");
                },
            );
            ligne(
                ui,
                "Mode Abysses",
                "Aquarium des grands fonds : eau sombre, neige marine, poissons qui luisent.",
                &mut |ui| {
                    ui.checkbox(&mut r.abysses, "Mode Abysses");
                },
            );
            ui.label(
                RichText::new(format!("Sauvegarde : {}", self.dossier))
                    .color(BRUME)
                    .size(13.0),
            );
            ui.label(
                RichText::new("Raccourci : Ctrl + Alt + A ouvre l'Écloserie.")
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
        Etat::Rien => chercher(ui, "Rechercher une mise à jour"),
        Etat::Recherche => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(RichText::new("Recherche en cours…").color(BRUME));
            });
        }
        Etat::AJour => {
            ui.label(RichText::new("Tu as la dernière version.").color(OR));
            chercher(ui, "Rechercher à nouveau");
        }
        Etat::Erreur(e) => {
            ui.label(RichText::new(e).color(ALERTE));
            chercher(ui, "Réessayer");
        }
        Etat::Dispo(a) => {
            ui.label(
                RichText::new(format!("Nouvelle version {} disponible", a.version))
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
                RichText::new(format!("Installer v{}", a.version))
                    .font(titre(14.0))
                    .color(ENCRE),
            )
            .fill(SABLE)
            .min_size(Vec2::new(220.0, 34.0));
            if ui.add(b).clicked() {
                m.telecharger();
            }
            ui.label(
                RichText::new("L'appli redémarre toute seule après l'installation.")
                    .color(BRUME)
                    .size(13.0),
            );
        }
        Etat::Telechargement(v, n, t) => {
            ui.label(RichText::new(format!("Téléchargement de la version {v}…")).color(BRUME));
            ui.add(
                egui::ProgressBar::new(n as f32 / t.max(1) as f32)
                    .show_percentage()
                    .desired_width(ui.available_width()),
            );
        }
        Etat::Pret(_) | Etat::Installation => {
            ui.label(RichText::new("Installation, redémarrage…").color(OR));
        }
        Etat::Echec(e) => {
            ui.label(RichText::new(e).color(ALERTE));
            chercher(ui, "Réessayer");
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
        ui.label(RichText::new("SHINY").font(titre(12.0)).color(SHINY))
            .on_hover_text(format!("Shiny : revenu ×{}", court_pct(MULT_SHINY)));
    }
    if let Some(m) = f.modif {
        ui.label(
            RichText::new(m.nom().to_uppercase())
                .font(titre(12.0))
                .color(c32(m.couleur())),
        )
        .on_hover_text(format!(
            "{} : revenu ×{}",
            m.nom(),
            court_pct(m.multiplicateur())
        ));
    }
}

/// « SHINY ! » en lettres qui scintillent (révélation).
fn texte_shiny(t: f64, taille: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    for (i, ch) in "SHINY !".chars().enumerate() {
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
    format!("{t:.2}").replace('.', ",")
}

fn court_pct(c: f64) -> String {
    if c.fract() == 0.0 {
        format!("{c:.0}")
    } else {
        format!("{c:.1}").replace('.', ",")
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

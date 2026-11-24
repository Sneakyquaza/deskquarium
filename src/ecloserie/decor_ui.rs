//! Onglet « Décor » de l'Écloserie (v0.9) : boutique d'objets de décor (achetés avec l'argent
//! du jeu) et éditeur « Modifier le décor » : aperçu de l'aquarium écran par écran, où l'on
//! pose, déplace (au sable) et retire les objets achetés.

use super::*;
use crate::decor::{Anim, Img, OBJETS};
use crate::jeu::MAX_EXEMPLAIRES;

/// Rectangle en pixels monde : x0, y0, x1, y1.
type Rect = (i32, i32, i32, i32);
/// Aperçu : clé, texture, zone de l'écran, rectangles des objets posés.
type Apercu = (String, TextureHandle, Rect, Vec<Rect>);

/// État de l'onglet Décor.
#[derive(Default)]
pub struct EtatDecor {
    /// Vrai : éditeur ; faux : boutique.
    pub editeur: bool,
    pub ecran: usize,
    icones: Vec<TextureHandle>,
    /// Aperçu : clé (thème, objets, écran, disposition), texture, zone de l'écran (pixels
    /// monde), rectangles des objets posés.
    apercu: Option<Apercu>,
    /// Objet posé sélectionné (index dans `Partie::decor_pose`).
    choisi: Option<usize>,
}

fn image_img(img: &Img) -> egui::ColorImage {
    egui::ColorImage::from_rgba_unmultiplied(
        [img.l as usize, img.h as usize],
        img.px.as_flattened(),
    )
}

/// Taille d'affichage d'une icône : agrandie d'un facteur entier pour tenir dans `max`.
fn taille_icone(img: &Img, max: f32) -> Vec2 {
    let k = (max / img.l.max(img.h) as f32).floor().max(1.0);
    let k = if img.l.max(img.h) as f32 > max {
        max / img.l.max(img.h) as f32
    } else {
        k
    };
    Vec2::new(img.l as f32 * k, img.h as f32 * k)
}

impl Ecloserie {
    pub(super) fn onglet_decor(&mut self, ui: &mut Ui, p: &mut Partie) {
        if self.decor.icones.is_empty() {
            let d = crate::decor::decor();
            self.decor.icones = d
                .objets
                .iter()
                .enumerate()
                .map(|(i, img)| {
                    ui.ctx().load_texture(
                        format!("decor{i}"),
                        image_img(img),
                        TextureOptions::NEAREST,
                    )
                })
                .collect();
        }
        ui.horizontal(|ui| {
            for (editeur, nom) in [(false, "Boutique"), (true, "Modifier le décor")] {
                if ui
                    .selectable_label(
                        self.decor.editeur == editeur,
                        RichText::new(nom).font(titre(14.0)),
                    )
                    .clicked()
                {
                    self.decor.editeur = editeur;
                }
            }
        });
        ui.add_space(6.0);
        if self.decor.editeur {
            self.editeur_decor(ui, p);
        } else {
            self.boutique_decor(ui, p);
        }
    }

    fn boutique_decor(&mut self, ui: &mut Ui, p: &mut Partie) {
        let colonnes = if ui.available_width() > 1000.0 {
            5
        } else if ui.available_width() > 620.0 {
            4
        } else {
            2
        };
        let mut achat = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for debut in (0..OBJETS.len()).step_by(colonnes) {
                ui.columns(colonnes, |cols| {
                    for (k, ui) in cols.iter_mut().enumerate() {
                        let i = debut + k;
                        let Some(o) = OBJETS.get(i) else { continue };
                        let img = &crate::decor::decor().objets[i];
                        biseau([0x7a, 0xc8, 0xb0]).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                ui.vertical_centered(|ui| {
                                    let (r, _) = ui.allocate_exact_size(
                                        Vec2::new(ui.available_width(), 104.0),
                                        egui::Sense::hover(),
                                    );
                                    let t = taille_icone(img, 100.0);
                                    let rect = egui::Rect::from_center_size(
                                        egui::pos2(r.center().x, r.bottom() - t.y / 2.0),
                                        t,
                                    );
                                    ui.painter().image(
                                        self.decor.icones[i].id(),
                                        rect,
                                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                        Color32::WHITE,
                                    );
                                    // Titre sur deux lignes de haut : les boutons restent alignés.
                                    ui.allocate_ui(Vec2::new(ui.available_width(), 34.0), |ui| {
                                        ui.set_min_height(34.0);
                                        ui.label(RichText::new(o.nom).font(titre(13.0)));
                                    });
                                });
                                let anim = match o.anim {
                                    Anim::Aucune => "",
                                    Anim::Bulles => "Animé : bulles",
                                    Anim::Geyser => "Animé : jets de bulles",
                                    Anim::Ondule => "Animé : ondule et luit",
                                    Anim::Lueur => "Animé : luit",
                                };
                                ui.label(
                                    RichText::new(if anim.is_empty() { "Immobile" } else { anim })
                                        .color(BRUME)
                                        .size(14.0),
                                );
                                let n = p.nb_achetes(o.cle);
                                ui.label(
                                    RichText::new(format!(
                                        "Possédés {n}/{MAX_EXEMPLAIRES} · posés {}",
                                        n - p.nb_libres(o.cle)
                                    ))
                                    .color(ECUME)
                                    .size(14.0),
                                );
                                let prix = p.prix_objet(o.cle).unwrap_or(f64::MAX);
                                let plein = n >= MAX_EXEMPLAIRES;
                                let assez = p.argent >= prix && !plein;
                                let texte = if plein {
                                    "Complet".to_string()
                                } else {
                                    format!("Acheter · {}", court(prix))
                                };
                                let b = egui::Button::new(
                                    RichText::new(texte)
                                        .font(titre(13.0))
                                        .color(if assez { ENCRE } else { BRUME }),
                                )
                                .fill(if assez { SABLE } else { VERRE_FONCE })
                                .stroke(Stroke::new(2.0, if assez { ENCRE } else { BISEAU_SOMBRE }))
                                .min_size(Vec2::new(ui.available_width(), 32.0));
                                if ui.add_enabled(assez, b).clicked() {
                                    achat = Some(o.cle);
                                }
                            });
                        });
                    }
                });
                ui.add_space(8.0);
            }
            ui.label(
                RichText::new(
                    "Les objets achetés se posent dans « Modifier le décor ». Le prix monte à chaque \
                     exemplaire. Le décor ne change pas les gains.",
                )
                .color(BRUME)
                .size(14.0),
            );
        });
        if let Some(cle) = achat {
            if p.acheter_objet(cle).is_ok() {
                // Posé tout de suite sur l'écran principal, à un endroit libre.
                let x = 0.2 + 0.6 * ((p.decor_pose.len() * 37) % 100) as f32 / 100.0;
                let _ = p.poser_objet(cle, 0, x);
                let nom = crate::decor::objet(cle).map_or("", |o| o.nom);
                self.info = Some((
                    format!("{nom} posé dans l'aquarium (Modifier le décor pour le placer)."),
                    ui.input(|i| i.time),
                ));
            }
        }
    }

    /// Aperçu de l'écran choisi : l'aquarium (décor seul) tel qu'il est dessiné au fond.
    fn apercu_decor(&mut self, ctx: &egui::Context, p: &Partie) {
        let (disp, e) = self
            .disposition
            .clone()
            .unwrap_or_else(|| (crate::ecrans::Disposition::simple(1920, 1080), 4));
        let n = disp.ecrans.len();
        self.decor.ecran = self.decor.ecran.min(n.saturating_sub(1));
        let cle = format!(
            "{}|{:?}|{}|{:?}|{}|{}|{}",
            p.reglages.theme,
            p.decor_pose,
            self.decor.ecran,
            disp,
            e,
            p.reglages.abysses,
            p.reglages.teinte
        );
        if self.decor.apercu.as_ref().is_some_and(|a| a.0 == cle) {
            return;
        }
        let (_, _, bl, bh) = disp.boite();
        let (l, h) = (
            (bl as usize).div_ceil(e as usize),
            (bh as usize).div_ceil(e as usize),
        );
        let zones = disp.zones(e);
        let mut aq = Aquarium::avec_zones(l, h, 1, &zones, &disp.travail(e));
        // Décor seul (le compteur est remplacé par sa zone, dessinée par l'éditeur).
        aq.avec_compteur = false;
        let mut vide = p.clone();
        vide.poissons.clear();
        aq.dessiner(&vide);
        let z = zones[self.decor.ecran];
        let f = crate::toile::FIN as i32;
        let (lf, hf) = (((z.2 - z.0) * f) as usize, ((z.3 - z.1) * f) as usize);
        let fond: Rgb = [0x2c, 0x4a, 0x66];
        let mut px = Vec::with_capacity(lf * hf * 4);
        for y in 0..hf {
            for x in 0..lf {
                let (sx, sy) = ((z.0 * f) as usize + x, (z.1 * f) as usize + y);
                let c = aq
                    .toile
                    .px
                    .get(sy * aq.toile.lf() + sx)
                    .copied()
                    .unwrap_or([0; 4]);
                let a = c[3] as u32;
                for k in 0..3 {
                    px.push((c[k] as u32 + fond[k] as u32 * (255 - a) / 255).min(255) as u8);
                }
                px.push(255);
            }
        }
        let img = egui::ColorImage::from_rgba_unmultiplied([lf, hf], &px);
        let tex = ctx.load_texture("apercu-decor", img, TextureOptions::NEAREST);
        self.decor.apercu = Some((cle, tex, z, aq.objets_places.clone()));
    }

    fn editeur_decor(&mut self, ui: &mut Ui, p: &mut Partie) {
        self.apercu_decor(ui.ctx(), p);
        let n_ecrans = self.disposition.as_ref().map_or(1, |d| d.0.ecrans.len());
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "Décor : {}",
                    crate::decor::THEMES[p.reglages.theme as usize]
                ))
                .font(titre(13.0)),
            );
            if n_ecrans > 1 {
                ui.add_space(12.0);
                for i in 0..n_ecrans {
                    let nom = if i == 0 {
                        "Écran 1 (principal)".to_string()
                    } else {
                        format!("Écran {}", i + 1)
                    };
                    if ui
                        .selectable_label(self.decor.ecran == i, RichText::new(nom).size(14.0))
                        .clicked()
                    {
                        self.decor.ecran = i;
                        self.decor.choisi = None;
                    }
                }
            }
        });
        let Some((_, tex, z, places)) = &self.decor.apercu else {
            return;
        };
        let (z, tex_id) = (*z, tex.id());
        let places = places.clone();
        let (zl, zh) = ((z.2 - z.0) as f32, (z.3 - z.1) as f32);
        // L'aperçu laisse la place aux boutons et à la liste des objets en dessous.
        let w = ui
            .available_width()
            .min(1100.0)
            .min((ui.available_height() - 190.0).max(120.0) * zl / zh);
        let k = w / zl;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(w, zh * k), egui::Sense::hover());
        let peintre = ui.painter_at(rect);
        peintre.image(
            tex_id,
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        let vers_ui = |r: (i32, i32, i32, i32)| {
            egui::Rect::from_min_max(
                egui::pos2(
                    rect.left() + (r.0 - z.0) as f32 * k,
                    rect.top() + (r.1 - z.1) as f32 * k,
                ),
                egui::pos2(
                    rect.left() + (r.2 - z.0) as f32 * k,
                    rect.top() + (r.3 - z.1) as f32 * k,
                ),
            )
        };
        // Zone du compteur (écran principal) : rien n'y est posé.
        if self.decor.ecran == 0 {
            let c = vers_ui((z.0 - 8, z.3 - 24, z.0 + 75, z.3));
            peintre.rect_filled(
                c,
                0.0,
                Color32::from_rgba_unmultiplied(0x08, 0x16, 0x28, 170),
            );
            peintre.text(
                c.center(),
                egui::Align2::CENTER_CENTER,
                "compteur",
                egui::FontId::proportional(12.0),
                BRUME,
            );
        }
        let mut deplacer: Option<(usize, f32)> = None;
        let mut retirer = None;
        for (i, pose) in p.decor_pose.iter().enumerate() {
            let ecran = if (pose.ecran as usize) < n_ecrans {
                pose.ecran as usize
            } else {
                0
            };
            let Some(&r) = places.get(i).filter(|_| ecran == self.decor.ecran) else {
                continue;
            };
            let ur = vers_ui(r);
            let rep = ui.interact(
                ur,
                ui.make_persistent_id(("objet-decor", i)),
                egui::Sense::click_and_drag(),
            );
            let choisi = self.decor.choisi == Some(i);
            if rep.dragged() || rep.drag_stopped() {
                // Glissé : il suit la souris en x, posé sur le sable au lâcher.
                let dx = rep.total_drag_delta().map_or(0.0, |d| d.x);
                let fantome = ur.translate(Vec2::new(dx, 0.0));
                peintre.rect_stroke(
                    fantome,
                    0.0,
                    Stroke::new(2.0, OR),
                    egui::StrokeKind::Outside,
                );
                if rep.drag_stopped() {
                    // Fraction de l'écran : inverse de la pose dans `Aquarium::decorer`.
                    let fi = crate::toile::FIN as i32;
                    let img = crate::decor::index_objet(&pose.objet)
                        .map(|j| &crate::decor::decor().objets[j]);
                    let demi = img.map_or(4, |im| im.l / fi / 2 + 1) as f32;
                    let cx = (r.0 + r.2) as f32 / 2.0 + dx / k;
                    let x = (cx - z.0 as f32 - demi) / (zl - 2.0 * demi).max(1.0);
                    deplacer = Some((i, x.clamp(0.0, 1.0)));
                }
                self.decor.choisi = Some(i);
            } else {
                let c = if choisi {
                    OR
                } else if rep.hovered() {
                    ECUME
                } else {
                    Color32::TRANSPARENT
                };
                peintre.rect_stroke(
                    ur,
                    0.0,
                    Stroke::new(if choisi { 2.0 } else { 1.0 }, c),
                    egui::StrokeKind::Outside,
                );
            }
            if rep.clicked() {
                self.decor.choisi = Some(i);
            }
            if rep.secondary_clicked() {
                retirer = Some(i);
            }
            rep.on_hover_text(
                crate::decor::objet(&pose.objet)
                    .map_or("", |o| o.nom)
                    .to_string()
                    + " · glisser pour déplacer, clic droit pour retirer",
            );
        }
        if let Some((i, x)) = deplacer {
            p.decor_pose[i].x = x;
        }
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let choisi = self.decor.choisi.filter(|&i| i < p.decor_pose.len());
            if ui
                .add_enabled(
                    choisi.is_some(),
                    egui::Button::new(RichText::new("Retourner").font(titre(13.0))),
                )
                .clicked()
            {
                if let Some(i) = choisi {
                    p.decor_pose[i].miroir = !p.decor_pose[i].miroir;
                }
            }
            if ui
                .add_enabled(
                    choisi.is_some(),
                    egui::Button::new(RichText::new("Retirer").font(titre(13.0))),
                )
                .clicked()
            {
                retirer = choisi;
            }
            if ui
                .add_enabled(
                    !p.decor_pose.is_empty(),
                    egui::Button::new(RichText::new("Tout retirer").font(titre(13.0))),
                )
                .clicked()
            {
                p.decor_pose.clear();
                self.decor.choisi = None;
            }
        });
        if let Some(i) = retirer.filter(|&i| i < p.decor_pose.len()) {
            p.decor_pose.remove(i);
            self.decor.choisi = None;
        }
        ui.add_space(6.0);
        ui.label(
            RichText::new("Objets à poser (achetés dans la Boutique) :")
                .color(BRUME)
                .size(14.0),
        );
        let mut poser = None;
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                for (i, o) in OBJETS.iter().enumerate() {
                    let libres = p.nb_libres(o.cle);
                    if libres == 0 {
                        continue;
                    }
                    let img = &crate::decor::decor().objets[i];
                    let b = egui::Button::image_and_text(
                        egui::Image::from_texture((
                            self.decor.icones[i].id(),
                            taille_icone(img, 48.0),
                        )),
                        RichText::new(format!("{} ×{libres}", o.nom)).size(13.0),
                    );
                    if ui
                        .add(b)
                        .on_hover_text("Poser au milieu de cet écran")
                        .clicked()
                    {
                        poser = Some(o.cle);
                    }
                }
            });
        });
        if let Some(cle) = poser {
            let x = 0.35 + 0.3 * ((p.decor_pose.len() * 41) % 100) as f32 / 100.0;
            if p.poser_objet(cle, self.decor.ecran as u8, x).is_ok() {
                self.decor.choisi = Some(p.decor_pose.len() - 1);
            }
        }
        if !p.decor_achete.values().any(|&n| n > 0) {
            ui.label(
                RichText::new(
                    "Aucun objet acheté pour l'instant : la Boutique en propose dès 150.",
                )
                .color(BRUME)
                .size(14.0),
            );
        }
        ui.label(
            RichText::new(
                "Clique un objet pour le choisir, glisse-le pour le déplacer (il reste sur le \
                 sable), clic droit pour le retirer. Rien ne se pose devant le compteur.",
            )
            .color(BRUME)
            .size(14.0),
        );
    }
}

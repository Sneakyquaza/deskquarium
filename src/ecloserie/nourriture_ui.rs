//! Onglet « Nourriture » de l'Écloserie (v0.16) : boutique des sept nourritures, réserve, et
//! jet dans l'aquarium (au hasard, ou au prochain clic sur le bureau).

use super::decor_ui::{image_img, taille_icone};
use super::*;
use crate::nourriture::{images, MAX_STOCK, NOURRITURES};

impl Ecloserie {
    pub(super) fn onglet_nourriture(&mut self, ui: &mut Ui, p: &mut Partie) {
        if self.icones_nourriture.is_empty() {
            self.icones_nourriture = images()
                .icones
                .iter()
                .enumerate()
                .map(|(i, img)| {
                    ui.ctx().load_texture(
                        format!("nourriture{i}"),
                        image_img(img),
                        TextureOptions::NEAREST,
                    )
                })
                .collect();
        }
        ui.label(
            RichText::new(tr(
                "Achète de la nourriture, puis jette-la dans l'aquarium : les poissons viennent la manger et le repas multiplie tes gains un moment.",
            ))
            .color(BRUME)
            .size(14.0),
        );
        match p.festin {
            Some(f) => ui.label(
                RichText::new(trf(
                    "Repas en cours : revenu ×{} · encore {}",
                    &[&decimale(f.nourriture().mult.to_string()), &duree(f.reste)],
                ))
                .font(titre(14.0))
                .color(OR),
            ),
            None => ui.label(
                RichText::new(tr(
                    "Un seul bonus à la fois : une meilleure nourriture le remplace, une moins bonne le prolonge un peu (au plus 3 repas d'avance).",
                ))
                .color(BRUME)
                .size(14.0),
            ),
        };
        if let Some((n, _)) = self.viser {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(trf(
                        "Clique sur le fond d'écran pour jeter : {}",
                        &[tr(NOURRITURES[n].nom)],
                    ))
                    .color(SABLE),
                );
                if ui.button(tr("Annuler")).clicked() {
                    self.viser = None;
                }
            });
        }
        ui.add_space(6.0);
        let colonnes = if ui.available_width() > 1000.0 {
            4
        } else if ui.available_width() > 620.0 {
            3
        } else {
            2
        };
        let maintenant = ui.input(|i| i.time);
        let mut achat = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for debut in (0..NOURRITURES.len()).step_by(colonnes) {
                ui.columns(colonnes, |cols| {
                    for (k, ui) in cols.iter_mut().enumerate() {
                        let i = debut + k;
                        let Some(nn) = NOURRITURES.get(i) else {
                            continue;
                        };
                        let img = &images().icones[i];
                        biseau([0xe8, 0x9a, 0x5a]).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                ui.vertical_centered(|ui| {
                                    let (r, _) = ui.allocate_exact_size(
                                        Vec2::new(ui.available_width(), 104.0),
                                        egui::Sense::hover(),
                                    );
                                    let t = taille_icone(img, 100.0);
                                    let rect = egui::Rect::from_center_size(r.center(), t);
                                    ui.painter().image(
                                        self.icones_nourriture[i].id(),
                                        rect,
                                        egui::Rect::from_min_max(
                                            egui::pos2(0.0, 0.0),
                                            egui::pos2(1.0, 1.0),
                                        ),
                                        Color32::WHITE,
                                    );
                                    ui.label(RichText::new(tr(nn.nom)).font(titre(13.0)));
                                });
                                ui.allocate_ui(Vec2::new(ui.available_width(), 40.0), |ui| {
                                    ui.set_min_height(40.0);
                                    ui.label(
                                        RichText::new(tr(nn.description)).color(BRUME).size(13.0),
                                    );
                                });
                                // Chiffres en Silkscreen (le 5 de Pixelify ressemble à un S).
                                ui.label(
                                    RichText::new(trf(
                                        "Revenu ×{} pendant {}",
                                        &[&decimale(nn.mult.to_string()), &duree(nn.duree)],
                                    ))
                                    .color(OR)
                                    .font(titre(12.0)),
                                );
                                if nn.chance > 1.0 {
                                    ui.label(
                                        RichText::new(trf(
                                            "Chance aux éclosions ×{}",
                                            &[&decimale(nn.chance.to_string())],
                                        ))
                                        .color(SHINY)
                                        .font(titre(12.0)),
                                    );
                                }
                                let stock = p.nourriture.get(i).copied().unwrap_or(0);
                                ui.label(
                                    RichText::new(trf("En réserve : {}", &[&stock.to_string()]))
                                        .color(ECUME)
                                        .font(titre(12.0)),
                                );
                                for n in [1u32, 10] {
                                    let prix = nn.prix * n as f64;
                                    let assez = p.argent >= prix && stock + n <= MAX_STOCK;
                                    let texte = if n == 1 {
                                        trf("Acheter · {}", &[&court(prix)])
                                    } else {
                                        trf("×10 · {}", &[&court(prix)])
                                    };
                                    let b = egui::Button::new(
                                        RichText::new(texte)
                                            .font(titre(13.0))
                                            .color(if assez { ENCRE } else { BRUME }),
                                    )
                                    .fill(if assez { SABLE } else { VERRE_FONCE })
                                    .stroke(Stroke::new(
                                        2.0,
                                        if assez { ENCRE } else { BISEAU_SOMBRE },
                                    ))
                                    .min_size(Vec2::new(ui.available_width(), 30.0));
                                    if ui.add_enabled(assez, b).clicked() {
                                        achat = Some((i, n));
                                    }
                                }
                                ui.horizontal(|ui| {
                                    let w = (ui.available_width() - 8.0) / 2.0;
                                    let jeter = egui::Button::new(
                                        RichText::new(tr("Jeter")).font(titre(13.0)),
                                    )
                                    .min_size(Vec2::new(w, 30.0));
                                    if ui
                                        .add_enabled(stock > 0, jeter)
                                        .on_hover_text(tr(
                                            "Tombe d'en haut, à un endroit au hasard.",
                                        ))
                                        .clicked()
                                    {
                                        son::jouer(Son::Clic, 1.0);
                                        self.jets.push(i);
                                    }
                                    let viser = egui::Button::new(
                                        RichText::new(tr("Viser")).font(titre(13.0)),
                                    )
                                    .min_size(Vec2::new(w, 30.0));
                                    if ui
                                        .add_enabled(stock > 0, viser)
                                        .on_hover_text(tr(
                                            "Le prochain clic sur le bureau (hors icônes) la jette à cet endroit.",
                                        ))
                                        .clicked()
                                    {
                                        son::jouer(Son::Clic, 1.0);
                                        self.viser = Some((i, std::time::Instant::now()));
                                    }
                                });
                            });
                        });
                    }
                });
                ui.add_space(8.0);
            }
        });
        if let Some((i, n)) = achat {
            match p.acheter_nourriture(i, n) {
                Ok(()) => son::jouer(Son::Piece, 1.0),
                Err(_) => {
                    self.info = Some((tr("Pas assez d'argent.").into(), maintenant));
                }
            }
        }
    }
}

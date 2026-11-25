//! Nourriture dans l'aquarium (v0.16) : morceaux jetés qui coulent jusqu'au sable, poissons
//! intéressés qui viennent les manger (la tête suit la nourriture), bouchées données à la
//! partie, dessin des morceaux et de la nourriture tenue au curseur. Tête penchée : dessin
//! RotSprite (`penche.rs`) échangé avec le dessin droit le temps du dessin et des effets.

use super::*;
use crate::nourriture::{images, Morceau, NOURRITURES};
use crate::penche::CRAN;

/// Pente maximale d'une espèce (radians) : 30° ; 22,5° pour les grandes, 15° pour les énormes
/// et les corps longs ; aucune pour les colossales (dessins trop grands à tourner), les vues
/// de dessus ou de face, les méduses, marcheurs et serpents qui traversent.
pub fn penche_max(e: &crate::especes::Espece, n: &Nageur) -> f32 {
    let vue = matches!(
        e.cle,
        "tortue"
            | "anomalocaris"
            | "trilobite"
            | "kraken"
            | "poulpe"
            | "cthulhu"
            | "oeil"
            | "hydre"
            | "ammonite"
    );
    let mv = matches!(
        e.mouvement,
        Mouvement::Marche
            | Mouvement::Abysse
            | Mouvement::Traverse
            | Mouvement::Pulse
            | Mouvement::Debout
            | Mouvement::Plane
            | Mouvement::Flotte
    );
    if vue || mv || e.taille >= Taille::Colossale || n.sprite.l * n.sprite.h > 60_000 {
        return 0.0;
    }
    let crans = if n.corps_long() || e.taille == Taille::Enorme {
        2.0
    } else if e.taille == Taille::Grande {
        3.0
    } else {
        4.0
    };
    crans * CRAN
}

/// Demi-taille d'un morceau (pixels monde).
fn demi(n: usize) -> (f32, f32) {
    let im = &images().eau[n];
    let f = FIN as f32;
    (im.l as f32 / f / 2.0, im.h as f32 / f / 2.0)
}

impl Nageur {
    /// Échange le dessin droit et le dessin penché (deux appels = retour au droit).
    pub(super) fn basculer(&mut self) {
        if let Some((_, p)) = &mut self.penche {
            std::mem::swap(&mut self.sprite, p);
        }
    }

    /// Bouche sur la toile (pixels monde), tête penchée comprise.
    pub fn bouche(&mut self) -> (f32, f32) {
        self.basculer();
        let b = self.monde(self.sprite.bouche);
        self.basculer();
        b
    }

    /// Bouche par rapport au centre, tourné à droite, sans pente (pixels monde).
    pub(super) fn decalage_bouche(&self) -> (f32, f32) {
        let (s, f) = (&self.sprite, FIN as f32);
        (
            (s.bouche.0 as f32 + 0.5 - s.l as f32 / 2.0) / f,
            (s.bouche.1 as f32 + 0.5 - s.h as f32 / 2.0) / f,
        )
    }

    /// La tête suit la nourriture visée (sinon, doucement, la montée ou la descente, un cran
    /// au plus) :
    /// ressort amorti critique (ni à-coup ni rebond), jamais au-delà de `penche_max` ; le cran
    /// affiché change avec un peu d'hystérésis (pas de va-et-vient entre deux crans).
    pub(super) fn pencher(&mut self, dt: f32, sl: f32) {
        let voulu = match self.repas {
            Some((fx, fy)) => (self.y - fy)
                .atan2((fx - self.x).abs().max(sl * 0.5))
                .clamp(-self.penche_max, self.penche_max),
            // Nage libre : un cran au plus, petits et moyens seulement (peu de dessins penchés
            // à fabriquer et garder ; les grands ne penchent que pour manger).
            None if self.sprite.taille >= Taille::Grande => 0.0,
            None => ((-self.vy).atan2(self.vx.abs().max(6.0)) * 0.35)
                .clamp(-CRAN, CRAN)
                .clamp(-self.penche_max, self.penche_max),
        };
        let w = if self.sprite.taille >= Taille::Grande {
            3.5
        } else {
            5.0
        };
        self.pente_v += (w * w * (voulu - self.pente) - 2.0 * w * self.pente_v) * dt;
        self.pente = (self.pente + self.pente_v * dt).clamp(-self.penche_max, self.penche_max);
        if (self.pente - self.cran as f32 * CRAN).abs() > CRAN * 0.65 {
            self.cran = (self.pente / CRAN).round() as i8;
        }
    }
}

impl Aquarium {
    /// Octets des dessins penchés gardés en cache (mesure).
    pub fn octets_penches(&self) -> usize {
        self.penches.octets()
    }

    /// Écran principal : x0, x1 (pixels monde).
    pub fn zone_principale(&self) -> (f32, f32) {
        (
            self.bassin.principal.0 as f32,
            self.bassin.principal.2 as f32,
        )
    }

    /// Jette la nourriture `n` : au point donné (clic sur le bureau), sinon d'en haut à un
    /// endroit au hasard de l'écran principal. Une pincée s'éparpille en plusieurs morceaux.
    pub fn jeter(&mut self, n: usize, pos: Option<(f32, f32)>) {
        let Some(nn) = NOURRITURES.get(n) else {
            return;
        };
        let b = &self.bassin;
        let (x0, _, x1, _) = b.principal;
        let (x, y) = match pos {
            Some((x, y)) if b.bas[b.col(x)] > 0 => (x, y),
            _ => {
                let x = x0 as f32 + (x1 - x0) as f32 * self.alea.entre(0.15, 0.85);
                (x, f32::MIN)
            }
        };
        let (_, dh) = demi(n);
        for k in 0..nn.morceaux {
            let eparpille = nn.morceaux > 1;
            let dx = if eparpille {
                self.alea.entre(-8.0, 8.0)
            } else {
                0.0
            };
            let dy = if eparpille && k > 0 {
                self.alea.entre(-4.0, 2.0)
            } else {
                0.0
            };
            let xm = x + dx;
            let c = b.col(xm);
            if b.bas[c] == 0 {
                continue;
            }
            let ym = (y + dy).clamp(b.ciel[c] as f32 + dh + 1.0, b.sable[c] as f32 - dh - 2.0);
            let mut m = Morceau::new(n, xm, ym, self.alea.f() * TAU);
            m.vy = self.alea.entre(0.0, nn.chute * 0.5);
            self.morceaux.push(m);
        }
    }

    /// Avant la nage : les morceaux coulent, chaque poisson intéressé choisit le plus proche
    /// (quelques poissons par morceau, le plus près d'abord).
    pub(super) fn nourriture_avant(&mut self, dt: f32) {
        let t = self.temps;
        let b = &self.bassin;
        for m in &mut self.morceaux {
            let c = b.col(m.x);
            let (_, dh) = demi(m.n);
            m.avancer(dt, t, b.sable[c] as f32 + 1.0, dh);
        }
        self.morceaux
            .retain(|m| !m.fini() && b.bas[b.col(m.x)] > 0 && m.x >= 0.0);
        let mut couples: Vec<(f32, usize, usize)> = Vec::new();
        for (i, n) in self.nageurs.iter_mut().enumerate() {
            n.repas = None;
            n.rassasie = (n.rassasie - dt).max(0.0);
            if n.fixe || n.tenu || n.fuite > 0.0 {
                continue;
            }
            if matches!(
                n.sprite.mouvement,
                Mouvement::Marche | Mouvement::Abysse | Mouvement::Traverse | Mouvement::Pulse
            ) {
                continue;
            }
            let sens = if n.gauche { -1.0 } else { 1.0 };
            for (j, m) in self.morceaux.iter().enumerate() {
                if m.age < crate::nourriture::REMARQUEE {
                    continue;
                }
                let nn = m.nourriture();
                if n.sprite.taille < nn.tailles.0 || n.sprite.taille > nn.tailles.1 {
                    continue;
                }
                // Corps long : seulement devant lui (il ne se retourne pas sous nos yeux).
                if n.corps_long() && (m.x - n.x) * sens < 0.0 {
                    continue;
                }
                let d = (m.x - n.x).hypot((m.y - n.y) * 1.2);
                let portee = 110.0 + n.sprite.lw() + if nn.bouchees > 1 { 80.0 } else { 0.0 };
                if d < portee {
                    couples.push((d, i, j));
                }
            }
        }
        couples.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut par_morceau = vec![0u32; self.morceaux.len()];
        for (_, i, j) in couples {
            let m = &self.morceaux[j];
            let max = if m.bouchees > 1 { 4 } else { 2 };
            if self.nageurs[i].repas.is_some() || par_morceau[j] >= max {
                continue;
            }
            par_morceau[j] += 1;
            self.nageurs[i].repas = Some((m.x, m.y));
        }
    }

    /// Après la nage : bouchées (bouche au contact du morceau), miettes, puis dessins penchés
    /// mis à jour (fabriqués peu à peu, `budget_penche`).
    pub(super) fn nourriture_apres(&mut self, dt: f32) {
        let _ = dt;
        for i in 0..self.nageurs.len() {
            // Entre deux bouchées, il reste devant la nourriture (`rassasie`).
            if self.nageurs[i].repas.is_none() || self.nageurs[i].rassasie > 0.0 {
                continue;
            }
            let (bx, by) = self.nageurs[i].bouche();
            let Some(j) = self.morceaux.iter().position(|m| {
                let (dw, dh) = demi(m.n);
                m.bouchees > 0
                    && (m.x - bx).abs() < dw.max(1.5) + 2.5
                    && (m.y - by).abs() < dh.max(1.5) + 2.5
            }) else {
                continue;
            };
            let m = &mut self.morceaux[j];
            let nn = m.nourriture();
            m.bouchees -= 1;
            m.croque = 0.25;
            self.bouchees
                .push((m.n, 1.0 / (nn.morceaux * nn.bouchees) as f64));
            let (mx, my) = (m.x, m.y);
            let im = &images().eau[m.n];
            self.nageurs[i].rassasie = 1.1;
            // Miettes de la couleur de la nourriture, et une bulle.
            for _ in 0..(3 + 2 * nn.bouchees.min(3)) {
                let k = self.alea.n(im.px.len() as u32) as usize;
                let p = im.px[k];
                if p[3] == 0 {
                    continue;
                }
                self.particules.push(Particule {
                    x: mx + self.alea.entre(-1.5, 1.5),
                    y: my + self.alea.entre(-1.5, 1.5),
                    vx: self.alea.entre(-7.0, 7.0),
                    vy: self.alea.entre(-6.0, 3.0),
                    vie: 0.0,
                    duree: self.alea.entre(0.4, 0.9),
                    c: [p[0], p[1], p[2]],
                    genre: Genre::Poussiere,
                });
            }
            self.bulles.push(Bulle {
                x: bx,
                y: by - 1.0,
                r: 1,
                v: 12.0,
                phase: self.alea.f() * TAU,
            });
        }
        // Dessins penchés : le cran voulu dès qu'il est prêt (sinon l'ancien reste affiché).
        for n in &mut self.nageurs {
            if n.cran == 0 || n.penche_max == 0.0 {
                n.penche = None;
                continue;
            }
            if n.penche.as_ref().is_some_and(|p| p.0 == n.cran) {
                continue;
            }
            if let Some(p) = self.penches.obtenir(&n.sprite, n.cran) {
                n.penche = Some((n.cran, p));
            }
        }
        self.penches.travailler(self.temps, self.budget_penche);
    }

    /// Les morceaux dans l'eau et posés (pixels fins). Flocons et vers voltigent en tombant
    /// (miroir), une bouchée fait un éclat clair, un morceau oublié s'efface en trame à la
    /// fin ; le festin luit.
    pub(super) fn dessiner_nourriture(&mut self) {
        let t = self.temps;
        let f = FIN as f32;
        for m in &self.morceaux {
            let im = &images().eau[m.n];
            let voltige = m.pose < 0.0 && matches!(m.nourriture().cle, "flocons" | "vers");
            let miroir = voltige && (t * 2.5 + m.phase).sin() > 0.0;
            let reste = crate::nourriture::DUREE_SUR_LE_SABLE - m.pose.max(0.0);
            let efface = (1.0 - reste / 2.0).clamp(0.0, 1.0);
            let (x0, y0) = (
                (m.x * f).round() as i32 - im.l / 2,
                (m.y * f).round() as i32 - im.h / 2,
            );
            let lueur = m.nourriture().cle == "festin";
            let pouls = 0.6 + 0.4 * (t * 2.2 + m.phase).sin();
            for y in 0..im.h {
                for x in 0..im.l {
                    let p = im.px(if miroir { im.l - 1 - x } else { x }, y);
                    if p[3] == 0 {
                        continue;
                    }
                    let (xx, yy) = (x0 + x, y0 + y);
                    if efface > 0.0 && bayer(xx, yy) < efface {
                        continue;
                    }
                    let mut c = [p[0], p[1], p[2]];
                    if m.croque > 0.0 {
                        c = melange(c, [255, 255, 255], 0.4);
                    }
                    if lueur && (xx + yy).rem_euclid(2) == 0 {
                        let a = (70.0 * pouls) as u8;
                        for (dx, dy) in [(2, 0), (-2, 0), (0, 2), (0, -2)] {
                            self.toile
                                .eclairer_fin(xx + dx, yy + dy, [0x9c, 0xf0, 0xff], a);
                        }
                    }
                    self.toile.poser_fin(xx, yy, c, p[3]);
                }
            }
        }
    }

    /// Nourriture prête à jeter : son dessin suit le curseur (on voit qu'un clic la jettera).
    pub(super) fn dessiner_viseur(&mut self) {
        let (Some(n), Some((cx, cy))) = (self.viser, self.souris.pos) else {
            return;
        };
        let Some(im) = images().eau.get(n) else {
            return;
        };
        let f = FIN as f32;
        let bob = (self.temps * 4.0).sin().round() as i32;
        let (x0, y0) = (
            (cx * f).round() as i32 - im.l / 2,
            (cy * f).round() as i32 - im.h - 6 + bob,
        );
        for y in 0..im.h {
            for x in 0..im.l {
                let p = im.px(x, y);
                if p[3] > 0 {
                    self.toile
                        .poser_fin(x0 + x, y0 + y, [p[0], p[1], p[2]], p[3]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Aquarium avec un seul poisson de l'espèce `cle`, au milieu.
    fn bocal(cle: &str) -> Aquarium {
        let e = crate::especes::par_cle(cle).unwrap();
        let mut p = Partie::nouvelle(0, 3);
        p.poissons[0].espece = e.cle.to_string();
        p.poissons[0].rarete = e.rarete;
        let mut aq = Aquarium::new(480, 270, 1);
        aq.budget_penche = None;
        aq.synchroniser(&p);
        aq
    }

    #[test]
    fn la_cage_se_mange_en_plusieurs_bouchees() {
        let mut aq = bocal("blanc");
        let x = aq.nageurs[0].x;
        aq.jeter(5, Some((x + 40.0, 60.0)));
        let mut bouchees = 0;
        for _ in 0..30 * 60 {
            aq.avancer(1.0 / 30.0);
            bouchees += aq.bouchees.drain(..).count();
            if aq.morceaux.is_empty() {
                break;
            }
        }
        assert_eq!(bouchees, 4, "la cage : 4 bouchées");
        assert!(aq.morceaux.is_empty(), "mangée");
        // Un tout petit poisson ne s'intéresse pas à la cage.
        let mut aq = bocal("neon");
        let x = aq.nageurs[0].x;
        aq.jeter(5, Some((x, 60.0)));
        for _ in 0..30 * 10 {
            aq.avancer(1.0 / 30.0);
        }
        assert!(aq.bouchees.is_empty());
    }

    #[test]
    fn flocons_manges_ou_oublies_sur_le_sable() {
        let mut aq = bocal("clown");
        let x = aq.nageurs[0].x;
        aq.jeter(0, Some((x, 40.0)));
        assert_eq!(aq.morceaux.len(), 6);
        let mut mange = 0.0;
        for _ in 0..30 * 90 {
            aq.avancer(1.0 / 30.0);
            mange += aq.bouchees.drain(..).map(|b| b.1).sum::<f64>();
        }
        // Tout est parti (mangé ou effacé après 25 s sur le sable), et au moins un morceau
        // mangé ; la part d'une pincée entière vaut un repas.
        assert!(aq.morceaux.is_empty());
        assert!(mange > 0.1 && mange <= 1.0 + 1e-9, "{mange}");
    }

    #[test]
    fn la_tete_suit_la_nourriture_dans_les_deux_sens() {
        for cle in ["clown", "blanc", "anguille"] {
            for dessus in [true, false] {
                for gauche in [false, true] {
                    let mut aq = bocal(cle);
                    let n = &mut aq.nageurs[0];
                    let max = n.penche_max;
                    assert!(max > 0.0 && max <= 4.0 * CRAN + 1e-6, "{cle}");
                    n.x = 240.0;
                    n.y = 135.0;
                    n.gauche = gauche;
                    let sens = if gauche { -1.0 } else { 1.0 };
                    let cible = (240.0 + sens * 30.0, if dessus { 40.0 } else { 230.0 });
                    for _ in 0..20 {
                        n.repas = Some(cible);
                        n.pencher(1.0 / 30.0, n.sprite.lw());
                    }
                    // Tête vers la nourriture, plafonnée, même signe dans les deux sens.
                    assert!(n.pente.abs() <= max + 1e-6);
                    assert_eq!(n.pente > 0.0, dessus, "{cle} gauche {gauche}");
                    assert!(n.pente.abs() > max * 0.5, "{cle} : {}", n.pente);
                    assert_ne!(n.cran, 0);
                    // Sans nourriture et sans vitesse verticale : elle revient à plat.
                    for _ in 0..90 {
                        n.repas = None;
                        n.pencher(1.0 / 30.0, n.sprite.lw());
                    }
                    assert!(n.pente.abs() < 0.02, "{cle}");
                    assert_eq!(n.cran, 0);
                }
            }
        }
        // Créatures sans pente : énormes, vues de dessus, méduses.
        for cle in ["kraken", "megalodon", "tortue", "meduse"] {
            let aq = bocal(cle);
            assert_eq!(aq.nageurs[0].penche_max, 0.0, "{cle}");
        }
    }
}

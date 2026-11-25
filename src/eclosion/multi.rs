//! Ouverture ×3 / ×10 (v0.11) : les œufs tombent en grille, tremblent ensemble, la lumière de
//! chacun monte par paliers jusqu'à SA rareté (ceux qui restent allumés le plus longtemps sont
//! les plus rares), éclatement commun, puis les cartes se retournent une à une, la plus rare en
//! dernier (pause, rayons, son à part).

use super::calques;
use super::paliers::{couleur_palier, palier};
use super::*;

/// Une carte de la grille.
struct Carte {
    id: u64,
    rarete: Rarete,
    shiny: bool,
    modif: Option<crate::jeu::Modif>,
    dore: bool,
    pitie: bool,
    palier: usize,
    sprite: crate::poisson::Sprite,
    fiss: Fissures,
    /// Place dans la grille (centre, pixels fins).
    x: i32,
    y: i32,
}

pub struct Multi {
    pub oeuf: usize,
    pub debut: f64,
    pub toile: Toile,
    pub texture: Option<eframe::egui::TextureHandle>,
    cartes: Vec<Carte>,
    /// Ordre de retournement (indices de `cartes`), la plus rare en dernier.
    ordre: Vec<usize>,
    trauma: f32,
    fait: u64,
    t_prec: f32,
    etincelles: Vec<Etincelle>,
    alea: Alea,
    pub sons: Vec<(Son, f32)>,
}

/// Taille de la grille (pixels monde).
pub const ML: i32 = 200;
pub const MH: i32 = 112;
const ARRIVEE: f32 = 0.7;
const PAS_PALIER: f32 = 0.6;

/// Rang de rareté d'une carte (tri : la plus rare se retourne en dernier).
fn rang(c: &Carte) -> (usize, Rarete, bool, bool) {
    (c.palier, c.rarete, c.shiny, c.modif.is_some())
}

impl Multi {
    /// `poissons` : (poisson éclos, œuf doré, pitié), dans l'ordre d'achat.
    pub fn nouvelle(poissons: &[(Poisson, bool, bool)], oeuf: usize, debut: f64) -> Multi {
        let n = poissons.len() as i32;
        let cols = n.min(5);
        let lignes = (n + cols - 1) / cols.max(1);
        let (cl, ch) = (ML * 2 / 5, MH * 2 / 2);
        let cartes = poissons
            .iter()
            .enumerate()
            .map(|(k, (f, dore, pitie))| {
                let (k, ki) = (k as i32, k as i32 % cols);
                let ligne = k / cols;
                let dans_ligne = if ligne == lignes - 1 {
                    n - ligne * cols
                } else {
                    cols
                };
                let x0 = ML - dans_ligne * cl / 2;
                Carte {
                    id: f.id,
                    rarete: f.rarete,
                    shiny: f.shiny,
                    modif: f.modif,
                    dore: *dore,
                    pitie: *pitie,
                    palier: palier(f.rarete).max(if f.shiny || f.modif.is_some() { 3 } else { 0 }),
                    sprite: sprite_ajuste(f, cl - 12, ch - 34),
                    fiss: Fissures::nouvelles(image_oeuf(oeuf), f.id),
                    x: (x0 + ki * cl + cl / 2) & !1,
                    y: (MH * 2 - (lignes - ligne) * ch + ch / 2 - 4) & !1,
                }
            })
            .collect::<Vec<_>>();
        let mut ordre: Vec<usize> = (0..cartes.len()).collect();
        ordre.sort_by_key(|&i| rang(&cartes[i]));
        Multi {
            oeuf,
            debut,
            toile: Toile::new(ML as usize, MH as usize),
            texture: None,
            ordre,
            trauma: 0.0,
            fait: 0,
            t_prec: 0.0,
            etincelles: Vec::new(),
            alea: Alea(poissons.first().map_or(1, |p| p.0.id) ^ 0x3017),
            sons: Vec::new(),
            cartes,
        }
    }

    fn palier_max(&self) -> usize {
        self.cartes.iter().map(|c| c.palier).max().unwrap_or(0)
    }

    /// Instant de l'éclatement commun.
    pub fn eclat(&self) -> f32 {
        ARRIVEE + 1.0 + PAS_PALIER * self.palier_max() as f32 + 0.5
    }

    /// Intervalle entre deux cartes qui se retournent.
    fn pas_carte(&self) -> f32 {
        if self.cartes.len() > 3 {
            0.26
        } else {
            0.45
        }
    }

    /// Instant où la carte de rang `k` (dans `ordre`) se retourne ; la dernière, si elle est
    /// rare, se fait attendre.
    fn retourne(&self, k: usize) -> f32 {
        let mut t = self.eclat() + 0.35 + k as f32 * self.pas_carte();
        if k + 1 == self.ordre.len() && self.derniere_rare() {
            t += 0.8;
        }
        t
    }

    fn derniere_rare(&self) -> bool {
        self.ordre
            .last()
            .is_some_and(|&i| self.cartes[i].palier >= 2 || self.cartes[i].shiny)
    }

    /// Fin de l'animation (secondes) : toutes les cartes retournées.
    pub fn fin(&self) -> f32 {
        self.retourne(self.ordre.len().saturating_sub(1)) + 0.7
    }

    /// Toutes les cartes sont retournées (texte et boutons sous la grille).
    pub fn revele(&self, maintenant: f64) -> bool {
        maintenant - self.debut >= (self.retourne(self.ordre.len().saturating_sub(1)) + 0.7) as f64
    }

    /// Clic : passe à la fin (toutes les cartes retournées).
    pub fn passer(&mut self, maintenant: f64) {
        let fin = (self.retourne(self.ordre.len().saturating_sub(1)) + 0.7) as f64;
        if maintenant - self.debut < fin {
            self.debut = maintenant - fin;
        }
    }

    /// Ids éclos, rareté, shiny (résumé sous la grille), dans l'ordre de la grille.
    pub fn ids(&self) -> Vec<u64> {
        self.cartes.iter().map(|c| c.id).collect()
    }

    fn une_fois(&mut self, b: u32) -> bool {
        let m = 1u64 << b;
        let neuf = self.fait & m == 0;
        self.fait |= m;
        neuf
    }

    pub fn dessiner(&mut self, t: f64) {
        let t = t as f32;
        let dt = (t - self.t_prec).clamp(0.0, 0.25);
        self.t_prec = t;
        for p in &mut self.etincelles {
            p.vie += dt;
            p.vy += 40.0 * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
        self.etincelles.retain(|p| p.vie < p.duree);
        self.trauma = (self.trauma - 1.6 * dt).max(0.0);
        let mut scene = Toile::new(ML as usize, MH as usize);
        scene.rect(0, 0, ML, MH, FOND, 255);
        let eclat = self.eclat();
        // Paliers communs : chaque saut fête les œufs qui montent encore.
        for j in 1..=self.palier_max() {
            let tj = ARRIVEE + 1.0 + PAS_PALIER * (j - 1) as f32;
            if t >= tj && self.une_fois(j as u32) {
                self.trauma = (self.trauma + 0.2 + 0.06 * j as f32).min(1.0);
                self.sons.push((Son::Palier(j as u8), 1.0));
            }
        }
        if t < eclat {
            self.oeufs(&mut scene, t);
        } else {
            if self.une_fois(10) {
                self.trauma = 0.7;
                let fort = self.palier_max() >= 3;
                self.sons
                    .push((if fort { Son::EclatFort } else { Son::Eclat }, 1.0));
                for k in 0..self.cartes.len() {
                    let (x, y, p) = (self.cartes[k].x, self.cartes[k].y, self.cartes[k].palier);
                    self.gerbe((x as f32, y as f32), 10, couleur_palier(p, 0.0), 110.0);
                }
            }
            // Avant la dernière carte rare : la grille tremble (suspense).
            let tr = self.retourne(self.ordre.len().saturating_sub(1));
            if self.derniere_rare() && (tr - 0.6..tr).contains(&t) {
                self.trauma = self.trauma.max(0.35);
            }
            self.rayons_derniere(&mut scene, t);
            self.cartes_dessin(&mut scene, t);
            if t - eclat < 0.15 {
                // Éclair de la couleur du meilleur palier, pas un gris plat.
                // Blanc, puis la couleur du plus beau palier.
                let v = (t - eclat) / 0.15;
                let c = melange([255, 255, 255], couleur_palier(self.palier_max(), t), v);
                calques::assombrir(&mut scene, c, 0.8 * (1.0 - v));
            }
        }
        for p in &self.etincelles {
            let a = (255.0 * (1.0 - p.vie / p.duree)).clamp(0.0, 255.0) as u8;
            scene.eclairer(p.x as i32 / 2, p.y as i32 / 2, p.c, a);
        }
        let k = self.trauma * self.trauma;
        let (ox, oy) = (
            (k * 4.0 * (t * 47.0).sin()).round() as i32 * 2,
            (k * 3.0 * (t * 53.0 + 1.3).sin()).round() as i32 * 2,
        );
        let toile = &mut self.toile;
        toile.rect(0, 0, ML, MH, FOND, 255);
        let (lf, hf) = (scene.lf() as i32, scene.hf() as i32);
        for y in 0..hf {
            for x in 0..lf {
                let (sx, sy) = (x - ox, y - oy);
                if sx >= 0 && sy >= 0 && sx < lf && sy < hf {
                    toile.px[(y * lf + x) as usize] = scene.px[(sy * lf + sx) as usize];
                }
            }
        }
    }

    fn gerbe(&mut self, (x, y): (f32, f32), n: usize, c: Rgb, vitesse: f32) {
        for _ in 0..n {
            let a = &mut self.alea;
            let ang = a.f() * TAU;
            let v = a.entre(0.3, 1.0) * vitesse;
            self.etincelles.push(Etincelle {
                x,
                y,
                vx: ang.cos() * v,
                vy: ang.sin() * v - vitesse * 0.3,
                vie: 0.0,
                duree: a.entre(0.4, 1.0),
                c,
            });
        }
    }

    /// Avant l'éclatement : les œufs tombent un à un, tremblent, leur lumière monte par
    /// paliers jusqu'à leur rareté puis reste (les plus rares continuent de monter).
    fn oeufs(&mut self, scene: &mut Toile, t: f32) {
        let n = self.cartes.len();
        for k in 0..n {
            let c = &self.cartes[k];
            let arrive = 0.06 * k as f32;
            if t < arrive {
                continue;
            }
            let u = ((t - arrive) / 0.3).min(1.0);
            let y = -(1.0 - u * u) * 140.0;
            if u >= 1.0 && self.fait & (1 << (20 + k)) == 0 {
                self.fait |= 1 << (20 + k);
                self.sons.push((Son::Chute, 0.9 + 0.03 * k as f32));
            }
            // Palier atteint par cet œuf à l'instant t.
            let atteint = ((t - (ARRIVEE + 1.0)) / PAS_PALIER + 1.0)
                .floor()
                .clamp(0.0, c.palier as f32) as usize;
            let monte = atteint < c.palier || t < ARRIVEE + 1.0;
            let amp = if monte {
                1.0 + 0.8 * atteint as f32
            } else {
                0.6
            };
            let penche = (t * (8.0 + atteint as f32 * 2.0) + k as f32).sin() * amp;
            let fiss = ((t - ARRIVEE) / 1.0).clamp(0.0, 0.6) + 0.1 * atteint as f32;
            let lum = (t > ARRIVEE + 0.4).then_some((
                super::paliers::lumiere_palier(atteint, t),
                0.5 + 0.12 * atteint as f32,
            ));
            if let Some((_, _)) = lum {
                let col = couleur_palier(atteint, t);
                calques::halo(
                    scene,
                    (c.x, c.y - 4),
                    26.0 + 6.0 * atteint as f32,
                    melange(col, FOND, 0.4),
                    0.5,
                    2,
                );
            }
            let img = image_oeuf(self.oeuf);
            dessiner_oeuf(
                scene,
                self.oeuf,
                (
                    c.x + (penche * 0.5).round() as i32,
                    c.y + img.h / 2 + y as i32,
                ),
                t + k as f32,
                penche,
                1.0,
                Some((&c.fiss, fiss.min(1.0))),
                lum,
                c.dore,
                false,
            );
        }
    }

    /// Rayons derrière la dernière carte (la plus rare), sous toutes les cartes.
    fn rayons_derniere(&self, scene: &mut Toile, t: f32) {
        let Some(&i) = self.ordre.last() else {
            return;
        };
        let derniere = self.derniere_rare();
        // Rayons dès la pause qui précède la dernière carte (suspense), puis derrière elle.
        let face = t >= self.retourne(self.ordre.len() - 1) - 0.6;
        let c = &self.cartes[i];
        // La plus rare en dernier : rayons derrière sa carte, de sa couleur.
        if face && derniere {
            let col = couleur_palier(c.palier, t);
            for k in 0..14 {
                let ang = k as f32 / 14.0 * TAU + t * 0.5;
                for r in 10..70 {
                    let (x, y) = (
                        c.x as f32 / 2.0 + ang.cos() * r as f32,
                        c.y as f32 / 2.0 + ang.sin() * r as f32,
                    );
                    scene.eclairer(
                        x.round() as i32,
                        y.round() as i32,
                        col,
                        (110.0 * (1.0 - r as f32 / 70.0)) as u8,
                    );
                }
            }
        }
    }

    /// Après l'éclatement : cartes de dos (couleur de l'œuf, point d'interrogation) qui se
    /// retournent une à une (largeur repliée par colonnes entières) sur la créature.
    fn cartes_dessin(&mut self, scene: &mut Toile, t: f32) {
        let n = self.ordre.len();
        let (cl, ch) = (ML * 2 / 5 - 6, MH * 2 / 2 - 6);
        for rang_k in 0..n {
            let i = self.ordre[rang_k];
            let tr = self.retourne(rang_k);
            let derniere = rang_k + 1 == n && self.derniere_rare();
            if t >= tr && self.une_fois(30 + rang_k as u32) {
                let c = &self.cartes[i];
                let s = if derniere {
                    Son::Jingle(c.rarete.index() as u8)
                } else if c.palier >= 2 || c.shiny {
                    Son::CarteRare
                } else {
                    Son::Carte
                };
                let (x, y, p) = (c.x, c.y, c.palier);
                self.sons.push((s, 1.0 + 0.02 * rang_k as f32));
                if derniere {
                    self.trauma = 0.8;
                }
                self.gerbe(
                    (x as f32, y as f32),
                    if p >= 2 { 24 } else { 8 },
                    couleur_palier(p, t),
                    90.0,
                );
            }
            let c = &self.cartes[i];
            // Repli : 1 → 0 avant tr, 0 → 1 après (0,09 s de chaque côté).
            let v = t - tr;
            let largeur = if v < -0.09 {
                1.0
            } else if v < 0.0 {
                -v / 0.09
            } else {
                (v / 0.09).min(1.0)
            };
            let face = v >= 0.0;
            let w = ((cl as f32 * largeur) as i32 / 2 * 2).max(2);
            let (x0, y0) = (c.x - w / 2, c.y - ch / 2);
            let bord = if face {
                // Cadre de la couleur de la rareté (Mythique ≠ Légendaire).
                c.rarete.couleur()
            } else {
                OEUFS[self.oeuf].coquille[1]
            };
            for y in 0..ch {
                for x in 0..w {
                    let cadre = x < 2 || y < 2 || x >= w - 2 || y >= ch - 2;
                    let col = if cadre && face && c.shiny {
                        // Shiny : cadre arc-en-ciel qui tourne.
                        hsv((x + y) as f32 / 60.0 + t * 0.8, 0.6, 1.0)
                    } else if cadre {
                        bord
                    } else if face {
                        melange(FOND, bord, 0.18)
                    } else {
                        melange(OEUFS[self.oeuf].coquille[0], [0x10, 0x20, 0x30], 0.55)
                    };
                    scene.poser_fin(x0 + x, y0 + y, col, 255);
                }
            }
            if !face {
                // Dos de carte : point d'interrogation dessiné (5 x 7, pixels monde ×2),
                // écrasé avec la carte qui se replie.
                const Q: [&str; 7] = [
                    ".###.", "#...#", "....#", "..##.", "..#..", ".....", "..#..",
                ];
                let col = OEUFS[self.oeuf].coquille[1];
                for (j, l) in Q.iter().enumerate() {
                    for (i, ch) in l.chars().enumerate() {
                        if ch == '#' {
                            let x = c.x + ((i as i32 - 2) * 4 * w) / cl;
                            let y = c.y - 14 + j as i32 * 4;
                            scene.rect(x / 2, y / 2, (2 * w / cl).max(1), 2, col, 255);
                        }
                    }
                }
                continue;
            }
            if largeur < 1.0 {
                continue;
            }
            // Créature (image 0), puis le nom de la rareté et les marques.
            let s = &c.sprite;
            let im = &s.images[0];
            let (sx, sy) = (c.x - s.l / 2, c.y - 6 - s.h / 2);
            let rebond = if v < 0.3 {
                (-(v / 0.3 * PI).sin() * 6.0) as i32
            } else {
                0
            };
            for yy in 0..s.h {
                for xx in 0..s.l {
                    let p = im.px((yy * s.l + xx) as usize);
                    if p[3] > 0 {
                        scene.poser_fin(sx + xx, sy + yy + rebond, [p[0], p[1], p[2]], 255);
                    }
                }
            }
            let col = c.rarete.couleur();
            let nom = c.rarete.nom().to_uppercase();
            let nom: String = nom.chars().take(9).collect();
            // Petite police sans agrandissement : tient dans la carte (« UNCOMMON »).
            // Trop long pour la carte (« UNCOMMON ») : coupé avec un point.
            let mut nom = nom;
            while crate::toile::largeur_texte(&nom) > cl / 2 - 6 && nom.chars().count() > 3 {
                let n = nom.trim_end_matches('.').chars().count();
                nom = nom.chars().take(n - 1).collect::<String>() + ".";
            }
            let lw = crate::toile::largeur_texte(&nom);
            scene.texte(
                c.x / 2 - lw / 2,
                (c.y + ch / 2) / 2 - 8,
                &nom,
                col,
                255,
                Some([0x04, 0x08, 0x10]),
            );
            let mut marques = Vec::new();
            if c.shiny {
                marques.push(([0xff, 0xb8, 0xe8], "S"));
            }
            if let Some(m) = c.modif {
                marques.push((m.couleur(), "M"));
            }
            if c.dore {
                marques.push(([0xff, 0xdc, 0x60], "D"));
            }
            if c.pitie {
                marques.push(([0xa8, 0xf0, 0xff], "G"));
            }
            for (j, (mc, l)) in marques.iter().enumerate() {
                let (mx, my) = (x0 / 2 + 2 + j as i32 * 6, y0 / 2 + 2);
                scene.rect(mx, my, 6, 7, [0x08, 0x12, 0x20], 255);
                scene.texte(mx + 1, my + 1, l, *mc, 255, None);
            }
            if c.shiny {
                let mut a = Alea(c.id ^ (t * 6.0) as u64);
                for _ in 0..2 {
                    let (x, y) = (
                        x0 + a.entre(4.0, w as f32 - 4.0) as i32,
                        y0 + a.entre(4.0, ch as f32 - 4.0) as i32,
                    );
                    calques::etoile(scene, (x, y), 2, [255, 200, 240], 2);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn poisson(id: u64, r: Rarete, shiny: bool) -> Poisson {
        let e = crate::especes::de_rarete(r).next().unwrap();
        Poisson {
            id,
            graine: id,
            rarete: r,
            taille: 1.0,
            niveau: 1,
            actif: true,
            oeuf: 0,
            espece: e.cle.to_string(),
            variante: 0,
            shiny,
            modif: None,
            verrou: false,
        }
    }

    #[test]
    fn la_plus_rare_en_dernier() {
        let rs = [
            Rarete::Rare,
            Rarete::Commun,
            Rarete::Legendaire,
            Rarete::PeuCommun,
            Rarete::Epique,
            Rarete::Commun,
            Rarete::Commun,
            Rarete::Rare,
            Rarete::Commun,
            Rarete::Commun,
        ];
        let v: Vec<(Poisson, bool, bool)> = rs
            .iter()
            .enumerate()
            .map(|(k, &r)| (poisson(k as u64 + 1, r, k == 6), false, false))
            .collect();
        let m = Multi::nouvelle(&v, 0, 0.0);
        assert_eq!(m.ids().len(), 10);
        // Ordre de retournement croissant ; la Légendaire (ou le shiny, palier 3) en dernier.
        let rangs: Vec<_> = m.ordre.iter().map(|&i| rang(&m.cartes[i])).collect();
        assert!(rangs.windows(2).all(|w| w[0] <= w[1]));
        assert_eq!(
            m.cartes[*m.ordre.last().unwrap()].rarete,
            Rarete::Legendaire
        );
        // Les cartes se retournent après l'éclatement, une à une, la dernière attendue.
        let tr: Vec<f32> = (0..10).map(|k| m.retourne(k)).collect();
        assert!(tr[0] > m.eclat());
        assert!(tr.windows(2).all(|w| w[1] > w[0]));
        assert!(tr[9] - tr[8] > tr[8] - tr[7]);
        // Toutes dans la grille, sans chevauchement.
        for a in &m.cartes {
            assert!(a.x > 0 && a.x < ML * 2 && a.y > 0 && a.y < MH * 2);
        }
        for (i, a) in m.cartes.iter().enumerate() {
            for b in &m.cartes[i + 1..] {
                assert!((a.x, a.y) != (b.x, b.y));
            }
        }
        let mut m = m;
        m.passer(0.0);
        assert!(m.revele(0.0));
    }
}

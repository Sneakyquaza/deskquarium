//! Scène d'éclosion propre à chaque œuf (v0.11), entre les paliers et l'éclatement, puis son
//! fond pendant la révélation. Les calques IA sont posés en pixels monde (2 pixels fins, la
//! grille de l'œuf agrandi) ; le code les fait tourner, glisser, s'ouvrir, fondre, briller.
//! Version renforcée (`fort` : haute rareté, shiny, modificateur) : plus longue, plus dense.

use super::calques::{self, calques, poser, poser_tourne};
use super::*;

/// Douce montée 0..1 entre `a` et `b`.
fn lisse(a: f32, b: f32, x: f32) -> f32 {
    let u = ((x - a) / (b - a)).clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

/// Nuit avec une zone éclairée : tout s'assombrit vers `c` (force `k`), sauf autour de
/// `lumiere` (centre, rayon, pixels fins) où l'ombre recule en trois paliers tramés.
fn nuit(t: &mut Toile, c: Rgb, k: f32, lumiere: Option<((i32, i32), f32)>) {
    let (lf, hf) = (t.lf() as i32, t.hf() as i32);
    for y in 0..hf {
        for x in 0..lf {
            let mut f = k;
            if let Some(((lx, ly), r)) = lumiere {
                let d = ((x - lx) as f32).hypot((y - ly) as f32) / r.max(1.0);
                let marche = ((1.0 - d) * 3.0 + crate::toile::bayer(x / 2, y / 2) * 0.7 - 0.35)
                    .floor()
                    .clamp(0.0, 3.0);
                f *= 1.0 - marche / 3.0;
            }
            let p = &mut t.px[(y * lf + x) as usize];
            let kk = (f.clamp(0.0, 1.0) * 256.0) as u32;
            for i in 0..3 {
                p[i] = ((p[i] as u32 * (256 - kk) + c[i] as u32 * kk) >> 8) as u8;
            }
        }
    }
}

impl Eclosion {
    /// Centre de l'œuf dans la scène (pixels fins).
    fn centre_oeuf(&self) -> (i32, i32) {
        (L, SOL - image_oeuf(self.oeuf).h * K / 2)
    }

    /// L'œuf, ses fêlures grandes ouvertes, dans la lumière du palier final, décalé de `dy`.
    fn oeuf_ouvert(&self, scene: &mut Toile, t: f32, dy: i32, couleur: Rgb, penche: f32) {
        dessiner_oeuf_k(
            scene,
            K,
            self.oeuf,
            (L + (penche * 0.5).round() as i32, SOL + dy),
            t,
            penche,
            1.0,
            Some((&self.fissures, 1.0)),
            Some((couleur, 1.0)),
            self.dore,
            false,
        );
    }

    /// La scène de l'œuf, `u` = 0..1 de sa durée.
    pub(super) fn scene_oeuf(&mut self, scene: &mut Toile, t: f32, u: f32, couleur: Rgb) {
        let fort = self.d.fort;
        let tremble = (t * 55.0).sin() * (1.5 + 2.0 * u) * if fort { 1.4 } else { 1.0 };
        match self.oeuf {
            COSMIQUE => self.scene_cosmique(scene, t, u, fort),
            ECLIPSE => self.scene_eclipse(scene, t, u, couleur, tremble),
            AMBRE => self.scene_ambre(scene, t, u, fort),
            ABYSSES => self.scene_abysses(scene, t, u, couleur),
            LAGON => self.scene_lagon(scene, t, u, couleur, tremble),
            RIVIERE => self.scene_riviere(scene, t, u, fort),
            DORE => self.scene_dore(scene, t, u, couleur, tremble, fort),
            _ => self.scene_arc(scene, t, u, couleur, tremble),
        }
        // Raretés hautes : la couleur du palier balaie l'écran.
        if fort {
            balayage(scene, couleur, u);
        }
        self.trauma = self.trauma.max(0.12 + 0.3 * u);
    }

    /// Cosmique : l'œuf s'effondre en tournant dans un trou noir (disque d'accrétion qui
    /// tourne, espace aspiré, étoiles qui y tombent), qui se resserre en un point.
    fn scene_cosmique(&mut self, scene: &mut Toile, t: f32, u: f32, fort: bool) {
        if self.une_fois(32) {
            self.son(Son::TrouNoir, 1.0);
        }
        let c = calques();
        let (cx, cy) = self.centre_oeuf();
        calques::assombrir(scene, [0x06, 0x04, 0x14], lisse(0.0, 0.3, u) * 0.9);
        // Étoiles qui tombent en spirale vers le centre.
        let mut a = Alea(self.id ^ 0xc05);
        let n = if fort { 110 } else { 70 };
        for _ in 0..n {
            let (a0, r0, v) = (a.f() * TAU, a.entre(30.0, 190.0), a.entre(0.5, 1.2));
            let w = (u * v * 1.6 + a.f()).fract();
            let r = r0 * (1.0 - w).powf(1.4);
            let ang = a0 + w * 4.0;
            let (x, y) = (cx as f32 + ang.cos() * r, cy as f32 + ang.sin() * r * 0.8);
            let col = [[255, 255, 255], [0xd8, 0xb8, 0xff], [0xa8, 0xf0, 0xff]][a.n(3) as usize];
            let al = (90.0 + 160.0 * w) as u8;
            scene.eclairer(x.round() as i32 / 2, y.round() as i32 / 2, col, al);
            if w > 0.6 {
                // Traînée d'un pixel vers l'extérieur.
                let (x2, y2) = (x + ang.cos() * 3.0, y + ang.sin() * 2.4);
                scene.eclairer(x2.round() as i32 / 2, y2.round() as i32 / 2, col, al / 2);
            }
        }
        // L'espace est aspiré et tourne autour du trou.
        let k = lisse(0.15, 0.6, u);
        calques::deformer(scene, (cx, cy), 160.0, 0.9 * k, 1.6 * k, 2);
        // L'œuf s'effondre en tournant.
        if u < 0.45 {
            let e = 1.0 - lisse(0.0, 0.45, u) * 0.95;
            poser_tourne(
                scene,
                image_oeuf(COSMIQUE),
                (cx, cy),
                2,
                u * u * 14.0,
                e,
                1.0,
                255,
                false,
                |c, _, _| c,
            );
        }
        // Trou noir : disque d'accrétion vu de biais qui tourne, se resserre à la fin.
        let e = lisse(0.2, 0.5, u) * (1.0 - lisse(0.86, 1.0, u)) * if fort { 0.95 } else { 0.85 };
        if e > 0.02 {
            calques::halo(scene, (cx, cy), 150.0 * e, [0xff, 0x90, 0x40], 0.5 * e, 2);
            poser_tourne(
                scene,
                &c.trou_noir,
                (cx, cy),
                2,
                -t * 2.6,
                e,
                0.62,
                255,
                false,
                |c, _, _| c,
            );
        }
        // Le point de lumière juste avant l'explosion.
        if u > 0.86 {
            let v = lisse(0.86, 1.0, u);
            calques::halo(scene, (cx, cy), 20.0 + 60.0 * v, [255, 255, 255], v, 2);
        }
    }

    /// Éclipse : l'œuf descend, la nuit tombe pendant que la lune couvre le soleil, la
    /// couronne flambe, l'anneau de diamant éclate, puis tout s'embrase.
    fn scene_eclipse(&mut self, scene: &mut Toile, t: f32, u: f32, couleur: Rgb, tremble: f32) {
        if self.une_fois(32) {
            self.son(Son::Eclipse, 1.0);
        }
        let c = calques();
        calques::assombrir(scene, [0x02, 0x02, 0x08], lisse(0.0, 0.55, u) * 0.92);
        let (sx, sy) = (L, 52);
        let totale = lisse(0.5, 0.62, u) * (1.0 - lisse(0.93, 1.0, u));
        // Couronne : plus vive à la totalité (le soleil couvert la laisse flamber).
        let s = &c.soleil;
        if totale > 0.0 {
            calques::halo(scene, (sx, sy), 120.0, [0xff, 0xb0, 0x50], 0.6 * totale, 2);
        }
        poser(scene, s, (sx - s.l, sy - s.h), 2, 255, |col| {
            let (h, sa, v) = rgb_hsv(col);
            hsv(h, sa, (v * (0.75 + 0.45 * totale)).min(1.0))
        });
        if totale > 0.0 {
            for y in 0..s.h {
                for x in 0..s.l {
                    if s.px(x, y)[3] == 0 {
                        continue;
                    }
                    let k = (t * 9.0 + bruit(x, y, 3) * TAU).sin();
                    if k > 0.3 {
                        let a = (totale * 120.0 * k) as u8;
                        scene.eclairer((sx - s.l) / 2 + x, (sy - s.h) / 2 + y, [255, 220, 140], a);
                    }
                }
            }
        }
        // La lune glisse de la gauche et couvre le soleil.
        let l = &c.lune;
        let mx = (sx - (150.0 * (1.0 - lisse(0.0, 0.6, u))) as i32) & !1;
        poser(scene, l, (mx - l.l, sy - l.h), 2, 255, |c| c);
        // Anneau de diamant : un point éblouissant au bord, puis l'éclair.
        if (0.78..0.97).contains(&u) {
            let v = lisse(0.78, 0.86, u);
            let a = &c.anneau;
            poser_tourne(
                scene,
                a,
                (sx, sy),
                1,
                0.0,
                1.25,
                1.0,
                (255.0 * v) as u8,
                true,
                |c, _, _| c,
            );
            if self.une_fois(33) {
                self.flash = t + 0.05;
                self.trauma = (self.trauma + 0.4).min(1.0);
                self.son(Son::Shiny, 0.8);
            }
        }
        // L'œuf descend sous le soleil, ses fêlures de lave brûlent.
        let dy = (lisse(0.0, 0.3, u) * 110.0) as i32 & !1;
        self.oeuf_ouvert(scene, t, dy, couleur, tremble);
        if u > 0.95 {
            calques::assombrir(scene, [255, 250, 230], lisse(0.95, 1.0, u));
        }
    }

    /// Ambre : l'œuf fond et coule en gouttes, le fossile se libère, s'illumine et s'anime.
    fn scene_ambre(&mut self, scene: &mut Toile, t: f32, u: f32, fort: bool) {
        if self.une_fois(32) {
            self.son(Son::Fonte, 1.0);
        }
        let c = calques();
        let img = image_oeuf(AMBRE);
        let (cx, cy) = self.centre_oeuf();
        let (x0, y0) = (L - img.l * K / 2, SOL - img.h * K);
        // Fonte : chaque colonne s'affaisse à sa vitesse ; flaque qui s'étale en bas.
        let m = lisse(0.0, 0.75, u);
        let ambre = &c.ambre_coque;
        for x in 0..img.l {
            let mx = (m * (0.65 + 0.45 * bruit(x, 9, 4))).min(1.0);
            for sy in 0..img.h {
                if ambre.px(x, sy)[3] == 0 {
                    continue;
                }
                let p = ambre.px(x, sy);
                let y = sy as f32;
                // La colonne se tasse vers le bas : ligne `sy` → bas - (bas - sy)(1 - mx).
                let y = img.h as f32 - (img.h as f32 - y) * (1.0 - mx * 0.85);
                let etale = 1.0 + 0.6 * mx * (sy as f32 / img.h as f32);
                let xx = img.l as f32 / 2.0 + (x as f32 - img.l as f32 / 2.0) * etale;
                let col = melange([p[0], p[1], p[2]], [0xff, 0xa8, 0x30], 0.25 * mx);
                scene.poser(
                    (x0 / 2) + xx.round() as i32,
                    (y0 / 2) + y.round() as i32,
                    col,
                    255,
                );
            }
        }
        // Gouttes d'ambre qui tombent des bords.
        if u < 0.7 && self.alea.f() < if fort { 0.12 } else { 0.08 } {
            let g = &c.gouttes[self.alea.n(c.gouttes.len().max(1) as u32) as usize];
            let x = cx as f32 + self.alea.entre(-50.0, 50.0);
            self.objets.push(Objet {
                img: g,
                x,
                y: (cy - 40) as f32,
                vx: self.alea.entre(-25.0, 25.0),
                vy: self.alea.entre(-60.0, 10.0),
                vie: 0.0,
                sol: Some((SOL + 2) as f32),
            });
        }
        // Le fossile, libéré, flotte au centre, s'illumine d'or et s'agite.
        let f = &c.fossile;
        let reveil = lisse(0.45, 0.95, u);
        let bouge = (t * (6.0 + 14.0 * reveil)).sin() * 0.12 * reveil;
        let fy = (cy - 6 - (reveil * 20.0) as i32) & !1;
        calques::halo(
            scene,
            (cx, fy),
            40.0 + 70.0 * reveil,
            [0xff, 0xc0, 0x50],
            0.3 + 0.7 * reveil,
            2,
        );
        poser_tourne(
            scene,
            f,
            (cx, fy),
            2,
            bouge,
            1.0,
            1.0,
            (120.0 + 135.0 * m) as u8,
            false,
            |col, x, y| {
                let or = melange(col, [0xff, 0xd8, 0x70], reveil * 0.9);
                if reveil > 0.5 && bruit(x, y, (t * 8.0) as i32) > 0.9 {
                    [255, 250, 220]
                } else {
                    or
                }
            },
        );
    }

    /// Abysses : noir total, des yeux s'allument un à un, une lanterne approche et éclaire.
    fn scene_abysses(&mut self, scene: &mut Toile, t: f32, u: f32, couleur: Rgb) {
        if self.une_fois(32) {
            self.son(Son::Abysse, 1.0);
        }
        let c = calques();
        let (cx, cy) = self.centre_oeuf();
        self.oeuf_ouvert(scene, t, 0, couleur, (t * 30.0).sin() * 1.2);
        // La lanterne arrive de loin (en haut à gauche), sa lumière grandit.
        let v = lisse(0.5, 0.95, u);
        let (lx, ly) = (
            (-30.0 + (cx as f32 - 40.0 + 30.0) * v) as i32 & !1,
            (20.0 + (cy as f32 - 70.0 - 20.0) * v) as i32 & !1,
        );
        let lumiere = (v > 0.0).then_some(((lx + 6, ly - 10), 40.0 + 150.0 * v));
        nuit(
            scene,
            [0x00, 0x02, 0x08],
            0.97 * lisse(0.0, 0.15, u),
            lumiere,
        );
        // Yeux autour, un à un (les grands au bord, les petits plus près) ; ils se ferment
        // quand la lumière arrive.
        let places = [
            (30, 40, true),
            (226, 120, true),
            (70, 150, false),
            (196, 46, false),
            (52, 100, false),
            (210, 170, false),
            (90, 24, false),
        ];
        for (k, &(x, y, grand)) in places.iter().enumerate() {
            let debut = 0.12 + k as f32 * 0.07;
            if u < debut {
                continue;
            }
            if self.une_fois(34 + k as u32) {
                self.son(Son::Oeil, 0.9 + 0.05 * k as f32);
            }
            let ouvert = lisse(debut, debut + 0.06, u) * (1.0 - lisse(0.8, 0.92, u));
            if ouvert <= 0.02 {
                continue;
            }
            if grand {
                poser_tourne(
                    scene,
                    &c.oeil,
                    (x, y),
                    1,
                    0.0,
                    1.0,
                    ouvert,
                    255,
                    false,
                    |c, _, _| c,
                );
            } else {
                let regard = if lx < x { -2 } else { 2 };
                petit_oeil(scene, (x & !1, y & !1), ouvert, regard, 2);
            }
        }
        if v > 0.0 {
            let lt = &c.lanterne;
            poser(scene, lt, (lx - lt.l / 2, ly - lt.h / 2), 1, 255, |c| c);
            calques::halo(
                scene,
                (lx + 6, ly - 10),
                24.0 + 30.0 * v,
                [0x90, 0xff, 0xf0],
                0.8,
                2,
            );
        }
    }

    /// Lagon : une vague balaie tout ; derrière elle, une huître géante ; elle s'ouvre, la
    /// perle brille de plus en plus.
    fn scene_lagon(&mut self, scene: &mut Toile, t: f32, u: f32, couleur: Rgb, tremble: f32) {
        if self.une_fois(32) {
            self.son(Son::Vague, 1.0);
        }
        let c = calques();
        let front = (-60.0 + lisse(0.0, 0.38, u) * 400.0) as i32;
        // Avant la vague : l'œuf ; après : l'huître (chacun de son côté du front).
        let mut avant = Toile::new(L as usize, HT as usize);
        self.oeuf_ouvert(&mut avant, t, 0, couleur, tremble);
        let mut apres = Toile::new(L as usize, HT as usize);
        let ouvre = lisse(0.4, 0.75, u);
        let (bx, by) = (L, SOL);
        let b = &c.huitre_bas;
        // Perle au creux de l'huître, de plus en plus lumineuse.
        let perle = lisse(0.5, 1.0, u);
        let (px, py) = (bx, by - b.h + 10);
        if ouvre > 0.1 {
            calques::halo(
                &mut apres,
                (px, py),
                30.0 + 90.0 * perle,
                [0xff, 0xe8, 0xf8],
                0.4 + 0.6 * perle,
                2,
            );
            let p = &c.perle;
            let blanc = lisse(0.85, 1.0, u);
            poser(&mut apres, p, (px - p.l / 2, py - p.h / 2), 1, 255, |col| {
                melange(col, [255, 255, 255], blanc)
            });
        }
        poser(&mut apres, b, (bx - b.l / 2, by - b.h), 1, 255, |c| c);
        let h = &c.huitre_haut;
        let lev = (ouvre * 70.0) as i32 & !1;
        poser_tourne(
            &mut apres,
            h,
            (bx, by - b.h - h.h / 2 + 6 - lev),
            1,
            0.0,
            1.0,
            1.0 - 0.25 * ouvre,
            255,
            false,
            |c, _, _| c,
        );
        if self.une_fois(33) {
            self.objets.clear();
        }
        if ouvre > 0.0 && self.une_fois(34) {
            self.son(Son::Coquillage, 1.0);
        }
        let lf = scene.lf() as i32;
        for y in 0..scene.hf() as i32 {
            for x in 0..lf {
                let src = if x > front { &avant } else { &apres };
                let p = src.px[(y * lf + x) as usize];
                if p[3] > 0 {
                    let d = crate::toile::demultiplier(p);
                    scene.poser_fin(x, y, [d[0], d[1], d[2]], d[3]);
                }
            }
        }
        // La vague : une bande d'eau qui roule, crête d'écume, en paliers de bleu.
        for x in (front - 90).max(0)..(front + 10).min(lf) {
            let k = (x - (front - 90)) as f32 / 100.0;
            let haut = (HT * 2) as f32 * (1.0 - 0.95 * (k * PI).sin().max(0.0))
                + (x as f32 * 0.15 + t * 9.0).sin() * 4.0;
            for y in (haut as i32).max(0)..HT * 2 {
                let prof = (y as f32 - haut) / 40.0;
                let col = if prof < 0.12 {
                    [0xf0, 0xff, 0xff]
                } else if prof < 0.5 {
                    [0x7c, 0xd8, 0xf0]
                } else if prof < 1.4 {
                    [0x34, 0x9c, 0xd0]
                } else {
                    [0x1c, 0x60, 0x9c]
                };
                scene.poser_fin(x & !1, y & !1, col, 235);
            }
        }
    }

    /// Rivière : un tourbillon aspire l'œuf qui tourne et s'enfonce, puis une gerbe d'eau.
    fn scene_riviere(&mut self, scene: &mut Toile, t: f32, u: f32, fort: bool) {
        if self.une_fois(32) {
            self.son(Son::Remous, 1.0);
        }
        let c = calques();
        let (cx, cy) = self.centre_oeuf();
        let k = lisse(0.0, 0.3, u);
        // Remous en spirale, trois tons d'eau et une ligne d'écume, sur la grille monde.
        for y in 0..HT {
            for x in 0..L {
                let (dx, dy) = ((x * 2 - cx) as f32, ((y * 2 - cy) as f32) * 1.3);
                let r = dx.hypot(dy);
                if r > 170.0 * k {
                    continue;
                }
                let ang = dy.atan2(dx);
                let s = (ang / TAU + r * 0.012 - t * 1.4).rem_euclid(1.0);
                let col = if s < 0.06 {
                    [0xe0, 0xfa, 0xff]
                } else if s < 0.4 {
                    [0x2c, 0x84, 0xb0]
                } else if s < 0.75 {
                    [0x1c, 0x5c, 0x88]
                } else {
                    [0x12, 0x40, 0x68]
                };
                let a = ((1.0 - r / 170.0) * 255.0 * k).min(230.0) as u8;
                scene.poser(x, y, col, a);
            }
        }
        // L'œuf tourne et s'enfonce au centre.
        if u < 0.6 {
            let e = 1.0 - lisse(0.05, 0.6, u) * 0.9;
            poser_tourne(
                scene,
                image_oeuf(RIVIERE),
                (cx, cy + (u * 30.0) as i32),
                2,
                u * u * 18.0,
                e,
                1.0,
                255,
                false,
                |c, _, _| c,
            );
        }
        // Gerbe d'eau qui jaillit du centre.
        if u > 0.6 {
            if self.une_fois(33) {
                self.son(Son::Vague, 1.3);
                self.trauma = (self.trauma + 0.4).min(1.0);
                let n = if fort { 50 } else { 30 };
                self.gerbe((cx as f32, cy as f32), n, [0xb0, 0xe8, 0xff], 140.0);
            }
            let s = &c.eclaboussure;
            let v = lisse(0.6, 0.8, u);
            let y = cy + 40 - (v * 30.0) as i32;
            poser(scene, s, (cx - s.l / 2, y - s.h), 1, 255, |c| c);
        }
    }

    /// Doré : pluie de pièces et de pierres qui rebondissent et s'entassent, puis l'éclat du
    /// trésor.
    fn scene_dore(
        &mut self,
        scene: &mut Toile,
        t: f32,
        u: f32,
        couleur: Rgb,
        tremble: f32,
        fort: bool,
    ) {
        if self.une_fois(32) {
            self.son(Son::Tresor, 1.0);
        }
        if u > 0.45 && self.une_fois(33) {
            self.son(Son::Tresor, 1.12);
        }
        let c = calques();
        let (cx, cy) = self.centre_oeuf();
        if u < 0.78 && self.alea.f() < if fort { 0.25 } else { 0.16 } {
            let p = &c.tresor[self.alea.n(c.tresor.len().max(1) as u32) as usize];
            let x = self.alea.entre(10.0, 246.0);
            let sol = SOL as f32 + 4.0 - self.alea.entre(0.0, 10.0);
            self.objets.push(Objet {
                img: p,
                x,
                y: -10.0,
                vx: self.alea.entre(-15.0, 15.0),
                vy: self.alea.entre(20.0, 80.0),
                vie: 0.0,
                sol: Some(sol),
            });
        }
        self.oeuf_ouvert(scene, t, 0, couleur, tremble);
        // L'éclat du trésor : l'or monte, étincelles de joaillier sur l'œuf.
        let v = lisse(0.6, 1.0, u);
        calques::halo(
            scene,
            (cx, cy),
            40.0 + 120.0 * v,
            [0xff, 0xd8, 0x60],
            0.3 + 0.7 * v,
            2,
        );
        let mut a = Alea(self.id ^ (t * 12.0) as u64);
        for _ in 0..(2.0 + 6.0 * v) as usize {
            let (x, y) = (
                cx + a.entre(-60.0, 60.0) as i32,
                cy + a.entre(-80.0, 80.0) as i32,
            );
            calques::etoile(scene, (x, y), 1 + a.n(3) as i32, [255, 230, 140], 2);
        }
    }

    /// Arc-en-ciel : l'œuf descend, un prisme se pose au-dessus, un rayon blanc le traverse
    /// et s'ouvre en éventail arc-en-ciel sur l'œuf, puis tout converge.
    fn scene_arc(&mut self, scene: &mut Toile, t: f32, u: f32, couleur: Rgb, tremble: f32) {
        let c = calques();
        let (px, py) = (L, 44);
        let dy = (lisse(0.0, 0.2, u) * 56.0) as i32 & !1;
        self.oeuf_ouvert(scene, t, dy, couleur, tremble);
        // Le prisme tombe en place.
        let v = lisse(0.08, 0.24, u);
        if v > 0.0 {
            let p = &c.prisme;
            let y = (py - ((1.0 - v) * 80.0) as i32) & !1;
            poser(scene, p, (px - p.l / 2, y - p.h / 2), 1, 255, |c| c);
        }
        // Le rayon blanc arrive de la gauche.
        let r = lisse(0.24, 0.34, u);
        if r > 0.0 {
            if self.une_fois(32) {
                self.son(Son::Prisme, 1.0);
            }
            let x1 = (px as f32 * r) as i32;
            for x in 0..x1 / 2 {
                let y = (py - 20) / 2 + (x * 20 / (px / 2)).min(10);
                scene.eclairer(x, y, [255, 255, 255], 230);
                scene.eclairer(x, y + 1, [255, 255, 240], 120);
            }
        }
        // L'éventail : sept bandes de couleur qui s'ouvrent vers le bas, plus larges ensuite.
        let f = lisse(0.34, 0.6, u);
        if f > 0.0 {
            let ouverture = 0.35 + 0.55 * lisse(0.6, 0.88, u) - 0.5 * lisse(0.9, 1.0, u);
            let long = 40.0 + 120.0 * f;
            let (ox, oy) = (px / 2, py / 2);
            for y in oy..HT {
                for x in 0..L {
                    let (dx, dy) = ((x - ox) as f32, (y - oy) as f32);
                    let d = dx.hypot(dy);
                    if d < 6.0 || d > long {
                        continue;
                    }
                    let ang = dx.atan2(dy); // 0 = vers le bas
                    let w = ang / ouverture.max(0.05) * 0.5 + 0.5;
                    if !(0.0..1.0).contains(&w) {
                        continue;
                    }
                    let bande = (w * 7.0).floor() / 7.0;
                    let col = hsv(0.0 - bande * 0.8 + 0.8, 0.7, 1.0);
                    let palier = (3.0 - (d / long * 3.0).floor()).clamp(1.0, 3.0);
                    scene.eclairer(x, y, col, (palier * 40.0 * f) as u8);
                }
            }
        }
        if u > 0.92 {
            calques::halo(
                scene,
                self.centre_oeuf(),
                30.0 + 600.0 * (u - 0.92),
                [255, 255, 255],
                1.0,
                2,
            );
        }
    }

    /// À l'éclatement : ce que la scène de l'œuf projette (pièces, gouttes, eau, étoiles).
    pub(super) fn eclat_theme(&mut self, (cx, cy): (f32, f32)) {
        let c = calques();
        match self.oeuf {
            DORE => {
                for _ in 0..if self.d.fort { 22 } else { 14 } {
                    let p = &c.tresor[self.alea.n(c.tresor.len().max(1) as u32) as usize];
                    let ang = self.alea.entre(PI * 1.05, PI * 1.95);
                    let v = self.alea.entre(120.0, 260.0);
                    self.objets.push(Objet {
                        img: p,
                        x: cx,
                        y: cy,
                        vx: ang.cos() * v,
                        vy: ang.sin() * v,
                        vie: 0.0,
                        sol: Some(SOL as f32 + 4.0),
                    });
                }
            }
            AMBRE => {
                for _ in 0..8 {
                    let g = &c.gouttes[self.alea.n(c.gouttes.len().max(1) as u32) as usize];
                    let ang = self.alea.entre(PI * 1.1, PI * 1.9);
                    let v = self.alea.entre(80.0, 180.0);
                    self.objets.push(Objet {
                        img: g,
                        x: cx,
                        y: cy,
                        vx: ang.cos() * v,
                        vy: ang.sin() * v,
                        vie: 0.0,
                        sol: Some(SOL as f32 + 2.0),
                    });
                }
            }
            RIVIERE | LAGON => self.gerbe((cx, cy), 50, [0xb8, 0xf0, 0xff], 160.0),
            COSMIQUE => {
                self.gerbe((cx, cy), 50, [0xe0, 0xc8, 0xff], 190.0);
                self.gerbe((cx, cy), 30, [0xa0, 0xf0, 0xff], 140.0);
            }
            ABYSSES => self.gerbe((cx, cy), 50, [0x70, 0xff, 0xe0], 120.0),
            ECLIPSE => self.gerbe((cx, cy), 60, [0xff, 0xc8, 0x50], 180.0),
            _ => {
                for k in 0..7 {
                    self.gerbe((cx, cy), 8, hsv(k as f32 / 7.0, 0.6, 1.0), 170.0);
                }
            }
        }
    }

    /// Fond de la révélation, propre à l'œuf (derrière les rayons et la créature).
    pub(super) fn fond_revelation(&mut self, scene: &mut Toile, apres: f32) {
        let c = calques();
        let (cx, cy) = self.centre_oeuf();
        match self.oeuf {
            COSMIQUE => {
                // La galaxie explose depuis le point, puis tourne lentement derrière.
                calques::assombrir(
                    scene,
                    [0x06, 0x04, 0x14],
                    0.9 * (1.0 - lisse(1.5, 3.0, apres)),
                );
                let e = 0.2 + 0.9 * lisse(0.0, 0.6, apres);
                let a = (255.0 * (1.0 - 0.55 * lisse(0.6, 1.6, apres))) as u8;
                poser_tourne(
                    scene,
                    &c.galaxie,
                    (cx, cy),
                    2,
                    apres * 0.5,
                    e,
                    0.8,
                    a,
                    false,
                    |c, _, _| c,
                );
            }
            ECLIPSE => {
                let v = 1.0 - lisse(0.0, 0.5, apres);
                calques::assombrir(scene, [0xff, 0xf0, 0xd0], 0.7 * v);
                let s = &c.soleil;
                calques::halo(scene, (L, 52), 110.0, [0xff, 0xc0, 0x60], 0.6, 2);
                poser(scene, s, (L - s.l, 52 - s.h), 2, 255, |c| c);
            }
            AMBRE => {
                calques::halo(scene, (cx, cy), 130.0, [0xff, 0xa8, 0x30], 0.5, 2);
                for x in -60..=60 {
                    let h = (3.0 * (1.0 - (x as f32 / 60.0).powi(2))).round() as i32;
                    for y in 0..=h {
                        let col = if y == h {
                            [0xff, 0xd0, 0x70]
                        } else {
                            [0xd8, 0x80, 0x20]
                        };
                        scene.poser(L / 2 + x, SOL / 2 + 2 - y, col, 255);
                    }
                }
            }
            ABYSSES => {
                nuit(scene, [0x00, 0x02, 0x08], 0.85, Some(((cx, cy), 120.0)));
                let mut a = Alea(self.id ^ 0xab1);
                for _ in 0..30 {
                    let (x, y) = (a.entre(0.0, 128.0), a.entre(0.0, 96.0));
                    let y = (y - apres * a.entre(2.0, 6.0)).rem_euclid(96.0);
                    let k = (apres * 2.0 + a.f() * TAU).sin();
                    if k > 0.0 {
                        scene.eclairer(x as i32, y as i32, [0x80, 0xff, 0xe8], (160.0 * k) as u8);
                    }
                }
            }
            LAGON => {
                let b = &c.huitre_bas;
                poser(scene, b, (L - b.l / 2, SOL - b.h), 1, 255, |c| c);
                let h = &c.huitre_haut;
                poser_tourne(
                    scene,
                    h,
                    (L, SOL - b.h - h.h / 2 + 6 - 70),
                    1,
                    0.0,
                    1.0,
                    0.75,
                    255,
                    false,
                    |c, _, _| c,
                );
            }
            RIVIERE => {
                // Ronds dans l'eau qui s'élargissent.
                for k in 0..3 {
                    let r = ((apres * 50.0 + k as f32 * 22.0) % 66.0) + 6.0;
                    let a = (200.0 * (1.0 - r / 72.0)) as u8;
                    for i in 0..120 {
                        let ang = i as f32 / 120.0 * TAU;
                        let (x, y) = (
                            64.0 + ang.cos() * r,
                            (SOL / 2 - 6) as f32 + ang.sin() * r * 0.25,
                        );
                        scene.eclairer(x.round() as i32, y.round() as i32, [0xc0, 0xf0, 0xff], a);
                    }
                }
            }
            DORE => {
                calques::halo(scene, (cx, cy), 150.0, [0xff, 0xd8, 0x60], 0.6, 2);
            }
            _ => {
                // Éventail arc-en-ciel qui tourne lentement derrière la créature.
                for y in 0..HT {
                    for x in 0..L {
                        let (dx, dy) = ((x * 2 - cx) as f32, (y * 2 - cy) as f32);
                        let ang = dy.atan2(dx) + apres * 0.4;
                        let bande = ((ang / TAU * 14.0).rem_euclid(14.0)).floor();
                        if bande as i32 % 2 == 0 {
                            let col = hsv(bande / 14.0, 0.6, 1.0);
                            scene.eclairer(x, y, col, 45);
                        }
                    }
                }
            }
        }
    }
}

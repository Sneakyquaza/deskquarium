//! L'aquarium dessiné sur la toile basse résolution : eau teintée, rayons, sable, rochers,
//! plantes, bulles, poissons (nage, banc, effets de rareté), « +x » et compteur.

use crate::jeu::{court, Alea, Partie, Rarete};
use crate::poisson::{self, Sprite, CONTOUR, CORPS, NAGEOIRE, VIDE};
use crate::toile::{bayer, hsv, largeur_texte, melange, Rgb, Toile};
use std::collections::{HashMap, VecDeque};
use std::f32::consts::TAU;
use std::sync::Arc;

/// Échelle entière toile → écran : ~480 pixels de toile en largeur.
pub fn echelle_pour(largeur_ecran: u32) -> u32 {
    ((largeur_ecran as f32 / 480.0).round() as u32).clamp(2, 8)
}

pub struct Nageur {
    pub id: u64,
    pub rarete: Rarete,
    pub sprite: Arc<Sprite>,
    pub x: f32,
    pub y: f32,
    vx: f32,
    vy: f32,
    cible: (f32, f32),
    croisiere: f32,
    pause: f32,
    minuteur: f32,
    anim: f32,
    pub gauche: bool,
    tourne: f32,
    phase: f32,
    banc: Option<(u64, f32, f32)>,
    traine: VecDeque<(f32, f32)>,
    glitch: f32,
    prochain_glitch: f32,
    /// Horloge de l'ondulation verticale.
    bob: f32,
    gain_dans: f32,
    revenu: f64,
    /// Immobile (éclosion, planche) : la traînée est simulée.
    pub fixe: bool,
}

struct Bulle {
    x: f32,
    y: f32,
    r: u8,
    v: f32,
    phase: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum Genre {
    Scintille,
    Poussiere,
    Braise,
    Eclat,
}

struct Particule {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    vie: f32,
    duree: f32,
    c: Rgb,
    genre: Genre,
}

struct Gain {
    x: f32,
    y: f32,
    texte: String,
    vie: f32,
}

struct Plante {
    x: i32,
    base: i32,
    hauteur: i32,
    epaisseur: i32,
    couleurs: [Rgb; 3],
    phase: f32,
    devant: bool,
    feuilles: bool,
}

pub struct Aquarium {
    pub toile: Toile,
    /// Eau teintée + sable + rochers, refait quand la teinte change.
    fond: Toile,
    rochers: Toile,
    teinte_fond: f32,
    sable: Vec<i32>,
    plantes: Vec<Plante>,
    sources: Vec<(i32, i32)>,
    pub nageurs: Vec<Nageur>,
    bulles: Vec<Bulle>,
    particules: Vec<Particule>,
    gains: Vec<Gain>,
    pub temps: f32,
    alea: Alea,
    sprites: HashMap<u64, Arc<Sprite>>,
    /// Message affiché au-dessus du compteur (gains hors ligne) et sa durée restante.
    pub message: Option<(String, f32)>,
    /// Compteur affiché (l'argent arrive en continu, on lisse l'affichage).
    pub avec_compteur: bool,
    /// Sable, plantes, rayons et bulles (faux pour la planche des raretés).
    pub decor: bool,
}

pub fn sprite_de(graine: u64, rarete: Rarete, taille: f32) -> Sprite {
    poisson::sprite(&poisson::genome(graine, rarete, taille))
}

impl Aquarium {
    pub fn new(l: usize, h: usize, graine: u64) -> Aquarium {
        let mut a = Alea(graine);
        let hs = ((h as f32 * 0.085) as i32).max(12);
        let p1 = a.f() * TAU;
        let p2 = a.f() * TAU;
        let sable: Vec<i32> = (0..l as i32)
            .map(|x| {
                let xf = x as f32;
                h as i32 - hs
                    + (2.5 * (xf * 0.021 + p1).sin() + 1.5 * (xf * 0.057 + p2).sin()).round() as i32
            })
            .collect();
        let mut plantes = Vec::new();
        let mut sources = Vec::new();
        // Touffes de plantes réparties, plus denses sur les côtés.
        let n = (l / 34).max(6);
        for i in 0..n {
            let mut x = ((i as f32 + a.entre(0.1, 0.9)) / n as f32 * l as f32) as i32;
            if a.f() < 0.35 {
                x = if a.f() < 0.5 {
                    a.n(l as u32 / 6) as i32
                } else {
                    l as i32 - 1 - a.n(l as u32 / 6) as i32
                };
            }
            let touffe = 1 + a.n(3);
            for _ in 0..touffe {
                let xx = (x + a.n(9) as i32 - 4).clamp(1, l as i32 - 2);
                let rouge = a.f() < 0.18;
                let couleurs = if rouge {
                    [[0x5a, 0x1c, 0x3c], [0xa8, 0x34, 0x5c], [0xe8, 0x6a, 0x7c]]
                } else if a.f() < 0.5 {
                    [[0x14, 0x48, 0x2c], [0x2c, 0x84, 0x44], [0x7c, 0xc8, 0x5c]]
                } else {
                    [[0x1c, 0x44, 0x34], [0x3c, 0x7c, 0x4c], [0x9c, 0xc0, 0x68]]
                };
                plantes.push(Plante {
                    x: xx,
                    base: sable[xx as usize] + 2,
                    hauteur: (a.entre(0.08, 0.27) * h as f32) as i32,
                    epaisseur: 2 + a.n(2) as i32,
                    couleurs,
                    phase: a.f() * TAU,
                    // Pas de plante devant le compteur (coin bas gauche).
                    devant: a.f() < 0.25 && xx > 90,
                    feuilles: a.f() < 0.6,
                });
            }
        }
        plantes.sort_by_key(|p| p.devant);
        let mut rochers = Toile::new(l, h);
        // Rochers (sources de bulles).
        let nr = 3 + l / 140;
        for i in 0..nr {
            let cx = ((i as f32 + a.entre(0.2, 0.8)) / nr as f32 * l as f32) as i32;
            let rl = a.entre(9.0, 22.0);
            let rh = rl * a.entre(0.45, 0.8);
            let base = sable[cx.clamp(0, l as i32 - 1) as usize] + 3;
            rocher(&mut rochers, cx, base, rl, rh, &mut a);
            sources.push((cx + (a.n(5) as i32 - 2), base - rh as i32));
        }
        Aquarium {
            toile: Toile::new(l, h),
            fond: Toile::new(l, h),
            rochers,
            teinte_fond: -1.0,
            sable,
            plantes,
            sources,
            nageurs: Vec::new(),
            bulles: Vec::new(),
            particules: Vec::new(),
            gains: Vec::new(),
            temps: 0.0,
            alea: a,
            sprites: HashMap::new(),
            message: None,
            avec_compteur: true,
            decor: true,
        }
    }

    fn l(&self) -> i32 {
        self.toile.l as i32
    }

    fn h(&self) -> i32 {
        self.toile.h as i32
    }

    /// Haut de la zone de nage autorisée au-dessus du sable.
    fn plancher(&self) -> f32 {
        (self.sable.iter().min().copied().unwrap_or(self.h()) - 8) as f32
    }

    /// Eau (teinte 0..1, 0 = invisible), sable tramé et rochers.
    fn refaire_fond(&mut self, teinte: f32) {
        self.teinte_fond = teinte;
        let (l, h) = (self.l(), self.h());
        let f = &mut self.fond;
        f.effacer();
        let haut: Rgb = [0x2a, 0x8c, 0xa8];
        let bas: Rgb = [0x0c, 0x3c, 0x5c];
        for y in 0..h {
            let t = y as f32 / h as f32;
            let c = melange(haut, bas, t);
            // Opacité max 50 %, un peu plus dense vers le fond.
            let a = teinte.clamp(0.0, 1.0) * (0.32 + 0.18 * t) * 255.0;
            for x in 0..l {
                // Tramage : l'opacité se quantifie par paliers de 8, sans dégradé lisse.
                let q = ((a / 8.0) + bayer(x, y)).floor() * 8.0;
                f.poser(x, y, c, q.clamp(0.0, 255.0) as u8);
            }
        }
        let sable: [Rgb; 4] = [
            [0x8a, 0x6c, 0x4c],
            [0xc4, 0x9e, 0x6c],
            [0xe0, 0xc0, 0x86],
            [0xf4, 0xe0, 0xae],
        ];
        let mut a = Alea(77);
        for x in 0..l {
            let top = self.sable[x as usize];
            for y in top..h {
                let prof = (y - top) as f32;
                let mut k: usize = if prof < 1.0 {
                    3
                } else if prof < 3.0 {
                    2
                } else {
                    1
                };
                let bruit = a.f();
                if bruit < 0.08 {
                    k = k.saturating_sub(1);
                } else if bruit > 0.97 && k < 3 {
                    k += 1;
                }
                // Petites rides plus sombres.
                if prof > 4.0 && ((x as f32 * 0.35 + y as f32 * 1.7).sin() > 0.93) {
                    k = 0;
                }
                f.poser(x, y, sable[k], 255);
            }
            // Ombre douce sous la ligne du sable.
            f.poser(x, top - 1, [0x0c, 0x2c, 0x3c], 50);
        }
        // Cailloux et étoiles de mer.
        for _ in 0..l / 12 {
            let x = a.n(l as u32) as i32;
            let y = self.sable[x as usize] + 3 + a.n(8) as i32;
            let c = [[0x6c, 0x64, 0x5c], [0xa0, 0x94, 0x84], [0xd8, 0xd0, 0xc0]][a.n(3) as usize];
            f.poser(x, y, c, 255);
            if a.f() < 0.4 {
                f.poser(x + 1, y, c, 255);
            }
        }
        for _ in 0..if l > 160 { 2 } else { 0 } {
            let x = 100 + a.n((l - 120) as u32) as i32;
            let y = self.sable[x as usize] + 5 + a.n(4) as i32;
            for (dx, dy) in [(0, 0), (-1, 0), (1, 0), (0, -1), (-2, 1), (2, 1), (0, 1)] {
                f.poser(x + dx, y + dy, [0xf0, 0x7c, 0x5c], 255);
            }
            f.poser(x, y, [0xff, 0xb0, 0x88], 255);
        }
        f.superposer(&self.rochers);
    }

    /// Met les nageurs en accord avec les poissons de l'aquarium (les plus rentables d'abord,
    /// `max_affiches` au plus).
    pub fn synchroniser(&mut self, partie: &Partie) {
        let mut actifs: Vec<_> = partie.poissons.iter().filter(|p| p.actif).collect();
        actifs.sort_by(|a, b| b.revenu().total_cmp(&a.revenu()));
        actifs.truncate(partie.reglages.max_affiches);
        self.nageurs.retain(|n| actifs.iter().any(|p| p.id == n.id));
        for p in &actifs {
            if let Some(n) = self.nageurs.iter_mut().find(|n| n.id == p.id) {
                n.revenu = p.revenu();
                continue;
            }
            let sprite = self
                .sprites
                .entry(p.graine)
                .or_insert_with(|| Arc::new(sprite_de(p.graine, p.rarete, p.taille)))
                .clone();
            let (l, plancher) = (self.l() as f32, self.plancher());
            let a = &mut self.alea;
            let x = a.entre(20.0, l - 20.0);
            let y = a.entre(20.0, plancher - 10.0);
            let mut n = Nageur::new(p.id, p.rarete, sprite, x, y, a);
            n.revenu = p.revenu();
            n.cible = (a.entre(20.0, l - 20.0), a.entre(20.0, plancher - 10.0));
            self.nageurs.push(n);
        }
        // Bancs : les petits communs suivent un meneur.
        let petits: Vec<u64> = self
            .nageurs
            .iter()
            .filter(|n| n.rarete <= Rarete::PeuCommun && n.sprite.l < 30)
            .map(|n| n.id)
            .collect();
        // Bancs de 5 au plus : le premier de chaque groupe mène.
        for n in self.nageurs.iter_mut() {
            let meneur = petits
                .iter()
                .position(|&id| id == n.id)
                .map(|i| petits[i / 5 * 5]);
            n.banc = match (meneur, n.banc) {
                (Some(m), Some(b)) if b.0 == m && n.id != m => Some(b),
                (Some(m), _) if n.id != m => {
                    let a = &mut self.alea;
                    Some((m, a.entre(-30.0, -8.0), a.entre(-12.0, 12.0)))
                }
                _ => None,
            };
        }
        // Les gros d'abord, les petits par-dessus.
        self.nageurs
            .sort_by_key(|n| std::cmp::Reverse(n.sprite.l * n.sprite.h));
    }

    /// Avance la simulation de `dt` secondes.
    pub fn avancer(&mut self, dt: f32) {
        let dt = dt.min(0.1);
        self.temps += dt;
        let t = self.temps;
        let (l, plancher) = (self.l() as f32, self.plancher());
        if let Some((_, reste)) = &mut self.message {
            *reste -= dt;
            if *reste <= 0.0 {
                self.message = None;
            }
        }
        let meneurs: HashMap<u64, (f32, f32, bool)> = self
            .nageurs
            .iter()
            .map(|n| (n.id, (n.x, n.y, n.gauche)))
            .collect();
        for i in 0..self.nageurs.len() {
            let n = &mut self.nageurs[i];
            n.bouger(dt, l, plancher, &meneurs, &mut self.alea);
            n.effets(dt, t, &mut self.particules, &mut self.alea);
            n.gain_dans -= dt;
            if n.gain_dans <= 0.0 {
                n.gain_dans = self.alea.entre(7.0, 14.0);
                if self.gains.len() < 4 && n.revenu > 0.0 && !n.fixe {
                    let montant = n.revenu * 10.0;
                    self.gains.push(Gain {
                        x: n.x,
                        y: n.y - n.sprite.h as f32 / 2.0 + 2.0,
                        texte: format!("+{}", court(montant)),
                        vie: 0.0,
                    });
                }
            }
            // Petite bulle par la bouche de temps en temps.
            if !n.fixe && self.alea.f() < dt * 0.08 {
                let (bx, by) = n.monde(n.sprite.bouche);
                self.bulles.push(Bulle {
                    x: bx,
                    y: by,
                    r: 1,
                    v: 9.0,
                    phase: self.alea.f() * TAU,
                });
            }
        }
        // Colonnes de bulles depuis les rochers.
        for &(sx, sy) in self.sources.iter().filter(|_| self.decor) {
            if self.alea.f() < dt * 0.9 {
                let r = if self.alea.f() < 0.3 { 2 } else { 1 };
                self.bulles.push(Bulle {
                    x: sx as f32 + self.alea.entre(-1.5, 1.5),
                    y: sy as f32,
                    r,
                    v: self.alea.entre(12.0, 22.0) + r as f32 * 3.0,
                    phase: self.alea.f() * TAU,
                });
            }
        }
        for b in &mut self.bulles {
            b.y -= b.v * dt;
            b.x += (t * 2.2 + b.phase).sin() * 4.0 * dt;
        }
        self.bulles.retain(|b| b.y > 2.0);
        for p in &mut self.particules {
            p.vie += dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
        self.particules.retain(|p| p.vie < p.duree);
        for g in &mut self.gains {
            g.vie += dt;
            g.y -= 7.0 * dt;
        }
        self.gains.retain(|g| g.vie < 2.4);
    }

    /// Dessine une image complète de l'aquarium sur `self.toile`.
    pub fn dessiner(&mut self, partie: &Partie) {
        if self.teinte_fond != partie.reglages.teinte {
            self.refaire_fond(partie.reglages.teinte);
        }
        let t = self.temps;
        if self.decor {
            self.toile.px.copy_from_slice(&self.fond.px);
            let plancher = self.plancher();
            rayons(&mut self.toile, t, plancher);
        } else {
            self.toile.effacer();
        }
        for p in self.plantes.iter().filter(|p| !p.devant && self.decor) {
            plante(&mut self.toile, p, t, false);
        }
        for n in &self.nageurs {
            n.dessiner(&mut self.toile, t, true);
        }
        dessiner_particules(&mut self.toile, &self.particules);
        for b in &self.bulles {
            bulle(&mut self.toile, b.x.round() as i32, b.y.round() as i32, b.r);
        }
        for p in self.plantes.iter().filter(|p| p.devant && self.decor) {
            plante(&mut self.toile, p, t, true);
        }
        for g in &self.gains {
            let a = (255.0 * (1.0 - ((g.vie - 1.6) / 0.8).max(0.0))) as u8;
            let w = largeur_texte(&g.texte) + 5;
            let x = g.x.round() as i32 - w / 2;
            let y = g.y.round() as i32;
            piece(&mut self.toile, x, y, a);
            self.toile.texte(
                x + 5,
                y,
                &g.texte,
                [0xff, 0xe4, 0x8c],
                a,
                Some([0x2a, 0x1a, 0x08]),
            );
        }
        if self.avec_compteur {
            self.compteur(partie);
        }
    }

    fn compteur(&mut self, partie: &Partie) {
        let argent = court(partie.argent);
        let revenu = format!("+{}/s", court(partie.revenu_par_s()));
        let w = largeur_texte(&argent).max(largeur_texte(&revenu) - 4) + 15;
        let (x, y) = (4, self.h() - 22);
        let t = &mut self.toile;
        // Cartouche sombre aux coins arrondis.
        t.rect(x + 1, y, w - 2, 18, [0x08, 0x16, 0x28], 190);
        t.rect(x, y + 1, 1, 16, [0x08, 0x16, 0x28], 190);
        t.rect(x + w - 1, y + 1, 1, 16, [0x08, 0x16, 0x28], 190);
        t.rect(x + 1, y, w - 2, 1, [0x5c, 0x9c, 0xb4], 200);
        piece(t, x + 4, y + 3, 255);
        t.texte(x + 10, y + 3, &argent, [0xff, 0xea, 0xa8], 255, None);
        t.texte(x + 6, y + 11, &revenu, [0x9c, 0xe4, 0xec], 255, None);
        if let Some((m, reste)) = &self.message {
            let a = (reste.min(2.0) / 2.0 * 255.0) as u8;
            let mw = largeur_texte(m) + 8;
            t.rect(
                x,
                y - 11,
                mw,
                9,
                [0x08, 0x16, 0x28],
                (a as u16 * 3 / 4) as u8,
            );
            t.texte(x + 4, y - 9, m, [0xff, 0xea, 0xa8], a, None);
        }
    }
}

impl Nageur {
    pub fn new(
        id: u64,
        rarete: Rarete,
        sprite: Arc<Sprite>,
        x: f32,
        y: f32,
        a: &mut Alea,
    ) -> Nageur {
        let taille = sprite.l as f32;
        Nageur {
            id,
            rarete,
            croisiere: a.entre(9.0, 15.0) + taille * 0.15,
            sprite,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            cible: (x, y),
            pause: 0.0,
            minuteur: a.entre(2.0, 6.0),
            anim: a.f() * 4.0,
            gauche: a.f() < 0.5,
            tourne: 0.0,
            phase: a.f() * TAU,
            banc: None,
            traine: VecDeque::new(),
            glitch: 0.0,
            bob: 0.0,
            prochain_glitch: a.entre(1.0, 3.0),
            gain_dans: a.entre(3.0, 12.0),
            revenu: 0.0,
            fixe: false,
        }
    }

    fn bouger(
        &mut self,
        dt: f32,
        l: f32,
        plancher: f32,
        meneurs: &HashMap<u64, (f32, f32, bool)>,
        a: &mut Alea,
    ) {
        self.bob += dt;
        if self.fixe {
            self.anim = (self.anim + dt * 6.0) % 4.0;
            return;
        }
        let marge_x = self.sprite.l as f32 / 2.0 + 4.0;
        let haut = self.sprite.h as f32 / 2.0 + 10.0;
        let bas = plancher - self.sprite.h as f32 / 2.0;
        self.minuteur -= dt;
        let mut vitesse = self.croisiere;
        if let Some((m, dx, dy)) = self.banc {
            if let Some(&(mx, my, mg)) = meneurs.get(&m) {
                let dx = if mg { -dx } else { dx };
                self.cible = (mx + dx, my + dy);
                vitesse *= 1.3;
                self.pause = 0.0;
            }
        } else if self.pause > 0.0 {
            self.pause -= dt;
            vitesse *= 0.12;
        } else if self.minuteur <= 0.0 || (self.x - self.cible.0).hypot(self.y - self.cible.1) < 6.0
        {
            self.minuteur = a.entre(4.0, 10.0);
            if a.f() < 0.22 {
                self.pause = a.entre(1.5, 4.0);
            }
            // Cible plutôt sur le côté (les poissons nagent en long).
            let dx = a.entre(60.0, 200.0) * if a.f() < 0.5 { -1.0 } else { 1.0 };
            self.cible = (
                (self.x + dx).clamp(marge_x, l - marge_x),
                (self.y + a.entre(-40.0, 40.0)).clamp(haut, bas.max(haut)),
            );
        }
        let (dx, dy) = (self.cible.0 - self.x, (self.cible.1 - self.y) * 0.6);
        let d = dx.hypot(dy).max(0.001);
        let freine = (d / 12.0).min(1.0);
        let (vx, vy) = (dx / d * vitesse * freine, dy / d * vitesse * freine);
        let k = (dt * 1.6).min(1.0);
        self.vx += (vx - self.vx) * k;
        self.vy += (vy - self.vy) * k;
        self.x = (self.x + self.vx * dt).clamp(marge_x - 2.0, l - marge_x + 2.0);
        self.y = (self.y + self.vy * dt).clamp(haut - 4.0, bas.max(haut) + 2.0);
        // Retournement avec hystérésis et image écrasée pendant 0,16 s.
        if (self.vx < -1.5 && !self.gauche) || (self.vx > 1.5 && self.gauche) {
            self.gauche = !self.gauche;
            self.tourne = 0.16;
        }
        self.tourne = (self.tourne - dt).max(0.0);
        let v = self.vx.hypot(self.vy);
        self.anim = (self.anim + dt * (2.5 + v * 0.35)) % 4.0;
        if self.rarete == Rarete::Mythique {
            let p = self.monde(self.sprite.queue);
            self.traine.push_front(p);
            self.traine.truncate(30);
        }
    }

    /// Position écran (toile) d'un point du sprite (coordonnées de l'image tournée à droite).
    pub fn monde(&self, (sx, sy): (i32, i32)) -> (f32, f32) {
        let s = &self.sprite;
        let x0 = self.x.round() - (s.l / 2) as f32;
        let y0 = self.dessin_y() - (s.h / 2) as f32;
        let sx = if self.gauche { s.l - 1 - sx } else { sx };
        (x0 + sx as f32, y0 + sy as f32)
    }

    fn dessin_y(&self) -> f32 {
        (self.y + (self.phase + self.bob * 1.3).sin() * 1.2).round()
    }

    fn effets(&mut self, dt: f32, t: f32, parts: &mut Vec<Particule>, a: &mut Alea) {
        let s = self.sprite.clone();
        let im = &s.images[0];
        let point_halo = |a: &mut Alea| {
            let h = &im.halo1;
            h[a.n(h.len() as u32) as usize]
        };
        match self.rarete {
            Rarete::Rare => {
                if a.f() < dt * 4.0 {
                    let corps: Vec<usize> = (0..im.classe.len())
                        .filter(|&i| im.classe[i] == CORPS)
                        .collect();
                    if let Some(&i) = corps.get(a.n(corps.len().max(1) as u32) as usize) {
                        let (x, y) = self.monde(((i as i32) % s.l, (i as i32) / s.l));
                        parts.push(Particule {
                            x,
                            y,
                            vx: 0.0,
                            vy: 0.0,
                            vie: 0.0,
                            duree: 0.6,
                            c: [255, 255, 255],
                            genre: Genre::Scintille,
                        });
                    }
                }
            }
            Rarete::Legendaire => {
                if a.f() < dt * 7.0 {
                    let (x, y) = self.monde(point_halo(a));
                    let c = if a.f() < 0.3 {
                        [255, 250, 210]
                    } else {
                        [255, 200, 70]
                    };
                    parts.push(Particule {
                        x,
                        y,
                        vx: a.entre(-2.0, 2.0),
                        vy: a.entre(-9.0, -4.0),
                        vie: 0.0,
                        duree: a.entre(1.0, 1.8),
                        c,
                        genre: Genre::Poussiere,
                    });
                }
            }
            Rarete::Mythique => {
                if a.f() < dt * 4.0 {
                    let (x, y) = self.monde(s.queue);
                    parts.push(Particule {
                        x,
                        y,
                        vx: a.entre(-3.0, 3.0),
                        vy: a.entre(2.0, 6.0),
                        vie: 0.0,
                        duree: a.entre(0.6, 1.2),
                        c: [255, 140, 60],
                        genre: Genre::Braise,
                    });
                }
            }
            Rarete::Irreel => {
                self.prochain_glitch -= dt;
                if self.prochain_glitch <= 0.0 {
                    self.glitch = 0.2;
                    self.prochain_glitch = a.entre(1.2, 3.0);
                }
                self.glitch = (self.glitch - dt).max(0.0);
                if a.f() < dt * 5.0 {
                    let (x, y) = self.monde(point_halo(a));
                    let c = hsv(t * 0.5 + a.f(), 0.6, 1.0);
                    parts.push(Particule {
                        x,
                        y,
                        vx: a.entre(-6.0, 6.0),
                        vy: a.entre(-6.0, 6.0),
                        vie: 0.0,
                        duree: 0.7,
                        c,
                        genre: Genre::Eclat,
                    });
                }
            }
            _ => {}
        }
    }

    /// Dessine le poisson et ses effets de rareté.
    pub fn dessiner(&self, toile: &mut Toile, t: f32, avec_halo: bool) {
        let s = &self.sprite;
        let k = (self.anim as usize) % 4;
        let im = &s.images[k];
        let x0 = self.x.round() as i32 - s.l / 2;
        let y0 = self.dessin_y() as i32 - s.h / 2;
        // Écrasement horizontal pendant le retournement.
        let ratio = if self.tourne > 0.0 {
            0.45 + 0.55 * (1.0 - self.tourne / 0.16 * 0.9).abs()
        } else {
            1.0
        };
        let cx = s.l as f32 / 2.0;
        let tx = |x: i32| -> i32 {
            let x = if self.gauche { s.l - 1 - x } else { x };
            x0 + (cx + (x as f32 + 0.5 - cx) * ratio).floor() as i32
        };
        let r = self.rarete;
        let pouls = 0.75 + 0.25 * (t * 2.4 + self.phase).sin();

        // Traînée (Mythique).
        if r == Rarete::Mythique {
            let pts: Vec<(f32, f32)> = if self.fixe {
                let (qx, qy) = self.monde(s.queue);
                let sens = if self.gauche { 1.0 } else { -1.0 };
                (0..30)
                    .map(|i| {
                        (
                            qx + sens * i as f32 * 0.9,
                            qy + (t * 5.0 - i as f32 * 0.35).sin() * 1.5 * (i as f32 / 30.0),
                        )
                    })
                    .collect()
            } else {
                self.traine.iter().copied().collect()
            };
            let n = pts.len().max(1) as f32;
            for (i, w) in pts.windows(2).enumerate() {
                let f = i as f32 / n;
                let c = if f < 0.3 {
                    melange([255, 236, 150], [255, 110, 70], f / 0.3)
                } else {
                    melange([255, 110, 70], [150, 20, 70], (f - 0.3) / 0.7)
                };
                let a = (230.0 * (1.0 - f)) as u8;
                let pas = ((w[1].0 - w[0].0).abs().max((w[1].1 - w[0].1).abs()).ceil() as i32)
                    .clamp(1, 8);
                for j in 0..pas {
                    let u = j as f32 / pas as f32;
                    let (px, py) = (
                        w[0].0 + (w[1].0 - w[0].0) * u,
                        w[0].1 + (w[1].1 - w[0].1) * u,
                    );
                    let (px, py) = (px.round() as i32, py.round() as i32);
                    toile.eclairer(px, py, c, a);
                    if f < 0.45 {
                        toile.eclairer(px, py - 1, c, a / 2);
                        toile.eclairer(px, py + 1, c, a / 3);
                    }
                }
            }
        }

        // Halos (lueurs).
        if avec_halo && r >= Rarete::Epique {
            let (c1, c2): (Rgb, Rgb) = match r {
                Rarete::Epique => ([0xc0, 0x70, 0xff], [0x80, 0x40, 0xe0]),
                Rarete::Legendaire => ([0xff, 0xd0, 0x50], [0xff, 0xa0, 0x20]),
                Rarete::Mythique => ([0xff, 0x50, 0x60], [0xc0, 0x20, 0x50]),
                Rarete::Celeste => ([0x70, 0xf0, 0xff], [0xa0, 0x70, 0xff]),
                _ => ([255, 255, 255], [255, 255, 255]),
            };
            let (a1, a2) = match r {
                Rarete::Epique => (90.0, 40.0),
                Rarete::Mythique => (80.0, 30.0),
                _ => (120.0, 55.0),
            };
            for &(x, y) in &im.halo1 {
                let c = if r == Rarete::Irreel {
                    hsv(t * 0.6 + x as f32 * 0.04 + y as f32 * 0.02, 0.7, 1.0)
                } else if r == Rarete::Celeste && bayer(x, y) < 0.5 {
                    c2
                } else {
                    c1
                };
                toile.eclairer(tx(x), y0 + y, c, (a1 * pouls) as u8);
            }
            for &(x, y) in &im.halo2 {
                // Second anneau tramé une case sur deux : lueur « pixel » sans flou.
                if (x + y).rem_euclid(2) == 0 {
                    let c = if r == Rarete::Irreel {
                        hsv(t * 0.6 + x as f32 * 0.04, 0.7, 1.0)
                    } else {
                        c2
                    };
                    toile.eclairer(tx(x), y0 + y, c, (a2 * pouls) as u8);
                }
            }
        }

        // Étoiles en orbite (Céleste), moitié arrière.
        let orbite = |toile: &mut Toile, devant: bool| {
            for i in 0..3 {
                let ang = t * 0.9 + i as f32 * TAU / 3.0 + self.phase;
                if (ang.sin() > 0.0) != devant {
                    continue;
                }
                let ox = x0 as f32 + s.l as f32 / 2.0 + ang.cos() * s.l as f32 * 0.55;
                let oy = y0 as f32 + s.h as f32 / 2.0 + ang.sin() * s.h as f32 * 0.42;
                let (ox, oy) = (ox.round() as i32, oy.round() as i32);
                let c = if i == 0 {
                    [255, 255, 255]
                } else {
                    [0x9c, 0xf4, 0xff]
                };
                toile.eclairer(ox, oy, c, 255);
                if ((t * 3.0 + i as f32 * 1.7).sin()) > 0.4 {
                    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        toile.eclairer(ox + dx, oy + dy, c, 110);
                    }
                }
            }
        };
        if r == Rarete::Celeste {
            orbite(toile, false);
        }

        // Copies décalées (Irréel, pendant un glitch).
        if r == Rarete::Irreel && self.glitch > 0.0 {
            for (dx, c) in [(-2, [255, 40, 200]), (2, [40, 230, 255])] {
                for y in 0..s.h {
                    for x in 0..s.l {
                        if im.classe[(y * s.l + x) as usize] != VIDE {
                            toile.eclairer(tx(x) + dx, y0 + y, c, 90);
                        }
                    }
                }
            }
        }

        // Corps.
        let reflet = (t * 0.7 + self.phase).rem_euclid(3.2) / 1.6 * (s.l + s.h) as f32 - s.h as f32;
        for y in 0..s.h {
            // Glitch : bandes de lignes décalées.
            let decal_ligne = if r == Rarete::Irreel
                && self.glitch > 0.0
                && ((y + (t * 40.0) as i32) / 3) % 3 == 0
            {
                if (y / 3) % 2 == 0 {
                    2
                } else {
                    -2
                }
            } else {
                0
            };
            for x in 0..s.l {
                let i = (y * s.l + x) as usize;
                let cl = im.classe[i];
                if cl == VIDE {
                    continue;
                }
                let p = im.px[i];
                let mut c = [p[0], p[1], p[2]];
                let mut yy = y0 + y;
                match r {
                    Rarete::PeuCommun if cl == CORPS => {
                        let d = x as f32 - (s.h - y) as f32 * 0.6 - reflet;
                        if (0.0..2.0).contains(&d) {
                            c = melange(c, [255, 255, 255], 0.55);
                        }
                    }
                    Rarete::Irreel => {
                        let h = x as f32 / s.l as f32 * 0.9 + y as f32 * 0.02 - t * 0.45;
                        if cl == CORPS || cl == NAGEOIRE {
                            let lum = (p[0] as f32 * 0.3 + p[1] as f32 * 0.55 + p[2] as f32 * 0.15)
                                / 255.0;
                            c = hsv(h, 0.55, (0.55 + lum * 0.5).min(1.0));
                        } else if cl == CONTOUR {
                            c = hsv(h + 0.5, 0.8, 0.45);
                            // Contours qui ondulent.
                            if (t * 6.0 + x as f32 * 0.8).sin() > 0.75 {
                                yy -= 1;
                            }
                        }
                    }
                    Rarete::Legendaire if cl == CORPS => {
                        // Reflet doré qui passe.
                        let d = x as f32 - (s.h - y) as f32 * 0.6 - reflet;
                        if (0.0..3.0).contains(&d) {
                            c = melange(c, [255, 230, 140], 0.6);
                        }
                    }
                    _ => {}
                }
                toile.poser(tx(x) + decal_ligne, yy, c, p[3]);
            }
        }
        // Étoiles du corps (Céleste) qui scintillent.
        if r == Rarete::Celeste {
            for &(x, y, ph) in &s.etoiles {
                let v = (t * 2.6 + ph * TAU).sin();
                if v > 0.1 {
                    let yy = y0 + y;
                    toile.poser(tx(x), yy, [255, 255, 255], if v > 0.6 { 255 } else { 150 });
                }
            }
            orbite(toile, true);
        }
    }
}

fn dessiner_particules(toile: &mut Toile, parts: &[Particule]) {
    for p in parts {
        let f = p.vie / p.duree;
        let (x, y) = (p.x.round() as i32, p.y.round() as i32);
        match p.genre {
            Genre::Scintille => {
                // Point → croix → point.
                toile.eclairer(x, y, p.c, 255);
                if (0.3..0.7).contains(&f) {
                    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        toile.eclairer(x + dx, y + dy, [200, 230, 255], 170);
                    }
                    if (0.42..0.58).contains(&f) {
                        for (dx, dy) in [(2, 0), (-2, 0), (0, 2), (0, -2)] {
                            toile.eclairer(x + dx, y + dy, [150, 200, 255], 90);
                        }
                    }
                }
            }
            Genre::Poussiere | Genre::Braise | Genre::Eclat => {
                // Scintillement : une image sur trois plus faible.
                let a = (255.0 * (1.0 - f)) as u8;
                let a = if ((p.vie * 20.0) as i32) % 3 == 0 {
                    a / 2
                } else {
                    a
                };
                toile.eclairer(x, y, p.c, a);
            }
        }
    }
}

fn piece(t: &mut Toile, x: i32, y: i32, a: u8) {
    let motif = [".###.", "#####", "##.##", "#####", ".###."];
    for (j, l) in motif.iter().enumerate() {
        for (i, b) in l.bytes().enumerate() {
            let c = match b {
                b'#' if j == 0 || i == 0 => [0xff, 0xf0, 0xa0],
                b'#' if j == 4 || i == 4 => [0xc0, 0x80, 0x20],
                b'#' => [0xff, 0xc8, 0x40],
                _ if j == 2 && i == 2 => [0xc0, 0x80, 0x20],
                _ => continue,
            };
            t.poser(x + i as i32, y + j as i32, c, a);
        }
    }
}

fn bulle(t: &mut Toile, x: i32, y: i32, r: u8) {
    let c = [0xd8, 0xf4, 0xff];
    if r <= 1 {
        for (dx, dy) in [(0, -1), (-1, 0), (1, 0), (0, 1)] {
            t.poser(x + dx, y + dy, c, 150);
        }
        t.poser(x - 1, y - 1, [255, 255, 255], 90);
    } else {
        for (dx, dy) in [
            (-1, -2),
            (0, -2),
            (1, -2),
            (-2, -1),
            (2, -1),
            (-2, 0),
            (2, 0),
            (-2, 1),
            (2, 1),
            (-1, 2),
            (0, 2),
            (1, 2),
        ] {
            t.poser(x + dx, y + dy, c, 160);
        }
        t.poser(x - 1, y - 1, [255, 255, 255], 230);
        t.poser(x, y, c, 30);
    }
}

/// Rayons de lumière obliques, tramés, très légers.
fn rayons(t: &mut Toile, temps: f32, plancher: f32) {
    let (l, h) = (t.l as i32, t.h as i32);
    let fin = plancher * 0.85;
    for i in 0..5 {
        let fi = i as f32;
        let base = (fi + 0.5) / 5.0 * l as f32 + (temps * 0.07 + fi * 1.9).sin() * 18.0;
        let larg = 9.0 + (fi * 2.3).sin().abs() * 14.0;
        let force = 0.6 + 0.4 * (temps * 0.5 + fi * 2.1).sin();
        for y in 0..fin as i32 {
            let fy = 1.0 - y as f32 / fin;
            let x0 = base - y as f32 * 0.38;
            for x in (x0 - larg) as i32..(x0 + larg) as i32 {
                if x < 0 || x >= l || y >= h {
                    continue;
                }
                let bord = 1.0 - ((x as f32 - x0).abs() / larg).powi(2);
                let a = 30.0 * force * fy * fy * bord;
                let q = ((a / 6.0) + bayer(x, y)).floor() * 6.0;
                if q > 0.0 {
                    t.eclairer(x, y, [190, 240, 255], q as u8);
                }
            }
        }
    }
}

fn plante(t: &mut Toile, p: &Plante, temps: f32, devant: bool) {
    for j in 0..p.hauteur {
        let f = j as f32 / p.hauteur as f32;
        let ondule = (temps * 0.9 + p.phase + j as f32 * 0.09).sin() * 5.0 * f.powf(1.4);
        let x = p.x as f32 + ondule;
        let y = p.base - j;
        let ep = if f > 0.85 { 1 } else { p.epaisseur };
        let xi = x.round() as i32;
        // Contour sombre à gauche, lumière à droite.
        let [sombre, moyen, clair] = p.couleurs;
        let moyen = if devant {
            moyen
        } else {
            melange(moyen, [0x10, 0x30, 0x40], 0.35)
        };
        t.poser(xi - 1, y, sombre, 255);
        for k in 0..ep {
            t.poser(xi + k, y, if k == ep - 1 { clair } else { moyen }, 255);
        }
        t.poser(xi + ep, y, sombre, 255);
        // Petites feuilles alternées.
        if p.feuilles && j > 4 && j % 6 == 0 && f < 0.9 {
            let cote = if (j / 6) % 2 == 0 { -1 } else { 1 };
            let bx = if cote < 0 { xi - 2 } else { xi + ep + 1 };
            t.poser(bx, y, moyen, 255);
            t.poser(bx + cote, y - 1, clair, 255);
            t.poser(bx, y + 1, sombre, 255);
        }
    }
}

fn rocher(t: &mut Toile, cx: i32, base: i32, rl: f32, rh: f32, a: &mut Alea) {
    let gris = a.f() < 0.5;
    let pal: [Rgb; 4] = if gris {
        [
            [0x2c, 0x30, 0x3c],
            [0x50, 0x58, 0x68],
            [0x78, 0x84, 0x90],
            [0xa4, 0xb0, 0xb4],
        ]
    } else {
        [
            [0x3c, 0x2c, 0x2c],
            [0x6c, 0x50, 0x44],
            [0x94, 0x74, 0x5c],
            [0xbc, 0x9c, 0x7c],
        ]
    };
    let bosses: Vec<f32> = (0..6).map(|_| a.entre(-0.12, 0.12)).collect();
    let dans = |x: i32, y: i32| {
        let u = (x - cx) as f32 / rl;
        let v = (base - y) as f32 / rh;
        if v < 0.0 {
            return false;
        }
        let b = bosses[((u + 1.0) * 2.9).clamp(0.0, 5.0) as usize];
        u * u + v * v < 1.0 + b
    };
    for y in base - rh as i32 - 2..=base {
        for x in cx - rl as i32 - 2..=cx + rl as i32 + 2 {
            if dans(x, y) {
                let u = (x - cx) as f32 / rl;
                let v = (base - y) as f32 / rh;
                // Lumière d'en haut à gauche.
                let lum = v * 0.8 - u * 0.4 + 0.2 + (bayer(x, y) - 0.5) * 0.35;
                let k = if lum > 0.85 {
                    3
                } else if lum > 0.45 {
                    2
                } else {
                    1
                };
                let k = if !dans(x, y - 1) && k < 3 { k + 1 } else { k };
                t.poser(x, y, pal[k], 255);
            } else if dans(x + 1, y) || dans(x - 1, y) || dans(x, y + 1) || dans(x, y - 1) {
                t.poser(x, y, pal[0], 255);
            }
        }
    }
    // Mousse sur le dessus.
    for x in cx - rl as i32..=cx + rl as i32 {
        for y in base - rh as i32 - 2..=base {
            if dans(x, y) {
                if a.f() < 0.45 {
                    t.poser(x, y, [0x4c, 0x8c, 0x44], 255);
                }
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echelle_entiere() {
        assert_eq!(echelle_pour(1920), 4);
        assert_eq!(echelle_pour(1366), 3);
        assert_eq!(echelle_pour(3840), 8);
        assert_eq!(echelle_pour(640), 2);
    }

    #[test]
    fn aquarium_tourne_sans_paniquer() {
        let mut p = Partie::nouvelle(0, 5);
        p.argent = 1e12;
        for i in 0..60 {
            let _ = p.acheter_oeuf(i % 4);
        }
        let mut aq = Aquarium::new(480, 270, 1);
        aq.synchroniser(&p);
        assert_eq!(aq.nageurs.len(), 40);
        for _ in 0..300 {
            aq.avancer(1.0 / 30.0);
            aq.dessiner(&p);
        }
        // Tous restent dans la toile.
        for n in &aq.nageurs {
            assert!(n.x >= 0.0 && n.x <= 480.0 && n.y >= 0.0 && n.y <= 270.0);
        }
        p.reglages.max_affiches = 5;
        aq.synchroniser(&p);
        assert_eq!(aq.nageurs.len(), 5);
    }
}

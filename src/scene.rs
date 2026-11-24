//! L'aquarium dessiné sur la toile basse résolution : eau teintée, rayons, sable, rochers,
//! plantes, bulles, poissons (nage, banc, effets de rareté), « +x » et compteur.

use crate::especes::{Mouvement, Taille};
use crate::jeu::{court, Alea, Partie, Poisson, Rarete};
use crate::poisson::{self, Sprite, CONTOUR, CORPS, LUMIERE, MARGE, NAGEOIRE, OEIL, VIDE};
use crate::toile::{bayer, hsv, largeur_texte, melange, Rgb, Toile, FIN};
use std::collections::{HashMap, VecDeque};
use std::f32::consts::TAU;
use std::sync::Arc;

/// Couleur de l'eau lointaine : les créatures du fond s'y fondent un peu.
const EAU_LOIN: Rgb = [0x10, 0x3c, 0x58];

/// Échelle entière toile → écran : ~480 pixels monde en largeur ; toujours paire (les
/// sprites sont posés au pixel fin, la moitié d'un pixel monde).
pub fn echelle_pour(largeur_ecran: u32) -> u32 {
    ((largeur_ecran as f32 / 960.0).round() as u32).clamp(1, 4) * crate::toile::FIN as u32
}

/// Ce que les autres imposent à un nageur : zones des yeux du fond à éviter, poussée de
/// séparation, places prises (positions et cibles).
type Voisinage<'a> = (&'a [(f32, f32, f32, f32)], (f32, f32), &'a [(f32, f32)]);

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
    /// Clignement des yeux : négatif pendant le clignement, sinon temps avant le prochain.
    cligne: f32,
    /// Horloge de l'ondulation verticale.
    bob: f32,
    /// Accélération en cours (phoque, calmar, gobie) ou descente (méduse), en secondes.
    elan: f32,
    /// Serpents colossaux : attente hors de l'écran avant la prochaine traversée.
    attente: f32,
    gain_dans: f32,
    revenu: f64,
    /// Immobile (éclosion, planche) : la traînée est simulée.
    pub fixe: bool,
    /// Clé de l'espèce (bancs d'une même espèce).
    pub espece: &'static str,
    /// Nage par poussées (vrai) et glissés, durée restante de la phase en cours.
    poussee: bool,
    battue: f32,
    /// Force du battement de queue (suit la poussée avec retard).
    vigueur: f32,
    /// Inclinaison du corps (pente dy/dx, en montée négative) et sa valeur visée.
    pente: f32,
    pente_visee: f32,
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
    phase: f32,
    devant: bool,
    /// Lignes visibles de l'algue (pixels fins), le reste sous le sable.
    visible: i32,
    /// Algue dessinée (décor IA) : image et sens.
    algue: usize,
    miroir: bool,
}

/// Eau de chaque colonne de la toile (pixels monde) : plusieurs écrans de tailles différentes
/// laissent des trous (colonnes ou bouts de colonnes hors écran) où l'on ne nage pas.
pub struct Bassin {
    /// Haut de l'eau (haut de l'écran le plus haut de la colonne).
    ciel: Vec<i32>,
    /// Dessus du sable.
    sable: Vec<i32>,
    /// Bas de l'écran le plus bas de la colonne ; 0 = colonne hors écran.
    bas: Vec<i32>,
    /// Écran principal (compteur en bas à gauche) : x0, y0, x1, y1.
    principal: (i32, i32, i32, i32),
}

impl Bassin {
    fn col(&self, x: f32) -> usize {
        (x.max(0.0) as usize).min(self.bas.len() - 1)
    }

    /// Eau où le centre d'un nageur peut aller sur les colonnes x0..x1 : (haut, plancher),
    /// None si une des colonnes est hors écran.
    pub fn eau(&self, x0: f32, x1: f32) -> Option<(f32, f32)> {
        let (a, b) = (self.col(x0), self.col(x1));
        let mut r = (f32::MIN, f32::MAX);
        for c in a..=b {
            if self.bas[c] == 0 {
                return None;
            }
            r = (
                r.0.max(self.ciel[c] as f32),
                r.1.min((self.sable[c] - 8) as f32),
            );
        }
        Some(r)
    }

    /// Hauteurs extrêmes de l'eau (serpents qui traversent tout).
    fn eau_totale(&self) -> (f32, f32) {
        let mut r = (f32::MAX, f32::MIN);
        for c in (0..self.bas.len()).filter(|&c| self.bas[c] > 0) {
            r = (
                r.0.min(self.ciel[c] as f32),
                r.1.max((self.sable[c] - 8) as f32),
            );
        }
        r
    }
}

pub struct Aquarium {
    pub toile: Toile,
    /// Eau teintée + sable + rochers, refait quand la teinte change.
    fond: Toile,
    rochers: Toile,
    teinte_fond: f32,
    bassin: Bassin,
    plantes: Vec<Plante>,
    sources: Vec<(i32, i32)>,
    pub nageurs: Vec<Nageur>,
    bulles: Vec<Bulle>,
    particules: Vec<Particule>,
    gains: Vec<Gain>,
    pub temps: f32,
    alea: Alea,
    sprites: HashMap<String, Arc<Sprite>>,
    /// Message affiché au-dessus du compteur (gains hors ligne) et sa durée restante.
    pub message: Option<(String, f32)>,
    /// Compteur affiché (l'argent arrive en continu, on lisse l'affichage).
    pub avec_compteur: bool,
    /// Sable, plantes, rayons et bulles (faux pour la planche des raretés).
    pub decor: bool,
}

/// Espèce d'un poisson (toujours connue après `Partie::migrer`, repli par la graine sinon).
pub fn espece_de(p: &Poisson) -> &'static crate::especes::Espece {
    crate::especes::par_cle(&p.espece)
        .unwrap_or_else(|| crate::especes::choisir(p.rarete, p.graine).0)
}

/// Nom affiché d'un poisson (espèce et variante).
pub fn nom_de(p: &Poisson) -> String {
    espece_de(p).nom_variante(p.variante as usize)
}

/// Sprite d'un poisson à sa taille dans l'aquarium.
pub fn sprite_de(p: &Poisson) -> Sprite {
    poisson::sprite(
        espece_de(p),
        p.variante as usize,
        poisson::echelle_individu(p.taille),
    )
}

/// Sprite d'un poisson réduit pour tenir dans `lmax` x `hmax` pixels (vignettes, éclosion).
pub fn sprite_ajuste(p: &Poisson, lmax: i32, hmax: i32) -> Sprite {
    let e = espece_de(p);
    let m = 2.0 * MARGE as f32;
    let k = ((lmax as f32 - m) / e.dims.0 as f32)
        .min((hmax as f32 - m) / e.dims.1 as f32)
        .min(poisson::echelle_individu(p.taille));
    poisson::sprite(e, p.variante as usize, k.max(0.05))
}

impl Aquarium {
    pub fn new(l: usize, h: usize, graine: u64) -> Aquarium {
        let z = [(0, 0, l as i32, h as i32)];
        Aquarium::avec_zones(l, h, graine, &z, &z)
    }

    /// Aquarium sur plusieurs écrans : `zones` = rectangles des écrans en pixels monde
    /// (x0, y0, x1, y1), le principal en premier (voir `ecrans.rs`). Hors des zones : rien.
    /// `travail` : zones de travail (sans la barre des tâches), même ordre : le sable et le
    /// compteur restent au-dessus de la barre (sous la barre, seulement du sable).
    pub fn avec_zones(
        l: usize,
        h: usize,
        graine: u64,
        zones: &[(i32, i32, i32, i32)],
        travail: &[(i32, i32, i32, i32)],
    ) -> Aquarium {
        let mut a = Alea(graine);
        let p1 = a.f() * TAU;
        let p2 = a.f() * TAU;
        let (mut ciel, mut sable, mut bas) = (vec![h as i32; l], vec![h as i32; l], vec![0; l]);
        for x in 0..l {
            let xi = x as i32;
            let dessus = zones.iter().filter(|z| z.0 <= xi && xi < z.2);
            let (Some(y0), Some((i, z))) = (
                dessus.map(|z| z.1.max(0)).min(),
                zones
                    .iter()
                    .enumerate()
                    .filter(|(_, z)| z.0 <= xi && xi < z.2)
                    .max_by_key(|(_, z)| z.3),
            ) else {
                continue;
            };
            let y1 = z.3.min(h as i32);
            let utile = travail.get(i).map_or(y1, |t| t.3.min(y1));
            let hs = (((utile - z.1) as f32 * 0.085) as i32).max(12);
            let xf = x as f32;
            ciel[x] = y0;
            bas[x] = y1;
            sable[x] = utile - hs
                + (2.5 * (xf * 0.021 + p1).sin() + 1.5 * (xf * 0.057 + p2).sin()).round() as i32;
        }
        // Raccord entre écrans (barre des tâches sur un seul) : le sable monte en pente douce
        // au lieu d'une marche (il ne descend jamais sous la zone de travail).
        for x in 1..l {
            if bas[x] > 0 && bas[x - 1] > 0 {
                sable[x] = sable[x].min(sable[x - 1] + 1);
            }
        }
        for x in (0..l.saturating_sub(1)).rev() {
            if bas[x] > 0 && bas[x + 1] > 0 {
                sable[x] = sable[x].min(sable[x + 1] + 1);
            }
        }
        let principal = travail
            .first()
            .or(zones.first())
            .copied()
            .unwrap_or((0, 0, l as i32, h as i32));
        let mut plantes = Vec::new();
        let mut sources = Vec::new();
        // Touffes de plantes réparties, plus denses sur les côtés.
        let n = (l / 34).max(6);
        for i in 0..n {
            let mut x = ((i as f32 + a.entre(0.1, 0.9)) / n as f32 * l as f32) as i32;
            if a.f() < 0.35 && zones.len() == 1 {
                x = if a.f() < 0.5 {
                    a.n(l as u32 / 6) as i32
                } else {
                    l as i32 - 1 - a.n(l as u32 / 6) as i32
                };
            }
            let touffe = 1 + a.n(3);
            for _ in 0..touffe {
                let xx = (x + a.n(9) as i32 - 4).clamp(1, l as i32 - 2);
                if bas[xx as usize] == 0 {
                    continue;
                }
                plantes.push(Plante {
                    x: xx,
                    base: sable[xx as usize] + 2,
                    phase: a.f() * TAU,
                    // Pas de plante devant le compteur (coin bas gauche de l'écran principal).
                    devant: a.f() < 0.25 && !(principal.0..principal.0 + 90).contains(&xx),
                    // 15-40 % de la hauteur de l'eau.
                    visible: (a.entre(0.15, 0.4)
                        * (sable[xx as usize] - ciel[xx as usize]) as f32
                        * FIN as f32) as i32,
                    algue: a.n(crate::decor::decor().algues.len() as u32) as usize,
                    miroir: a.f() < 0.5,
                });
            }
        }
        plantes.sort_by_key(|p| p.devant);
        let mut rochers = Toile::new(l, h);
        // Rochers et coraux dessinés (décor IA), posés dans le sable ; sources de bulles.
        let pierres = &crate::decor::decor().pierres;
        let nr = 4 + l / 90;
        let f = FIN as i32;
        for i in 0..nr {
            let cx = ((i as f32 + a.entre(0.2, 0.8)) / nr as f32 * l as f32) as i32;
            let c = cx.clamp(0, l as i32 - 1) as usize;
            let img = &pierres[a.n(pierres.len() as u32) as usize];
            if bas[c] == 0 || (principal.0..principal.0 + 70).contains(&cx) {
                continue;
            }
            let base = (sable[c] + 4) * f;
            crate::decor::poser(
                &mut rochers,
                img,
                (cx * f - img.l / 2, base - img.h),
                a.f() < 0.5,
                (EAU_LOIN, 0.1),
            );
            sources.push((cx + (a.n(5) as i32 - 2), (base - img.h) / f + 2));
        }
        Aquarium {
            toile: Toile::new(l, h),
            fond: Toile::new(l, h),
            rochers,
            teinte_fond: -1.0,
            bassin: Bassin {
                ciel,
                sable,
                bas,
                principal,
            },
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

    /// Eau (teinte 0..1, 0 = invisible), sable tramé et rochers.
    fn refaire_fond(&mut self, teinte: f32) {
        self.teinte_fond = teinte;
        let l = self.l();
        let f = &mut self.fond;
        f.effacer();
        let haut: Rgb = [0x2a, 0x8c, 0xa8];
        let bas: Rgb = [0x0c, 0x3c, 0x5c];
        let b = &self.bassin;
        for x in 0..l {
            let (y0, y1) = (b.ciel[x as usize], b.bas[x as usize]);
            for y in y0..y1 {
                let t = (y - y0) as f32 / (y1 - y0).max(1) as f32;
                let c = melange(haut, bas, t);
                // Opacité max 50 %, un peu plus dense vers le fond.
                let a = teinte.clamp(0.0, 1.0) * (0.32 + 0.18 * t) * 255.0;
                // Tramage : l'opacité se quantifie par paliers de 8, sans dégradé lisse.
                let q = ((a / 8.0) + bayer(x, y)).floor() * 8.0;
                f.poser(x, y, c, q.clamp(0.0, 255.0) as u8);
            }
        }
        // Récif lointain (décor IA) fondu dans l'eau, derrière le sable, répété en miroir.
        let d = crate::decor::decor();
        let fi = FIN as i32;
        let miroir = |xf: i32, n: i32| {
            let k = xf.rem_euclid(2 * n);
            if k < n {
                k
            } else {
                2 * n - 1 - k
            }
        };
        let lo = &d.lointain;
        for xf in 0..l * fi {
            let x = (xf / fi) as usize;
            if b.bas[x] == 0 {
                continue;
            }
            let sx = miroir(xf, lo.l);
            let y0 = (b.sable[x] + 6) * fi - lo.h;
            for y in 0..lo.h {
                let p = lo.px(sx, y);
                if p[3] > 0 && y0 + y >= b.ciel[x] * fi {
                    let c = crate::decor::dans_l_eau(p, EAU_LOIN, 0.5);
                    f.poser_fin(xf, y0 + y, c, 150);
                }
            }
        }
        // Sable dessiné (décor IA) : la bande, répétée en miroir sans le contour de ses bouts,
        // suit la ligne du sable de chaque colonne ; dessous, jusqu'au bas de l'écran (sous la
        // barre des tâches), ses lignes du bas répétées.
        let sb = &d.sable;
        let dessus = sb.dessus();
        let (g, lg) = (3, (sb.l - 6).max(1));
        let haut_moyen = dessus[g as usize..(g + lg) as usize].iter().sum::<i32>() / lg;
        let fin_s = (sb.h - 3).max(9);
        // Sous la bande : sable uni tramé, des trois couleurs les plus fréquentes de la bande
        // (plus sombre en profondeur), pas de motif répété.
        let mut comptes: Vec<([u8; 4], u32)> = Vec::new();
        for y in sb.h / 2..fin_s {
            for x in g..g + lg {
                let p = sb.px(x, y);
                if p[3] == 0 {
                    continue;
                }
                match comptes.iter_mut().find(|c| c.0 == p) {
                    Some(c) => c.1 += 1,
                    None => comptes.push((p, 1)),
                }
            }
        }
        comptes.sort_by_key(|c| std::cmp::Reverse(c.1));
        let mut tons: Vec<Rgb> = comptes
            .iter()
            .take(3)
            .map(|c| crate::decor::dans_l_eau(c.0, EAU_LOIN, 0.14))
            .collect();
        tons.sort_by_key(|c| c[0] as u32 + c[1] as u32 + c[2] as u32);
        let mut alea = Alea(77);
        for xf in 0..l * fi {
            let x = (xf / fi) as usize;
            if b.bas[x] == 0 {
                continue;
            }
            let sx = g + miroir(xf, lg);
            let decal = b.sable[x] * fi - haut_moyen;
            for yf in (decal + dessus[sx as usize]).max(0)..b.bas[x] * fi {
                let r = yf - decal;
                if r < fin_s {
                    let p = sb.px(sx, r);
                    if p[3] > 0 {
                        let c = crate::decor::dans_l_eau(p, EAU_LOIN, 0.14);
                        f.poser_fin(xf, yf, c, 255);
                    }
                } else if !tons.is_empty() {
                    let prof = ((r - fin_s) as f32 / 30.0).min(1.0);
                    let k = ((1.0 - prof) * 1.6 + alea.f() * 1.2) as usize;
                    let c = melange(tons[k.min(tons.len() - 1)], [0x40, 0x30, 0x20], prof * 0.35);
                    f.poser_fin(xf, yf, c, 255);
                }
            }
            // Ombre douce sous la ligne du sable.
            f.poser_fin(xf, decal + dessus[sx as usize] - 1, [0x0c, 0x2c, 0x3c], 50);
        }
        f.superposer(&self.rochers);
    }

    /// Met les nageurs en accord avec les poissons de l'aquarium (les plus rentables d'abord,
    /// `max_affiches` au plus).
    pub fn synchroniser(&mut self, partie: &Partie) {
        let mut actifs: Vec<_> = partie.poissons.iter().filter(|p| p.actif).collect();
        actifs.sort_by(|a, b| b.revenu().total_cmp(&a.revenu()));
        // Lisibilité : une créature colossale compte pour 2 places, et pas
        // plus de 2 colossales à la fois (les autres rapportent quand même).
        let mut places = partie.reglages.max_affiches;
        let mut colossales = 0;
        actifs.retain(|p| {
            let t = espece_de(p).taille;
            let ok = t.places() <= places && (t != Taille::Colossale || colossales < 2);
            if ok {
                places -= t.places();
                colossales += (t == Taille::Colossale) as usize;
            }
            ok
        });
        // Lisibilité (lot 4) : les grandes créatures couvrent au plus ~20 % de l'eau, les
        // moyennes ~9 %, les petites ~7 % (il y en a toujours ; plus d'écrans, plus de poissons) ;
        // les meilleures d'abord, les autres rapportent quand même.
        let eau: f32 = (0..self.bassin.bas.len())
            .filter(|&c| self.bassin.bas[c] > 0)
            .map(|c| (self.bassin.sable[c] - self.bassin.ciel[c]).max(0) as f32)
            .sum();
        let mut budget = [eau * 0.07, eau * 0.09, eau * 0.2];
        let mut premier = true;
        actifs.retain(|p| {
            let e = espece_de(p);
            let k = match e.taille {
                Taille::Minuscule | Taille::Petite => 0,
                Taille::Moyenne => 1,
                _ => 2,
            };
            // Surface du dessin en pixels monde (une silhouette remplit ~55 % de sa boîte).
            let surface = e.dims.0 as f32 * e.dims.1 as f32 / (FIN * FIN) as f32 * 0.55;
            let ok = surface <= budget[k] || premier;
            if ok {
                budget[k] -= surface;
                premier = false;
            }
            ok
        });
        self.nageurs.retain(|n| actifs.iter().any(|p| p.id == n.id));
        for p in &actifs {
            if let Some(n) = self.nageurs.iter_mut().find(|n| n.id == p.id) {
                n.revenu = p.revenu();
                continue;
            }
            let cle = format!(
                "{}/{}/{:.2}",
                p.espece,
                p.variante,
                poisson::echelle_individu(p.taille)
            );
            let sprite = self
                .sprites
                .entry(cle)
                .or_insert_with(|| Arc::new(sprite_de(p)))
                .clone();
            // Arrivée à une place libre (loin des autres et de leurs cibles), sur un écran.
            let occupes: Vec<(f32, f32)> = self
                .nageurs
                .iter()
                .flat_map(|n| [(n.x, n.y), n.cible])
                .collect();
            let a = &mut self.alea;
            let taille = (sprite.lw(), sprite.hw());
            let (x, y) = place_libre(a, &occupes, &self.bassin, taille, false);
            let mut n = Nageur::new(p.id, p.rarete, sprite, x, y, a);
            n.revenu = p.revenu();
            n.espece = espece_de(p).cle;
            n.cible = place_libre(a, &occupes, &self.bassin, taille, false);
            self.nageurs.push(n);
        }
        // Bancs lâches d'une même espèce : 5 au plus pour les poissons de banc, 3 pour les
        // autres petits communs ; le premier de chaque groupe mène.
        let mut groupes: Vec<(&'static str, Vec<u64>)> = Vec::new();
        for n in &self.nageurs {
            let max = match n.sprite.mouvement {
                Mouvement::Banc => 5,
                Mouvement::Nage if n.rarete == Rarete::Commun => 3,
                _ => continue,
            };
            match groupes
                .iter_mut()
                .find(|g| g.0 == n.espece && g.1.len() < max)
            {
                Some(g) => g.1.push(n.id),
                None => groupes.push((n.espece, vec![n.id])),
            }
        }
        for n in self.nageurs.iter_mut() {
            let meneur = groupes.iter().find(|g| g.1.contains(&n.id)).map(|g| g.1[0]);
            n.banc = match (meneur, n.banc) {
                (Some(m), Some(b)) if b.0 == m && n.id != m => Some(b),
                (Some(m), _) if n.id != m => {
                    let a = &mut self.alea;
                    // Écart à la taille du poisson : le banc ne se chevauche pas.
                    let (lw, hw) = (n.sprite.lw(), n.sprite.hw());
                    Some((m, -lw * a.entre(1.0, 2.4), hw * a.entre(-2.2, 2.2)))
                }
                _ => None,
            };
        }
        // Le fond d'abord, puis les gros, les petits par-dessus.
        self.nageurs.sort_by_key(|n| {
            std::cmp::Reverse((n.sprite.taille.profondeur() > 0.0, n.sprite.l * n.sprite.h))
        });
    }

    /// Avance la simulation de `dt` secondes.
    pub fn avancer(&mut self, dt: f32) {
        let dt = dt.min(0.1);
        self.temps += dt;
        let t = self.temps;
        let (l, ht) = (self.l() as f32, self.h() as f32);
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
        // Les nageurs du premier plan ne s'arrêtent pas devant les yeux des créatures du fond.
        let evite: Vec<(f32, f32, f32, f32)> = self
            .nageurs
            .iter()
            .filter(|n| n.sprite.taille >= Taille::Enorme)
            .filter_map(Nageur::zone_yeux)
            .collect();
        let pousses = separation(&self.nageurs);
        // Places prises (positions et cibles) : les nouvelles cibles vont vers l'eau libre.
        let occupes: Vec<(f32, f32)> = self
            .nageurs
            .iter()
            .filter(|n| !n.fixe)
            .flat_map(|n| [(n.x, n.y), n.cible])
            .collect();
        for (i, &pousse) in pousses.iter().enumerate() {
            let n = &mut self.nageurs[i];
            let ev: &[(f32, f32, f32, f32)] = if n.sprite.taille >= Taille::Enorme {
                &[]
            } else {
                &evite
            };
            n.bouger(
                dt,
                (l, ht),
                &self.bassin,
                &meneurs,
                (ev, pousse, &occupes),
                &mut self.alea,
            );
            n.effets(dt, t, &mut self.particules, &mut self.alea);
            n.gain_dans -= dt;
            if n.gain_dans <= 0.0 {
                n.gain_dans = self.alea.entre(7.0, 14.0);
                if self.gains.len() < 4 && n.revenu > 0.0 && !n.fixe {
                    let montant = n.revenu * 10.0;
                    self.gains.push(Gain {
                        x: n.x,
                        y: n.y - n.sprite.hw() / 2.0 + 2.0,
                        texte: format!("+{}", court(montant)),
                        vie: 0.0,
                    });
                }
            }
            // Petite bulle par la bouche de temps en temps.
            if !n.fixe && n.sprite.taille < Taille::Colossale && self.alea.f() < dt * 0.08 {
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
        // Une bulle éclate à la surface de son écran (jamais dans un trou entre écrans).
        let b = &self.bassin;
        self.bulles.retain(|u| {
            let c = b.col(u.x);
            b.bas[c] > 0 && u.y > b.ciel[c] as f32 + 2.0
        });
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
            rayons(&mut self.toile, t, &self.bassin);
        } else {
            self.toile.effacer();
        }
        // Créatures du fond derrière les plantes, les autres devant ; le kraken passe devant
        // les autres créatures du lointain (ses yeux restent visibles).
        let fond = |n: &&Nageur| n.sprite.taille.profondeur() > 0.0;
        let abysse = |n: &&Nageur| n.sprite.mouvement == Mouvement::Abysse;
        for n in self.nageurs.iter().filter(fond).filter(|n| !abysse(n)) {
            n.dessiner(&mut self.toile, t, true);
        }
        for n in self.nageurs.iter().filter(fond).filter(abysse) {
            n.dessiner(&mut self.toile, t, true);
        }
        // Le kraken sort du sable : on repose le sable par-dessus le bas de son corps, en
        // petites buttes sur ses bras.
        if self.decor {
            let lf = self.toile.lf();
            let f = FIN as i32;
            for n in self.nageurs.iter().filter(abysse) {
                let (xa, xb) = (n.x - n.sprite.lw() * 0.45, n.x + n.sprite.lw() * 0.45);
                let ph = n.phase;
                for (x, &top) in self.bassin.sable.iter().enumerate() {
                    let hf = self.bassin.bas[x] * f;
                    if hf == 0 {
                        continue;
                    }
                    let xf = x as f32;
                    let dedans = ((xf - xa).min(xb - xf) / 12.0).clamp(0.0, 1.0);
                    let butte = (dedans
                        * (3.0 + 2.5 * (xf * 0.11 + ph).sin() + 1.5 * (xf * 0.29 + ph * 2.0).sin()))
                    .max(0.0)
                    .round() as i32;
                    for fx in 0..f {
                        let col = x * FIN + fx as usize;
                        for y in ((top + 2 - butte) * f).max(0)..hf {
                            // Sable décalé vers le haut sur la butte ; tout en bas, le sable d'origine.
                            let src = if y + butte * f < hf { y + butte * f } else { y };
                            self.toile.px[y as usize * lf + col] =
                                self.fond.px[src as usize * lf + col];
                        }
                    }
                }
            }
        }
        for p in self.plantes.iter().filter(|p| !p.devant && self.decor) {
            plante(&mut self.toile, p, t, false);
        }
        for n in self.nageurs.iter().filter(fond) {
            n.dessiner_yeux(&mut self.toile);
        }
        for n in self.nageurs.iter().filter(|n| !fond(n)) {
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
        let (x, y) = (self.bassin.principal.0 + 4, self.bassin.principal.3 - 22);
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
        let taille = sprite.lw();
        let croisiere = match sprite.mouvement {
            Mouvement::Nage => a.entre(9.0, 15.0) + taille.min(40.0) * 0.15,
            Mouvement::Banc => a.entre(12.0, 16.0),
            Mouvement::Lent => a.entre(5.0, 8.0) + taille * 0.02,
            Mouvement::Fonce | Mouvement::Jet => a.entre(9.0, 13.0),
            Mouvement::Ondule => a.entre(6.0, 10.0),
            Mouvement::Plane => a.entre(7.0, 11.0),
            Mouvement::Pulse => a.entre(6.0, 8.0),
            Mouvement::Fond => a.entre(5.0, 8.0),
            Mouvement::Debout => a.entre(2.5, 4.0),
            Mouvement::Marche => a.entre(4.0, 7.0),
            Mouvement::Flotte => a.entre(2.5, 4.5),
            Mouvement::Abysse => a.entre(1.5, 3.0),
            Mouvement::Traverse => a.entre(10.0, 16.0),
        };
        // Lot 3 : les dessins sont plus grands, ils nagent plus vite pour traverser l'écran
        // (lot 4 : encore plus, la relecture les trouvait lents).
        let croisiere = croisiere * 1.75;
        let nb = sprite.images.len() as f32;
        Nageur {
            id,
            rarete,
            croisiere,
            gauche: a.f() < 0.5 && sprite.mouvement != Mouvement::Pulse,
            sprite,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            cible: (x, y),
            pause: 0.0,
            minuteur: a.entre(2.0, 6.0),
            anim: a.f() * nb,
            tourne: 0.0,
            phase: a.f() * TAU,
            banc: None,
            traine: VecDeque::new(),
            glitch: 0.0,
            bob: 0.0,
            elan: 0.0,
            attente: 0.0,
            prochain_glitch: a.entre(1.0, 3.0),
            cligne: a.entre(1.0, 8.0),
            gain_dans: a.entre(3.0, 12.0),
            revenu: 0.0,
            fixe: false,
            espece: "",
            poussee: a.f() < 0.5,
            battue: a.entre(0.2, 1.2),
            vigueur: 1.0,
            pente: 0.0,
            pente_visee: 0.0,
        }
    }

    /// Cycles d'animation par seconde selon la façon de bouger et la vitesse.
    fn cadence(&self, v: f32) -> f32 {
        // Vitesse d'animation qui varie doucement (rien de mécanique).
        let souffle = 1.0 + 0.22 * (self.bob * 0.37 + self.phase).sin();
        // Le battement suit la vitesse et la poussée (presque arrêté pendant un glissé).
        let g = self.vigueur;
        souffle
            * match self.sprite.mouvement {
                Mouvement::Nage | Mouvement::Banc => (0.3 + v * 0.07) * (0.3 + 1.0 * g),
                Mouvement::Lent => (0.2 + v * 0.03) * (0.5 + 0.6 * g),
                Mouvement::Fonce => (0.3 + v * 0.05) * (0.4 + 0.8 * g),
                Mouvement::Ondule => 0.35 + v * 0.06,
                // Battements d'ailes puis planés, ailes presque immobiles.
                Mouvement::Plane => 0.04 + 0.42 * g.powi(2),
                Mouvement::Pulse => 0.55,
                Mouvement::Jet => 0.4 + self.elan * 0.6,
                Mouvement::Fond => 0.7 + self.elan,
                Mouvement::Debout => 0.9,
                Mouvement::Marche => self.vx.abs() * 0.3,
                Mouvement::Flotte => 0.25,
                Mouvement::Abysse => 0.2,
                Mouvement::Traverse => 0.3 + v * 0.02,
            }
    }

    /// Durée du retournement (plus lente pour les grandes créatures).
    fn duree_tour(&self) -> f32 {
        0.26 * (1.0 + self.sprite.taille.profondeur() * 3.0)
    }

    fn bouger(
        &mut self,
        dt: f32,
        (l, ht): (f32, f32),
        b: &Bassin,
        meneurs: &HashMap<u64, (f32, f32, bool)>,
        (evite, pousse, occupes): Voisinage,
        a: &mut Alea,
    ) {
        self.bob += dt;
        let nb = self.sprite.images.len() as f32;
        if self.fixe {
            self.anim = (self.anim + dt * self.cadence(8.0) * nb) % nb;
            return;
        }
        let mv = self.sprite.mouvement;
        let (sl, sh) = (self.sprite.lw(), self.sprite.hw());
        let marge_x = sl / 2.0 + 4.0;
        // L'eau sous tout le corps (plusieurs écrans : jamais dans les trous entre eux).
        let (dessus, plancher) = b
            .eau(self.x - sl / 2.0, self.x + sl / 2.0)
            .or_else(|| b.eau(self.x, self.x))
            .unwrap_or_else(|| b.eau_totale());
        let mut haut = dessus + sh / 2.0 + 10.0;
        let bas = plancher - sh / 2.0;
        if mv == Mouvement::Fond {
            haut = haut.max(bas - 24.0);
        }
        self.minuteur -= dt;
        self.elan = (self.elan - dt).max(0.0);
        let mut vitesse = self.croisiere;
        match mv {
            Mouvement::Marche => {
                // Sur le sable : avance vers une cible le long du fond, s'arrête souvent.
                let sol = |x: f32| b.sable[b.col(x)] as f32;
                if self.pause > 0.0 {
                    self.pause -= dt;
                    self.vx *= 0.8;
                } else {
                    if self.minuteur <= 0.0 || (self.x - self.cible.0).abs() < 3.0 {
                        self.minuteur = a.entre(3.0, 8.0);
                        self.pause = if a.f() < 0.45 { a.entre(1.0, 4.0) } else { 0.0 };
                        let dx = a.entre(25.0, 120.0) * if a.f() < 0.5 { -1.0 } else { 1.0 };
                        self.cible.0 = (self.x + dx).clamp(marge_x, l - marge_x);
                    }
                    let d = self.cible.0 - self.x;
                    self.vx += (d.signum() * vitesse * (d.abs() / 6.0).min(1.0) - self.vx)
                        * (dt * 3.0).min(1.0);
                }
                let nx = (self.x + (self.vx + pousse.0 * 2.5) * dt).clamp(marge_x, l - marge_x);
                if b.eau(nx - sl / 2.0, nx + sl / 2.0).is_some() {
                    self.x = nx;
                } else {
                    // Bord d'un écran sans voisin à cette hauteur : demi-tour.
                    self.vx = 0.0;
                    self.minuteur = 0.0;
                }
                let y0 = sol(self.x) + 2.0 + MARGE as f32 - sh;
                self.y = y0 + (sh / 2.0).floor();
                self.anim = (self.anim + dt * self.cadence(0.0) * nb) % nb;
                return;
            }
            Mouvement::Abysse => {
                // Tout en bas, le bas du corps sous le bord de l'écran ; dérive lente.
                if self.minuteur <= 0.0 {
                    self.minuteur = a.entre(8.0, 16.0);
                    self.cible.0 = a.entre(sl * 0.4, (l - sl * 0.4).max(sl * 0.4));
                }
                let d = self.cible.0 - self.x;
                self.vx += (d.signum() * vitesse * (d.abs() / 20.0).min(1.0) - self.vx) * dt;
                self.x += self.vx * dt;
                // Bas du corps dans le sable (au-dessus de la barre des tâches : ses yeux restent
                // visibles).
                let c = b.col(self.x);
                let fond = match b.bas[c] {
                    0 => ht,
                    _ => b.sable[c] as f32 + 22.0,
                };
                self.y = fond - sh * 0.95 + sh / 2.0;
                self.anim = (self.anim + dt * self.cadence(0.0) * nb) % nb;
                return;
            }
            Mouvement::Traverse => {
                let dehors = self.x < -sl / 2.0 - 2.0 || self.x > l + sl / 2.0 + 2.0;
                if self.attente > 0.0 {
                    self.attente -= dt;
                    if self.attente <= 0.0 {
                        // Nouvelle traversée depuis un bord, à une hauteur au hasard.
                        self.gauche = a.f() < 0.5;
                        self.x = if self.gauche { l + sl / 2.0 } else { -sl / 2.0 };
                        // Toute la hauteur des écrans (il passe aussi derrière les trous).
                        let (t0, t1) = b.eau_totale();
                        let (y0, y1) = (t0 + sh / 2.0 + 10.0, t1 - sh / 2.0);
                        self.y = a.entre(y0.min(y1), y1.max(y0));
                    }
                    return;
                }
                let sens = if self.gauche { -1.0 } else { 1.0 };
                self.vx = sens * vitesse;
                self.x += self.vx * dt;
                self.y += (self.bob * 0.35 + self.phase).sin() * 4.0 * dt;
                if dehors && (self.x - l / 2.0) * sens > 0.0 {
                    self.attente = a.entre(6.0, 18.0);
                }
                self.anim = (self.anim + dt * self.cadence(vitesse) * nb) % nb;
                return;
            }
            Mouvement::Pulse => {
                // Méduse : la cloche se contracte (première moitié du cycle) et la pousse vers
                // le haut ; entre deux, elle coule doucement. Arrivée en haut, elle redescend.
                let cycle = self.anim / nb;
                let poussee = if self.elan > 0.0 {
                    0.0
                } else {
                    (cycle * TAU).sin().max(0.0) * vitesse * 1.6
                };
                if self.y < haut + 6.0 {
                    self.elan = a.entre(5.0, 9.0);
                }
                let vy = -poussee + if self.elan > 0.0 { 5.0 } else { 2.5 };
                self.vy += (vy - self.vy) * (dt * 3.0).min(1.0);
                self.vx = (self.bob * 0.21 + self.phase).sin() * 3.5 + pousse.0 * 2.0;
                self.vy += pousse.1 * 2.0 * dt;
                self.x = (self.x + self.vx * dt).clamp(marge_x, l - marge_x);
                self.y = (self.y + self.vy * dt).clamp(haut - 4.0, bas.max(haut) + 2.0);
                if self.y >= bas.max(haut) {
                    self.elan = 0.0;
                }
                self.anim = (self.anim + dt * self.cadence(0.0) * nb) % nb;
                return;
            }
            _ => {}
        }
        if let Some((m, dx, dy)) = self.banc {
            if let Some(&(mx, my, mg)) = meneurs.get(&m) {
                // Banc lâche : la place de chacun dérive lentement autour du meneur.
                let dx = dx + (self.bob * 0.37 + self.phase).sin() * sl * 0.6;
                let dy = dy + (self.bob * 0.29 + self.phase * 2.0).sin() * sh * 1.2;
                let dx = if mg { -dx } else { dx };
                self.cible = (
                    (mx + dx).clamp(marge_x, l - marge_x),
                    (my + dy).clamp(haut, bas.max(haut)),
                );
                vitesse *= 1.25;
                self.pause = 0.0;
            }
        } else if self.pause > 0.0 {
            self.pause -= dt;
            vitesse *= 0.12;
        } else if self.minuteur <= 0.0 || (self.x - self.cible.0).hypot(self.y - self.cible.1) < 6.0
        {
            self.minuteur = a.entre(7.0, 14.0);
            let pause = match mv {
                Mouvement::Lent | Mouvement::Flotte | Mouvement::Ondule => 0.08,
                Mouvement::Debout => 0.4,
                _ => 0.15,
            };
            if a.f() < pause {
                self.pause = a.entre(1.5, 4.0);
            }
            // Cible dans l'eau libre, sur tous les écrans et toute la hauteur de sa zone ;
            // plutôt sur le côté (les poissons nagent en long).
            let mut best = (f32::MIN, self.cible);
            // Une fois sur deux il flâne près d'où il est (sinon tous les poissons passent leur
            // temps à traverser le milieu de l'écran) ; sinon cap vers l'eau libre.
            let local = a.f() < 0.5;
            for _ in 0..3 {
                let c = if local {
                    let x = (self.x + a.entre(-90.0, 90.0)).clamp(marge_x, l - marge_x);
                    match b.eau(x - sl / 2.0, x + sl / 2.0) {
                        Some((t, f)) => {
                            let (y0, y1) = (t + sh / 2.0 + 10.0, f - sh / 2.0);
                            let y0 = if mv == Mouvement::Fond {
                                y0.max(y1 - 24.0)
                            } else {
                                y0
                            };
                            (
                                x,
                                (self.y + a.entre(-30.0, 30.0)).clamp(y0.min(y1), y1.max(y0)),
                            )
                        }
                        None => continue,
                    }
                } else {
                    place_libre(a, occupes, b, (sl, sh), mv == Mouvement::Fond)
                };
                let (ex, ey) = ((c.0 - self.x).abs(), (c.1 - self.y).abs());
                let libre = occupes
                    .iter()
                    .map(|&(ox, oy)| (c.0 - ox).hypot((c.1 - oy) * 1.6))
                    .fold(f32::MAX, f32::min);
                let score = ex.min(200.0) - (ey - ex * 0.6).max(0.0) * 2.0 + libre.min(80.0);
                if score > best.0 {
                    best = (score, c);
                }
            }
            self.cible = best.1;
            // Cible devant des yeux du fond : à côté d'eux (au hasard, pas tous au même endroit).
            for &(x0, y0, x1, y1) in evite {
                let m = 14.0 + sh / 2.0;
                if (x0 - m..x1 + m).contains(&self.cible.0)
                    && (y0 - m..y1 + m).contains(&self.cible.1)
                {
                    self.cible.0 = if a.f() < 0.5 {
                        (x0 - m - a.entre(0.0, 80.0)).max(marge_x)
                    } else {
                        (x1 + m + a.entre(0.0, 80.0)).min(l - marge_x)
                    };
                }
            }
        }
        // Accélérations : foncer (phoque, dauphin), jets (calmars), bonds (fond).
        match mv {
            Mouvement::Fonce if self.elan <= 0.0 && a.f() < dt * 0.1 => self.elan = 1.0,
            Mouvement::Jet | Mouvement::Fond if self.elan <= 0.0 && a.f() < dt * 0.4 => {
                self.elan = 0.7
            }
            _ => {}
        }
        // Nage par poussées et glissés : quelques coups de queue qui accélèrent, puis le
        // poisson glisse sur son élan en ralentissant (anguilles : ondulation continue).
        self.battue -= dt;
        if self.battue <= 0.0 {
            self.poussee = !self.poussee || mv == Mouvement::Ondule;
            let (p, g) = match mv {
                Mouvement::Lent => ((2.0, 4.5), (1.5, 3.5)),
                Mouvement::Plane => ((1.2, 2.2), (1.8, 4.0)),
                Mouvement::Fonce => ((0.6, 1.4), (0.8, 2.0)),
                _ => ((0.5, 1.5), (0.4, 1.3)),
            };
            let (d0, d1) = if self.poussee { p } else { g };
            self.battue = a.entre(d0, d1);
        }
        let (fort, doux) = match mv {
            Mouvement::Lent => (1.12, 0.75),
            Mouvement::Plane => (1.2, 0.7),
            Mouvement::Ondule => (1.0, 1.0),
            _ => (1.3, 0.5),
        };
        vitesse *= if self.poussee { fort } else { doux };
        // Allure qui varie doucement (pas de vitesse constante).
        vitesse *= 1.0 + 0.15 * (self.bob * 0.23 + self.phase * 3.0).sin();
        vitesse *= match mv {
            Mouvement::Fonce => 1.0 + 2.6 * self.elan,
            Mouvement::Jet => 0.35 + 3.6 * self.elan,
            Mouvement::Fond => 0.3 + 2.4 * self.elan,
            _ => 1.0,
        };
        if self.tourne > 0.0 {
            // Ralentit pendant le demi-tour.
            vitesse *= 0.55;
        }
        let (dx, dy) = (self.cible.0 - self.x, (self.cible.1 - self.y) * 0.6);
        let d = dx.hypot(dy).max(0.001);
        let freine = (d / 12.0).min(1.0);
        // Trajectoire courbe : la direction voulue oscille lentement autour de la cible.
        let ang = 0.45 * (self.bob * 0.41 + self.phase).sin()
            + 0.2 * (self.bob * 0.93 + self.phase * 1.7).sin();
        let (ca, sa) = (ang.cos(), ang.sin());
        let (ux, uy) = (dx / d, dy / d);
        let (ux, uy) = (ux * ca - uy * sa, ux * sa + uy * ca);
        let (vx, vy) = (ux * vitesse * freine, uy * vitesse * freine * 0.8);
        // Poussée : rattrape vite la vitesse voulue ; glissé : ralentit lentement (élan).
        let k = if self.elan > 0.0 {
            4.0
        } else if self.poussee {
            2.2
        } else {
            0.7
        };
        let k = (dt * k).min(1.0);
        self.vx += (vx - self.vx) * k;
        self.vy += (vy - self.vy) * k;
        // Séparation : s'écarte des voisins qui le chevauchent ; trop serré, il change de cible.
        self.vx += pousse.0 * 3.0 * dt;
        self.vy += pousse.1 * 3.0 * dt;
        if pousse.0.hypot(pousse.1) > 3.0 && self.banc.is_none() {
            self.minuteur = self.minuteur.min(0.4);
        }
        // Passage d'un écran à l'autre : seulement s'il y a de l'eau à cette hauteur.
        let nx = (self.x + self.vx * dt).clamp(marge_x - 2.0, l - marge_x + 2.0);
        match b.eau(nx - sl / 2.0, nx + sl / 2.0) {
            Some((t, f)) if self.y - sh / 2.0 >= t - 2.0 && self.y + sh / 2.0 <= f + 8.0 => {
                self.x = nx
            }
            _ => {
                self.vx *= -0.3;
                self.minuteur = self.minuteur.min(0.2);
            }
        }
        self.y = (self.y + self.vy * dt).clamp(haut - 4.0, bas.max(haut) + 2.0);
        // Retournement avec hystérésis et image écrasée pendant le demi-tour.
        if (self.vx < -1.5 && !self.gauche) || (self.vx > 1.5 && self.gauche) {
            self.gauche = !self.gauche;
            self.tourne = self.duree_tour();
        }
        self.tourne = (self.tourne - dt).max(0.0);
        // Corps incliné dans le sens de la montée ou de la descente (vue de côté seulement).
        self.pente_visee = match mv {
            Mouvement::Fond | Mouvement::Debout | Mouvement::Flotte => 0.0,
            _ => (self.vy / self.vx.abs().max(5.0)).clamp(-0.4, 0.4) * 0.8,
        };
        self.pente += (self.pente_visee - self.pente) * (dt * 2.5).min(1.0);
        let v = self.vx.hypot(self.vy);
        // Battement de queue : fort pendant la poussée, presque arrêté pendant le glissé.
        let vigueur = if self.poussee || mv == Mouvement::Ondule {
            1.0
        } else {
            0.25
        };
        self.vigueur += (vigueur - self.vigueur) * (dt * 3.0).min(1.0);
        self.anim = (self.anim + dt * self.cadence(v) * nb) % nb;
        if self.rarete == Rarete::Mythique {
            let p = self.monde(self.sprite.queue);
            self.traine.push_front(p);
            self.traine.truncate(30);
        }
    }

    /// Position écran (toile) d'un point du sprite (coordonnées de l'image tournée à droite).
    pub fn monde(&self, (sx, sy): (i32, i32)) -> (f32, f32) {
        let s = &self.sprite;
        let (x0, y0) = self.origine_fine();
        let sx = if self.gauche { s.l - 1 - sx } else { sx };
        let f = FIN as f32;
        ((x0 + sx) as f32 / f, (y0 + sy) as f32 / f)
    }

    /// Coin haut gauche du sprite sur la toile, en pixels fins.
    fn origine_fine(&self) -> (i32, i32) {
        let f = FIN as f32;
        (
            (self.x * f).round() as i32 - self.sprite.l / 2,
            (self.dessin_y() * f).round() as i32 - self.sprite.h / 2,
        )
    }

    fn dessin_y(&self) -> f32 {
        let (amp, vit) = match self.sprite.mouvement {
            Mouvement::Marche => (0.0, 1.0),
            // Le kraken se soulève lentement.
            Mouvement::Abysse => (1.5, 0.45),
            Mouvement::Debout | Mouvement::Flotte => (2.5, 1.3),
            _ => (1.2, 1.3),
        };
        let f = FIN as f32;
        ((self.y + (self.phase + self.bob * vit).sin() * amp) * f).round() / f
    }

    fn effets(&mut self, dt: f32, t: f32, parts: &mut Vec<Particule>, a: &mut Alea) {
        self.cligne -= dt;
        if self.cligne < -0.15 {
            self.cligne = a.entre(3.0, 9.0);
        }
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
            Rarete::Prehistorique => {
                // Sédiments et poussière d'ambre qui se détachent et coulent lentement.
                if a.f() < dt * 10.0 {
                    let (x, y) = self.monde(point_halo(a));
                    let c = [[0xd8, 0xb0, 0x70], [0xb0, 0x80, 0x40], [0xf4, 0xdc, 0xa4]]
                        [a.n(3) as usize];
                    parts.push(Particule {
                        x,
                        y,
                        vx: a.entre(-2.5, 2.5),
                        vy: a.entre(1.0, 4.5),
                        vie: 0.0,
                        duree: a.entre(2.0, 3.6),
                        c,
                        genre: Genre::Poussiere,
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

    /// Dessine le poisson et ses effets de rareté. Le sprite est posé au pixel fin, les
    /// effets (traînée, étoiles en orbite) au pixel monde.
    pub fn dessiner(&self, toile: &mut Toile, t: f32, avec_halo: bool) {
        let s = &self.sprite;
        let k = (self.anim as usize) % s.images.len();
        let im = &s.images[k];
        let (x0, y0) = self.origine_fine();
        let fw = FIN as f32;
        // Écrasement horizontal pendant le retournement (demi-tour rapide, pas de saut).
        let ratio = if self.tourne > 0.0 {
            0.5 + 0.5 * (1.0 - 2.0 * (self.tourne / self.duree_tour())).abs()
        } else {
            1.0
        };
        let cx = s.l as f32 / 2.0;
        // Pendant la première moitié du demi-tour, l'ancien sens.
        let gauche = self.gauche ^ (self.tourne > self.duree_tour() * 0.5);
        let colonnes: Vec<i32> = (0..s.l)
            .map(|x| {
                let x = if gauche { s.l - 1 - x } else { x };
                x0 + (cx + (x as f32 + 0.5 - cx) * ratio).floor() as i32
            })
            .collect();
        let tx = |x: i32| -> i32 { colonnes[x as usize] };
        let decal = self.decalages();
        let ty = |x: i32, y: i32| -> i32 { y0 + y + decal[x as usize] };
        let r = self.rarete;
        let pouls = 0.75 + 0.25 * (t * 2.4 + self.phase).sin();
        // Immobile (éclosion, planches) : couleurs vraies, sans l'eau du lointain. Sinon l'eau
        // du lointain à moitié : le dessin reste lisible.
        let prof = if self.fixe {
            0.0
        } else {
            s.taille.profondeur() * 0.5
        };
        let cligne = self.cligne < 0.0 && !self.fixe;

        // Traînée de flammes (Mythique) : part du bout de la queue, épaisse puis effilée,
        // bord qui vacille (pixels fins).
        if r == Rarete::Mythique {
            let (qx, qy) = self.monde(s.queue);
            let pts: Vec<(f32, f32)> = if self.fixe {
                let sens = if gauche { 1.0 } else { -1.0 };
                (0..30)
                    .map(|i| {
                        (
                            qx + sens * i as f32 * 0.9,
                            qy + (t * 5.0 - i as f32 * 0.35).sin() * 1.5 * (i as f32 / 30.0),
                        )
                    })
                    .collect()
            } else {
                std::iter::once((qx, qy))
                    .chain(self.traine.iter().copied())
                    .collect()
            };
            let n = pts.len().max(1) as f32;
            let epais = (s.hw() * 0.18).clamp(1.5, 5.0);
            for (i, w) in pts.windows(2).enumerate() {
                let f = i as f32 / n;
                let pas = ((w[1].0 - w[0].0).abs().max((w[1].1 - w[0].1).abs()) * fw).ceil() as i32;
                for j in 0..pas.clamp(1, 16) {
                    let u = j as f32 / pas.max(1) as f32;
                    let (px, py) = (
                        (w[0].0 + (w[1].0 - w[0].0) * u) * fw,
                        (w[0].1 + (w[1].1 - w[0].1) * u) * fw,
                    );
                    let vacille = 1.0 + 0.3 * (t * 17.0 + i as f32 * 1.3).sin();
                    let ray = (epais * fw * (1.0 - f).powf(1.3) * vacille).max(0.6);
                    for dy in -(ray.ceil() as i32)..=(ray.ceil() as i32) {
                        let k = dy.abs() as f32 / ray;
                        if k > 1.0 {
                            continue;
                        }
                        // Cœur jaune pâle, bords orange puis rouge sombre.
                        let chaleur = (1.0 - f) * (1.0 - k * 0.8);
                        let c = if chaleur > 0.6 {
                            melange([255, 200, 90], [255, 246, 190], (chaleur - 0.6) / 0.4)
                        } else {
                            melange([150, 20, 50], [255, 110, 50], chaleur / 0.6)
                        };
                        let a = (220.0 * (1.0 - f) * (1.0 - k * k * 0.6)) as u8;
                        toile.eclairer_fin(px.round() as i32, py.round() as i32 + dy, c, a);
                    }
                }
            }
        }

        // Halos (lueurs), pixels fins. L'Irréel n'a pas d'anneau : son contour est irisé.
        if avec_halo && r >= Rarete::Epique && r != Rarete::Irreel {
            let (c1, c2): (Rgb, Rgb) = match r {
                Rarete::Epique => ([0xc0, 0x70, 0xff], [0x80, 0x40, 0xe0]),
                Rarete::Legendaire => ([0xff, 0xd0, 0x50], [0xff, 0xa0, 0x20]),
                Rarete::Mythique => ([0xff, 0x50, 0x60], [0xc0, 0x20, 0x50]),
                Rarete::Prehistorique => ([0xff, 0xb8, 0x50], [0x9c, 0x5c, 0x20]),
                _ => ([0x70, 0xf0, 0xff], [0xa0, 0x70, 0xff]),
            };
            let (a1, a2) = match r {
                Rarete::Epique => (90.0, 40.0),
                Rarete::Mythique => (80.0, 30.0),
                _ => (120.0, 55.0),
            };
            for &(x, y) in &im.halo1 {
                let c = if r == Rarete::Celeste && bayer(x, y) < 0.5 {
                    c2
                } else {
                    c1
                };
                toile.eclairer_fin(tx(x), ty(x, y), c, (a1 * pouls) as u8);
            }
            for &(x, y) in &im.halo2 {
                // Second anneau tramé une case sur deux : lueur « pixel » sans flou.
                if (x + y).rem_euclid(2) == 0 {
                    toile.eclairer_fin(tx(x), ty(x, y), c2, (a2 * pouls) as u8);
                }
            }
        }

        // Étoiles en orbite (Céleste), moitié arrière (pixels monde).
        let (mx, my) = (
            (x0 as f32 + s.l as f32 / 2.0) / fw,
            (y0 as f32 + s.h as f32 / 2.0) / fw,
        );
        let orbite = |toile: &mut Toile, devant: bool| {
            for i in 0..3 {
                let ang = t * 0.9 + i as f32 * TAU / 3.0 + self.phase;
                if (ang.sin() > 0.0) != devant {
                    continue;
                }
                let ox = mx + ang.cos() * s.lw() * 0.55;
                let oy = my + ang.sin() * s.hw() * 0.42;
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
                            toile.eclairer_fin(tx(x) + dx, ty(x, y), c, 90);
                        }
                    }
                }
            }
        }

        // Corps.
        let irise: Vec<(Rgb, Rgb, f32)> = if r == Rarete::Irreel {
            (0..s.l)
                .map(|x| {
                    let h = x as f32 / s.l as f32 * 0.9 - t * 0.45;
                    (
                        hsv(h, 0.6, 0.75),
                        hsv(h + 0.5, 0.9, 1.0),
                        0.45 + 0.1 * (t * 2.0 - x as f32 * 0.05).sin(),
                    )
                })
                .collect()
        } else {
            Vec::new()
        };
        let reflet = (t * 0.7 + self.phase).rem_euclid(3.2) / 1.6 * (s.l + s.h) as f32 - s.h as f32;
        for y in 0..s.h {
            // Glitch : bandes de lignes décalées.
            let decal_ligne = if r == Rarete::Irreel
                && self.glitch > 0.0
                && ((y + (t * 80.0) as i32) / 6) % 3 == 0
            {
                if (y / 6) % 2 == 0 {
                    1
                } else {
                    -1
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
                let p = im.px(i);
                let mut c = [p[0], p[1], p[2]];
                if cligne && cl == OEIL {
                    // Paupière fermée : la peau autour (texture du dessin gardée), trait sombre en
                    // bas de l'œil.
                    let bas = im.classe.get(i + s.l as usize).is_none_or(|&b| b != OEIL);
                    c = if bas {
                        melange(s.paupiere, [0x08, 0x06, 0x0c], 0.55)
                    } else {
                        melange(c, s.paupiere, 0.8)
                    };
                }
                match r {
                    Rarete::PeuCommun if cl == CORPS => {
                        let d = x as f32 - (s.h - y) as f32 * 0.6 - reflet;
                        if (0.0..2.0).contains(&d) {
                            c = melange(c, [255, 255, 255], 0.25);
                        }
                    }
                    Rarete::Irreel => {
                        // Reflets arc-en-ciel posés sur le dessin (qui reste lisible), contour irisé
                        // qui chatoie (couleurs par colonne, calculées une fois par image).
                        let (corps, bord, chatoie) = &irise[x as usize];
                        if cl == CORPS || cl == NAGEOIRE {
                            let lum =
                                (p[0] as u32 * 77 + p[1] as u32 * 140 + p[2] as u32 * 38) >> 8;
                            let arc = melange(
                                *corps,
                                [255, 255, 255],
                                (lum as f32 / 255.0 - 0.5).max(0.0),
                            );
                            c = melange(c, arc, 0.24);
                        } else if cl == CONTOUR {
                            c = melange(c, *bord, *chatoie);
                        }
                    }
                    Rarete::Prehistorique => {
                        // Lueur d'ambre large et douce qui passe, contour couleur d'ambre.
                        if cl == CONTOUR {
                            c = melange(c, [0xc8, 0x78, 0x24], 0.45);
                        } else {
                            let d = x as f32 - (s.h - y) as f32 * 0.6 - reflet;
                            if (0.0..14.0).contains(&d) {
                                let k = 1.0 - (d - 7.0).abs() / 7.0;
                                c = melange(c, [0xff, 0xcc, 0x70], 0.18 * k);
                            }
                        }
                    }
                    Rarete::Legendaire if cl == CORPS => {
                        // Éclat doré fin et court qui glisse sur le corps.
                        let d = x as f32 - (s.h - y) as f32 * 0.6 - reflet;
                        let centre = (y as f32 - s.h as f32 * 0.4).abs() < s.h as f32 * 0.22;
                        if (0.0..2.0).contains(&d) && centre {
                            c = melange(c, [255, 236, 160], 0.3);
                        }
                    }
                    _ => {}
                }
                // L'eau du lointain n'efface ni le contour ni les yeux et lumières (qui luisent).
                if prof > 0.0 && cl != OEIL && cl != LUMIERE && cl != CONTOUR {
                    c = melange(c, EAU_LOIN, prof);
                }
                toile.poser_fin(tx(x) + decal_ligne, ty(x, y), c, p[3]);
            }
        }
        // Leurre lumineux (baudroie…) : point vif et lueur tramée qui pulse (pixels monde).
        if let Some(lu) = s.lueur {
            let (lx, ly) = self.monde(lu);
            let (lx, ly) = (lx as i32, ly as i32);
            let c: Rgb = [0xc8, 0xff, 0xf0];
            let rr = 2.0 + 3.0 * pouls;
            for dy in -6..=6 {
                for dx in -6..=6 {
                    let d = ((dx * dx + dy * dy) as f32).sqrt();
                    if d <= rr && (1.0 - d / rr) > bayer(lx + dx, ly + dy) * 0.9 {
                        toile.eclairer(lx + dx, ly + dy, c, (40.0 + 90.0 * (1.0 - d / rr)) as u8);
                    }
                }
            }
        }
        // Étoiles du corps (Céleste) qui scintillent.
        if r == Rarete::Celeste {
            for &(x, y, ph) in &s.etoiles {
                let v = (t * 2.6 + ph * TAU).sin();
                if v > 0.1 {
                    let (xx, yy) = (tx(x), ty(x, y));
                    toile.poser_fin(xx, yy, [255, 255, 255], if v > 0.6 { 255 } else { 150 });
                    if v > 0.8 {
                        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                            toile.eclairer_fin(xx + dx, yy + dy, [200, 230, 255], 110);
                        }
                    }
                }
            }
            orbite(toile, true);
        }
    }

    /// Décalage vertical de chaque colonne du dessin (pixels fins, image tournée à droite) :
    /// corps incliné en montée / descente, et queue en retard sur la tête quand la pente change
    /// (le corps se plie). Les grandes créatures s'inclinent moins.
    fn decalages(&self) -> Vec<i32> {
        let s = &self.sprite;
        let k = (60.0 / s.lw()).min(1.0);
        let (p, pv) = (self.pente * k, self.pente_visee * k);
        let flex = ((pv - p) * 0.8).clamp(-0.25, 0.25);
        let (cx, l) = (s.l as f32 / 2.0, s.l as f32);
        (0..s.l)
            .map(|x| {
                let u = x as f32 + 0.5 - cx;
                (p * u + flex * u * u / l).round() as i32
            })
            .collect()
    }

    /// Repose les yeux et lumières par-dessus les plantes du fond (créatures du lointain).
    pub fn dessiner_yeux(&self, toile: &mut Toile) {
        let s = &self.sprite;
        if self.tourne > 0.0 || (self.cligne < 0.0 && !self.fixe) {
            return;
        }
        let im = &s.images[(self.anim as usize) % s.images.len()];
        let (x0, y0) = self.origine_fine();
        let decal = self.decalages();
        for y in 0..s.h {
            for x in 0..s.l {
                let i = (y * s.l + x) as usize;
                if im.classe[i] == OEIL || im.classe[i] == LUMIERE {
                    let xx = if self.gauche { s.l - 1 - x } else { x };
                    let p = im.px(i);
                    let yy = y0 + y + decal[x as usize];
                    toile.poser_fin(x0 + xx, yy, [p[0], p[1], p[2]], 255);
                }
            }
        }
    }

    /// Zone des yeux en pixels monde (x0, y0, x1, y1), pour que les autres nageurs l'évitent.
    pub fn zone_yeux(&self) -> Option<(f32, f32, f32, f32)> {
        let s = &self.sprite;
        let im = &s.images[0];
        let mut z: Option<(i32, i32, i32, i32)> = None;
        for (i, &c) in im.classe.iter().enumerate() {
            if c == OEIL {
                let (x, y) = (i as i32 % s.l, i as i32 / s.l);
                z = Some(match z {
                    None => (x, y, x, y),
                    Some((a, b, c, d)) => (a.min(x), b.min(y), c.max(x), d.max(y)),
                });
            }
        }
        let (a, b, c, d) = z?;
        let (p, q) = (self.monde((a, b)), self.monde((c, d)));
        Some((p.0.min(q.0), p.1.min(q.1), p.0.max(q.0), p.1.max(q.1)))
    }
}

/// Place dans l'eau (sur un écran, corps `sl`x`sh` entier, près du sable si `fond`) la plus
/// loin possible des points `occupes` : meilleur de 10 tirages, distance verticale comptée
/// 1,6 fois (les poissons s'étalent sur tous les écrans).
fn place_libre(
    a: &mut Alea,
    occupes: &[(f32, f32)],
    b: &Bassin,
    (sl, sh): (f32, f32),
    fond: bool,
) -> (f32, f32) {
    let l = b.bas.len() as f32;
    let p = b.principal;
    let (t0, t1) = b.eau_totale();
    let mut best = (f32::MIN, ((p.0 + p.2) as f32 / 2.0, (t0 + t1) / 2.0));
    for _ in 0..10 {
        let x = a.entre(sl / 2.0 + 4.0, (l - sl / 2.0 - 4.0).max(sl / 2.0 + 4.0));
        let Some((t, f)) = b.eau(x - sl / 2.0, x + sl / 2.0) else {
            continue;
        };
        let (y0, y1) = (t + sh / 2.0 + 10.0, f - sh / 2.0);
        let y0 = if fond { y0.max(y1 - 24.0) } else { y0 };
        let p = (x, a.entre(y0.min(y1), y1.max(y0)));
        let libre = occupes
            .iter()
            .map(|&(ox, oy)| (p.0 - ox).hypot((p.1 - oy) * 1.6))
            .fold(f32::MAX, f32::min);
        if libre > best.0 {
            best = (libre, p);
        }
    }
    best.1
}

/// Poussée de séparation de chaque nageur (pixels monde de chevauchement à rattraper) :
/// deux créatures du même plan (devant / lointain) ne se superposent pas ; la plus petite
/// s'écarte le plus ; un banc reste groupé mais sans se chevaucher.
fn separation(nageurs: &[Nageur]) -> Vec<(f32, f32)> {
    let mut p = vec![(0.0f32, 0.0f32); nageurs.len()];
    for (i, a) in nageurs.iter().enumerate() {
        if a.fixe || a.attente > 0.0 {
            continue;
        }
        let groupe_a = a.banc.map_or(a.id, |b| b.0);
        for (j, b) in nageurs.iter().enumerate().skip(i + 1) {
            if b.fixe || b.attente > 0.0 {
                continue;
            }
            // Plans différents : le petit du premier plan s'écarte doucement de la géante du
            // fond (pas de poisson posé sur sa tête), la géante ne bouge pas pour lui ; les
            // serpents qui traversent ne repoussent personne.
            let autre_plan =
                (a.sprite.taille.profondeur() > 0.0) != (b.sprite.taille.profondeur() > 0.0);
            if autre_plan
                && (a.sprite.mouvement == Mouvement::Traverse
                    || b.sprite.mouvement == Mouvement::Traverse)
            {
                continue;
            }
            let meme_banc = groupe_a == b.banc.map_or(b.id, |c| c.0);
            let k = if autre_plan {
                0.65
            } else if meme_banc {
                0.55
            } else {
                0.85
            };
            let rx = (a.sprite.lw() + b.sprite.lw()) * 0.5 * k + 3.0;
            let ry = (a.sprite.hw() + b.sprite.hw()) * 0.5 * k + 2.0;
            let (dx, dy) = (a.x - b.x, a.y - b.y);
            let q = (dx / rx).hypot(dy / ry);
            if q >= 1.0 {
                continue;
            }
            // Directions : à l'écart l'un de l'autre (au même point : l'un à gauche, l'autre à droite).
            let (ux, uy) = if q > 0.01 {
                (dx / rx / q, dy / ry / q)
            } else {
                (1.0, 0.0)
            };
            let (ma, mb) = (a.sprite.lw() * a.sprite.hw(), b.sprite.lw() * b.sprite.hw());
            let f = 1.0 - q;
            let (fa, fb) = if autre_plan {
                let a_devant = a.sprite.taille.profondeur() == 0.0;
                (
                    if a_devant { f * 0.7 } else { 0.0 },
                    if a_devant { 0.0 } else { f * 0.7 },
                )
            } else {
                (f * mb / (ma + mb), f * ma / (ma + mb))
            };
            p[i].0 += ux * rx * fa;
            p[i].1 += uy * ry * fa;
            p[j].0 -= ux * rx * fb;
            p[j].1 -= uy * ry * fb;
        }
    }
    p
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
fn rayons(t: &mut Toile, temps: f32, b: &Bassin) {
    let (l, h) = (t.l as i32, t.h as i32);
    let n = (l / 96).max(5);
    for i in 0..n {
        let fi = i as f32;
        let base = (fi + 0.5) / n as f32 * l as f32 + (temps * 0.07 + fi * 1.9).sin() * 18.0;
        let larg = 9.0 + (fi * 2.3).sin().abs() * 14.0;
        let force = 0.6 + 0.4 * (temps * 0.5 + fi * 2.1).sin();
        // Chaque rayon descend depuis le haut de l'eau de sa colonne (écrans décalés).
        let c = b.col(base);
        if b.bas[c] == 0 {
            continue;
        }
        let (y0, fin) = (b.ciel[c], (b.sable[c] - 8 - b.ciel[c]) as f32 * 0.85);
        for dy in 0..fin as i32 {
            let y = y0 + dy;
            let fy = 1.0 - dy as f32 / fin;
            let x0 = base - dy as f32 * 0.38;
            for x in (x0 - larg) as i32..(x0 + larg) as i32 {
                if x < 0 || x >= l || y >= h || y < b.ciel[x as usize] || y >= b.bas[x as usize] {
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
    // Algue dessinée qui ondule ; celles du fond plus fondues dans l'eau.
    let img = &crate::decor::decor().algues[p.algue];
    let f = FIN as i32;
    let eau = (EAU_LOIN, if devant { 0.08 } else { 0.32 });
    let pied = (p.x * f, (p.base + 2) * f);
    crate::decor::algue(t, img, pied, p.visible, temps, p.phase, eau, p.miroir);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echelle_entiere() {
        assert_eq!(echelle_pour(1920), 4);
        assert_eq!(echelle_pour(1366), 2);
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
        // Limite des places : 40, une colossale en prend 2 (2 colossales au plus), et la surface.
        let places: usize = aq.nageurs.iter().map(|n| n.sprite.taille.places()).sum();
        let colossales = aq
            .nageurs
            .iter()
            .filter(|n| n.sprite.taille == Taille::Colossale)
            .count();
        assert!(
            places <= 40 && !aq.nageurs.is_empty() && colossales <= 2,
            "{places} {colossales}"
        );
        for _ in 0..300 {
            aq.avancer(1.0 / 30.0);
            aq.dessiner(&p);
        }
        // Tous restent dans la toile (les serpents colossaux sortent entre deux traversées).
        for n in aq
            .nageurs
            .iter()
            .filter(|n| n.sprite.mouvement != Mouvement::Traverse)
        {
            assert!(n.x >= 0.0 && n.x <= 480.0 && n.y >= 0.0 && n.y <= 270.0);
        }
        p.reglages.max_affiches = 5;
        aq.synchroniser(&p);
        let places: usize = aq.nageurs.iter().map(|n| n.sprite.taille.places()).sum();
        assert!(!aq.nageurs.is_empty() && places <= 5);
    }

    #[test]
    fn poissons_etales_sans_entassement() {
        // Aquarium de démo : après une minute, personne n'est entassé (au plus 2 voisins du
        // même plan dont le centre est dans sa moitié de corps) et les poissons bougent tous.
        let p = crate::capture::partie_demo();
        let mut aq = Aquarium::new(480, 270, 1);
        aq.synchroniser(&p);
        let mut parcours = vec![0.0f32; aq.nageurs.len()];
        let mut pire = 0;
        for k in 0..1800 {
            let avant: Vec<(f32, f32)> = aq.nageurs.iter().map(|n| (n.x, n.y)).collect();
            aq.avancer(1.0 / 30.0);
            for (i, n) in aq.nageurs.iter().enumerate() {
                parcours[i] += (n.x - avant[i].0).hypot(n.y - avant[i].1);
            }
            if k < 300 {
                continue;
            }
            for a in &aq.nageurs {
                let plan = a.sprite.taille.profondeur() > 0.0;
                let voisins = aq
                    .nageurs
                    .iter()
                    .filter(|b| b.id != a.id && (b.sprite.taille.profondeur() > 0.0) == plan)
                    .filter(|b| {
                        (a.x - b.x).abs() < a.sprite.lw() * 0.25 + 1.0
                            && (a.y - b.y).abs() < a.sprite.hw() * 0.25 + 1.0
                    })
                    .count();
                pire = pire.max(voisins);
            }
        }
        assert!(pire <= 2, "{pire} voisins entassés");
        for (i, n) in aq.nageurs.iter().enumerate() {
            if matches!(
                n.sprite.mouvement,
                Mouvement::Nage | Mouvement::Banc | Mouvement::Lent
            ) {
                assert!(
                    parcours[i] > 60.0,
                    "{} : {} px en 60 s",
                    n.espece,
                    parcours[i]
                );
            }
        }
    }

    #[test]
    fn personne_dans_les_trous_entre_ecrans() {
        // Principal 1920x1080 à droite d'un 2560x1440 plus haut : les trous au-dessus et
        // au-dessous du principal ne sont pas de l'eau.
        let d =
            crate::ecrans::Disposition::depuis_texte("1920x1080+0+0,2560x1440+-2560+-200").unwrap();
        let p = crate::capture::partie_demo();
        let mut aq = Aquarium::avec_zones(1120, 360, 1, &d.zones(4), &d.travail(4));
        aq.synchroniser(&p);
        for k in 0..1800 {
            aq.avancer(1.0 / 30.0);
            if k % 10 != 0 {
                continue;
            }
            for n in aq
                .nageurs
                .iter()
                .filter(|n| !matches!(n.sprite.mouvement, Mouvement::Traverse | Mouvement::Abysse))
            {
                // Le dessin (sans ses marges d'animation) reste sur un écran.
                let (dl, dh) = (n.sprite.lw() * 0.3, n.sprite.hw() * 0.3);
                let ok = aq
                    .bassin
                    .eau(n.x - dl, n.x + dl)
                    .is_some_and(|(t, _)| n.y - dh >= t - 3.0);
                assert!(ok, "{} dans un trou : {} {}", n.espece, n.x, n.y);
            }
        }
    }
}

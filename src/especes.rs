//! Catalogue des espèces : nom, rareté, classe de taille, façon de bouger, palette et variantes
//! de couleur, et la fonction qui dessine chaque image de son sprite (sous-modules par rareté).

mod abysses;
mod grands;
mod petits;
mod recif;

use crate::art::{lum, opaque, rampe, ton, Dessin, Px, Rampe, P};
use crate::jeu::{Alea, Rarete};
use crate::toile::{hsv, Rgb};

/// Classe de taille : longueur typique sur la toile (~480 px de large) et place à l'écran.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Taille {
    /// ~8-12 px.
    Minuscule,
    /// ~13-22 px.
    Petite,
    /// ~23-40 px.
    Moyenne,
    /// ~40-80 px.
    Grande,
    /// Baleines, orque, grands requins : nagent au second plan.
    Enorme,
    /// Créatures irréelles qui prennent une grande part de l'écran, tout au fond.
    Colossale,
}

impl Taille {
    /// Places prises dans la limite des poissons affichés (lisibilité de l'écran).
    pub fn places(self) -> usize {
        match self {
            Taille::Enorme => 2,
            Taille::Colossale => 4,
            _ => 1,
        }
    }

    /// Profondeur 0 (devant) .. 1 (tout au fond) : teinte d'eau et ordre de dessin.
    pub fn profondeur(self) -> f32 {
        match self {
            Taille::Enorme => 0.22,
            Taille::Colossale => 0.38,
            _ => 0.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mouvement {
    /// Poisson : cibles, pauses, battement de queue.
    Nage,
    /// Petits poissons en bancs.
    Banc,
    /// Gros animaux tranquilles.
    Lent,
    /// Phoque, dauphin : nage souple et accélérations soudaines.
    Fonce,
    /// Anguilles, serpents : corps ondulant.
    Ondule,
    /// Raies, tortues : battements d'ailes et planés.
    Plane,
    /// Méduses : pulsations qui les font monter, puis elles se laissent couler.
    Pulse,
    /// Calmars : jets par à-coups.
    Jet,
    /// Gobies, crevettes, poulpes : près du sable, petits bonds.
    Fond,
    /// Hippocampe : debout, lent, se balance.
    Debout,
    /// Crabe, isopode : marche sur le sable.
    Marche,
    /// Dieux et yeux des abysses : dérive lente en suspension.
    Flotte,
    /// Kraken : posé tout en bas, tentacules qui montent.
    Abysse,
    /// Serpents colossaux : traversent l'écran d'un bord à l'autre puis disparaissent.
    Traverse,
}

/// Variante de couleur : décalage de teinte (tours), saturation et valeur des couleurs 0 et 1.
pub struct Variante {
    pub nom: &'static str,
    pub teinte: f32,
    pub sat: f32,
    pub val: f32,
}

const fn v(nom: &'static str, teinte: f32, sat: f32, val: f32) -> Variante {
    Variante {
        nom,
        teinte,
        sat,
        val,
    }
}

/// Palette d'une variante : 4 couleurs et leurs rampes.
pub struct Pal {
    pub c: [Rgb; 4],
    pub r: [Rampe; 4],
}

pub type Art = fn(&mut Dessin, f32, &Pal);

pub struct Espece {
    /// Identifiant stable (sauvegarde).
    pub cle: &'static str,
    pub nom: &'static str,
    pub rarete: Rarete,
    pub taille: Taille,
    pub mouvement: Mouvement,
    /// Taille du dessin à l'échelle 1 (sans la marge).
    pub dims: (i32, i32),
    pub images: u8,
    pub couleurs: [Rgb; 4],
    pub variantes: &'static [Variante],
    /// Poids du tirage dans sa rareté (1 = normal).
    pub poids: f32,
    pub art: Art,
}

impl Espece {
    pub fn palette(&self, variante: usize) -> Pal {
        let v = &self.variantes[variante.min(self.variantes.len() - 1)];
        let mut c = self.couleurs;
        for k in c.iter_mut().take(2) {
            let (h, s, vv) = crate::art::rgb_hsv(*k);
            *k = hsv(
                h + v.teinte,
                (s * v.sat).clamp(0.0, 1.0),
                (vv * v.val).clamp(0.0, 1.0),
            );
        }
        Pal { c, r: c.map(rampe) }
    }

    /// Nom complet d'un individu (« Requin-marteau albinos »).
    pub fn nom_variante(&self, variante: usize) -> String {
        match self.variantes.get(variante).map(|v| v.nom) {
            Some(n) if !n.is_empty() => format!("{} {n}", self.nom),
            _ => self.nom.to_string(),
        }
    }
}

/// Variantes courantes.
pub const V_POISSON: &[Variante] = &[
    v("", 0.0, 1.0, 1.0),
    v("doré", 0.06, 1.0, 1.08),
    v("azur", 0.5, 1.0, 1.0),
    v("albinos", 0.0, 0.15, 1.25),
];
pub const V_SOMBRE: &[Variante] = &[
    v("", 0.0, 1.0, 1.0),
    v("des grands fonds", 0.08, 0.8, 0.7),
    v("pâle", 0.0, 0.35, 1.2),
];
pub const V_VIF: &[Variante] = &[
    v("", 0.0, 1.0, 1.0),
    v("pourpre", 0.82, 1.0, 1.0),
    v("émeraude", 0.33, 1.0, 1.0),
    v("solaire", 0.12, 1.0, 1.1),
];
pub const V_COSMIQUE: &[Variante] = &[
    v("", 0.0, 1.0, 1.0),
    v("aurorale", 0.3, 1.0, 1.05),
    v("pourpre", 0.85, 1.0, 1.0),
];

pub fn catalogue() -> &'static [Espece] {
    static CAT: std::sync::OnceLock<Vec<Espece>> = std::sync::OnceLock::new();
    CAT.get_or_init(|| {
        let mut c = Vec::new();
        c.extend(petits::especes());
        c.extend(recif::especes());
        c.extend(grands::especes());
        c.extend(abysses::especes());
        c
    })
}

pub fn par_cle(cle: &str) -> Option<&'static Espece> {
    catalogue().iter().find(|e| e.cle == cle)
}

pub fn de_rarete(r: Rarete) -> impl Iterator<Item = &'static Espece> {
    catalogue().iter().filter(move |e| e.rarete == r)
}

/// Espèce et variante d'un poisson, tirées de sa graine (pondérées par `poids`).
/// Sert à l'éclosion et à la migration des sauvegardes du lot 1.
pub fn choisir(r: Rarete, graine: u64) -> (&'static Espece, u8) {
    let mut a = Alea(graine ^ 0x5bec_1e5a_u64.rotate_left(17));
    let liste: Vec<&Espece> = de_rarete(r).collect();
    let total: f32 = liste.iter().map(|e| e.poids).sum();
    let mut x = a.f() * total;
    let mut e = liste[liste.len() - 1];
    for &c in &liste {
        x -= c.poids;
        if x < 0.0 {
            e = c;
            break;
        }
    }
    // La variante de base sort une fois sur deux.
    let var = if a.f() < 0.5 {
        0
    } else {
        1 + a.n(e.variantes.len().max(2) as u32 - 1) as u8
    };
    (e, var.min(e.variantes.len() as u8 - 1))
}

// ---------- outils communs aux dessins ----------

/// Ondulation d'un corps de poisson : déplacement de la ligne médiane (forte vers la queue).
pub fn plier(t: f32, b: f32, amp: f32) -> f32 {
    let k = (1.0 - t / 0.65).max(0.0);
    b * amp * k * k
}

/// Peinture d'un corps à contre-ombrage : dos (rampe `dos`) au-dessus de `seuil`, ventre clair.
pub fn contre<'a>(dos: &'a Rampe, ventre: &'a Rampe, seuil: f32) -> impl Fn(&P) -> Option<Px> + 'a {
    move |p| {
        if p.v > seuil {
            opaque(ton(ventre, lum(p.v) * 0.6 + 0.4, p.x, p.y))
        } else {
            opaque(ton(dos, lum(p.v), p.x, p.y))
        }
    }
}

/// Volume simple d'une seule rampe.
pub fn volume(r: &Rampe) -> impl Fn(&P) -> Option<Px> + '_ {
    move |p| opaque(ton(r, lum(p.v), p.x, p.y))
}

/// Nageoire translucide : plus claire et plus transparente vers le bord (`t` du polygone ou
/// distance donnée), rayons d'un ton plus sombre une colonne sur `pas`.
pub fn membrane(r: &Rampe, a: u8) -> impl Fn(&P) -> Option<Px> + '_ {
    move |p| {
        let c = ton(r, 0.35 + 0.4 * (1.0 - p.v.abs()), p.x, p.y);
        Some([c[0], c[1], c[2], a])
    }
}

#[derive(Clone, Copy)]
pub enum Caudale {
    Fourche,
    Ronde,
    Croissant,
    Voile,
}

/// Contour d'une nageoire caudale qui part de `(bx, by)` vers la gauche ; `b` = battement -1..1.
pub fn caudale(f: Caudale, bx: f32, by: f32, long: f32, ouv: f32, b: f32) -> Vec<(f32, f32)> {
    let base = (ouv * 0.28).max(0.8);
    let dy = b * long * 0.35;
    let serre = 1.0 - 0.18 * b.abs();
    let o = ouv * serre;
    match f {
        Caudale::Fourche => vec![
            (bx + 0.5, by - base),
            (bx - long, by - o + dy),
            (bx - long * 0.55, by + dy * 0.6),
            (bx - long, by + o + dy),
            (bx + 0.5, by + base),
        ],
        Caudale::Croissant => vec![
            (bx + 0.5, by - base * 0.6),
            (bx - long * 0.55, by - o * 0.6 + dy * 0.5),
            (bx - long, by - o + dy),
            (bx - long * 0.62, by - o * 0.35 + dy * 0.7),
            (bx - long * 0.35, by + dy * 0.5),
            (bx - long * 0.62, by + o * 0.35 + dy * 0.7),
            (bx - long, by + o + dy),
            (bx - long * 0.55, by + o * 0.6 + dy * 0.5),
            (bx + 0.5, by + base * 0.6),
        ],
        Caudale::Ronde | Caudale::Voile => {
            let n = 9;
            let mut v = vec![(bx + 0.5, by - base)];
            let lg = if matches!(f, Caudale::Voile) {
                long
            } else {
                long * 0.9
            };
            for i in 0..=n {
                let a = -1.0 + 2.0 * i as f32 / n as f32;
                let ondule = if matches!(f, Caudale::Voile) {
                    1.0 - 0.18 * ((a * 3.0 + b) * 2.0).sin().abs()
                } else {
                    1.0
                };
                let r = lg * ondule * (1.0 - 0.12 * a * a);
                v.push((
                    bx - r * (a * 0.9).cos(),
                    by + a * o + dy * (0.6 + 0.4 * ondule),
                ));
            }
            v.push((bx + 0.5, by + base));
            v
        }
    }
}

/// Rapproche les points de `ancre` (facteur `k` < 1) : sert au liseré des nageoires.
pub fn retrecir(pts: &[(f32, f32)], ancre: (f32, f32), k: f32) -> Vec<(f32, f32)> {
    pts.iter()
        .map(|&(x, y)| (ancre.0 + (x - ancre.0) * k, ancre.1 + (y - ancre.1) * k))
        .collect()
}

/// Nageoire à liseré : polygone dans la rampe `bord`, puis l'intérieur rétréci dans `fond`.
pub fn bordee(
    d: &mut Dessin,
    pts: &[(f32, f32)],
    ancre: (f32, f32),
    fond: &Rampe,
    bord: &Rampe,
    k: f32,
) {
    use crate::poisson::NAGEOIRE;
    d.poly(pts, NAGEOIRE, |q| opaque(ton(bord, 0.45, q.x, q.y)));
    d.poly(&retrecir(pts, ancre, k), NAGEOIRE, |q| {
        opaque(ton(fond, 0.4 + 0.35 * (1.0 - q.t), q.x, q.y))
    });
}

/// Fiche d'espèce (poids de tirage 1).
#[allow(clippy::too_many_arguments)]
pub fn esp(
    cle: &'static str,
    nom: &'static str,
    rarete: Rarete,
    taille: Taille,
    mouvement: Mouvement,
    dims: (i32, i32),
    images: u8,
    couleurs: [Rgb; 4],
    variantes: &'static [Variante],
    art: Art,
) -> Espece {
    Espece {
        cle,
        nom,
        rarete,
        taille,
        mouvement,
        dims,
        images,
        couleurs,
        variantes,
        poids: 1.0,
        art,
    }
}

/// Point tourné de `ang` radians autour de `o`.
pub fn tourner(o: (f32, f32), p: (f32, f32), ang: f32) -> (f32, f32) {
    let (s, c) = ang.sin_cos();
    let (dx, dy) = (p.0 - o.0, p.1 - o.1);
    (o.0 + dx * c - dy * s, o.1 + dx * s + dy * c)
}

/// Ligne ondulante de `n` segments (tentacules) : départ, direction (angle), longueur,
/// rayon de départ et de bout, amplitude et phase de la vague.
#[allow(clippy::too_many_arguments)]
pub fn tentacule(
    depart: (f32, f32),
    ang: f32,
    long: f32,
    r0: f32,
    r1: f32,
    amp: f32,
    phase: f32,
    boucle: f32,
) -> Vec<(f32, f32, f32)> {
    let n = ((long / 2.5) as usize).clamp(4, 40);
    let mut pts = Vec::with_capacity(n + 1);
    let (mut x, mut y) = depart;
    let mut a = ang;
    let pas = long / n as f32;
    for i in 0..=n {
        let u = i as f32 / n as f32;
        pts.push((x, y, r0 + (r1 - r0) * u));
        // La vague courbe le tentacule, `boucle` l'enroule vers le bout.
        a += (u * 5.0 - phase).sin() * amp / n as f32 + boucle * u * u / n as f32;
        x += a.cos() * pas;
        y += a.sin() * pas;
    }
    pts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_coherent() {
        let c = catalogue();
        assert!(c.len() >= 40, "{} espèces", c.len());
        for r in Rarete::TOUTES {
            assert!(
                de_rarete(r).count() >= 3,
                "{} : moins de 3 espèces",
                r.nom()
            );
        }
        let mut cles: Vec<&str> = c.iter().map(|e| e.cle).collect();
        let mut noms: Vec<&str> = c.iter().map(|e| e.nom).collect();
        cles.sort();
        noms.sort();
        cles.dedup();
        noms.dedup();
        assert_eq!(cles.len(), c.len(), "clés en double");
        assert_eq!(noms.len(), c.len(), "noms en double");
        for e in c {
            assert!(
                !e.variantes.is_empty() && e.images > 0 && e.poids > 0.0,
                "{}",
                e.cle
            );
        }
    }

    #[test]
    fn tirage_par_graine() {
        for r in Rarete::TOUTES {
            let mut vues = std::collections::HashSet::new();
            for g in 0..2000u64 {
                let (e, v) = choisir(r, g.wrapping_mul(0x9e37_79b9_7f4a_7c15));
                assert_eq!(e.rarete, r);
                assert!((v as usize) < e.variantes.len());
                assert_eq!(
                    choisir(r, g.wrapping_mul(0x9e37_79b9_7f4a_7c15)).0.cle,
                    e.cle
                );
                vues.insert(e.cle);
            }
            // Toutes les espèces d'une rareté finissent par sortir.
            assert_eq!(vues.len(), de_rarete(r).count(), "{}", r.nom());
        }
    }
}

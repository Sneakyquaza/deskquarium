//! Sprite d'un individu : l'espèce (catalogue `especes.rs`) dessine chaque image, ici on ajoute
//! la marge, le contour « sel-out », les classes de pixels et les halos des effets de rareté
//! (dessinés au rendu dans `scene.rs`).

use crate::especes::{Espece, Mouvement, Taille};
use crate::jeu::{Alea, Forme, Modif};

/// Classe d'un pixel du sprite (sert aux effets : arc-en-ciel sur le corps, pas sur l'œil…).
pub const VIDE: u8 = 0;
pub const CONTOUR: u8 = 1;
pub const CORPS: u8 = 2;
pub const NAGEOIRE: u8 = 3;
pub const OEIL: u8 = 4;
/// Lumière propre (leurre, taches lumineuses) : luit à travers l'eau, ne cligne pas.
pub const LUMIERE: u8 = 5;

pub struct Image {
    /// Index dans `pal` (0 = vide) : un octet par pixel (mémoire : les images des grandes
    /// créatures sont nombreuses).
    pub idx: Vec<u8>,
    /// Couleurs RGBA non prémultipliées du sprite (partagées par ses images).
    pub pal: std::sync::Arc<Vec<[u8; 4]>>,
    pub classe: Vec<u8>,
    /// Halo à 1 et 2 pixels autour de la silhouette (lueurs).
    pub halo1: Vec<(i32, i32)>,
    pub halo2: Vec<(i32, i32)>,
}

impl Image {
    /// Couleur RGBA du pixel `i`.
    #[inline]
    pub fn px(&self, i: usize) -> [u8; 4] {
        self.pal[self.idx[i] as usize]
    }
}

pub struct Sprite {
    pub l: i32,
    pub h: i32,
    pub images: Vec<Image>,
    /// Point d'attache de la queue (traînée), bouche (bulles), leurre lumineux, dans l'image 0,
    /// tournée à droite.
    pub queue: (i32, i32),
    pub bouche: (i32, i32),
    pub lueur: Option<(i32, i32)>,
    /// Étoiles du corps (Céleste) : position et phase.
    pub etoiles: Vec<(i32, i32, f32)>,
    pub mouvement: Mouvement,
    pub taille: Taille,
    /// Couleur de la paupière (clignement des yeux).
    pub paupiere: [u8; 3],
    /// Dessin sombre (luminosité moyenne du corps basse) : contre-jour plus fort en Abysses.
    pub sombre: f32,
}

impl Sprite {
    /// Taille en pixels monde (le sprite est en pixels fins).
    pub fn lw(&self) -> f32 {
        self.l as f32 / crate::toile::FIN as f32
    }

    pub fn hw(&self) -> f32 {
        self.h as f32 / crate::toile::FIN as f32
    }
}

pub const MARGE: i32 = 3;

/// Facteur d'échelle d'un individu : ±10 % autour de la taille de l'espèce selon `taille`
/// (0,6..1,5, celle qui multiplie aussi le revenu).
pub fn echelle_individu(taille: f32) -> f32 {
    0.9 + 0.2 * ((taille - 0.6) / 0.9).clamp(0.0, 1.0)
}

/// Sprite d'une espèce, variante et échelle données (déterministe).
pub fn sprite(e: &Espece, variante: usize, echelle: f32) -> Sprite {
    sprite_forme(e, variante, echelle, Forme::default())
}

/// Sprite d'un individu shiny et/ou modifié (palette recolorée, taille de l'Ombre).
pub fn sprite_forme(e: &Espece, variante: usize, echelle: f32, forme: Forme) -> Sprite {
    sprite_png(e, variante, echelle, png_espece(e.cle), forme)
}

/// Shiny : vraie recoloration (comme un Pokémon chromatique), pas juste plus clair. La teinte
/// dominante du dessin tourne d'un pas propre à l'espèce (les rampes d'ombre restent) et les
/// gris du corps prennent une teinte complémentaire ; yeux, lumières, contour sombre et
/// reflets blancs gardés.
fn recolorer_shiny(src: &mut [[u8; 4]], cle: &str) {
    use crate::toile::{hsv, rgb_hsv};
    // Teinte dominante (histogramme des pixels colorés).
    let mut histo = [0f32; 24];
    for p in src.iter().filter(|p| p[3] == 255) {
        let (h, s, v) = rgb_hsv([p[0], p[1], p[2]]);
        if s > 0.25 && v > 0.2 {
            histo[((h.rem_euclid(1.0) * 24.0) as usize).min(23)] += s;
        }
    }
    let dominante = (0..24)
        .max_by(|&a, &b| histo[a].total_cmp(&histo[b]))
        .map_or(0.0, |k| (k as f32 + 0.5) / 24.0);
    // Pas propre à l'espèce, toujours loin des variantes (jamais moins d'un tiers de tour).
    let h = cle.bytes().fold(0x811c_9dc5u32, |h, b| {
        (h ^ b as u32).wrapping_mul(0x0100_0193)
    });
    let pas = [0.36, 0.45, 0.55, 0.62, 0.7][(h % 5) as usize];
    let gris = (dominante + pas + 0.5).rem_euclid(1.0);
    for p in src.iter_mut().filter(|p| p[3] == 255) {
        let (hh, s, v) = rgb_hsv([p[0], p[1], p[2]]);
        let c = if s >= 0.3 {
            hsv(hh + pas, (s * 1.08).min(1.0), v)
        } else if (0.2..0.93).contains(&v) {
            // Gris et couleurs pâles du corps : teintés franchement, l'ombrage gardé.
            hsv(gris, 0.34 + s * 0.5, v)
        } else {
            continue;
        };
        *p = [c[0], c[1], c[2], p[3]];
    }
}

/// Modificateur : teinte légère ou effet du corps cuit dans la palette (l'aura et ce qui bouge
/// sont dessinés au rendu, `scene.rs`).
fn recolorer_modif(src: &mut [[u8; 4]], m: Modif, w: i32) {
    use crate::toile::{hsv, melange, rgb_hsv};
    for (i, p) in src.iter_mut().enumerate().filter(|(_, p)| p[3] == 255) {
        let (x, y) = (i as i32 % w, i as i32 / w);
        let c = [p[0], p[1], p[2]];
        let (hh, s, v) = rgb_hsv(c);
        let lum = (c[0] as f32 * 0.3 + c[1] as f32 * 0.59 + c[2] as f32 * 0.11) / 255.0;
        let n = match m {
            // Or : rampe dorée selon la luminosité, à 40 % (le dessin reste lisible).
            Modif::Dore => melange(c, hsv(0.12, 0.75 - lum * 0.4, 0.35 + lum * 0.75), 0.4),
            // Diamant : blanc bleuté, facettes en diagonale plus claires.
            Modif::Diamant => {
                let facette = (x + y).rem_euclid(9) == 0 || (x - y).rem_euclid(13) == 0;
                let b = melange(c, [0xd8, 0xf4, 0xff], 0.38);
                if facette && v > 0.25 {
                    melange(b, [255, 255, 255], 0.35)
                } else {
                    b
                }
            }
            // Électrique : couleurs plus vives, un peu froides.
            Modif::Electrique => melange(
                hsv(hh, (s * 1.25).min(1.0), v.max(0.15)),
                [0x70, 0xf0, 0xff],
                0.16,
            ),
            // Arc-en-ciel : bandes de teinte le long du corps (elles tournent au rendu).
            Modif::ArcEnCiel => melange(c, hsv(x as f32 / w as f32, 0.7, 0.45 + lum * 0.6), 0.42),
            // Ombre : plus sombre, violacée, désaturée.
            Modif::Ombre => {
                let sombre = hsv(hh, s * 0.6, v * 0.55);
                melange(sombre, [0x2c, 0x14, 0x44], 0.3)
            }
            // Cristal : pâle et froid (posé translucide au rendu), facettes.
            Modif::Cristal => {
                let facette = (x * 2 + y).rem_euclid(11) == 0;
                let b = melange(c, [0xc0, 0xf0, 0xf8], 0.5);
                if facette {
                    melange(b, [255, 255, 255], 0.45)
                } else {
                    b
                }
            }
            // Galaxie : ciel de nuit (bleu nuit → violet → rose selon l'ombrage).
            Modif::Galaxie => {
                let g = if lum < 0.5 {
                    melange([0x10, 0x0c, 0x38], [0x5c, 0x2c, 0x9c], lum * 2.0)
                } else {
                    melange([0x5c, 0x2c, 0x9c], [0xe8, 0x8c, 0xe8], (lum - 0.5) * 2.0)
                };
                melange(c, g, 0.75)
            }
        };
        *p = [n[0], n[1], n[2], p[3]];
    }
}

/// Sprite pixel art d'une espèce (image IA nettoyée par `tools/pixeliser.py`).
fn png_espece(cle: &str) -> &'static [u8] {
    match cle {
        "clown" => include_bytes!("../assets/especes/clown.png"),
        "neon" => include_bytes!("../assets/especes/neon.png"),
        "guppy" => include_bytes!("../assets/especes/guppy.png"),
        "rouge" => include_bytes!("../assets/especes/rouge.png"),
        "sardine" => include_bytes!("../assets/especes/sardine.png"),
        "gobie" => include_bytes!("../assets/especes/gobie.png"),
        "chirurgien" => include_bytes!("../assets/especes/chirurgien.png"),
        "ange" => include_bytes!("../assets/especes/ange.png"),
        "globe" => include_bytes!("../assets/especes/globe.png"),
        "hippocampe" => include_bytes!("../assets/especes/hippocampe.png"),
        "crabe" => include_bytes!("../assets/especes/crabe.png"),
        "crevette" => include_bytes!("../assets/especes/crevette.png"),
        "meduse" => include_bytes!("../assets/especes/meduse.png"),
        "poulpe" => include_bytes!("../assets/especes/poulpe.png"),
        "murene" => include_bytes!("../assets/especes/murene.png"),
        "anguille" => include_bytes!("../assets/especes/anguille.png"),
        "rascasse" => include_bytes!("../assets/especes/rascasse.png"),
        "tortue" => include_bytes!("../assets/especes/tortue.png"),
        "calamar" => include_bytes!("../assets/especes/calamar.png"),
        "manta" => include_bytes!("../assets/especes/manta.png"),
        "marteau" => include_bytes!("../assets/especes/marteau.png"),
        "espadon" => include_bytes!("../assets/especes/espadon.png"),
        "phoque" => include_bytes!("../assets/especes/phoque.png"),
        "dauphin" => include_bytes!("../assets/especes/dauphin.png"),
        "mola" => include_bytes!("../assets/especes/mola.png"),
        "blanc" => include_bytes!("../assets/especes/blanc.png"),
        "orque" => include_bytes!("../assets/especes/orque.png"),
        "requin_baleine" => include_bytes!("../assets/especes/requin_baleine.png"),
        "narval" => include_bytes!("../assets/especes/narval.png"),
        "calamar_geant" => include_bytes!("../assets/especes/calamar_geant.png"),
        "baudroie" => include_bytes!("../assets/especes/baudroie.png"),
        "dragon_feuillu" => include_bytes!("../assets/especes/dragon_feuillu.png"),
        "coelacanthe" => include_bytes!("../assets/especes/coelacanthe.png"),
        "lutin" => include_bytes!("../assets/especes/lutin.png"),
        "bosse" => include_bytes!("../assets/especes/bosse.png"),
        "isopode" => include_bytes!("../assets/especes/isopode.png"),
        "hydre" => include_bytes!("../assets/especes/hydre.png"),
        "cadborosaure" => include_bytes!("../assets/especes/cadborosaure.png"),
        "baleine_stellaire" => include_bytes!("../assets/especes/baleine_stellaire.png"),
        "meduse_nebuleuse" => include_bytes!("../assets/especes/meduse_nebuleuse.png"),
        "raie_aurore" => include_bytes!("../assets/especes/raie_aurore.png"),
        "carpe_comete" => include_bytes!("../assets/especes/carpe_comete.png"),
        "kraken" => include_bytes!("../assets/especes/kraken.png"),
        "cthulhu" => include_bytes!("../assets/especes/cthulhu.png"),
        "leviathan" => include_bytes!("../assets/especes/leviathan.png"),
        "oeil" => include_bytes!("../assets/especes/oeil.png"),
        "bloop" => include_bytes!("../assets/especes/bloop.png"),
        "grand_abyssal" => include_bytes!("../assets/especes/grand_abyssal.png"),
        "lusca" => include_bytes!("../assets/especes/lusca.png"),
        "jormungandr" => include_bytes!("../assets/especes/jormungandr.png"),
        "loch_ness" => include_bytes!("../assets/especes/loch_ness.png"),
        "megalodon" => include_bytes!("../assets/especes/megalodon.png"),
        "mosasaure" => include_bytes!("../assets/especes/mosasaure.png"),
        "liopleurodon" => include_bytes!("../assets/especes/liopleurodon.png"),
        "ichthyosaure" => include_bytes!("../assets/especes/ichthyosaure.png"),
        "dunkleosteus" => include_bytes!("../assets/especes/dunkleosteus.png"),
        "helicoprion" => include_bytes!("../assets/especes/helicoprion.png"),
        "anomalocaris" => include_bytes!("../assets/especes/anomalocaris.png"),
        "ammonite" => include_bytes!("../assets/especes/ammonite.png"),
        "basilosaurus" => include_bytes!("../assets/especes/basilosaurus.png"),
        "trilobite" => include_bytes!("../assets/especes/trilobite.png"),
        _ => panic!("espèce sans image : {cle}"),
    }
}

/// Pixels fins par pixel du dessin : les créatures colossales sont dessinées en gros pixels
/// (l'image IA ne tient pas assez de cases pour leur taille à l'écran).
fn zoom_espece(cle: &str) -> i32 {
    match cle {
        "orque" | "leviathan" | "grand_abyssal" | "jormungandr" | "megalodon" | "mosasaure"
        | "basilosaurus" => 2,
        _ => 1,
    }
}

/// Taille d'un sprite pixel art (lue dans l'en-tête PNG), en pixels fins.
pub fn dims_png(cle: &str) -> (i32, i32) {
    let b = png_espece(cle);
    let n = |i: usize| u32::from_be_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]) as i32;
    let z = zoom_espece(cle);
    (n(16) * z, n(20) * z)
}

/// Façon de déformer l'unique dessin pour l'animer.
#[derive(Clone, Copy, Debug)]
enum Anim {
    /// Poissons (tournés à droite) : battement de queue qui part du corps, flexion du corps.
    /// `zone` = part de la longueur (depuis la queue) qui bat, `force` multiplie l'amplitude.
    Queue { zone: f32, force: f32 },
    /// Anguilles, serpents : vague sur tout le corps, de la tête vers la queue.
    Ondule { force: f32 },
    /// Raies, tortues vues de dessus : ailes qui battent, la vague court vers le bout.
    Ailes { force: f32 },
    /// Méduses : la cloche se contracte, les filaments suivent en retard.
    Cloche,
    /// Kraken, poulpe, Cthulhu, œil, hydre : bras qui ondulent depuis le centre (cx, cy en
    /// part de la taille), chacun à son rythme ; le manteau respire.
    Bras { cx: f32, cy: f32, force: f32 },
    /// Crabe, isopode : pattes qui trottinent, corps qui sautille.
    Pattes,
    /// Hippocampe : debout, la queue se balance.
    Balance,
}

fn anim_de(e: &Espece, w: f32, h: f32) -> Anim {
    use Mouvement as M;
    match (e.cle, e.mouvement) {
        ("kraken", _) => Anim::Bras {
            cx: 0.5,
            cy: 0.36,
            force: 1.0,
        },
        ("poulpe", _) => Anim::Bras {
            cx: 0.5,
            cy: 0.3,
            force: 2.2,
        },
        ("cthulhu", _) => Anim::Bras {
            cx: 0.5,
            cy: 0.22,
            force: 1.5,
        },
        ("oeil", _) => Anim::Bras {
            cx: 0.5,
            cy: 0.35,
            force: 1.4,
        },
        ("hydre", _) => Anim::Bras {
            cx: 0.5,
            cy: 0.85,
            force: 1.2,
        },
        // Coquille immobile, tentacules (à droite) qui ondulent.
        ("ammonite", _) => Anim::Bras {
            cx: 0.4,
            cy: 0.5,
            force: 2.2,
        },
        ("calamar" | "calamar_geant" | "lusca", _) => Anim::Queue {
            zone: 0.75,
            force: 1.4,
        },
        ("tortue", _) => Anim::Ailes { force: 0.85 },
        ("anomalocaris", _) => Anim::Ailes { force: 0.5 },
        // Vu de dessus : le corps segmenté ondule doucement en marchant.
        ("trilobite", _) => Anim::Ondule { force: 0.35 },
        ("dragon_feuillu", _) => Anim::Ondule { force: 0.5 },
        (_, M::Ondule | M::Traverse) => Anim::Ondule { force: 1.0 },
        (_, M::Plane) => Anim::Ailes { force: 1.0 },
        (_, M::Pulse) => Anim::Cloche,
        (_, M::Marche) => Anim::Pattes,
        (_, M::Debout) if h > w => Anim::Balance,
        (_, M::Abysse | M::Flotte) => Anim::Bras {
            cx: 0.5,
            cy: 0.35,
            force: 1.0,
        },
        (_, M::Lent) => Anim::Queue {
            zone: 0.7,
            force: 1.0,
        },
        _ => Anim::Queue {
            zone: 0.62,
            force: 1.0,
        },
    }
}

fn lisse(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Déplacement (pixels fins) du point (x, y) du dessin à la phase `ph` (0..2π, un cycle).
/// Toutes les vagues sont des multiples entiers de `ph` : le cycle boucle sans saut.
/// Champ lisse (pente < 1 px/px) : l'image tirée à rebours n'a ni déchirure ni trou.
fn deplacement(a: Anim, (w, h): (f32, f32), x: f32, y: f32, ph: f32) -> (f32, f32) {
    use std::f32::consts::PI;
    match a {
        Anim::Queue { zone, force } => {
            // u : 0 au bout de la queue, 1 au museau.
            let u = x / w;
            let amp = (h * 0.14).clamp(1.6, 10.0) * force;
            let kq = ((zone - u) / zone).clamp(0.0, 1.0).powf(1.5);
            // Battement qui part du corps vers la queue (la queue est en retard).
            let queue = amp * kq * (ph - (1.0 - u) * 2.2).sin();
            // Flexion douce de tout le corps (mouvement secondaire, en retard d'un quart).
            let flex = amp * 0.5 * (ph - 0.9 - u * PI).sin() * (0.35 + 0.65 * (1.0 - u));
            // La nageoire caudale se raccourcit quand elle passe de profil.
            let raccourci = amp * 0.5 * kq * kq * (ph - (1.0 - u) * 2.2).cos().abs();
            (raccourci, queue + flex)
        }
        Anim::Ondule { force } => {
            let u = x / w;
            let lambda = (w * 0.55).clamp(40.0, 260.0);
            let amp = (h * 0.16).clamp(2.0, 12.0) * force;
            let env = 0.3 + 0.7 * (1.0 - u).powf(0.8);
            let k = std::f32::consts::TAU / lambda;
            let vague = (ph + x * k).sin() + 0.25 * (2.0 * ph + x * k * 2.0 + 1.0).sin();
            (0.0, amp * env * vague)
        }
        Anim::Ailes { force } => {
            let cy = h * 0.5;
            let t = (y - cy) / cy;
            let ta = t.abs();
            // La vague court du corps vers le bout de l'aile ; le bout trace une boucle.
            let o = ph - ta * 1.5 - (1.0 - x / w) * 0.6;
            let dy = (y - cy) * 0.2 * force * ta * o.sin();
            let dx = w * 0.035 * force * ta * ta * (o - 0.8).cos();
            (dx, dy)
        }
        Anim::Cloche => {
            let cx = w * 0.5;
            let yb = h * 0.45;
            // Contraction rapide, détente lente.
            let p = ph.sin() + 0.25 * (2.0 * ph).sin();
            if y <= yb {
                let v = y / yb;
                ((x - cx) * 0.1 * p * v, -y * 0.07 * p)
            } else {
                let vt = (y - yb) / (h - yb);
                let dx =
                    (x - cx) * 0.1 * p * (1.0 - vt) + w * 0.09 * vt * (ph - 1.6 - vt * 3.0).sin();
                (dx, -yb * 0.07 * p * (1.0 - vt * 0.5))
            }
        }
        Anim::Bras { cx, cy, force } => {
            let (cx, cy) = (w * cx, h * cy);
            let (dx, dy) = (x - cx, y - cy);
            let r = (dx * dx + dy * dy * 1.4).sqrt();
            let rmax = (w * 0.5).max(h - cy).max(cy);
            let r0 = h * 0.24;
            let k = lisse((r - r0) / (rmax - r0));
            let th = dy.atan2(dx);
            // Chaque bras (direction) a son amplitude, son retard et son rythme.
            let amp = (h * 0.05).clamp(3.0, 12.0) * force * (0.75 + 0.25 * (3.0 * th + 0.7).sin());
            let phi = 1.7 * th.sin() + 1.1 * (2.0 * th).cos();
            let rythme = 0.5 + 0.5 * (2.0 * th + 1.3).sin();
            let kk = std::f32::consts::TAU / (0.6 * rmax);
            let vague = (1.0 - rythme) * (ph - r * kk + phi).sin()
                + rythme * (2.0 * ph - r * kk * 1.3 + phi).sin();
            // Ondulation de travers + les bouts qui s'enroulent (tirés vers le centre).
            let (rx, ry) = if r > 0.01 {
                (dx / r, dy / r)
            } else {
                (0.0, 0.0)
            };
            let travers = amp * k * vague;
            let enroule = -amp * 0.45 * k * k * (1.0 + (ph - r * kk + phi + 0.8).cos()) * 0.5;
            let mut ddx = -ry * travers + rx * enroule;
            let mut ddy = rx * travers + ry * enroule;
            // Le corps entier ondule un peu (pas de partie figée).
            ddx += 1.5 * (ph + y * 0.04).sin() * (1.0 - k);
            // Le manteau (haut de la tête) respire doucement.
            if dy < 0.0 {
                ddy += dy * 0.035 * ph.sin() * (1.0 - k);
                ddx += dx * 0.012 * ph.sin() * (1.0 - k);
            }
            (ddx, ddy)
        }
        Anim::Pattes => {
            // Pattes (bas) qui trottinent en alternance, pinces/antennes (haut des côtés) qui
            // se lèvent, corps qui sautille.
            let v = lisse((y / h - 0.45) / 0.3);
            let cote = ((x / w - 0.5).abs() * 2.0 - 0.35).max(0.0) / 0.65;
            let haut = (1.0 - y / h).clamp(0.0, 1.0);
            let dx = 3.6 * v * (2.0 * ph + x * 0.18).sin();
            let dy = 1.4 * (2.0 * ph).sin() * (1.0 - v)
                - 3.0 * cote * haut * (1.0 - v) * (ph + if x < w * 0.5 { 0.0 } else { 1.6 }).sin();
            (dx, dy)
        }
        Anim::Balance => {
            let v = y / h;
            let amp = (w * 0.2).clamp(2.0, 7.0);
            let dx = amp * v.powf(1.5) * (ph - v * 2.5).sin()
                + amp * 0.3 * (2.0 * ph + v * 3.0).sin() * v;
            (dx, 0.0)
        }
    }
}

/// Images par cycle selon la taille (plus de détail = plus d'images pour rester fluide).
pub fn images_par_cycle(t: Taille) -> usize {
    match t {
        Taille::Minuscule | Taille::Petite => 16,
        Taille::Moyenne => 20,
        _ => 24,
    }
}

/// Sprite d'une espèce pixel art : variante de couleur (teinte décalée selon la saturation,
/// les rampes d'ombre restent), réduction éventuelle (vignettes), puis N images déformées
/// à partir de l'unique dessin. Alpha du PNG : 255 corps, 254 œil, 253 lumière.
fn sprite_png(e: &Espece, variante: usize, echelle: f32, png: &[u8], forme: Forme) -> Sprite {
    let img = image::load_from_memory(png)
        .expect("sprite d'espèce embarqué lisible")
        .to_rgba8();
    let (w0, h0) = (img.width() as i32, img.height() as i32);
    let z = zoom_espece(e.cle);
    let (mut w, mut h) = (w0 * z, h0 * z);
    let mut src: Vec<[u8; 4]> = (0..w * h)
        .map(|i| img.get_pixel(((i % w) / z) as u32, ((i / w) / z) as u32).0)
        .collect();
    let v = &e.variantes[variante.min(e.variantes.len() - 1)];
    if v.teinte != 0.0 || v.sat != 1.0 || v.val != 1.0 {
        for p in src.iter_mut().filter(|p| p[3] >= 128) {
            let (hh, s, vv) = crate::toile::rgb_hsv([p[0], p[1], p[2]]);
            // Les gris (blancs, noirs, contour) gardent leur teinte : seules les couleurs tournent.
            let poids = ((s - 0.08) / 0.25).clamp(0.0, 1.0);
            let c = crate::toile::hsv(
                hh + v.teinte * poids,
                (s * v.sat).clamp(0.0, 1.0),
                (vv * v.val).clamp(0.0, 1.0),
            );
            *p = [c[0], c[1], c[2], p[3]];
        }
    }
    // Le contour sombre et les yeux (alpha 254/253) ne sont jamais recolorés : seuls les
    // pixels de corps (alpha 255).
    if forme.shiny {
        recolorer_shiny(&mut src, e.cle);
    }
    if let Some(m) = forme.modif {
        recolorer_modif(&mut src, m, w);
    }
    // Ombre : un peu plus grande (+12 %), agrandie au plus proche (jamais réduite).
    if forme.modif == Some(Modif::Ombre) && echelle >= 0.85 {
        let (nw, nh) = (w * 28 / 25, h * 28 / 25);
        src = (0..nw * nh)
            .map(|i| src[((i / nw) * h / nh * w + (i % nw) * w / nw) as usize])
            .collect();
        (w, h) = (nw, nh);
    }
    // Vignettes : réduction par la couleur la plus fréquente de chaque case (palette gardée).
    // Aux échelles d'individu (±10 %), le dessin reste à sa taille native.
    let k = if echelle < 0.85 { echelle } else { 1.0 };
    if k < 1.0 {
        let (nw, nh) = (
            ((w as f32 * k).round() as i32).max(4),
            ((h as f32 * k).round() as i32).max(4),
        );
        let mut r = vec![[0u8; 4]; (nw * nh) as usize];
        for y in 0..nh {
            for x in 0..nw {
                let (sx0, sx1) = (x * w / nw, ((x + 1) * w / nw).max(x * w / nw + 1));
                let (sy0, sy1) = (y * h / nh, ((y + 1) * h / nh).max(y * h / nh + 1));
                let mut comptes: Vec<([u8; 4], u32)> = Vec::new();
                let (mut plein, mut total) = (0, 0);
                for yy in sy0..sy1 {
                    for xx in sx0..sx1 {
                        let p = src[(yy * w + xx) as usize];
                        total += 1;
                        if p[3] >= 128 {
                            plein += 1;
                            match comptes.iter_mut().find(|c| c.0 == p) {
                                Some(c) => c.1 += 1,
                                None => comptes.push((p, 1)),
                            }
                        }
                    }
                }
                if plein * 2 >= total {
                    r[(y * nw + x) as usize] = comptes.iter().max_by_key(|c| c.1).unwrap().0;
                }
            }
        }
        (src, w, h) = (r, nw, nh);
    }
    let anim = anim_de(e, w as f32, h as f32);
    let n = if k < 1.0 {
        1
    } else {
        images_par_cycle(e.taille)
    };
    // Marge : le plus grand déplacement du cycle (mesuré), plus la marge des halos.
    let mut dmax = 0.0f32;
    for f in 0..n {
        let ph = f as f32 / n as f32 * std::f32::consts::TAU;
        for y in 0..h {
            for x in 0..w {
                let (dx, dy) = deplacement(anim, (w as f32, h as f32), x as f32, y as f32, ph);
                dmax = dmax.max(dx.abs()).max(dy.abs());
            }
        }
    }
    let bord = MARGE + dmax.ceil() as i32 + 2;
    let (l, ht) = (w + 2 * bord, h + 2 * bord);
    // Palette du sprite (le dessin a une quarantaine de couleurs ; variante et réduction n'en
    // ajoutent pas).
    let mut couleurs: Vec<[u8; 4]> = vec![[0; 4]];
    let src_idx: Vec<u8> = src
        .iter()
        .map(|p| {
            if p[3] < 128 {
                return 0;
            }
            let c = [p[0], p[1], p[2], 255];
            match couleurs.iter().position(|&q| q == c) {
                Some(k) => k as u8,
                None if couleurs.len() < 256 => {
                    couleurs.push(c);
                    (couleurs.len() - 1) as u8
                }
                // raccourci : au-delà de 255 couleurs, la plus proche (jamais vu : ≤ 41).
                None => (1..couleurs.len())
                    .min_by_key(|&k| {
                        (0..3)
                            .map(|j| (couleurs[k][j] as i32 - c[j] as i32).pow(2))
                            .sum::<i32>()
                    })
                    .unwrap() as u8,
            }
        })
        .collect();
    let pal = std::sync::Arc::new(couleurs);
    let mut images = Vec::with_capacity(n);
    for f in 0..n {
        let ph = if n == 1 {
            0.0
        } else {
            f as f32 / n as f32 * std::f32::consts::TAU
        };
        let mut px = vec![0u8; (l * ht) as usize];
        let mut cl = vec![VIDE; (l * ht) as usize];
        for y in 0..ht {
            for x in 0..l {
                let (sx, sy) = (x - bord, y - bord);
                let (dx, dy) = if n == 1 {
                    (0.0, 0.0)
                } else {
                    // Champ pris au bord du dessin pour les pixels de la marge (borné).
                    let (fx, fy) = (sx.clamp(0, w - 1) as f32, sy.clamp(0, h - 1) as f32);
                    deplacement(anim, (w as f32, h as f32), fx, fy, ph)
                };
                let (sx, sy) = (sx - dx.round() as i32, sy - dy.round() as i32);
                if sx >= 0 && sy >= 0 && sx < w && sy < h {
                    let p = src[(sy * w + sx) as usize];
                    if p[3] >= 128 {
                        let i = (y * l + x) as usize;
                        px[i] = src_idx[(sy * w + sx) as usize];
                        cl[i] = match p[3] {
                            254 => OEIL,
                            253 => LUMIERE,
                            _ => CORPS,
                        };
                    }
                }
            }
        }
        // Trou d'un pixel laissé par la déformation (deux pixels qui se rejoignent) : bouché
        // avec son voisin de gauche.
        for y in 1..ht - 1 {
            for x in 1..l - 1 {
                let i = (y * l + x) as usize;
                let plein = |j: usize| cl[j] != VIDE;
                if cl[i] == VIDE
                    && plein(i - 1)
                    && plein(i + 1)
                    && plein(i - l as usize)
                    && plein(i + l as usize)
                {
                    px[i] = px[i - 1];
                    cl[i] = if cl[i - 1] == CONTOUR {
                        CORPS
                    } else {
                        cl[i - 1]
                    };
                }
            }
        }
        // Bord de la silhouette = contour (le dessin a déjà son trait sombre).
        let avant = cl.clone();
        for y in 0..ht {
            for x in 0..l {
                let i = (y * l + x) as usize;
                let vide = |xx: i32, yy: i32| {
                    xx < 0 || yy < 0 || xx >= l || yy >= ht || avant[(yy * l + xx) as usize] == VIDE
                };
                if avant[i] == CORPS
                    && (vide(x - 1, y) || vide(x + 1, y) || vide(x, y - 1) || vide(x, y + 1))
                {
                    cl[i] = CONTOUR;
                }
            }
        }
        let (halo1, halo2) = halos(&cl, l, ht);
        images.push(Image {
            idx: px,
            pal: pal.clone(),
            classe: cl,
            halo1,
            halo2,
        });
    }
    // Paupière : couleur moyenne du corps qui touche les yeux, un peu assombrie.
    let im0 = &images[0];
    let (mut somme, mut nb) = ([0u32; 3], 0u32);
    let mut lum = (0i64, 0i64, 0i64);
    for y in 1..ht - 1 {
        for x in 1..l - 1 {
            let i = (y * l + x) as usize;
            if im0.classe[i] == LUMIERE {
                lum = (lum.0 + x as i64, lum.1 + y as i64, lum.2 + 1);
            }
            if im0.classe[i] != CORPS {
                continue;
            }
            let pres = [i - 1, i + 1, i - l as usize, i + l as usize]
                .iter()
                .any(|&j| im0.classe[j] == OEIL);
            if pres {
                for (c, v) in somme.iter_mut().zip(im0.px(i)) {
                    *c += v as u32;
                }
                nb += 1;
            }
        }
    }
    let paupiere = if nb > 0 {
        let m = somme.map(|c| (c / nb) as u8);
        crate::toile::melange(m, [0x10, 0x0c, 0x18], 0.2)
    } else {
        [0x20, 0x18, 0x20]
    };
    let lueur = (lum.2 > 0).then(|| ((lum.0 / lum.2) as i32, (lum.1 / lum.2) as i32));
    // Queue et bouche : pixels du dessin les plus à gauche et à droite (milieu de la colonne) :
    // (la traînée part vraiment du bout de la queue, les bulles de la bouche).
    let bande = |x: i32| {
        let ys: Vec<i32> = (0..ht)
            .filter(|&y| im0.classe[(y * l + x) as usize] != VIDE)
            .collect();
        (!ys.is_empty()).then(|| ys[ys.len() / 2])
    };
    let queue = (0..l)
        .find_map(|x| bande(x).map(|y| (x + 1, y)))
        .unwrap_or((bord, bord + h / 2));
    let bouche = (0..l)
        .rev()
        .find_map(|x| bande(x).map(|y| (x, y)))
        .unwrap_or((bord + w - 1, bord + h / 2));
    let mut etoiles = Vec::new();
    // Étoiles et facettes qui scintillent : Céleste, Galaxie, Diamant, Cristal.
    let scintille = matches!(
        forme.modif,
        Some(Modif::Galaxie | Modif::Diamant | Modif::Cristal)
    );
    if e.rarete == crate::jeu::Rarete::Celeste || scintille {
        let mut a = Alea(e.cle.len() as u64 * 977 + variante as u64);
        for _ in 0..((w + h) / 5).max(4) {
            for _essai in 0..30 {
                let (x, y) = (a.n(l as u32) as i32, a.n(ht as u32) as i32);
                if images[0].classe[(y * l + x) as usize] == CORPS {
                    etoiles.push((x, y, a.f()));
                    break;
                }
            }
        }
    }
    // Luminosité moyenne du corps (contre-jour des dessins sombres, mode Abysses).
    let (mut sl, mut sn) = (0f32, 0f32);
    for (i, &c) in images[0].classe.iter().enumerate() {
        if c == CORPS {
            let p = images[0].px(i);
            sl += p[0] as f32 * 0.3 + p[1] as f32 * 0.59 + p[2] as f32 * 0.11;
            sn += 1.0;
        }
    }
    let sombre = (1.0 - sl / sn.max(1.0) / 255.0 / 0.4).clamp(0.0, 1.0);
    Sprite {
        l,
        h: ht,
        images,
        queue,
        bouche,
        lueur,
        etoiles,
        mouvement: e.mouvement,
        taille: e.taille,
        paupiere,
        sombre,
    }
}

type Points = Vec<(i32, i32)>;

fn halos(cl: &[u8], w: i32, h: i32) -> (Points, Points) {
    let plein =
        |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h && cl[(y * w + x) as usize] != VIDE;
    let mut h1 = Vec::new();
    let mut h2 = Vec::new();
    for y in -2..h + 2 {
        for x in -2..w + 2 {
            if plein(x, y) {
                continue;
            }
            let mut d = 9;
            for dy in -2i32..=2 {
                for dx in -2i32..=2 {
                    if plein(x + dx, y + dy) {
                        d = d.min(
                            dx.abs().max(dy.abs()) + if dx.abs() + dy.abs() == 4 { 1 } else { 0 },
                        );
                    }
                }
            }
            match d {
                1 => h1.push((x, y)),
                2 => h2.push((x, y)),
                _ => {}
            }
        }
    }
    (h1, h2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::especes::catalogue;

    #[test]
    fn sprites_deterministes_et_bornes() {
        for e in catalogue() {
            for var in 0..e.variantes.len() {
                let s = sprite(e, var, 1.0);
                assert_eq!(s.images.len(), images_par_cycle(e.taille), "{}", e.cle);
                for im in &s.images {
                    assert!(
                        im.classe.contains(&CONTOUR) && im.classe.contains(&CORPS),
                        "{} sans corps ou contour",
                        e.cle
                    );
                    // Le contour ne touche aucun bord (marge pour les halos).
                    let bord = |x: i32, y: i32| im.classe[(y * s.l + x) as usize] == VIDE;
                    assert!(
                        (0..s.l).all(|x| bord(x, 0) && bord(x, s.h - 1))
                            && (0..s.h).all(|y| bord(0, y) && bord(s.l - 1, y)),
                        "{} touche le bord",
                        e.cle
                    );
                }
                let s2 = sprite(e, var, 1.0);
                for k in 0..s.images.len() {
                    assert_eq!(s2.images[k].idx, s.images[k].idx, "{} image {k}", e.cle);
                }
            }
        }
    }

    #[test]
    fn sprites_png_sans_trou_ni_dechirure() {
        // Les images déformées gardent la silhouette : ni trou d'un pixel au milieu du corps,
        // ni pixels perdus (aire à ±3 % de l'image d'origine).
        for e in catalogue() {
            let s = sprite(e, 0, 1.0);
            let (w, h) = dims_png(e.cle);
            let img = image::load_from_memory(png_espece(e.cle))
                .unwrap()
                .to_rgba8();
            let z = zoom_espece(e.cle);
            assert_eq!((img.width() as i32 * z, img.height() as i32 * z), (w, h));
            assert_eq!(e.dims, (w, h), "{}", e.cle);
            let aire0 = (img.pixels().filter(|p| p.0[3] >= 128).count() as i32 * z * z) as f32;
            let trous = |cl: &[u8]| {
                let mut n = 0;
                for y in 1..s.h - 1 {
                    for x in 1..s.l - 1 {
                        let v = |xx: i32, yy: i32| cl[(yy * s.l + xx) as usize] == VIDE;
                        if v(x, y) && !v(x - 1, y) && !v(x + 1, y) && !v(x, y - 1) && !v(x, y + 1) {
                            n += 1;
                        }
                    }
                }
                n
            };
            for (k, im) in s.images.iter().enumerate() {
                let aire = im.classe.iter().filter(|&&c| c != VIDE).count() as f32;
                // Les ailes s'étirent et se replient (±15 %) ; une déchirure perdrait bien plus.
                let r = aire / aire0;
                assert!((0.85..1.15).contains(&r), "{} image {k} : aire {r}", e.cle);
                assert_eq!(trous(&im.classe), 0, "{} image {k} : trous", e.cle);
            }
        }
    }
}

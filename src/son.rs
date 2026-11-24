//! Sons du jeu : WAV IMA ADPCM mono 22050 Hz embarqués (assets/sons, faits par
//! tools/sons/convertir.sh), décodés une seule fois, mélangés par `Mixeur`.
//! Windows : sortie waveOut sur un fil à part, ouverte au premier son et refermée après
//! 3 s de silence (une appli au repos ne coûte rien). Ailleurs : `jouer` ne fait rien, mais
//! le même mixeur sert à `rendre_wav` (son des vidéos de capture).

use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, OnceLock};

/// Fréquence d'échantillonnage du mixeur et de la sortie.
const FE: u32 = 22050;
/// Au-delà, la voix la plus ancienne est coupée.
const MAX_VOIX: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Son {
    /// Fêlure de coquille, 3 variantes (0..3).
    Fele(u8),
    Chute,
    /// Palier de lumière de la fêlure (blanc, bleu, violet, or, arc-en-ciel : 0..5).
    Palier(u8),
    Montee,
    Eclat,
    EclatFort,
    /// Jingle de révélation, indice = `Rarete::index()` (0..9).
    Jingle(u8),
    Shiny,
    Modif,
    Piece,
    Clic,
    Carte,
    CarteRare,
    TrouNoir,
    Eclipse,
    Fonte,
    Abysse,
    Oeil,
    Vague,
    Coquillage,
    Remous,
    Tresor,
    Prisme,
}

macro_rules! sons {
    ($($n:literal),* $(,)?) => {
        [$(include_bytes!(concat!("../assets/sons/", $n, ".wav")).as_slice()),*]
    };
}

/// Fichiers embarqués, dans l'ordre de `indice`.
static FICHIERS: [&[u8]; 37] = sons![
    "fele1",
    "fele2",
    "fele3",
    "chute",
    "palier0",
    "palier1",
    "palier2",
    "palier3",
    "palier4",
    "montee",
    "eclat",
    "eclat_fort",
    "jingle0",
    "jingle1",
    "jingle2",
    "jingle3",
    "jingle4",
    "jingle5",
    "jingle6",
    "jingle7",
    "jingle8",
    "shiny",
    "modif",
    "piece",
    "clic",
    "carte",
    "carte_rare",
    "trou_noir",
    "eclipse",
    "fonte",
    "abysse",
    "oeil",
    "vague",
    "coquillage",
    "remous",
    "tresor",
    "prisme",
];

fn indice(s: Son) -> usize {
    match s {
        Son::Fele(n) => n.min(2) as usize,
        Son::Chute => 3,
        Son::Palier(n) => 4 + n.min(4) as usize,
        Son::Montee => 9,
        Son::Eclat => 10,
        Son::EclatFort => 11,
        Son::Jingle(n) => 12 + n.min(8) as usize,
        Son::Shiny => 21,
        Son::Modif => 22,
        Son::Piece => 23,
        Son::Clic => 24,
        Son::Carte => 25,
        Son::CarteRare => 26,
        Son::TrouNoir => 27,
        Son::Eclipse => 28,
        Son::Fonte => 29,
        Son::Abysse => 30,
        Son::Oeil => 31,
        Son::Vague => 32,
        Son::Coquillage => 33,
        Son::Remous => 34,
        Son::Tresor => 35,
        Son::Prisme => 36,
    }
}

/// Son décodé : échantillons mono et leur fréquence.
struct Echantillons {
    donnees: Vec<i16>,
    fe: u32,
}

/// Tous les sons, décodés au premier usage. Un fichier illisible donne un son vide.
fn echantillons(s: Son) -> Arc<Echantillons> {
    static SONS: OnceLock<Vec<Arc<Echantillons>>> = OnceLock::new();
    SONS.get_or_init(|| {
        FICHIERS
            .iter()
            .map(|o| {
                Arc::new(decoder(o).unwrap_or(Echantillons {
                    donnees: Vec::new(),
                    fe: FE,
                }))
            })
            .collect()
    })[indice(s)]
    .clone()
}

fn u16_le(o: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_le_bytes(o.get(i..i + 2)?.try_into().ok()?))
}

fn u32_le(o: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_le_bytes(o.get(i..i + 4)?.try_into().ok()?))
}

/// WAV mono : PCM 16 bits ou IMA ADPCM (0x11). Autre chose : `None`.
fn decoder(o: &[u8]) -> Option<Echantillons> {
    if o.get(0..4)? != b"RIFF" || o.get(8..12)? != b"WAVE" {
        return None;
    }
    let (mut fmt, mut data) = (None, None);
    let mut i = 12;
    while i + 8 <= o.len() {
        let taille = u32_le(o, i + 4)? as usize;
        let corps = o.get(i + 8..(i + 8).saturating_add(taille).min(o.len()))?;
        match &o[i..i + 4] {
            b"fmt " => fmt = Some(corps),
            b"data" => data = Some(corps),
            _ => {}
        }
        i = i.saturating_add(8 + taille + (taille & 1));
    }
    let (fmt, data) = (fmt?, data?);
    let (format, canaux, fe) = (u16_le(fmt, 0)?, u16_le(fmt, 2)?, u32_le(fmt, 4)?);
    let (bloc, bits) = (u16_le(fmt, 12)? as usize, u16_le(fmt, 14)?);
    if canaux != 1 || fe == 0 {
        return None;
    }
    let donnees = match (format, bits) {
        (1, 16) => data
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&c| i16::from_le_bytes(c))
            .collect(),
        (0x11, 4) if bloc > 4 => data.chunks(bloc).flat_map(bloc_ima).collect(),
        _ => return None,
    };
    Some(Echantillons { donnees, fe })
}

const PAS_IMA: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66,
    73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449,
    494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272,
    2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493,
    10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767,
];
const INDICE_IMA: [i32; 8] = [-1, -1, -1, -1, 2, 4, 6, 8];

/// Un bloc IMA ADPCM mono : en-tête (échantillon i16, indice de pas) puis 2 quartets par octet,
/// le bas d'abord. Un bloc tronqué (fin de fichier) donne ce qu'il contient.
fn bloc_ima(b: &[u8]) -> Vec<i16> {
    if b.len() < 4 {
        return Vec::new();
    }
    let mut pred = i16::from_le_bytes([b[0], b[1]]) as i32;
    let mut idx = (b[2] as i32).clamp(0, 88);
    let mut sortie = Vec::with_capacity(1 + (b.len() - 4) * 2);
    sortie.push(pred as i16);
    for &octet in &b[4..] {
        for q in [octet & 0x0f, octet >> 4] {
            let pas = PAS_IMA[idx as usize];
            let mut d = pas >> 3;
            if q & 4 != 0 {
                d += pas;
            }
            if q & 2 != 0 {
                d += pas >> 1;
            }
            if q & 1 != 0 {
                d += pas >> 2;
            }
            pred = if q & 8 != 0 { pred - d } else { pred + d }.clamp(-32768, 32767);
            idx = (idx + INDICE_IMA[(q & 7) as usize]).clamp(0, 88);
            sortie.push(pred as i16);
        }
    }
    sortie
}

struct Voix {
    son: Arc<Echantillons>,
    pos: f64,
    pas: f64,
}

/// Mélangeur pur : voix lues à leur vitesse (interpolation linéaire), somme, volume,
/// limiteur doux, i16.
#[derive(Default)]
pub struct Mixeur {
    voix: Vec<Voix>,
}

impl Mixeur {
    pub fn ajouter(&mut self, s: Son, hauteur: f32) {
        self.ajouter_echantillons(echantillons(s), hauteur);
    }

    fn ajouter_echantillons(&mut self, son: Arc<Echantillons>, hauteur: f32) {
        let h = if hauteur.is_finite() {
            hauteur.clamp(0.25, 4.0)
        } else {
            1.0
        };
        if son.donnees.is_empty() {
            return;
        }
        if self.voix.len() >= MAX_VOIX {
            self.voix.remove(0);
        }
        let pas = h as f64 * son.fe as f64 / FE as f64;
        self.voix.push(Voix { son, pos: 0.0, pas });
    }

    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn actif(&self) -> bool {
        !self.voix.is_empty()
    }

    /// Remplit `sortie` (écrasée) ; les voix finies sont retirées.
    pub fn melanger(&mut self, sortie: &mut [i16], volume: f32) {
        let v = if volume.is_finite() {
            volume.clamp(0.0, 1.0)
        } else {
            0.0
        };
        for e in sortie.iter_mut() {
            let mut somme = 0.0f32;
            for voix in &mut self.voix {
                let d = &voix.son.donnees;
                let i = voix.pos as usize;
                if i < d.len() {
                    let a = d[i] as f32;
                    let b = d.get(i + 1).map_or(0.0, |&x| x as f32);
                    let t = (voix.pos - i as f64) as f32;
                    somme += a + (b - a) * t;
                    voix.pos += voix.pas;
                }
            }
            *e = (limiter(somme / 32768.0 * v) * 32767.0) as i16;
        }
        self.voix
            .retain(|voix| (voix.pos as usize) < voix.son.donnees.len());
    }
}

/// Linéaire jusqu'à 0,75, puis courbe douce qui n'atteint jamais 1.
fn limiter(x: f32) -> f32 {
    let a = x.abs();
    if a <= 0.75 {
        x
    } else {
        (0.75 + 0.25 * ((a - 0.75) / 0.25).tanh()).copysign(x)
    }
}

static VOLUME: AtomicU32 = AtomicU32::new(0x3f80_0000); // 1.0
static MUET: AtomicBool = AtomicBool::new(false);

/// Volume général (0..1) et coupure, pris en compte au tampon suivant (~30 ms).
pub fn regler(volume: f32, muet: bool) {
    let v = if volume.is_finite() {
        volume.clamp(0.0, 1.0)
    } else {
        1.0
    };
    VOLUME.store(v.to_bits(), Ordering::Relaxed);
    MUET.store(muet, Ordering::Relaxed);
}

#[cfg_attr(not(windows), allow(dead_code))]
fn volume_effectif() -> f32 {
    if MUET.load(Ordering::Relaxed) {
        0.0
    } else {
        f32::from_bits(VOLUME.load(Ordering::Relaxed))
    }
}

/// Joue un son (hauteur = vitesse de lecture, 1.0 normale). Ne bloque pas, n'échoue jamais.
#[cfg(windows)]
pub fn jouer(s: Son, hauteur: f32) {
    sortie::envoyer(s, hauteur);
}

#[cfg(not(windows))]
pub fn jouer(_s: Son, _hauteur: f32) {}

/// Mélange hors ligne (instant en s, son, vitesse) en WAV PCM 16 bits mono 22050 Hz.
pub fn rendre_wav(
    evenements: &[(f64, Son, f32)],
    duree: f64,
    volume: f32,
    chemin: &Path,
) -> std::io::Result<()> {
    let n = (duree.max(0.0) * FE as f64) as usize;
    let mut ev: Vec<_> = evenements.to_vec();
    ev.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut mix = Mixeur::default();
    let mut ech = vec![0i16; n];
    let mut fait = 0;
    for (t, s, h) in ev {
        let debut = ((t.max(0.0) * FE as f64) as usize).min(n);
        mix.melanger(&mut ech[fait..debut], volume);
        fait = debut;
        mix.ajouter(s, h);
    }
    mix.melanger(&mut ech[fait..], volume);

    let octets = (n * 2) as u32;
    let mut o = Vec::with_capacity(44 + n * 2);
    o.extend_from_slice(b"RIFF");
    o.extend_from_slice(&(36 + octets).to_le_bytes());
    o.extend_from_slice(b"WAVEfmt ");
    o.extend_from_slice(&16u32.to_le_bytes());
    o.extend_from_slice(&1u16.to_le_bytes()); // PCM
    o.extend_from_slice(&1u16.to_le_bytes()); // mono
    o.extend_from_slice(&FE.to_le_bytes());
    o.extend_from_slice(&(FE * 2).to_le_bytes());
    o.extend_from_slice(&2u16.to_le_bytes());
    o.extend_from_slice(&16u16.to_le_bytes());
    o.extend_from_slice(b"data");
    o.extend_from_slice(&octets.to_le_bytes());
    for e in ech {
        o.extend_from_slice(&e.to_le_bytes());
    }
    std::fs::File::create(chemin)?.write_all(&o)
}

/// Sortie waveOut : un fil qui reçoit les sons par canal et remplit 4 tampons de ~30 ms.
#[cfg(windows)]
mod sortie {
    use super::{volume_effectif, Mixeur, Son, FE};
    use std::ptr::{null, null_mut};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::sync::OnceLock;
    use std::time::{Duration, Instant};
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Media::Audio::*;
    use windows_sys::Win32::System::Threading::{CreateEventW, WaitForSingleObject};

    const NB_TAMPONS: usize = 4;
    const TAILLE: usize = FE as usize * 30 / 1000;
    const TAILLE_TETE: u32 = std::mem::size_of::<WAVEHDR>() as u32;

    static CANAL: OnceLock<Option<Sender<(Son, f32)>>> = OnceLock::new();
    static SIGNALE: AtomicBool = AtomicBool::new(false);

    /// Une seule ligne de journal par session, quoi qu'il arrive ensuite.
    fn signaler(m: &str) {
        if !SIGNALE.swap(true, Ordering::Relaxed) {
            crate::journal::ecrire(m);
        }
    }

    pub fn envoyer(s: Son, hauteur: f32) {
        let canal = CANAL.get_or_init(|| {
            let (tx, rx) = channel();
            match std::thread::Builder::new()
                .name("son".into())
                .spawn(move || boucle(rx))
            {
                Ok(_) => Some(tx),
                Err(_) => {
                    signaler("son : fil audio impossible");
                    None
                }
            }
        });
        if let Some(tx) = canal {
            let _ = tx.send((s, hauteur));
        }
    }

    /// En-tête et données d'un tampon : en boîte, adresses fixes tant que le pilote les tient.
    struct Tampon {
        tete: Box<WAVEHDR>,
        donnees: Box<[i16; TAILLE]>,
        en_file: bool,
    }

    struct Peripherique {
        hwo: HWAVEOUT,
        evt: HANDLE,
        tampons: Vec<Tampon>,
    }

    impl Peripherique {
        fn ouvrir() -> Option<Peripherique> {
            // SAFETY : appels waveOut documentés ; en-têtes et tampons en boîte, préparés une
            // fois, gardés jusqu'à `drop` qui réinitialise la sortie avant de les libérer.
            unsafe {
                let evt = CreateEventW(null(), 0, 0, null());
                if evt.is_null() {
                    return None;
                }
                let fmt = WAVEFORMATEX {
                    wFormatTag: WAVE_FORMAT_PCM as u16,
                    nChannels: 1,
                    nSamplesPerSec: FE,
                    nAvgBytesPerSec: FE * 2,
                    nBlockAlign: 2,
                    wBitsPerSample: 16,
                    cbSize: 0,
                };
                let mut hwo: HWAVEOUT = null_mut();
                if waveOutOpen(&mut hwo, WAVE_MAPPER, &fmt, evt as usize, 0, CALLBACK_EVENT) != 0 {
                    CloseHandle(evt);
                    return None;
                }
                let mut p = Peripherique {
                    hwo,
                    evt,
                    tampons: Vec::with_capacity(NB_TAMPONS),
                };
                for _ in 0..NB_TAMPONS {
                    let mut donnees = Box::new([0i16; TAILLE]);
                    let mut tete = Box::new(WAVEHDR {
                        lpData: donnees.as_mut_ptr() as *mut u8,
                        dwBufferLength: (TAILLE * 2) as u32,
                        ..Default::default()
                    });
                    if waveOutPrepareHeader(hwo, &mut *tete, TAILLE_TETE) != 0 {
                        return None; // `drop` referme ce qui est déjà ouvert
                    }
                    p.tampons.push(Tampon {
                        tete,
                        donnees,
                        en_file: false,
                    });
                }
                Some(p)
            }
        }

        /// Remplit et envoie chaque tampon libre (jamais envoyé ou rendu par le pilote).
        fn remplir(&mut self, mix: &mut Mixeur) {
            let volume = volume_effectif();
            for t in &mut self.tampons {
                // SAFETY : drapeaux écrits par le pilote ; WAVEHDR est compacté (packed) : lecture
                // non alignée par pointeur brut.
                let drapeaux = unsafe {
                    std::ptr::addr_of!((*(&*t.tete as *const WAVEHDR)).dwFlags).read_unaligned()
                };
                let fait = drapeaux & WHDR_DONE != 0;
                if t.en_file && !fait {
                    continue;
                }
                mix.melanger(&mut t.donnees[..], volume);
                // SAFETY : en-tête préparé sur cette sortie, rendu par le pilote (ou jamais envoyé).
                t.en_file = unsafe { waveOutWrite(self.hwo, &mut *t.tete, TAILLE_TETE) } == 0;
            }
        }
    }

    impl Drop for Peripherique {
        fn drop(&mut self) {
            // SAFETY : la réinitialisation rend tous les tampons avant leur libération.
            unsafe {
                waveOutReset(self.hwo);
                for t in &mut self.tampons {
                    waveOutUnprepareHeader(self.hwo, &mut *t.tete, TAILLE_TETE);
                }
                waveOutClose(self.hwo);
                CloseHandle(self.evt);
            }
        }
    }

    fn boucle(rx: Receiver<(Son, f32)>) {
        let mut mix = Mixeur::default();
        let mut sortie: Option<Peripherique> = None;
        let mut dernier_son = Instant::now();
        loop {
            if sortie.is_none() {
                // Sortie fermée : on dort jusqu'au prochain son.
                let Ok((s, h)) = rx.recv() else { return };
                mix.ajouter(s, h);
                if !mix.actif() {
                    continue;
                }
                sortie = Peripherique::ouvrir();
                if sortie.is_none() {
                    signaler("son : sortie audio impossible à ouvrir (waveOutOpen)");
                    mix = Mixeur::default();
                    continue;
                }
                dernier_son = Instant::now();
            }
            while let Ok((s, h)) = rx.try_recv() {
                mix.ajouter(s, h);
            }
            let Some(p) = sortie.as_mut() else { continue };
            p.remplir(&mut mix);
            if mix.actif() {
                dernier_son = Instant::now();
            } else if dernier_son.elapsed() > Duration::from_secs(3) {
                sortie = None; // `drop` referme la sortie
                continue;
            }
            // SAFETY : évènement valide tant que la sortie est ouverte.
            unsafe { WaitForSingleObject(p.evt, 50) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tous() -> Vec<Son> {
        let mut v: Vec<Son> = (0..3).map(Son::Fele).collect();
        v.extend((0..5).map(Son::Palier));
        v.extend((0..9).map(Son::Jingle));
        v.extend([
            Son::Chute,
            Son::Montee,
            Son::Eclat,
            Son::EclatFort,
            Son::Shiny,
            Son::Modif,
            Son::Piece,
            Son::Clic,
            Son::Carte,
            Son::CarteRare,
            Son::TrouNoir,
            Son::Eclipse,
            Son::Fonte,
            Son::Abysse,
            Son::Oeil,
            Son::Vague,
            Son::Coquillage,
            Son::Remous,
            Son::Tresor,
            Son::Prisme,
        ]);
        v
    }

    #[test]
    fn chaque_son_se_decode() {
        let v = tous();
        let indices: std::collections::HashSet<_> = v.iter().map(|&s| indice(s)).collect();
        assert_eq!(indices.len(), FICHIERS.len(), "un fichier par son");
        for s in v {
            let e = echantillons(s);
            let duree = e.donnees.len() as f32 / e.fe as f32;
            assert_eq!(e.fe, FE, "{s:?}");
            assert!((0.04..4.0).contains(&duree), "{s:?} : {duree} s");
            assert!(
                e.donnees.iter().any(|&x| x.unsigned_abs() > 1000),
                "{s:?} muet"
            );
        }
    }

    /// WAV IMA ADPCM minimal : en-tête RIFF + fmt 0x11 + data.
    fn wav_ima(bloc: &[u8]) -> Vec<u8> {
        let mut o = b"RIFF\0\0\0\0WAVEfmt ".to_vec();
        o.extend_from_slice(&20u32.to_le_bytes());
        for v in [0x11u16, 1] {
            o.extend_from_slice(&v.to_le_bytes());
        }
        o.extend_from_slice(&FE.to_le_bytes());
        o.extend_from_slice(&FE.to_le_bytes());
        for v in [bloc.len() as u16, 4, 2, 9] {
            o.extend_from_slice(&v.to_le_bytes());
        }
        o.extend_from_slice(b"data");
        o.extend_from_slice(&(bloc.len() as u32).to_le_bytes());
        o.extend_from_slice(bloc);
        o
    }

    #[test]
    fn decodage_ima_connu() {
        // Prédicteur 100, indice 0 ; quartets 0, 7, puis des 0.
        let e = decoder(&wav_ima(&[100, 0, 0, 0, 0x70, 0, 0, 0])).unwrap();
        assert_eq!(e.donnees, [100, 100, 111, 113, 114, 115, 116, 117, 118]);
        assert!(decoder(b"RIFF n'importe quoi").is_none());
        assert!(decoder(&[]).is_none());
    }

    fn synthetique(n: usize, valeur: i16) -> Arc<Echantillons> {
        Arc::new(Echantillons {
            donnees: vec![valeur; n],
            fe: FE,
        })
    }

    #[test]
    fn mixeur_limite_et_volume() {
        let mut m = Mixeur::default();
        for _ in 0..20 {
            m.ajouter_echantillons(synthetique(500, 30000), 1.0);
        }
        assert_eq!(m.voix.len(), MAX_VOIX);
        let mut s = [0i16; 100];
        m.melanger(&mut s, 1.0);
        // Somme ~15x la pleine échelle : saturée sans repliement de signe.
        assert!(s.iter().all(|&x| x > 24000));
        m.melanger(&mut s, 0.0);
        assert!(s.iter().all(|&x| x == 0));
        let mut m = Mixeur::default();
        m.ajouter_echantillons(synthetique(500, 10000), 1.0);
        m.melanger(&mut s, 0.5);
        assert!(s.iter().all(|&x| (x - 5000).abs() <= 1));
        regler(0.5, true);
        assert_eq!(volume_effectif(), 0.0);
        regler(0.5, false);
        assert_eq!(volume_effectif(), 0.5);
        regler(1.0, false);
    }

    #[test]
    fn vitesse_double_moitie_de_duree() {
        let mut m = Mixeur::default();
        m.ajouter_echantillons(synthetique(1000, 1000), 2.0);
        let mut n = 0;
        let mut s = [0i16; 1];
        while m.actif() {
            m.melanger(&mut s, 1.0);
            n += 1;
        }
        assert_eq!(n, 500);
    }

    #[test]
    fn rendu_wav_valide() {
        let dossier = std::env::temp_dir().join(format!("aquarium_son_{}", std::process::id()));
        std::fs::create_dir_all(&dossier).unwrap();
        let f = dossier.join("essai.wav");
        rendre_wav(
            &[(0.5, Son::Clic, 1.0), (0.0, Son::Piece, 1.0)],
            2.0,
            0.8,
            &f,
        )
        .unwrap();
        let o = std::fs::read(&f).unwrap();
        assert_eq!(o.len(), 44 + 2 * 44100);
        let e = decoder(&o).unwrap();
        assert_eq!(e.donnees.len(), 44100);
        assert!(e.donnees[..2000].iter().any(|&x| x != 0));
        assert!(e.donnees[40000..].iter().all(|&x| x == 0));
        let _ = std::fs::remove_file(&f);
    }
}

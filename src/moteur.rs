//! Windows (v0.12) : le fond d'écran tourne dans notre propre boucle, sans eframe ni OpenGL
//! (avant : une fenêtre eframe cachée de 1 px redessinée à chaque image pour garder la boucle
//! en vie). Le fil principal fait les images du fond (`Appli::pas`) puis dort jusqu'à la
//! suivante, ou jusqu'à une commande de l'icône / du raccourci.
//! L'Écloserie s'ouvre dans un fil à elle : `eframe::run_native` crée la fenêtre et son
//! contexte OpenGL, et revient quand on la ferme (fenêtre, contexte et mémoire d'egui
//! détruits) ; winit ne permet qu'une boucle d'événements par processus, eframe la garde dans
//! ce fil et la relance à l'ouverture suivante (`run_app_on_demand`). Fermée, rien ne tourne :
//! le fil attend sur un canal.

use crate::appli::{ips_de, Appli};
use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

/// Contexte egui de l'Écloserie ouverte (premier plan, repeint après une mise à jour).
static CTX: Mutex<Option<egui::Context>> = Mutex::new(None);
/// Le fil de l'Écloserie a une fenêtre ouverte (ou en cours d'ouverture).
static OUVERTE: AtomicBool = AtomicBool::new(false);
/// Une commande ou la fermeture de l'Écloserie attend la boucle principale.
static REVEILLE: AtomicBool = AtomicBool::new(false);

fn verrou(a: &Mutex<Appli>) -> MutexGuard<'_, Appli> {
    a.lock().unwrap_or_else(|e| e.into_inner())
}

fn ctx_ecloserie() -> Option<egui::Context> {
    CTX.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

pub fn lancer(mut appli: Appli) -> Result<(), String> {
    let principal = std::thread::current();
    let reveil = move || {
        REVEILLE.store(true, Ordering::Release);
        principal.unpark();
    };
    let r = &appli.partie.reglages;
    let voulu = (r.mode_fenetre, r.incrustation);
    match crate::windows::Systeme::demarrer(voulu.0, voulu.1, reveil.clone()) {
        Ok(s) => {
            appli.systeme = Some(s);
            appli.affichage_applique = Some(voulu);
        }
        Err(e) => {
            crate::journal::ecrire(&format!("démarrage impossible : {e}"));
            return Err(e);
        }
    }
    appli.ecloserie.maj = Some(crate::maj::Maj::new(|| {
        if let Some(c) = ctx_ecloserie() {
            c.request_repaint();
        }
    }));
    crate::journal::ecrire("v0.12 : boucle du fond sans eframe, Écloserie à la demande");
    let etat = Arc::new(Mutex::new(appli));
    let (ouvrir, demandes) = mpsc::channel::<()>();
    {
        let etat = etat.clone();
        let reveil = reveil.clone();
        std::thread::Builder::new()
            .name("ecloserie".into())
            .spawn(move || fil_ecloserie(etat, demandes, reveil))
            .map_err(|e| format!("fil de l'Écloserie : {e}"))?;
    }
    loop {
        let debut = Instant::now();
        let (attente, focus) = {
            let mut a = verrou(&etat);
            let attente = a.pas(None, (64, 64));
            if a.quitter {
                break;
            }
            if a.ecloserie_ouverte && !OUVERTE.swap(true, Ordering::AcqRel) {
                a.focus_ecloserie = false;
                let _ = ouvrir.send(());
            }
            (attente, std::mem::take(&mut a.focus_ecloserie))
        };
        // Hors du verrou : le fil de l'Écloserie peut l'attendre pendant ce temps.
        if focus {
            if let Some(c) = ctx_ecloserie() {
                c.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
        }
        // Attente jusqu'à l'image suivante : courte = sommeil précis (minuterie haute
        // résolution de std) ; longue (tout est en pause) = réveillé par une commande.
        let fin = debut + attente;
        if attente > Duration::from_millis(100) {
            while !REVEILLE.swap(false, Ordering::AcqRel) {
                let n = Instant::now();
                if n >= fin {
                    break;
                }
                std::thread::park_timeout(fin - n);
            }
        } else {
            REVEILLE.store(false, Ordering::Release);
            let n = Instant::now();
            if fin > n {
                std::thread::sleep(fin - n);
            }
        }
    }
    // Quitter : l'Écloserie se ferme avec le processus (la partie est sauvée ici).
    if let Some(c) = ctx_ecloserie() {
        c.send_viewport_cmd(egui::ViewportCommand::Close);
    }
    verrou(&etat).fermer();
    Ok(())
}

/// Fil de l'Écloserie : à chaque demande, une fenêtre eframe qui vit jusqu'à sa fermeture.
fn fil_ecloserie(
    etat: Arc<Mutex<Appli>>,
    demandes: mpsc::Receiver<()>,
    reveil: impl Fn() + Clone + Send + 'static,
) {
    while demandes.recv().is_ok() {
        let icone = Arc::new(crate::appli::icone());
        let titre = crate::i18n::tr("Écloserie");
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title(titre)
                .with_app_id("aquarium-ecloserie")
                .with_inner_size([960.0, 720.0])
                .with_min_inner_size([640.0, 520.0])
                .with_icon(icone),
            renderer: eframe::Renderer::Glow,
            // La boucle d'événements vit dans ce fil, pas dans le fil principal.
            event_loop_builder: Some(Box::new(|b| {
                use winit::platform::windows::EventLoopBuilderExtWindows;
                b.with_any_thread(true);
            })),
            ..Default::default()
        };
        let e = etat.clone();
        crate::journal::ecrire("Écloserie ouverte");
        let r = eframe::run_native(
            "aquarium-ecloserie",
            options,
            Box::new(move |cc| {
                crate::ecloserie::installer_style(&cc.egui_ctx);
                *CTX.lock().unwrap_or_else(|e| e.into_inner()) = Some(cc.egui_ctx.clone());
                Ok(Box::new(FenetreEcloserie { etat: e }))
            }),
        );
        *CTX.lock().unwrap_or_else(|e| e.into_inner()) = None;
        if let Err(e) = &r {
            crate::journal::ecrire(&format!("Écloserie : fenêtre / OpenGL : {e}"));
        }
        {
            let mut a = verrou(&etat);
            a.ecloserie_ouverte = false;
            a.ecloserie.liberer();
            if let Err(e) = r {
                a.ecloserie.bandeau = Some(format!("fenêtre : {e}"));
            }
            a.sauver();
        }
        // Le tas rend au système la mémoire libérée par egui et OpenGL.
        unsafe {
            use windows_sys::Win32::System::Memory::{GetProcessHeap, HeapCompact};
            HeapCompact(GetProcessHeap(), 0);
        }
        crate::journal::ecrire("Écloserie fermée (fenêtre, OpenGL et images libérés)");
        OUVERTE.store(false, Ordering::Release);
        reveil();
    }
}

struct FenetreEcloserie {
    etat: Arc<Mutex<Appli>>,
}

impl eframe::App for FenetreEcloserie {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut g = verrou(&self.etat);
        let a = &mut *g;
        a.ecloserie.ui(ui, &mut a.partie);
        // L'argent et les animations bougent : repeinte au rythme du fond.
        let ips = ips_de(a.partie.reglages.ips);
        if a.quitter {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        drop(g);
        ui.ctx()
            .request_repaint_after(Duration::from_secs_f32(1.0 / ips as f32));
    }
}

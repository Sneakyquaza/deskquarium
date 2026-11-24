//! Partie Windows : incrustation derrière les icônes (WorkerW), fond d'écran actuel,
//! icône de notification + menu, raccourci global Ctrl+Alt+A, plein écran, démarrage
//! avec Windows (HKCU\…\Run), rafraîchissement du fond d'écran à la fermeture.
//! JAMAIS testé sur un vrai Windows (serveur Linux) : vérifié par compilation croisée.

use crate::appli::Commande;
use eframe::egui;
use std::ffi::c_void;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering};
use std::sync::{mpsc, Mutex, OnceLock};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, S_OK, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    GetMonitorInfoW, GetSysColor, MapWindowPoints, MonitorFromWindow, COLOR_DESKTOP, MONITORINFO,
    MONITOR_DEFAULTTONEAREST,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegGetValueW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ, RRF_RT_REG_SZ,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
};
use windows_sys::Win32::UI::Shell::{
    SHQueryUserNotificationState, Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD,
    NIM_DELETE, NOTIFYICONDATAW, QUNS_BUSY, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN,
};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

fn large(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

pub fn alerte(m: &str) {
    unsafe {
        MessageBoxW(
            null_mut(),
            large(m).as_ptr(),
            large("Aquarium").as_ptr(),
            MB_OK | MB_ICONWARNING,
        );
    }
}

static CANAL: OnceLock<Mutex<mpsc::Sender<Commande>>> = OnceLock::new();
static CTX: OnceLock<egui::Context> = OnceLock::new();
static EN_PAUSE: AtomicBool = AtomicBool::new(false);
static FENETRE_SYS: AtomicPtr<c_void> = AtomicPtr::new(null_mut());
static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);
const WM_ICONE: u32 = WM_APP + 1;

pub struct Systeme {
    recu: mpsc::Receiver<Commande>,
    fil: Option<std::thread::JoinHandle<()>>,
}

impl Systeme {
    /// Incruste la fenêtre racine derrière les icônes et lance le fil de l'icône de notification.
    pub fn demarrer(frame: &eframe::Frame, ctx: &egui::Context) -> Result<Systeme, String> {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let h = frame
            .window_handle()
            .map_err(|e| format!("fenêtre : {e}"))?;
        let RawWindowHandle::Win32(w) = h.as_raw() else {
            return Err("fenêtre : pas une fenêtre Win32".into());
        };
        incruster(w.hwnd.get() as HWND)?;
        let (tx, rx) = mpsc::channel();
        let _ = CANAL.set(Mutex::new(tx));
        let _ = CTX.set(ctx.clone());
        let fil = std::thread::spawn(fil_systeme);
        Ok(Systeme {
            recu: rx,
            fil: Some(fil),
        })
    }

    pub fn commandes(&self) -> Vec<Commande> {
        self.recu.try_iter().collect()
    }

    pub fn signaler_pause(&self, p: bool) {
        EN_PAUSE.store(p, Ordering::Relaxed);
    }

    /// Retire l'icône, attend le fil, puis redessine le fond d'écran (pas d'image figée).
    pub fn arreter(mut self) {
        let h = FENETRE_SYS.load(Ordering::Acquire);
        if !h.is_null() {
            unsafe {
                SendMessageW(h, WM_CLOSE, 0, 0);
            }
        }
        if let Some(f) = self.fil.take() {
            let _ = f.join();
        }
        rafraichir_fond();
    }
}

/// Technique WorkerW (comme Wallpaper Engine / Lively) : le message 0x052C à Progman crée
/// un WorkerW derrière les icônes ; notre fenêtre devient son enfant.
fn incruster(hwnd: HWND) -> Result<(), String> {
    unsafe {
        let progman = FindWindowW(large("Progman").as_ptr(), null());
        if progman.is_null() {
            return Err(
                "Bureau Windows introuvable (Progman). L'Explorateur est-il lancé ?".into(),
            );
        }
        let mut res = 0usize;
        SendMessageTimeoutW(progman, 0x052C, 0xD, 0x1, SMTO_NORMAL, 1000, &mut res);
        SendMessageTimeoutW(progman, 0x052C, 0, 0, SMTO_NORMAL, 1000, &mut res);
        let mut worker: HWND = null_mut();
        EnumWindows(Some(chercher_worker), &mut worker as *mut HWND as LPARAM);
        if worker.is_null() {
            // Windows 11 24H2 : le WorkerW est un enfant de Progman.
            worker = FindWindowExW(progman, null_mut(), large("WorkerW").as_ptr(), null());
        }
        if worker.is_null() {
            return Err("Calque du fond d'écran introuvable (WorkerW).".into());
        }
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        let style =
            (style & !(WS_POPUP | WS_CAPTION | WS_THICKFRAME | WS_SYSMENU)) | WS_CHILD | WS_VISIBLE;
        SetWindowLongPtrW(hwnd, GWL_STYLE, style as isize);
        SetParent(hwnd, worker);
        // Écran principal : (0,0)-(cx,cy) en coordonnées écran, ramené dans le repère du WorkerW.
        let mut pt = POINT { x: 0, y: 0 };
        MapWindowPoints(null_mut(), worker, &mut pt, 1);
        let (cx, cy) = (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN));
        SetWindowPos(
            hwnd,
            null_mut(),
            pt.x,
            pt.y,
            cx,
            cy,
            SWP_NOZORDER | SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED,
        );
    }
    Ok(())
}

unsafe extern "system" fn chercher_worker(h: HWND, l: LPARAM) -> i32 {
    unsafe {
        let vue = FindWindowExW(h, null_mut(), large("SHELLDLL_DefView").as_ptr(), null());
        if !vue.is_null() {
            let w = FindWindowExW(null_mut(), h, large("WorkerW").as_ptr(), null());
            if !w.is_null() {
                *(l as *mut HWND) = w;
                return 0;
            }
        }
    }
    1
}

fn chemin_fond() -> Vec<u16> {
    let mut buf = vec![0u16; 1024];
    unsafe {
        SystemParametersInfoW(
            SPI_GETDESKWALLPAPER,
            buf.len() as u32,
            buf.as_mut_ptr() as *mut c_void,
            0,
        );
    }
    let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    buf.truncate(n);
    buf
}

fn lire_registre(cle: &str, valeur: &str) -> Option<String> {
    let mut buf = vec![0u16; 260];
    let mut taille = (buf.len() * 2) as u32;
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            large(cle).as_ptr(),
            large(valeur).as_ptr(),
            RRF_RT_REG_SZ,
            null_mut(),
            buf.as_mut_ptr() as *mut c_void,
            &mut taille,
        )
    };
    (r == 0).then(|| String::from_utf16_lossy(&buf[..(taille as usize / 2).saturating_sub(1)]))
}

/// Fond d'écran actuel à la taille de l'écran (Remplir, Ajuster, Étirer, Centrer ;
/// mosaïque et étendu traités comme Remplir), ou la couleur unie du bureau.
pub fn fond_bureau(l: u32, h: u32) -> Option<image::RgbaImage> {
    let c = unsafe { GetSysColor(COLOR_DESKTOP) };
    let uni = image::Rgba([
        (c & 0xff) as u8,
        (c >> 8 & 0xff) as u8,
        (c >> 16 & 0xff) as u8,
        255,
    ]);
    let chemin = String::from_utf16_lossy(&chemin_fond());
    if chemin.is_empty() {
        return Some(image::RgbaImage::from_pixel(l, h, uni));
    }
    let img = image::open(&chemin).ok()?.to_rgba8();
    let style = lire_registre("Control Panel\\Desktop", "WallpaperStyle").unwrap_or_default();
    let mosaique = lire_registre("Control Panel\\Desktop", "TileWallpaper").as_deref() == Some("1");
    Some(match (style.trim(), mosaique) {
        ("2", _) => image::imageops::resize(&img, l, h, image::imageops::FilterType::Triangle),
        ("6", _) | ("0", false) => {
            let e = if style.trim() == "6" {
                (l as f32 / img.width() as f32).min(h as f32 / img.height() as f32)
            } else {
                1.0
            };
            let (nl, nh) = (
                ((img.width() as f32 * e) as u32).max(1),
                ((img.height() as f32 * e) as u32).max(1),
            );
            let petit =
                image::imageops::resize(&img, nl, nh, image::imageops::FilterType::Triangle);
            let mut fond = image::RgbaImage::from_pixel(l, h, uni);
            image::imageops::overlay(
                &mut fond,
                &petit,
                (l as i64 - nl as i64) / 2,
                (h as i64 - nh as i64) / 2,
            );
            fond
        }
        _ => crate::fond::couvrir(&img, l, h),
    })
}

/// Réapplique le fond d'écran actuel : le bureau se redessine sans l'aquarium.
pub fn rafraichir_fond() {
    let mut chemin = chemin_fond();
    chemin.push(0);
    unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            chemin.as_mut_ptr() as *mut c_void,
            SPIF_SENDCHANGE,
        );
    }
}

/// Une appli en plein écran au premier plan (jeu, vidéo, présentation) ?
pub fn plein_ecran() -> bool {
    unsafe {
        let mut etat = 0;
        if SHQueryUserNotificationState(&mut etat) == S_OK
            && matches!(
                etat,
                QUNS_BUSY | QUNS_RUNNING_D3D_FULL_SCREEN | QUNS_PRESENTATION_MODE
            )
        {
            return true;
        }
        let fg = GetForegroundWindow();
        if fg.is_null() {
            return false;
        }
        let mut classe = [0u16; 64];
        let n = GetClassNameW(fg, classe.as_mut_ptr(), classe.len() as i32) as usize;
        let classe = String::from_utf16_lossy(&classe[..n]);
        if classe == "Progman" || classe == "WorkerW" {
            return false;
        }
        let mut r: RECT = std::mem::zeroed();
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = size_of::<MONITORINFO>() as u32;
        if GetWindowRect(fg, &mut r) == 0
            || GetMonitorInfoW(MonitorFromWindow(fg, MONITOR_DEFAULTTONEAREST), &mut info) == 0
        {
            return false;
        }
        let m = info.rcMonitor;
        r.left <= m.left && r.top <= m.top && r.right >= m.right && r.bottom >= m.bottom
    }
}

/// Lancement au démarrage de la session : valeur « Aquarium » de HKCU\…\Run.
pub fn demarrage(actif: bool) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    unsafe {
        let mut cle: HKEY = null_mut();
        let r = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            large("Software\\Microsoft\\Windows\\CurrentVersion\\Run").as_ptr(),
            0,
            null(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            null(),
            &mut cle,
            null_mut(),
        );
        if r != 0 {
            return Err(format!(
                "Démarrage avec Windows : registre inaccessible ({r})."
            ));
        }
        let nom = large("Aquarium");
        let r = if actif {
            let v = large(&format!("\"{}\"", exe.display()));
            RegSetValueExW(
                cle,
                nom.as_ptr(),
                0,
                REG_SZ,
                v.as_ptr() as *const u8,
                (v.len() * 2) as u32,
            )
        } else {
            // Absente : rien à faire.
            let r = RegDeleteValueW(cle, nom.as_ptr());
            if r == 2 {
                0
            } else {
                r
            }
        };
        RegCloseKey(cle);
        if r != 0 {
            return Err(format!("Démarrage avec Windows : écriture refusée ({r})."));
        }
    }
    Ok(())
}

fn envoyer(c: Commande) {
    if let Some(m) = CANAL.get() {
        if let Ok(tx) = m.lock() {
            let _ = tx.send(c);
        }
    }
    if let Some(ctx) = CTX.get() {
        ctx.request_repaint();
    }
}

fn donnees_icone(h: HWND) -> NOTIFYICONDATAW {
    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = h;
    nid.uID = 1;
    nid
}

fn ajouter_icone(h: HWND) {
    // Icône 32x32 : le poisson du jeu (BGRA, masque ET vide : l'alpha suffit).
    let rgba = crate::appli::icone_rgba();
    let bgra: Vec<u8> = rgba
        .chunks(4)
        .flat_map(|p| [p[2], p[1], p[0], p[3]])
        .collect();
    let masque = [0u8; 32 * 32 / 8];
    unsafe {
        let icone = CreateIcon(
            GetModuleHandleW(null()),
            32,
            32,
            1,
            32,
            masque.as_ptr(),
            bgra.as_ptr(),
        );
        let mut nid = donnees_icone(h);
        nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.uCallbackMessage = WM_ICONE;
        nid.hIcon = if icone.is_null() {
            LoadIconW(null_mut(), IDI_APPLICATION)
        } else {
            icone
        };
        let tip = large("Aquarium : clic pour l'Écloserie");
        nid.szTip[..tip.len()].copy_from_slice(&tip);
        Shell_NotifyIconW(NIM_ADD, &nid);
    }
}

fn menu(h: HWND) {
    unsafe {
        let m = CreatePopupMenu();
        AppendMenuW(m, MF_STRING, 1, large("Ouvrir l'Écloserie").as_ptr());
        let pause = if EN_PAUSE.load(Ordering::Relaxed) {
            MF_CHECKED
        } else {
            0
        };
        AppendMenuW(m, MF_STRING | pause, 2, large("Pause").as_ptr());
        AppendMenuW(m, MF_SEPARATOR, 0, null());
        AppendMenuW(m, MF_STRING, 3, large("Quitter").as_ptr());
        let mut pt = POINT { x: 0, y: 0 };
        GetCursorPos(&mut pt);
        // Sans premier plan, le menu ne se ferme pas quand on clique ailleurs.
        SetForegroundWindow(h);
        let choix = TrackPopupMenu(
            m,
            TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_BOTTOMALIGN,
            pt.x,
            pt.y,
            0,
            h,
            null(),
        );
        PostMessageW(h, WM_NULL, 0, 0);
        DestroyMenu(m);
        match choix {
            1 => envoyer(Commande::Ouvrir),
            2 => envoyer(Commande::Pause),
            3 => envoyer(Commande::Quitter),
            _ => {}
        }
    }
}

unsafe extern "system" fn proc_sys(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_ICONE => {
                match (l as u32) & 0xffff {
                    WM_LBUTTONUP => envoyer(Commande::Ouvrir),
                    WM_RBUTTONUP | WM_CONTEXTMENU => menu(h),
                    _ => {}
                }
                0
            }
            WM_HOTKEY => {
                envoyer(Commande::Ouvrir);
                0
            }
            WM_CLOSE => {
                Shell_NotifyIconW(NIM_DELETE, &donnees_icone(h));
                DestroyWindow(h);
                0
            }
            WM_DESTROY => {
                FENETRE_SYS.store(null_mut(), Ordering::Release);
                PostQuitMessage(0);
                0
            }
            // Explorateur relancé : l'icône est à remettre.
            m if m != 0 && m == TASKBAR_CREATED.load(Ordering::Relaxed) => {
                ajouter_icone(h);
                0
            }
            _ => DefWindowProcW(h, msg, w, l),
        }
    }
}

/// Fil de l'icône de notification et du raccourci global (fenêtre cachée + boucle de messages).
fn fil_systeme() {
    unsafe {
        let hinst = GetModuleHandleW(null());
        let classe = large("AquariumSysteme");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(proc_sys),
            hInstance: hinst,
            lpszClassName: classe.as_ptr(),
            ..std::mem::zeroed()
        };
        RegisterClassW(&wc);
        let h = CreateWindowExW(
            0,
            classe.as_ptr(),
            large("Aquarium").as_ptr(),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            hinst,
            null(),
        );
        if h.is_null() {
            return;
        }
        FENETRE_SYS.store(h, Ordering::Release);
        TASKBAR_CREATED.store(
            RegisterWindowMessageW(large("TaskbarCreated").as_ptr()),
            Ordering::Relaxed,
        );
        ajouter_icone(h);
        // Raccourci facultatif : s'il est déjà pris, l'icône reste le moyen d'ouvrir.
        RegisterHotKey(h, 1, MOD_CONTROL | MOD_ALT | MOD_NOREPEAT, 'A' as u32);
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

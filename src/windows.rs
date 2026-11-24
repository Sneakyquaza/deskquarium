//! Partie Windows : incrustation derrière les icônes (WorkerW), fond d'écran actuel,
//! icône de notification + menu, raccourci global Ctrl+Alt+A, plein écran, démarrage
//! avec Windows (HKCU\…\Run), rafraîchissement du fond d'écran à la fermeture.
//! Multi-écrans (lot 4) : la fenêtre couvre tout le bureau virtuel (EnumDisplayMonitors),
//! fond d'écran de chaque écran (IDesktopWallpaper).
//! JAMAIS testé sur un vrai Windows (serveur Linux) : vérifié par compilation croisée.

use crate::appli::Commande;
use crate::ecrans::{Disposition, Ecran};
use eframe::egui;
use std::ffi::c_void;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering};
use std::sync::{mpsc, Mutex, OnceLock};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, S_OK, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, GetSysColor, MapWindowPoints, MonitorFromWindow,
    COLOR_DESKTOP, HDC, HMONITOR, MONITORINFO, MONITOR_DEFAULTTONEAREST, MONITOR_DEFAULTTOPRIMARY,
};
use windows_sys::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_ALL,
    COINIT_MULTITHREADED,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegGetValueW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ,
    RRF_RT_REG_SZ,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
};
use windows_sys::Win32::UI::Shell::{
    SHQueryUserNotificationState, ShellExecuteW, Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP,
    NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW, QUNS_PRESENTATION_MODE,
    QUNS_RUNNING_D3D_FULL_SCREEN,
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
/// Infobulle de l'icône (mise à jour disponible).
static INFOBULLE: Mutex<String> = Mutex::new(String::new());
const WM_ICONE: u32 = WM_APP + 1;

pub struct Systeme {
    recu: mpsc::Receiver<Commande>,
    fil: Option<std::thread::JoinHandle<()>>,
    /// Calque du fond d'écran : fenêtre Win32 à nous, dessinée en GDI (voir `calque.rs`).
    calque: crate::calque::Calque,
    /// Parent actuel du calque (Progman ou WorkerW) ; nul en mode fenêtre.
    parent: HWND,
    methode: u8,
}

static MODE_FENETRE: AtomicBool = AtomicBool::new(false);
static ABYSSES: AtomicBool = AtomicBool::new(false);

impl Systeme {
    /// Cache la fenêtre eframe (elle ne sert plus qu'à faire tourner l'appli et l'Écloserie),
    /// crée le calque du fond, lance le fil de l'icône de notification, puis incruste le calque
    /// derrière les icônes (ou le met en fenêtre au fond si `mode_fenetre` ou si ça échoue).
    pub fn demarrer(
        frame: &eframe::Frame,
        ctx: &egui::Context,
        mode_fenetre: bool,
        methode: u8,
    ) -> Result<Systeme, String> {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let h = frame
            .window_handle()
            .map_err(|e| format!("fenêtre : {e}"))?;
        let RawWindowHandle::Win32(w) = h.as_raw() else {
            return Err("fenêtre : pas une fenêtre Win32".into());
        };
        journal_version();
        unsafe { cacher_racine(w.hwnd.get() as HWND) };
        let calque = crate::calque::Calque::creer()?;
        let (tx, rx) = mpsc::channel();
        let _ = CANAL.set(Mutex::new(tx));
        let _ = CTX.set(ctx.clone());
        let fil = std::thread::spawn(fil_systeme);
        let mut s = Systeme {
            recu: rx,
            fil: Some(fil),
            calque,
            parent: null_mut(),
            methode,
        };
        s.appliquer(mode_fenetre, methode);
        Ok(s)
    }

    /// Incruste le calque (méthode 0 = auto, 1 = Progman + calque WS_EX_LAYERED, 2 = Progman
    /// sans, 3 = WorkerW) ou le passe en mode fenêtre. Un échec d'incrustation donne le mode
    /// fenêtre : on voit toujours l'aquarium, et le journal dit pourquoi. Renvoie l'erreur.
    pub fn appliquer(&mut self, mode_fenetre: bool, methode: u8) -> Option<String> {
        let mut erreur = None;
        let h = self.calque.hwnd;
        self.methode = methode;
        if !mode_fenetre {
            match incruster(h, methode) {
                Ok(p) => {
                    self.parent = p;
                    MODE_FENETRE.store(false, Ordering::Relaxed);
                    return None;
                }
                Err(e) => {
                    crate::journal::ecrire(&format!("incrustation impossible : {e}"));
                    erreur = Some(format!("{e} L'aquarium passe en mode fenêtre."));
                }
            }
        }
        unsafe { en_fenetre(h) };
        self.parent = null_mut();
        MODE_FENETRE.store(true, Ordering::Relaxed);
        erreur
    }

    /// Écrans couverts par le calque : tout le bureau virtuel, ou en mode fenêtre la zone de
    /// travail de l'écran principal (jamais la barre des tâches).
    pub fn disposition(&self) -> Option<Disposition> {
        if self.parent.is_null() {
            let t = zone_travail()?;
            return Disposition::nouvelle(vec![Ecran {
                x: t.left,
                y: t.top,
                l: t.right - t.left,
                h: t.bottom - t.top,
                principal: true,
                travail: None,
            }]);
        }
        disposition()
    }

    /// Les écrans ont changé (branché, débranché, déplacé) : le calque recouvre le nouveau
    /// bureau virtuel.
    pub fn recaler(&self) {
        unsafe {
            if self.parent.is_null() {
                en_fenetre(self.calque.hwnd);
            } else {
                placer(self.calque.hwnd, self.parent, None);
            }
        }
    }

    pub fn presenter(&self, toile: &crate::toile::Toile, e: usize) {
        self.calque.presenter(toile, e);
    }

    pub fn fond(&self, img: &image::RgbaImage) {
        self.calque.fond(img);
    }

    /// Mire de couleurs 5 s sur le calque (le voit-on ?).
    pub fn tester(&self) {
        let nom = if self.parent.is_null() {
            "mode fenêtre".to_string()
        } else {
            format!(
                "méthode {} (parent {})",
                self.methode,
                classe_de(self.parent)
            )
        };
        self.calque.tester(&nom);
    }

    pub fn commandes(&self) -> Vec<Commande> {
        self.recu.try_iter().collect()
    }

    pub fn signaler_pause(&self, p: bool) {
        EN_PAUSE.store(p, Ordering::Relaxed);
    }

    /// État du mode Abysses pour la coche du menu.
    /// Infobulle de l'icône de notification (« mise à jour disponible »…).
    pub fn infobulle(&self, t: &str) {
        *INFOBULLE.lock().unwrap_or_else(|e| e.into_inner()) = t.to_string();
        let h = FENETRE_SYS.load(Ordering::Acquire);
        if h.is_null() {
            return;
        }
        let mut nid = donnees_icone(h);
        nid.uFlags = NIF_TIP;
        remplir_infobulle(&mut nid);
        unsafe {
            Shell_NotifyIconW(NIM_MODIFY, &nid);
        }
    }

    pub fn signaler_abysses(&self, a: bool) {
        ABYSSES.store(a, Ordering::Relaxed);
    }

    /// Retire l'icône et le calque, attend le fil, puis redessine le fond d'écran.
    pub fn arreter(mut self) {
        self.calque.detruire();
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

/// La fenêtre eframe ne montre plus rien : 1 pixel, transparente, sans clic, tout au fond.
unsafe fn cacher_racine(h: HWND) {
    unsafe {
        let ex = GetWindowLongPtrW(h, GWL_EXSTYLE) as u32;
        SetWindowLongPtrW(
            h,
            GWL_EXSTYLE,
            (ex | WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT) as isize,
        );
        SetLayeredWindowAttributes(h, 0, 0, LWA_ALPHA);
        SetWindowPos(
            h,
            HWND_BOTTOM,
            0,
            0,
            1,
            1,
            SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
        crate::journal::ecrire(&format!(
            "fenêtre eframe cachée : {h:?} rect {} exstyle {:#x}",
            rect_de(h),
            GetWindowLongPtrW(h, GWL_EXSTYLE) as u32
        ));
    }
}

/// Zone de travail de l'écran principal (sans la barre des tâches).
fn zone_travail() -> Option<RECT> {
    unsafe {
        let pt = POINT { x: 0, y: 0 };
        let m = windows_sys::Win32::Graphics::Gdi::MonitorFromPoint(pt, MONITOR_DEFAULTTOPRIMARY);
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = size_of::<MONITORINFO>() as u32;
        (GetMonitorInfoW(m, &mut info) != 0).then_some(info.rcWork)
    }
}

/// Version de Windows dans le journal (le numéro de build décide de la disposition du bureau).
fn journal_version() {
    let cle = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion";
    let v = |n| lire_registre_dans(HKEY_LOCAL_MACHINE, cle, n).unwrap_or_default();
    // ProductName dit encore « Windows 10 » sous Windows 11 : le build tranche.
    let build = v("CurrentBuild");
    let nom = if build.parse::<u32>().is_ok_and(|b| b >= 22000) {
        "Windows 11"
    } else {
        "Windows 10"
    };
    crate::journal::ecrire(&format!(
        "--- aquarium {} : {nom} {} {} build {build} ---",
        env!("CARGO_PKG_VERSION"),
        v("EditionID"),
        v("DisplayVersion"),
    ));
}

fn classe_de(h: HWND) -> String {
    let mut c = [0u16; 64];
    let n = unsafe { GetClassNameW(h, c.as_mut_ptr(), c.len() as i32) } as usize;
    String::from_utf16_lossy(&c[..n])
}

fn rect_de(h: HWND) -> String {
    let mut r: RECT = unsafe { std::mem::zeroed() };
    unsafe { GetWindowRect(h, &mut r) };
    format!("({},{})-({},{})", r.left, r.top, r.right, r.bottom)
}

/// Incrustation derrière les icônes, deux dispositions du bureau reconnues à l'exécution :
/// - avant Windows 11 24H2 : le message 0x052C à Progman crée un WorkerW de premier niveau
///   derrière celui qui porte les icônes (SHELLDLL_DefView) ; notre fenêtre devient son enfant ;
/// - 24H2 et plus (25H2…) : SHELLDLL_DefView et le WorkerW restent ENFANTS de Progman ; notre
///   fenêtre devient enfant de Progman, juste sous SHELLDLL_DefView et au-dessus du WorkerW,
///   avec WS_EX_LAYERED (Progman n'a plus de surface de redirection : sans calque, un enfant
///   n'est pas composé ; correctif de Lively Wallpaper). Renvoie le parent choisi.
fn incruster(hwnd: HWND, methode: u8) -> Result<HWND, String> {
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
        let vue_progman = FindWindowExW(
            progman,
            null_mut(),
            large("SHELLDLL_DefView").as_ptr(),
            null(),
        );
        let worker_enfant = FindWindowExW(progman, null_mut(), large("WorkerW").as_ptr(), null());
        let mut worker_haut: HWND = null_mut();
        EnumWindows(
            Some(chercher_worker),
            &mut worker_haut as *mut HWND as LPARAM,
        );
        let recent = !vue_progman.is_null();
        crate::journal::ecrire(&format!(
            "Progman {progman:?} {} ; DefView dans Progman {vue_progman:?} ; WorkerW enfant de \
             Progman {worker_enfant:?} ; WorkerW de premier niveau {worker_haut:?} ; disposition {} ; \
             méthode demandée {methode}",
            rect_de(progman),
            if recent { "24H2+" } else { "classique" },
        ));
        // Méthode : 1 Progman + calque, 2 Progman sans calque, 3 WorkerW.
        let m = match methode {
            1..=3 => methode,
            _ if recent => 1,
            _ => 3,
        };
        let worker = if worker_haut.is_null() {
            worker_enfant
        } else {
            worker_haut
        };
        let parent = if m == 3 { worker } else { progman };
        if parent.is_null() {
            return Err("Calque du fond d'écran introuvable (WorkerW).".into());
        }
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        let style =
            (style & !(WS_POPUP | WS_CAPTION | WS_THICKFRAME | WS_SYSMENU)) | WS_CHILD | WS_VISIBLE;
        use windows_sys::Win32::Foundation::{GetLastError, SetLastError};
        SetLastError(0);
        SetWindowLongPtrW(hwnd, GWL_STYLE, style as isize);
        let e_style = GetLastError();
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 & !WS_EX_TOOLWINDOW;
        let ex = if m == 1 {
            ex | WS_EX_LAYERED
        } else {
            ex & !WS_EX_LAYERED
        };
        SetLastError(0);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex as isize);
        let e_ex = GetLastError();
        // Calque WS_EX_LAYERED réappliqué à chaque incrustation (opaque : alpha 255).
        let e_calque = if m == 1 {
            if SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA) == 0 {
                GetLastError()
            } else {
                0
            }
        } else {
            0
        };
        let ancien = SetParent(hwnd, parent);
        let e_parent = GetLastError();
        crate::journal::ecrire(&format!(
            "erreurs : style {e_style}, exstyle {e_ex}, calque {e_calque}, SetParent {e_parent} \
             (ancien parent {ancien:?})"
        ));
        if GetParent(hwnd) != parent {
            return Err(format!(
                "Rattachement au bureau refusé (SetParent, erreur {e_parent})."
            ));
        }
        // Sous les icônes : juste après SHELLDLL_DefView dans l'ordre des enfants de Progman.
        let apres = (m != 3 && !vue_progman.is_null()).then_some(vue_progman);
        placer(hwnd, parent, apres);
        if m != 3 && !worker_enfant.is_null() {
            SetWindowPos(
                worker_enfant,
                HWND_BOTTOM,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
        crate::journal::ecrire(&format!(
            "incrustée : méthode {m}, parent {parent:?} ({}), rect {}, style {:#x}, exstyle {:#x}, \
             visible {}",
            classe_de(GetParent(hwnd)),
            rect_de(hwnd),
            GetWindowLongPtrW(hwnd, GWL_STYLE) as u32,
            GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32,
            IsWindowVisible(hwnd) != 0,
        ));
        // Ordre des enfants de Progman, du haut vers le bas (le calque doit être juste sous
        // SHELLDLL_DefView, le WorkerW en dessous).
        let mut c = GetWindow(progman, GW_CHILD);
        let mut i = 0;
        while !c.is_null() && i < 20 {
            crate::journal::ecrire(&format!(
                "  enfant {i} de Progman : {c:?} {} {} visible {} exstyle {:#x}{}",
                classe_de(c),
                rect_de(c),
                IsWindowVisible(c) != 0,
                GetWindowLongPtrW(c, GWL_EXSTYLE) as u32,
                if c == hwnd { " <- aquarium" } else { "" }
            ));
            c = GetWindow(c, GW_HWNDNEXT);
            i += 1;
        }
        Ok(parent)
    }
}

/// Mode fenêtre : fenêtre sans bord de premier niveau sur la zone de travail de l'écran
/// principal, au fond de la pile (au-dessus du bureau, sous les autres applis et la barre).
unsafe fn en_fenetre(hwnd: HWND) {
    unsafe {
        if !GetParent(hwnd).is_null() {
            SetParent(hwnd, null_mut());
        }
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        SetWindowLongPtrW(
            hwnd,
            GWL_STYLE,
            ((style & !(WS_CHILD | WS_CAPTION | WS_THICKFRAME)) | WS_POPUP | WS_VISIBLE) as isize,
        );
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        SetWindowLongPtrW(
            hwnd,
            GWL_EXSTYLE,
            ((ex & !WS_EX_LAYERED) | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE) as isize,
        );
        // Zone de travail de l'écran principal : la barre des tâches reste visible.
        let t = zone_travail().unwrap_or(RECT {
            left: 0,
            top: 0,
            right: GetSystemMetrics(SM_CXSCREEN),
            bottom: GetSystemMetrics(SM_CYSCREEN) - 48,
        });
        SetWindowPos(
            hwnd,
            HWND_BOTTOM,
            t.left,
            t.top,
            t.right - t.left,
            t.bottom - t.top,
            SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED,
        );
        crate::journal::ecrire(&format!("mode fenêtre : rect {}", rect_de(hwnd)));
    }
}

/// Tout le bureau virtuel (tous les écrans, coordonnées écran) ramené dans le repère du parent
/// (nul : coordonnées écran). `apres` : place dans l'ordre d'empilement, sinon inchangée.
unsafe fn placer(hwnd: HWND, parent: HWND, apres: Option<HWND>) {
    unsafe {
        let (x, y, l, h) = match disposition() {
            Some(d) => d.boite(),
            None => (
                GetSystemMetrics(SM_XVIRTUALSCREEN),
                GetSystemMetrics(SM_YVIRTUALSCREEN),
                GetSystemMetrics(SM_CXVIRTUALSCREEN),
                GetSystemMetrics(SM_CYVIRTUALSCREEN),
            ),
        };
        let mut pt = POINT { x, y };
        if !parent.is_null() {
            MapWindowPoints(null_mut(), parent, &mut pt, 1);
        }
        let ordre = if apres.is_some() { 0 } else { SWP_NOZORDER };
        SetWindowPos(
            hwnd,
            apres.unwrap_or(null_mut()),
            pt.x,
            pt.y,
            l,
            h,
            ordre | SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED,
        );
    }
}

/// Écrans de Windows en pixels physiques (le processus est « per-monitor DPI aware », réglé
/// par winit) : EnumDisplayMonitors + GetMonitorInfoW, aucune disposition en dur.
pub fn disposition() -> Option<Disposition> {
    unsafe extern "system" fn un(m: HMONITOR, _: HDC, _: *mut RECT, l: LPARAM) -> i32 {
        unsafe {
            let v = &mut *(l as *mut Vec<Ecran>);
            let mut info: MONITORINFO = std::mem::zeroed();
            info.cbSize = size_of::<MONITORINFO>() as u32;
            if GetMonitorInfoW(m, &mut info) != 0 {
                let r = info.rcMonitor;
                v.push(Ecran {
                    x: r.left,
                    y: r.top,
                    l: r.right - r.left,
                    h: r.bottom - r.top,
                    principal: info.dwFlags & MONITORINFOF_PRIMARY != 0,
                    // Zone de travail : le compteur et le sable restent au-dessus de la barre.
                    travail: Some((
                        info.rcWork.left,
                        info.rcWork.top,
                        info.rcWork.right - info.rcWork.left,
                        info.rcWork.bottom - info.rcWork.top,
                    )),
                });
            }
        }
        1
    }
    let mut v: Vec<Ecran> = Vec::new();
    unsafe {
        EnumDisplayMonitors(
            null_mut(),
            null(),
            Some(un),
            &mut v as *mut Vec<Ecran> as LPARAM,
        );
    }
    Disposition::nouvelle(v)
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
    lire_registre_dans(HKEY_CURRENT_USER, cle, valeur)
}

fn lire_registre_dans(racine: HKEY, cle: &str, valeur: &str) -> Option<String> {
    let mut buf = vec![0u16; 260];
    let mut taille = (buf.len() * 2) as u32;
    let r = unsafe {
        RegGetValueW(
            racine,
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

/// Couleur unie du bureau (pas d'image, bords des modes Centrer / Ajuster).
fn couleur_bureau() -> image::Rgba<u8> {
    let c = unsafe { GetSysColor(COLOR_DESKTOP) };
    image::Rgba([
        (c & 0xff) as u8,
        (c >> 8 & 0xff) as u8,
        (c >> 16 & 0xff) as u8,
        255,
    ])
}

/// Image posée sur `l`x`h` selon le mode de Windows (DESKTOP_WALLPAPER_POSITION : 0 Centrer,
/// 1 Mosaïque, 2 Étirer, 3 Ajuster, 4 Remplir, 5 Étendre = Remplir sur la boîte).
fn poser_image(
    img: &image::RgbaImage,
    l: u32,
    h: u32,
    mode: i32,
    uni: image::Rgba<u8>,
) -> image::RgbaImage {
    match mode {
        2 => image::imageops::resize(img, l, h, image::imageops::FilterType::Triangle),
        0 | 3 => {
            let e = if mode == 3 {
                (l as f32 / img.width() as f32).min(h as f32 / img.height() as f32)
            } else {
                1.0
            };
            let (nl, nh) = (
                ((img.width() as f32 * e) as u32).max(1),
                ((img.height() as f32 * e) as u32).max(1),
            );
            let petit = image::imageops::resize(img, nl, nh, image::imageops::FilterType::Triangle);
            let mut fond = image::RgbaImage::from_pixel(l, h, uni);
            image::imageops::overlay(
                &mut fond,
                &petit,
                (l as i64 - nl as i64) / 2,
                (h as i64 - nh as i64) / 2,
            );
            fond
        }
        1 => {
            let mut fond = image::RgbaImage::from_pixel(l, h, uni);
            for y in (0..h).step_by(img.height().max(1) as usize) {
                for x in (0..l).step_by(img.width().max(1) as usize) {
                    image::imageops::overlay(&mut fond, img, x as i64, y as i64);
                }
            }
            fond
        }
        _ => crate::fond::couvrir(img, l, h),
    }
}

/// Fond d'écran d'un écran selon IDesktopWallpaper : rectangle, chemin de l'image (vide =
/// couleur unie), identifiant de l'écran (pour le réappliquer à la sortie).
struct FondEcran {
    rect: RECT,
    chemin: String,
    id: Vec<u16>,
}

/// IDesktopWallpaper (shobjidl_core.h) : table des méthodes dans l'ordre de l'interface
/// (windows-sys ne fournit pas les interfaces COM).
#[repr(C)]
struct VtblFond {
    query_interface: usize,
    add_ref: usize,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
    set_wallpaper: unsafe extern "system" fn(*mut c_void, *const u16, *const u16) -> i32,
    get_wallpaper: unsafe extern "system" fn(*mut c_void, *const u16, *mut *mut u16) -> i32,
    get_monitor_device_path_at: unsafe extern "system" fn(*mut c_void, u32, *mut *mut u16) -> i32,
    get_monitor_device_path_count: unsafe extern "system" fn(*mut c_void, *mut u32) -> i32,
    get_monitor_rect: unsafe extern "system" fn(*mut c_void, *const u16, *mut RECT) -> i32,
    set_background_color: usize,
    get_background_color: usize,
    set_position: usize,
    get_position: unsafe extern "system" fn(*mut c_void, *mut i32) -> i32,
}

const IID_IDESKTOPWALLPAPER: windows_sys::core::GUID =
    windows_sys::core::GUID::from_u128(0xb92b56a9_8b55_4e14_9a89_0199bbb6f93b);

/// Appelle `f` avec l'objet IDesktopWallpaper, dans un fil à part (COM initialisé pour lui
/// seul, sans toucher à l'appartement du fil de la fenêtre).
fn avec_fonds<T: Send + 'static>(
    f: impl FnOnce(*mut c_void, &VtblFond) -> Option<T> + Send + 'static,
) -> Option<T> {
    std::thread::spawn(move || unsafe {
        let init = CoInitializeEx(null(), COINIT_MULTITHREADED as u32);
        let mut obj: *mut c_void = null_mut();
        let r = CoCreateInstance(
            &windows_sys::Win32::UI::Shell::DesktopWallpaper,
            null_mut(),
            CLSCTX_ALL,
            &IID_IDESKTOPWALLPAPER,
            &mut obj,
        );
        let res = if r >= 0 && !obj.is_null() {
            let vt = &**(obj as *mut *const VtblFond);
            let res = f(obj, vt);
            (vt.release)(obj);
            res
        } else {
            None
        };
        if init >= 0 {
            CoUninitialize();
        }
        res
    })
    .join()
    .ok()
    .flatten()
}

/// Fond de chaque écran et mode de placement (commun à tous les écrans sous Windows).
fn fonds_ecrans() -> Option<(Vec<FondEcran>, i32)> {
    avec_fonds(|o, vt| unsafe {
        let mut n = 0u32;
        if (vt.get_monitor_device_path_count)(o, &mut n) < 0 {
            return None;
        }
        let mut v = Vec::new();
        for i in 0..n {
            let mut id: *mut u16 = null_mut();
            if (vt.get_monitor_device_path_at)(o, i, &mut id) < 0 || id.is_null() {
                continue;
            }
            let lire = |p: *mut u16| {
                let mut k = 0;
                while *p.add(k) != 0 {
                    k += 1;
                }
                std::slice::from_raw_parts(p, k).to_vec()
            };
            let ident = lire(id);
            let mut rect: RECT = std::mem::zeroed();
            let mut chemin: *mut u16 = null_mut();
            // Écran inactif (dupliqué, éteint) : GetMonitorRECT échoue, on le saute.
            if (vt.get_monitor_rect)(o, id, &mut rect) >= 0 {
                let c = if (vt.get_wallpaper)(o, id, &mut chemin) >= 0 && !chemin.is_null() {
                    String::from_utf16_lossy(&lire(chemin))
                } else {
                    String::new()
                };
                v.push(FondEcran {
                    rect,
                    chemin: c,
                    id: ident.into_iter().chain(Some(0)).collect(),
                });
            }
            if !chemin.is_null() {
                CoTaskMemFree(chemin as *const c_void);
            }
            CoTaskMemFree(id as *const c_void);
        }
        let mut pos = 4;
        (vt.get_position)(o, &mut pos);
        Some((v, pos))
    })
}

/// Mode de placement lu dans le registre (repli sans IDesktopWallpaper).
fn mode_registre() -> i32 {
    let style = lire_registre("Control Panel\\Desktop", "WallpaperStyle").unwrap_or_default();
    let mosaique = lire_registre("Control Panel\\Desktop", "TileWallpaper").as_deref() == Some("1");
    match (style.trim(), mosaique) {
        (_, true) => 1,
        ("0", false) => 0,
        ("2", _) => 2,
        ("6", _) => 3,
        ("22", _) => 5,
        _ => 4,
    }
}

/// Fond d'écran actuel de chaque écran, assemblé sur la boîte englobante de `d` (trous entre
/// écrans en noir : hors écran, jamais vus).
pub fn fond_bureau(d: &Disposition) -> Option<image::RgbaImage> {
    let (bx, by, bl, bh) = d.boite();
    let uni = couleur_bureau();
    let mut out = image::RgbaImage::from_pixel(bl as u32, bh as u32, image::Rgba([0, 0, 0, 255]));
    let (fonds, mode) = match fonds_ecrans() {
        Some((f, m)) if !f.is_empty() => (f, m),
        _ => (Vec::new(), mode_registre()),
    };
    let commun = String::from_utf16_lossy(&chemin_fond());
    let ouvrir = |c: &str| {
        (!c.is_empty())
            .then(|| image::open(c).ok())
            .flatten()
            .map(|i| i.to_rgba8())
    };
    // Étendre : une seule image sur toute la boîte.
    let etendu = if mode == 5 {
        let c = fonds
            .first()
            .map(|f| f.chemin.clone())
            .unwrap_or(commun.clone());
        ouvrir(&c).map(|img| crate::fond::couvrir(&img, bl as u32, bh as u32))
    } else {
        None
    };
    for e in &d.ecrans {
        let (ox, oy) = ((e.x - bx) as i64, (e.y - by) as i64);
        let morceau = if let Some(g) = &etendu {
            image::imageops::crop_imm(g, ox as u32, oy as u32, e.l as u32, e.h as u32).to_image()
        } else {
            // L'écran de même rectangle chez IDesktopWallpaper, sinon le fond commun.
            let c = fonds
                .iter()
                .find(|f| f.rect.left == e.x && f.rect.top == e.y)
                .map(|f| f.chemin.clone())
                .unwrap_or_else(|| commun.clone());
            match ouvrir(&c) {
                Some(img) => poser_image(&img, e.l as u32, e.h as u32, mode, uni),
                None => image::RgbaImage::from_pixel(e.l as u32, e.h as u32, uni),
            }
        };
        image::imageops::replace(&mut out, &morceau, ox, oy);
    }
    Some(out)
}

/// Réapplique le fond d'écran actuel : le bureau se redessine sans l'aquarium. Plusieurs
/// écrans : chaque écran reçoit à nouveau SON image (jamais une image commune).
pub fn rafraichir_fond() {
    if let Some((fonds, _)) = fonds_ecrans().filter(|(f, _)| f.len() >= 2) {
        let _ = avec_fonds(move |o, vt| unsafe {
            for f in fonds.iter().filter(|f| !f.chemin.is_empty()) {
                let c = large(&f.chemin);
                (vt.set_wallpaper)(o, f.id.as_ptr(), c.as_ptr());
            }
            Some(())
        });
        return;
    }
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

/// Une appli en plein écran au premier plan (jeu, vidéo, présentation) ? Renvoie la raison
/// (journal). Jamais le bureau lui-même ni une fenêtre de l'aquarium.
pub fn plein_ecran() -> Option<String> {
    unsafe {
        let mut etat = 0;
        if SHQueryUserNotificationState(&mut etat) == S_OK
            && matches!(etat, QUNS_RUNNING_D3D_FULL_SCREEN | QUNS_PRESENTATION_MODE)
        {
            return Some(format!("état de notification {etat}"));
        }
        let fg = GetForegroundWindow();
        if fg.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(fg, &mut pid);
        let classe = classe_de(fg);
        if pid == std::process::id()
            || matches!(
                classe.as_str(),
                "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd"
            )
        {
            return None;
        }
        let mut r: RECT = std::mem::zeroed();
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = size_of::<MONITORINFO>() as u32;
        if GetWindowRect(fg, &mut r) == 0
            || GetMonitorInfoW(MonitorFromWindow(fg, MONITOR_DEFAULTTONEAREST), &mut info) == 0
        {
            return None;
        }
        let m = info.rcMonitor;
        (r.left <= m.left && r.top <= m.top && r.right >= m.right && r.bottom >= m.bottom)
            .then(|| format!("fenêtre {classe} {} plein écran", rect_de(fg)))
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
        remplir_infobulle(&mut nid);
        Shell_NotifyIconW(NIM_ADD, &nid);
    }
}

fn remplir_infobulle(nid: &mut NOTIFYICONDATAW) {
    let t = INFOBULLE.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let t = if t.is_empty() {
        "Aquarium : clic pour l'Écloserie".to_string()
    } else {
        t
    };
    let mut tip = large(&t);
    // 128 caractères au plus, zéro final compris.
    tip.truncate(nid.szTip.len() - 1);
    tip.push(0);
    nid.szTip[..tip.len()].copy_from_slice(&tip);
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
        let abysses = if ABYSSES.load(Ordering::Relaxed) {
            MF_CHECKED
        } else {
            0
        };
        AppendMenuW(m, MF_STRING | abysses, 8, large("Mode Abysses").as_ptr());
        AppendMenuW(m, MF_SEPARATOR, 0, null());
        let fenetre = if MODE_FENETRE.load(Ordering::Relaxed) {
            MF_CHECKED
        } else {
            0
        };
        AppendMenuW(m, MF_STRING | fenetre, 4, large("Mode fenêtre").as_ptr());
        AppendMenuW(
            m,
            MF_STRING,
            5,
            large("Autre méthode d'incrustation").as_ptr(),
        );
        AppendMenuW(
            m,
            MF_STRING,
            7,
            large("Test d'incrustation (couleurs)").as_ptr(),
        );
        AppendMenuW(m, MF_STRING, 6, large("Ouvrir le journal").as_ptr());
        AppendMenuW(
            m,
            MF_STRING,
            9,
            large("Rechercher une mise à jour").as_ptr(),
        );
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
            4 => envoyer(Commande::ModeFenetre),
            5 => envoyer(Commande::AutreMethode),
            7 => envoyer(Commande::Tester),
            8 => envoyer(Commande::Abysses),
            9 => envoyer(Commande::ChercherMaj),
            6 => {
                let f = large(&crate::journal::chemin().display().to_string());
                ShellExecuteW(
                    null_mut(),
                    large("open").as_ptr(),
                    f.as_ptr(),
                    null(),
                    null(),
                    SW_SHOWNORMAL,
                );
            }
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
            // Explorateur relancé : l'icône est à remettre, l'aquarium à réincruster.
            m if m != 0 && m == TASKBAR_CREATED.load(Ordering::Relaxed) => {
                crate::journal::ecrire("Explorateur relancé (TaskbarCreated)");
                ajouter_icone(h);
                envoyer(Commande::Rattacher);
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

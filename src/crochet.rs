//! Souris au bureau (v0.9, Windows) : position du curseur pour la curiosité des poissons et
//! crochet souris bas niveau (WH_MOUSE_LL) pour attraper un poisson.
//!
//! Le crochet tourne sur son propre fil avec sa boucle de messages et reste très rapide : il
//! ne lit que des atomiques publiés par le fil du jeu (« armé » : poisson attrapable sous ce
//! point, à cet instant) et la règle pure de `souris.rs` ; il ne bloque jamais, n'envoie
//! aucun message synchrone. Les clics à rendre au bureau sont rejoués APRÈS coup (SendInput
//! depuis la boucle du fil, jamais dans le crochet) ; nos propres injections passent
//! (LLMHF_INJECTED). Coordonnées : pixels physiques du bureau virtuel (processus
//! PerMonitorV2, voir le manifeste), comme GetCursorPos, WindowFromPoint et le calque.

use crate::souris::{Decision, Pince, Suite};
use std::cell::RefCell;
use std::ffi::c_void;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc;
use windows_sys::Win32::Foundation::{CloseHandle, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::ScreenToClient;
use windows_sys::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows_sys::Win32::System::Memory::{
    VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
};
use windows_sys::Win32::System::SystemInformation::GetTickCount;
use windows_sys::Win32::System::Threading::{
    GetCurrentThreadId, OpenProcess, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
};
use windows_sys::Win32::UI::Controls::{LVHITTESTINFO, LVM_HITTEST};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_VIRTUALDESK,
    MOUSEINPUT, VK_MENU,
};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

fn large(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

/// Poisson attrapable publié par le jeu : point (px physiques), instant (GetTickCount, 0 =
/// rien), poisson ; Alt exigé.
static ARME_X: AtomicI32 = AtomicI32::new(0);
static ARME_Y: AtomicI32 = AtomicI32::new(0);
static ARME_T: AtomicU32 = AtomicU32::new(0);
static ARME_ID: AtomicU64 = AtomicU64::new(0);
static ALT: AtomicBool = AtomicBool::new(false);
/// Un armement plus vieux que ça ne compte plus (le jeu publie à chaque image, 15-60 i/s).
const ARME_MS: u32 = 250;
/// Rejouer au bureau (message du fil du crochet) : wparam 1 = clic, 2 = appui en (x, y).
const WM_REJOUER: u32 = WM_APP + 9;

thread_local! {
    static PINCE: RefCell<Pince> = RefCell::new(Pince::default());
    static ENVOI: RefCell<Option<mpsc::Sender<Suite>>> = const { RefCell::new(None) };
}

/// Le fil du jeu publie à chaque image le poisson attrapable sous le curseur (ou rien).
pub fn armer(arme: Option<(i32, i32, u64)>, alt: bool) {
    ALT.store(alt, Ordering::Relaxed);
    match arme {
        Some((x, y, id)) => {
            ARME_X.store(x, Ordering::Relaxed);
            ARME_Y.store(y, Ordering::Relaxed);
            ARME_ID.store(id, Ordering::Relaxed);
            ARME_T.store(unsafe { GetTickCount() } | 1, Ordering::Release);
        }
        None => ARME_T.store(0, Ordering::Release),
    }
}

/// Poisson attrapable au point (lu dans le crochet : atomiques seulement).
fn arme(pt: POINT) -> Option<u64> {
    let t = ARME_T.load(Ordering::Acquire);
    if t == 0 || unsafe { GetTickCount() }.wrapping_sub(t) > ARME_MS {
        return None;
    }
    let (x, y) = (
        ARME_X.load(Ordering::Relaxed),
        ARME_Y.load(Ordering::Relaxed),
    );
    if (pt.x - x).abs() > 12 || (pt.y - y).abs() > 12 {
        return None;
    }
    if ALT.load(Ordering::Relaxed) && unsafe { GetAsyncKeyState(VK_MENU as i32) } >= 0 {
        return None;
    }
    Some(ARME_ID.load(Ordering::Relaxed))
}

unsafe extern "system" fn crochet(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        if code == HC_ACTION as i32 && l != 0 {
            let m = &*(l as *const MSLLHOOKSTRUCT);
            // Nos propres injections (clic rendu au bureau) et celles des autres passent.
            if m.flags & LLMHF_INJECTED == 0 {
                let (d, suite) = PINCE.with(|p| {
                    let Ok(mut p) = p.try_borrow_mut() else {
                        return (Decision::Laisser, Suite::Rien);
                    };
                    match w as u32 {
                        WM_LBUTTONDOWN => p.appui(m.time, m.pt.x, m.pt.y, arme(m.pt)),
                        WM_MOUSEMOVE if p.occupe() => p.mouvement(m.time, m.pt.x, m.pt.y),
                        WM_LBUTTONUP => p.relache(m.time),
                        _ => (Decision::Laisser, Suite::Rien),
                    }
                });
                if suite != Suite::Rien {
                    ENVOI.with(|e| {
                        if let Some(e) = e.borrow().as_ref() {
                            let _ = e.send(suite);
                        }
                    });
                    let fil = GetCurrentThreadId();
                    match suite {
                        Suite::Clic(_) => {
                            PostThreadMessageW(fil, WM_REJOUER, 1, 0);
                        }
                        Suite::RendreAppui(x, y) => {
                            let xy = ((x as u32 as u64) | ((y as u32 as u64) << 32)) as LPARAM;
                            PostThreadMessageW(fil, WM_REJOUER, 2, xy);
                        }
                        _ => {}
                    }
                }
                if d == Decision::Avaler {
                    return 1;
                }
            }
        }
        CallNextHookEx(null_mut(), code, w, l)
    }
}

fn souris(dx: i32, dy: i32, drapeaux: u32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: drapeaux,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Coordonnées absolues SendInput (0..65535 sur le bureau virtuel).
fn absolu(x: i32, y: i32) -> (i32, i32) {
    unsafe {
        let (vx, vy) = (
            GetSystemMetrics(SM_XVIRTUALSCREEN),
            GetSystemMetrics(SM_YVIRTUALSCREEN),
        );
        let (vl, vh) = (
            GetSystemMetrics(SM_CXVIRTUALSCREEN).max(2),
            GetSystemMetrics(SM_CYVIRTUALSCREEN).max(2),
        );
        (
            ((x - vx) as i64 * 65535 / (vl - 1) as i64) as i32,
            ((y - vy) as i64 * 65535 / (vh - 1) as i64) as i32,
        )
    }
}

/// Rend au bureau ce qu'on lui a pris : un clic complet, ou l'appui de départ (le rectangle
/// de sélection part de là) puis retour au curseur.
fn rejouer(genre: usize, l: LPARAM) {
    let entrees: Vec<INPUT> = if genre == 1 {
        vec![
            souris(0, 0, MOUSEEVENTF_LEFTDOWN),
            souris(0, 0, MOUSEEVENTF_LEFTUP),
        ]
    } else {
        let (x, y) = (l as u64 as u32 as i32, ((l as u64) >> 32) as u32 as i32);
        let mut ici = POINT { x, y };
        unsafe { GetCursorPos(&mut ici) };
        let abs = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
        let (ax, ay) = absolu(x, y);
        let (bx, by) = absolu(ici.x, ici.y);
        vec![
            souris(ax, ay, abs),
            souris(0, 0, MOUSEEVENTF_LEFTDOWN),
            souris(bx, by, abs),
        ]
    };
    unsafe {
        SendInput(
            entrees.len() as u32,
            entrees.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
    }
}

/// Fil du crochet : installé à la création, retiré à l'arrêt (WM_QUIT).
pub struct Crochet {
    fil: Option<std::thread::JoinHandle<()>>,
    id: u32,
    recu: mpsc::Receiver<Suite>,
}

impl Crochet {
    pub fn demarrer() -> Option<Crochet> {
        let (tx, rx) = mpsc::channel();
        let (tid_tx, tid_rx) = mpsc::channel();
        let fil = std::thread::spawn(move || unsafe {
            ENVOI.with(|e| *e.borrow_mut() = Some(tx));
            let h = SetWindowsHookExW(WH_MOUSE_LL, Some(crochet), null_mut(), 0);
            // La file de messages du fil existe dès le premier appel (PeekMessage).
            let mut msg: MSG = std::mem::zeroed();
            PeekMessageW(&mut msg, null_mut(), 0, 0, PM_NOREMOVE);
            let _ = tid_tx.send((!h.is_null()).then(|| GetCurrentThreadId()));
            if h.is_null() {
                return;
            }
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                if msg.message == WM_REJOUER {
                    rejouer(msg.wParam, msg.lParam);
                    continue;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            UnhookWindowsHookEx(h);
        });
        match tid_rx.recv() {
            Ok(Some(id)) => {
                crate::journal::ecrire("souris : crochet installé");
                Some(Crochet {
                    fil: Some(fil),
                    id,
                    recu: rx,
                })
            }
            _ => {
                crate::journal::ecrire("souris : crochet impossible (SetWindowsHookEx)");
                let _ = fil.join();
                None
            }
        }
    }

    /// Évènements pour le jeu depuis la dernière image.
    pub fn suites(&self) -> Vec<Suite> {
        self.recu.try_iter().collect()
    }
}

impl Drop for Crochet {
    fn drop(&mut self) {
        armer(None, false);
        unsafe {
            PostThreadMessageW(self.id, WM_QUIT, 0, 0);
        }
        if let Some(f) = self.fil.take() {
            let _ = f.join();
        }
    }
}

/// Curseur sur le bureau (pas sur une autre fenêtre) : position physique et fenêtre du bureau
/// sous lui (la liste des icônes, ou le calque en mode fenêtre). None sinon.
pub fn curseur_bureau(calque: HWND) -> Option<(POINT, HWND)> {
    unsafe {
        let mut pt = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut pt) == 0 {
            return None;
        }
        let h = WindowFromPoint(pt);
        if h.is_null() {
            return None;
        }
        let racine = GetAncestor(h, GA_ROOT);
        let classe = crate::windows::classe_de(h);
        let bureau = h == calque
            || racine == calque
            || classe == "SysListView32"
            || classe == "SHELLDLL_DefView"
            || matches!(
                crate::windows::classe_de(racine).as_str(),
                "Progman" | "WorkerW"
            );
        bureau.then_some((pt, h))
    }
}

/// Une icône du bureau sous le point ? LVM_HITTEST sur la liste des icônes d'Explorer : la
/// structure vit dans SA mémoire (VirtualAllocEx), message avec délai (jamais bloqué par un
/// Explorer figé). Pas la liste des icônes (calque en mode fenêtre) : pas d'icône. None =
/// on ne sait pas (on n'attrape pas).
pub fn icone_sous(fenetre: HWND, pt: POINT) -> Option<bool> {
    let liste = match crate::windows::classe_de(fenetre).as_str() {
        "SysListView32" => fenetre,
        // Le cadre de la liste des icônes : sa liste.
        "SHELLDLL_DefView" => unsafe {
            FindWindowExW(fenetre, null_mut(), large("SysListView32").as_ptr(), null())
        },
        // Le bureau sans sa liste (Explorer qui redémarre…) : on ne sait pas.
        "Progman" | "WorkerW" => null_mut(),
        // Notre calque (mode fenêtre) : pas d'icône.
        _ => return Some(false),
    };
    if liste.is_null() {
        return None;
    }
    unsafe {
        let mut p = pt;
        ScreenToClient(liste, &mut p);
        let mut pid = 0u32;
        GetWindowThreadProcessId(liste, &mut pid);
        let h = OpenProcess(
            PROCESS_VM_OPERATION | PROCESS_VM_READ | PROCESS_VM_WRITE,
            0,
            pid,
        );
        if h.is_null() {
            return None;
        }
        let taille = std::mem::size_of::<LVHITTESTINFO>();
        let dist: *mut c_void =
            VirtualAllocEx(h, null(), taille, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
        let mut res = None;
        if !dist.is_null() {
            let info = LVHITTESTINFO {
                pt: p,
                flags: 0,
                iItem: -1,
                iSubItem: 0,
                iGroup: 0,
            };
            if WriteProcessMemory(
                h,
                dist,
                &info as *const LVHITTESTINFO as *const c_void,
                taille,
                null_mut(),
            ) != 0
            {
                let mut r = 0usize;
                if SendMessageTimeoutW(
                    liste,
                    LVM_HITTEST,
                    0,
                    dist as LPARAM,
                    SMTO_ABORTIFHUNG | SMTO_BLOCK,
                    50,
                    &mut r,
                ) != 0
                {
                    // Index de l'icône sous le point, -1 si aucune.
                    res = Some(r as isize as i32 >= 0);
                }
            }
            VirtualFreeEx(h, dist, 0, MEM_RELEASE);
        }
        CloseHandle(h);
        res
    }
}

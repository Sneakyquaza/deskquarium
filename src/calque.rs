//! Windows : le calque du fond d'écran est une fenêtre Win32 à nous (pas la fenêtre eframe),
//! dessinée en GDI sur le processeur : fond d'écran (image de chaque écran) puis toile de
//! l'aquarium agrandie au plus proche. Son propre fil et sa propre boucle de messages
//! (WM_PAINT : tout ce qui est demandé est recomposé) ; l'appli lui envoie chaque image
//! (`presenter`) : v0.12, seules les cases changées sont recomposées (`compo.rs`) et copiées,
//! et rien n'est copié sur un écran recouvert (elles attendent qu'il se découvre).
//! Raison (v0.4, Windows 11 25H2) : la fenêtre OpenGL d'eframe incrustée dans Progman ne se
//! voyait jamais et winit gardait sa taille d'avant (640x360).

use std::ffi::c_void;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

/// Image à montrer : toile (cases envoyées et à envoyer) et fond d'écran.
#[derive(Default)]
struct Image {
    compo: crate::compo::Compo,
    /// Fond d'écran (BGRA, taille du calque) ; remplacé quand les écrans changent.
    fond: Vec<u32>,
    fl: i32,
    fh: i32,
    /// Écrans recouverts (coordonnées du calque) : rien n'y est copié.
    caches: Vec<crate::compo::Rect>,
    /// Morceau composé en cours de copie.
    tampon: Vec<u32>,
}

#[derive(Default)]
struct Partage {
    image: Mutex<Image>,
    /// Test d'incrustation : bandes de couleur et nom de la méthode jusqu'à l'instant donné.
    test: Mutex<Option<(Instant, String)>>,
}

static PARTAGE: OnceLock<Arc<Partage>> = OnceLock::new();
static FENETRE: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

pub struct Calque {
    pub hwnd: HWND,
}

fn large(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

impl Calque {
    /// Crée la fenêtre du calque (cachée, sans parent) dans son propre fil.
    pub fn creer() -> Result<Calque, String> {
        let _ = PARTAGE.set(Arc::new(Partage::default()));
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || unsafe {
            let hinst = GetModuleHandleW(null());
            let classe = large("DeskquariumCalque");
            let wc = WNDCLASSW {
                lpfnWndProc: Some(proc_calque),
                hInstance: hinst,
                lpszClassName: classe.as_ptr(),
                hCursor: LoadCursorW(null_mut(), IDC_ARROW),
                ..std::mem::zeroed()
            };
            RegisterClassW(&wc);
            let h = CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                classe.as_ptr(),
                large("Deskquarium (fond)").as_ptr(),
                WS_POPUP,
                0,
                0,
                64,
                64,
                null_mut(),
                null_mut(),
                hinst,
                null(),
            );
            FENETRE.store(h, Ordering::Release);
            let _ = tx.send(h as usize);
            if h.is_null() {
                return;
            }
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        });
        let h = rx
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| "calque du fond : fil sans réponse".to_string())? as HWND;
        if h.is_null() {
            return Err(format!(
                "calque du fond : CreateWindowEx a échoué ({})",
                unsafe { windows_sys::Win32::Foundation::GetLastError() }
            ));
        }
        crate::journal::ecrire(&format!("calque GDI créé : {h:?}"));
        Ok(Calque { hwnd: h })
    }

    /// Nouvelle image de l'aquarium (toile RGBA prémultiplié agrandie `k` fois à l'écran) :
    /// les cases changées sont composées et copiées tout de suite sur la fenêtre (GetDC, depuis
    /// le fil de l'appli), sauf sur les écrans recouverts et pendant la mire.
    pub fn presenter(&self, toile: &crate::toile::Toile, k: usize) {
        let Some(p) = PARTAGE.get() else {
            return;
        };
        let test = p
            .test
            .lock()
            .ok()
            .and_then(|t| t.clone())
            .is_some_and(|(fin, _)| Instant::now() < fin);
        let Ok(mut im) = p.image.lock() else {
            return;
        };
        im.compo.recevoir(toile, (k / crate::toile::FIN).max(1));
        if test {
            return;
        }
        unsafe {
            let mut r: RECT = std::mem::zeroed();
            GetClientRect(self.hwnd, &mut r);
            let caches = im.caches.clone();
            let rects = im
                .compo
                .rectangles(r.right - r.left, r.bottom - r.top, &caches);
            if rects.is_empty() {
                return;
            }
            let dc = GetDC(self.hwnd);
            if dc.is_null() {
                return;
            }
            for rc in rects {
                copier(&mut im, dc, rc);
            }
            ReleaseDC(self.hwnd, dc);
        }
    }

    /// Fond d'écran sous l'aquarium (taille du calque).
    pub fn fond(&self, img: &image::RgbaImage) {
        if let Some(p) = PARTAGE.get() {
            if let Ok(mut im) = p.image.lock() {
                im.fl = img.width() as i32;
                im.fh = img.height() as i32;
                im.fond = img
                    .pixels()
                    .map(|c| 0xff00_0000 | (c[0] as u32) << 16 | (c[1] as u32) << 8 | c[2] as u32)
                    .collect();
                im.compo.tout_salir();
            }
        }
        unsafe { InvalidateRect(self.hwnd, null(), 0) };
    }

    /// Écrans recouverts (coordonnées du calque) : on n'y copie plus rien ; découverts, leurs
    /// cases en retard partent à l'image suivante.
    pub fn cacher(&self, caches: Vec<crate::compo::Rect>) {
        if let Some(p) = PARTAGE.get() {
            if let Ok(mut im) = p.image.lock() {
                im.caches = caches;
            }
        }
    }

    /// Bandes de couleur vives et nom de la méthode pendant 5 s : se voit-il ?
    pub fn tester(&self, nom: &str) {
        if let Some(p) = PARTAGE.get() {
            if let Ok(mut t) = p.test.lock() {
                *t = Some((Instant::now() + Duration::from_secs(5), nom.to_string()));
            }
        }
        crate::journal::ecrire(&format!("test d'incrustation (couleurs) : {nom}"));
        unsafe { InvalidateRect(self.hwnd, null(), 0) };
    }

    pub fn detruire(&self) {
        unsafe { PostMessageW(self.hwnd, WM_CLOSE, 0, 0) };
    }
}

fn bmi(l: i32, h: i32) -> BITMAPINFO {
    let mut b: BITMAPINFO = unsafe { std::mem::zeroed() };
    b.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    b.bmiHeader.biWidth = l;
    // Hauteur négative : lignes de haut en bas.
    b.bmiHeader.biHeight = -h;
    b.bmiHeader.biPlanes = 1;
    b.bmiHeader.biBitCount = 32;
    b.bmiHeader.biCompression = BI_RGB;
    b
}

/// Compose le rectangle `rc` (coordonnées du calque) et le copie sur `dc`.
unsafe fn copier(im: &mut Image, dc: HDC, rc: crate::compo::Rect) {
    unsafe {
        let Image {
            compo,
            fond,
            fl,
            fh,
            tampon,
            ..
        } = im;
        compo.composer(fond, *fl as usize, *fh as usize, rc, tampon);
        let b = bmi(rc.l, rc.h);
        StretchDIBits(
            dc,
            rc.x,
            rc.y,
            rc.l,
            rc.h,
            0,
            0,
            rc.l,
            rc.h,
            tampon.as_ptr() as *const c_void,
            &b,
            DIB_RGB_COLORS,
            SRCCOPY,
        );
    }
}

/// WM_PAINT : la zone demandée est recomposée (fenêtre découverte, fond changé, fin de la
/// mire) ; la mire se dessine dans un tampon (pas de scintillement).
unsafe fn peindre(h: HWND) {
    unsafe {
        let mut ps: PAINTSTRUCT = std::mem::zeroed();
        let dc = BeginPaint(h, &mut ps);
        let mut r: RECT = std::mem::zeroed();
        GetClientRect(h, &mut r);
        let (l, ht) = (r.right - r.left, r.bottom - r.top);
        let test = PARTAGE
            .get()
            .and_then(|p| p.test.lock().ok().and_then(|t| t.clone()))
            .filter(|(fin, _)| Instant::now() < *fin);
        if l > 0 && ht > 0 {
            if let Some((_, nom)) = test {
                let tampon = CreateCompatibleDC(dc);
                let bmp = CreateCompatibleBitmap(dc, l, ht);
                let ancien = SelectObject(tampon, bmp);
                bandes(tampon, l, ht, &nom);
                BitBlt(dc, 0, 0, l, ht, tampon, 0, 0, SRCCOPY);
                SelectObject(tampon, ancien);
                DeleteObject(bmp);
                DeleteDC(tampon);
                // Se redessine jusqu'à la fin du test, même sans nouvelle image.
                InvalidateRect(h, null(), 0);
            } else if let Some(p) = PARTAGE.get() {
                if let Ok(mut im) = p.image.lock() {
                    let z = ps.rcPaint;
                    let (x0, y0) = (z.left.max(0), z.top.max(0));
                    let (x1, y1) = (z.right.min(l), z.bottom.min(ht));
                    if x1 > x0 && y1 > y0 {
                        // Par bandes de 64 lignes : le tampon reste petit.
                        let mut y = y0;
                        while y < y1 {
                            let hh = (y1 - y).min(64);
                            let rc = crate::compo::Rect {
                                x: x0,
                                y,
                                l: x1 - x0,
                                h: hh,
                            };
                            copier(&mut im, dc, rc);
                            y += hh;
                        }
                    }
                }
            }
        }
        EndPaint(h, &ps);
    }
}

/// Mire de test : bandes verticales vives et grand texte.
unsafe fn bandes(dc: HDC, l: i32, h: i32, nom: &str) {
    unsafe {
        let couleurs = [0x0000ff, 0x00ffff, 0x00ff00, 0xffff00, 0xff0000, 0xff00ff];
        let n = couleurs.len() as i32 * 4;
        for i in 0..n {
            let r = RECT {
                left: l * i / n,
                top: 0,
                right: l * (i + 1) / n,
                bottom: h,
            };
            let b = CreateSolidBrush(couleurs[i as usize % couleurs.len()]);
            FillRect(dc, &r, b);
            DeleteObject(b);
        }
        let police = CreateFontW(
            h / 8,
            0,
            0,
            0,
            FW_BOLD as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            0,
            0,
            0,
            0,
            large("Segoe UI").as_ptr(),
        );
        let ancienne = SelectObject(dc, police);
        SetBkMode(dc, OPAQUE as i32);
        SetBkColor(dc, 0);
        SetTextColor(dc, 0xffffff);
        let mut r = RECT {
            left: 0,
            top: 0,
            right: l,
            bottom: h,
        };
        let t: Vec<u16> = format!("DESKQUARIUM VISIBLE : {nom}")
            .encode_utf16()
            .collect();
        DrawTextW(
            dc,
            t.as_ptr(),
            t.len() as i32,
            &mut r,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
        SelectObject(dc, ancienne);
        DeleteObject(police);
    }
}

unsafe extern "system" fn proc_calque(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                peindre(h);
                0
            }
            WM_ERASEBKGND => 1,
            // Jamais de clic ni de focus : le bureau et ses icônes restent au-dessus.
            WM_NCHITTEST => HTTRANSPARENT as LRESULT,
            WM_MOUSEACTIVATE => MA_NOACTIVATE as LRESULT,
            WM_CLOSE => {
                DestroyWindow(h);
                0
            }
            WM_DESTROY => {
                FENETRE.store(null_mut(), Ordering::Release);
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(h, msg, w, l),
        }
    }
}

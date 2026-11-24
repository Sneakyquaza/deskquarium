//! Windows : le calque du fond d'écran est une fenêtre Win32 à nous (pas la fenêtre eframe),
//! dessinée en GDI sur le processeur : fond d'écran (image de chaque écran) puis toile de
//! l'aquarium agrandie au plus proche (AlphaBlend, prémultiplié). Son propre fil et sa propre
//! boucle de messages ; l'appli lui envoie chaque image (`presenter`).
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

/// Image à montrer : toile (BGRA prémultiplié, pixels fins) et son agrandissement entier.
#[derive(Default)]
struct Image {
    toile: Vec<u8>,
    tl: i32,
    th: i32,
    k: i32,
    /// Fond d'écran (BGRA, taille du calque) ; remplacé quand les écrans changent.
    fond: Vec<u8>,
    fl: i32,
    fh: i32,
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
            let classe = large("AquariumCalque");
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
                large("Aquarium (fond)").as_ptr(),
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

    /// Nouvelle image de l'aquarium (toile RGBA prémultiplié agrandie `k` fois à l'écran).
    pub fn presenter(&self, toile: &crate::toile::Toile, k: usize) {
        if let Some(p) = PARTAGE.get() {
            if let Ok(mut im) = p.image.lock() {
                im.tl = toile.lf() as i32;
                im.th = toile.hf() as i32;
                im.k = k as i32;
                im.toile.clear();
                im.toile
                    .extend(toile.px.iter().flat_map(|c| [c[2], c[1], c[0], c[3]]));
            }
        }
        unsafe { InvalidateRect(self.hwnd, null(), 0) };
    }

    /// Fond d'écran sous l'aquarium (taille du calque).
    pub fn fond(&self, img: &image::RgbaImage) {
        if let Some(p) = PARTAGE.get() {
            if let Ok(mut im) = p.image.lock() {
                im.fl = img.width() as i32;
                im.fh = img.height() as i32;
                im.fond = img.pixels().flat_map(|c| [c[2], c[1], c[0], 255]).collect();
            }
        }
        unsafe { InvalidateRect(self.hwnd, null(), 0) };
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

/// Dessine tout dans un tampon (pas de scintillement) puis le copie sur la fenêtre.
unsafe fn peindre(h: HWND) {
    unsafe {
        let mut ps: PAINTSTRUCT = std::mem::zeroed();
        let dc = BeginPaint(h, &mut ps);
        let mut r: RECT = std::mem::zeroed();
        GetClientRect(h, &mut r);
        let (l, ht) = (r.right - r.left, r.bottom - r.top);
        if l > 0 && ht > 0 {
            let tampon = CreateCompatibleDC(dc);
            let bmp = CreateCompatibleBitmap(dc, l, ht);
            let ancien = SelectObject(tampon, bmp);
            SetStretchBltMode(tampon, COLORONCOLOR);
            let test = PARTAGE
                .get()
                .and_then(|p| p.test.lock().ok().and_then(|t| t.clone()))
                .filter(|(fin, _)| Instant::now() < *fin);
            if let Some((_, nom)) = test {
                bandes(tampon, l, ht, &nom);
                // Se redessine jusqu'à la fin du test, même sans nouvelle image.
                InvalidateRect(h, null(), 0);
            } else if let Some(p) = PARTAGE.get() {
                if let Ok(im) = p.image.lock() {
                    if im.fond.len() == (im.fl * im.fh * 4) as usize && im.fl > 0 {
                        let b = bmi(im.fl, im.fh);
                        StretchDIBits(
                            tampon,
                            0,
                            0,
                            im.fl,
                            im.fh,
                            0,
                            0,
                            im.fl,
                            im.fh,
                            im.fond.as_ptr() as *const c_void,
                            &b,
                            DIB_RGB_COLORS,
                            SRCCOPY,
                        );
                    } else {
                        let noir = CreateSolidBrush(0);
                        FillRect(tampon, &r, noir);
                        DeleteObject(noir);
                    }
                    if im.toile.len() == (im.tl * im.th * 4) as usize && im.tl > 0 {
                        // Toile dans une section DIB (AlphaBlend demande un DC source).
                        let b = bmi(im.tl, im.th);
                        let mut bits: *mut c_void = null_mut();
                        let dib =
                            CreateDIBSection(tampon, &b, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
                        if !dib.is_null() && !bits.is_null() {
                            std::ptr::copy_nonoverlapping(
                                im.toile.as_ptr(),
                                bits as *mut u8,
                                im.toile.len(),
                            );
                            let src = CreateCompatibleDC(tampon);
                            let a = SelectObject(src, dib);
                            let f = BLENDFUNCTION {
                                BlendOp: AC_SRC_OVER as u8,
                                BlendFlags: 0,
                                SourceConstantAlpha: 255,
                                AlphaFormat: AC_SRC_ALPHA as u8,
                            };
                            let k = (im.k / crate::toile::FIN as i32).max(1);
                            AlphaBlend(
                                tampon,
                                0,
                                0,
                                im.tl * k,
                                im.th * k,
                                src,
                                0,
                                0,
                                im.tl,
                                im.th,
                                f,
                            );
                            SelectObject(src, a);
                            DeleteDC(src);
                            DeleteObject(dib);
                        }
                    }
                }
            }
            BitBlt(dc, 0, 0, l, ht, tampon, 0, 0, SRCCOPY);
            SelectObject(tampon, ancien);
            DeleteObject(bmp);
            DeleteDC(tampon);
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
        let t: Vec<u16> = format!("AQUARIUM VISIBLE : {nom}").encode_utf16().collect();
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

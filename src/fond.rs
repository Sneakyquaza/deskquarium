//! Fond d'écran sous l'aquarium. La fenêtre de rendu enfant de WorkerW n'a pas de
//! transparence simple : on relit le fond d'écran actuel de Windows et on le dessine
//! nous-mêmes dessous (voir `windows.rs`). Hors Windows : image `--fond` ou fond factice.

use image::RgbaImage;

/// Fond factice (crépuscule sur des montagnes) pour les tests et captures Linux.
pub fn factice(l: u32, h: u32) -> RgbaImage {
    let mut img = RgbaImage::new(l, h);
    let lerp = |a: [f32; 3], b: [f32; 3], t: f32| {
        [0, 1, 2].map(|k| a[k] + (b[k] - a[k]) * t.clamp(0.0, 1.0))
    };
    let ciel_haut = [0x22 as f32, 0x2c as f32, 0x5c as f32];
    let ciel_bas = [0xf2 as f32, 0x9c as f32, 0x78 as f32];
    let mont1 = [0x4a as f32, 0x44 as f32, 0x78 as f32];
    let mont2 = [0x2a as f32, 0x26 as f32, 0x50 as f32];
    let lac = [0x1a as f32, 0x1c as f32, 0x3c as f32];
    let horizon = h as f32 * 0.62;
    for y in 0..h {
        for x in 0..l {
            let (xf, yf) = (x as f32 / l as f32, y as f32);
            let mut c = lerp(ciel_haut, ciel_bas, yf / horizon);
            // Soleil.
            let (sx, sy) = (l as f32 * 0.68, horizon - h as f32 * 0.08);
            let d = ((x as f32 - sx).powi(2) + (yf - sy).powi(2)).sqrt() / (h as f32 * 0.07);
            if d < 1.0 {
                c = lerp(c, [255.0, 226.0, 170.0], 1.0);
            } else {
                c = lerp(
                    c,
                    [255.0, 200.0, 150.0],
                    (1.0 - (d - 1.0) / 4.0).max(0.0) * 0.35,
                );
            }
            let m1 = horizon
                - h as f32 * (0.18 + 0.08 * (xf * 7.0).sin() + 0.05 * (xf * 17.0 + 1.0).sin());
            let m2 = horizon
                - h as f32 * (0.08 + 0.05 * (xf * 11.0 + 2.0).sin() + 0.03 * (xf * 29.0).sin());
            if yf > m1 {
                c = lerp(mont1, mont2, (yf - m1) / (horizon - m1 + 1.0) * 0.6);
            }
            if yf > m2 {
                c = mont2;
            }
            if yf > horizon {
                c = lerp(lac, mont2, ((yf - horizon) / 40.0).sin().abs() * 0.3);
            }
            img.put_pixel(x, y, image::Rgba([c[0] as u8, c[1] as u8, c[2] as u8, 255]));
        }
    }
    img
}

/// Redimensionne un fond pour couvrir `l`x`h` (mode « Remplir » de Windows : recadré au centre).
pub fn couvrir(img: &RgbaImage, l: u32, h: u32) -> RgbaImage {
    let e = (l as f32 / img.width() as f32).max(h as f32 / img.height() as f32);
    let (nl, nh) = (
        ((img.width() as f32 * e).ceil() as u32).max(l),
        ((img.height() as f32 * e).ceil() as u32).max(h),
    );
    let grand = image::imageops::resize(img, nl, nh, image::imageops::FilterType::Triangle);
    image::imageops::crop_imm(&grand, (nl - l) / 2, (nh - h) / 2, l, h).to_image()
}

/// Compose le fond (pleine résolution) et la toile agrandie `e` fois au plus proche.
pub fn composer(fond: &RgbaImage, toile: &crate::toile::Toile, e: usize) -> RgbaImage {
    let mut out = fond.clone();
    let (l, h) = (out.width() as usize, out.height() as usize);
    for y in 0..h {
        // `e` : pixels écran par pixel monde ; la toile est en pixels fins.
        let k = (e / crate::toile::FIN).max(1);
        let ty = y / k;
        if ty >= toile.hf() {
            break;
        }
        for x in 0..l {
            let tx = x / k;
            if tx >= toile.lf() {
                break;
            }
            let s = toile.px[ty * toile.lf() + tx];
            if s[3] == 0 {
                continue;
            }
            let d = out.get_pixel_mut(x as u32, y as u32);
            let inv = 255 - s[3] as u16;
            for (dk, sk) in d.0.iter_mut().zip(s).take(3) {
                *dk = (sk as u16 + (*dk as u16 * inv + 127) / 255).min(255) as u8;
            }
        }
    }
    out
}

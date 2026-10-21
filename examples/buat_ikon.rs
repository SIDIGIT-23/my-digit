//! Membuat ikon aplikasi dari `assets/logo.png`:
//!
//! - `assets/icon-macos.png` (1024 px): ubin putih "squircle" bergaya macOS
//!   dengan margin dan bayangan sesuai grid ikon Apple, untuk `.icns` dan Dock.
//! - `assets/icon-windows.png` (256 px): ubin yang sama tanpa margin lebar,
//!   untuk `.ico` dan taskbar Windows.
//!
//! Jalankan ulang setelah mengganti logo:
//! `cargo run --example buat_ikon`

use image::imageops::{self, FilterType};
use image::{GrayImage, Luma, Rgba, RgbaImage};

/// Kepangkatan superelips; 5 mendekati sudut "continuous corner" Apple.
const N: f32 = 5.0;

fn main() {
    let logo = image::open("assets/logo.png")
        .expect("assets/logo.png")
        .into_rgba8();
    let logo = pangkas_transparan(&logo);

    // Grid ikon macOS: kanvas 1024, badan ubin 824 px di tengah.
    let mac = ubin(&logo, 1024, 824, 0.70, true);
    mac.save("assets/icon-macos.png").unwrap();

    // Windows: ubin hampir memenuhi kanvas, tanpa bayangan.
    let win = ubin(&logo, 1024, 960, 0.74, false);
    imageops::resize(&win, 256, 256, FilterType::Lanczos3)
        .save("assets/icon-windows.png")
        .unwrap();

    println!("Tersimpan: assets/icon-macos.png, assets/icon-windows.png");
}

/// Ubin putih bersudut superelips berisi logo.
fn ubin(logo: &RgbaImage, kanvas: u32, badan: u32, porsi_logo: f32, bayangan: bool) -> RgbaImage {
    let mut out = RgbaImage::new(kanvas, kanvas);
    let pusat = kanvas as f32 / 2.0;
    let jari = badan as f32 / 2.0;
    let topeng = topeng_superelips(kanvas, pusat, pusat, jari);

    if bayangan {
        // Bayangan lembut di bawah ubin.
        let mut geser = GrayImage::new(kanvas, kanvas);
        let dy = (kanvas as f32 * 0.012) as u32;
        for y in 0..kanvas - dy {
            for x in 0..kanvas {
                geser.put_pixel(x, y + dy, *topeng.get_pixel(x, y));
            }
        }
        let kabur = imageops::blur(&geser, kanvas as f32 * 0.014);
        for (x, y, p) in out.enumerate_pixels_mut() {
            let a = kabur.get_pixel(x, y).0[0] as f32 * 0.30;
            *p = Rgba([0, 0, 0, a as u8]);
        }
    }

    // Isi ubin: putih dengan gradasi tipis ke abu-abu terang di bawah,
    // dan garis tepi sangat halus agar tetap terlihat di latar putih.
    let tepi = topeng_superelips(kanvas, pusat, pusat, jari - kanvas as f32 * 0.002);
    let atas = jari - pusat; // negatif: koordinat y relatif atas badan
    for (x, y, p) in out.enumerate_pixels_mut() {
        let a = topeng.get_pixel(x, y).0[0] as f32 / 255.0;
        if a == 0.0 {
            continue;
        }
        let t = ((y as f32 + atas) / badan as f32).clamp(0.0, 1.0);
        let abu = 255.0 - 11.0 * t;
        let garis = a - tepi.get_pixel(x, y).0[0] as f32 / 255.0;
        let warna = abu * (1.0 - garis * 0.10);
        *p = campur(
            *p,
            Rgba([
                warna as u8,
                warna as u8,
                (warna + 2.0 * t).min(255.0) as u8,
                255,
            ]),
            a,
        );
    }

    // Logo di tengah ubin.
    let sisi = (badan as f32 * porsi_logo) as u32;
    let (lw, lh) = logo.dimensions();
    let skala = sisi as f32 / lw.max(lh) as f32;
    let (w, h) = ((lw as f32 * skala) as u32, (lh as f32 * skala) as u32);
    let kecil = imageops::resize(logo, w, h, FilterType::Lanczos3);
    let (ox, oy) = ((kanvas - w) / 2, (kanvas - h) / 2);
    for (x, y, p) in kecil.enumerate_pixels() {
        let tujuan = out.get_pixel_mut(ox + x, oy + y);
        *tujuan = campur(
            *tujuan,
            Rgba([p.0[0], p.0[1], p.0[2], 255]),
            p.0[3] as f32 / 255.0,
        );
    }
    out
}

/// Topeng superelips |x|^N + |y|^N <= 1 dengan anti-alias 4×4 sampel.
fn topeng_superelips(kanvas: u32, cx: f32, cy: f32, jari: f32) -> GrayImage {
    let mut m = GrayImage::new(kanvas, kanvas);
    for (x, y, p) in m.enumerate_pixels_mut() {
        let mut kena = 0;
        for sy in 0..4 {
            for sx in 0..4 {
                let u = ((x as f32 + (sx as f32 + 0.5) / 4.0) - cx).abs() / jari;
                let v = ((y as f32 + (sy as f32 + 0.5) / 4.0) - cy).abs() / jari;
                if u.powf(N) + v.powf(N) <= 1.0 {
                    kena += 1;
                }
            }
        }
        *p = Luma([(kena * 255 / 16) as u8]);
    }
    m
}

/// Tumpuk `atas` di atas `bawah` dengan opasitas `a` (0..1).
fn campur(bawah: Rgba<u8>, atas: Rgba<u8>, a: f32) -> Rgba<u8> {
    let ab = bawah.0[3] as f32 / 255.0;
    let ao = a + ab * (1.0 - a);
    if ao == 0.0 {
        return Rgba([0, 0, 0, 0]);
    }
    let k = |i: usize| ((atas.0[i] as f32 * a + bawah.0[i] as f32 * ab * (1.0 - a)) / ao) as u8;
    Rgba([k(0), k(1), k(2), (ao * 255.0) as u8])
}

/// Buang tepi transparan agar logo bisa dipusatkan dengan tepat.
fn pangkas_transparan(img: &RgbaImage) -> RgbaImage {
    let (w, h) = img.dimensions();
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for (x, y, p) in img.enumerate_pixels() {
        if p.0[3] > 8 {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
    }
    if x1 < x0 {
        return img.clone();
    }
    imageops::crop_imm(img, x0, y0, x1 - x0 + 1, y1 - y0 + 1).to_image()
}

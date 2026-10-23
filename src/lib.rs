//! My Digit — Aplikasi Bendahara Keuangan Lembaga.
//! made by SIDIGIT-23 Proyek IT SMP Negeri 23 Dumai
//!
//! Satu kode untuk desktop (Windows, macOS) dan Android. Desktop masuk
//! lewat [`jalankan`] di `main.rs`; Android lewat `android_main`.

mod berkas;
mod branding;
mod db;
mod excel;
mod format;
mod instansi;
mod laporan;
mod model;
mod pdf;
mod ui;

use eframe::egui;

/// Jalankan aplikasi desktop.
pub fn jalankan() -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(branding::APP_NAME)
        .with_app_id("my-digit")
        .with_icon(branding::window_icon())
        .with_inner_size([1180.0, 760.0])
        .with_min_inner_size([360.0, 560.0]);
    // Pengembangan: uji tata letak ponsel di desktop, mis. MY_DIGIT_LEBAR=390.
    if let Some(lebar) = std::env::var("MY_DIGIT_LEBAR")
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
    {
        viewport = viewport.with_inner_size([lebar, 844.0]);
    }
    // macOS: judul jendela transparan agar sidebar menyatu dengan tombol jendela,
    // seperti aplikasi bawaan Apple.
    if cfg!(target_os = "macos") {
        viewport = viewport
            .with_fullsize_content_view(true)
            .with_titlebar_shown(false)
            .with_title_shown(false);
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        branding::APP_NAME,
        options,
        Box::new(|cc| Ok(Box::new(ui::MyDigitApp::new(cc)))),
    )
}

/// Titik masuk Android (dipanggil NativeActivity).
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    // Data disimpan di folder internal aplikasi (lihat `db::folder_data`).
    if let Some(dir) = app.internal_data_path() {
        // SAFETY: dipanggil sekali di awal, sebelum ada thread lain.
        unsafe { std::env::set_var("MY_DIGIT_DATA_DIR", dir) };
    }
    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };
    let _ = eframe::run_native(
        branding::APP_NAME,
        options,
        Box::new(|cc| Ok(Box::new(ui::MyDigitApp::new(cc)))),
    );
}

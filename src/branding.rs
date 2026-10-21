//! Identitas aplikasi My Digit.
//!
//! Semua teks identitas dikumpulkan di sini supaya judul jendela, splash,
//! halaman "Tentang Aplikasi", dan footer PDF selalu seragam.

/// Nama aplikasi, dipakai juga sebagai judul jendela.
pub const APP_NAME: &str = "My Digit";

/// Versi diambil dari Cargo.toml.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Deskripsi singkat aplikasi.
pub const APP_TAGLINE: &str = "Aplikasi Bendahara Keuangan Lembaga";

/// Nama tim pembuat.
pub const MAKER: &str = "SIDIGIT-23";

/// Teks kredit di splash dan halaman "Tentang Aplikasi".
pub const CREDIT: &str = "made by SIDIGIT-23 Proyek IT SMP Negeri 23 Dumai";

/// Footer kecil di setiap halaman laporan PDF.
pub const PDF_FOOTER: &str = "Dibuat dengan My Digit — SIDIGIT-23";

/// Logo aplikasi My Digit (`assets/logo.png`), ditanam ke dalam binary.
///
/// Logo ini milik aplikasi dan terpisah dari logo lembaga pengguna, yang
/// diatur di profil lembaga (lihat [`crate::lembaga::ProfilLembaga`]).
pub const LOGO_PNG: &[u8] = include_bytes!("../assets/logo.png");

/// Ikon jendela dari logo aplikasi.
pub fn window_icon() -> eframe::egui::IconData {
    let img = image::load_from_memory(LOGO_PNG)
        .expect("assets/logo.png harus berupa PNG yang valid")
        .into_rgba8();
    let (width, height) = img.dimensions();
    eframe::egui::IconData {
        rgba: img.into_raw(),
        width,
        height,
    }
}

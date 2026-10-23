//! Simpan dan buka berkas sesuai platform.
//!
//! Desktop memakai dialog bawaan sistem. Android belum punya dialog berkas,
//! jadi ekspor disimpan langsung ke `Download/My Digit` (atau folder data
//! aplikasi bila folder Download tidak bisa ditulis).

use std::path::{Path, PathBuf};

/// Pengguna dapat memilih berkas dari perangkat (dialog "Buka").
pub const BISA_MEMILIH: bool = cfg!(not(target_os = "android"));

/// Tentukan lokasi simpan untuk berkas `nama`. `None` bila dibatalkan.
#[cfg(not(target_os = "android"))]
pub fn pilih_simpan(nama: &str, label: &str, ekstensi: &[&str]) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_file_name(nama)
        .add_filter(label, ekstensi)
        .save_file()
}

#[cfg(target_os = "android")]
pub fn pilih_simpan(nama: &str, _label: &str, _ekstensi: &[&str]) -> Option<PathBuf> {
    Some(folder_unduhan().join(nama))
}

/// Pilih berkas untuk dibuka. Di Android selalu `None` (belum didukung).
#[cfg(not(target_os = "android"))]
pub fn pilih_buka(label: &str, ekstensi: &[&str]) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter(label, ekstensi)
        .pick_file()
}

#[cfg(target_os = "android")]
pub fn pilih_buka(_label: &str, _ekstensi: &[&str]) -> Option<PathBuf> {
    None
}

/// Buka berkas atau folder dengan aplikasi bawaan sistem (desktop saja).
pub fn buka(path: &Path) {
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(path).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("explorer").arg(path).spawn();
    #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
    let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    #[cfg(target_os = "android")]
    let _ = path;
}

/// Folder Download publik; bila tidak bisa ditulis, folder data aplikasi.
#[cfg(target_os = "android")]
fn folder_unduhan() -> PathBuf {
    let publik = PathBuf::from("/storage/emulated/0/Download/My Digit");
    let uji = publik.join(".uji-tulis");
    if std::fs::create_dir_all(&publik).is_ok() && std::fs::write(&uji, b"").is_ok() {
        let _ = std::fs::remove_file(&uji);
        return publik;
    }
    let cadangan = crate::db::folder_data().join("ekspor");
    let _ = std::fs::create_dir_all(&cadangan);
    cadangan
}

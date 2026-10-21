//! Menanam ikon dan info versi ke my-digit.exe saat build di Windows.

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "My Digit");
        res.set(
            "FileDescription",
            "My Digit — Aplikasi Bendahara Keuangan Lembaga",
        );
        res.set("CompanyName", "SIDIGIT-23 — Proyek IT SMP Negeri 23 Dumai");
        res.set(
            "LegalCopyright",
            "© SIDIGIT-23 — Proyek IT SMP Negeri 23 Dumai",
        );
        res.compile().expect("gagal menanam ikon Windows");
    }
}

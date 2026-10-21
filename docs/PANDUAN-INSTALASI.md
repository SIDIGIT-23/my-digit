# Panduan Instalasi & Pembuatan Installer — My Digit

My Digit berjalan di **Windows 10/11 (64-bit)** dan **macOS 11 Big Sur atau
lebih baru** (Apple Silicon maupun Intel).

## Untuk pengguna

### Windows
1. Unduh `my-digit_x.y.z_x64-setup.exe`.
2. Klik dua kali, lalu ikuti petunjuk. Installer tidak memerlukan hak
   administrator karena memasang aplikasi untuk pengguna saat ini.
3. Jika muncul **"Windows melindungi PC Anda"** (SmartScreen), klik
   **Info selengkapnya → Tetap jalankan**. Peringatan ini muncul karena
   installer belum ditandatangani sertifikat code signing.
4. Buka **My Digit** dari menu Start.

### macOS
1. Unduh `My Digit_x.y.z_aarch64.dmg` (Mac M1/M2/M3/M4) atau `…_x64.dmg`
   (Mac Intel).
2. Buka berkas `.dmg`, lalu seret **My Digit** ke folder **Applications**.
3. Pada pembukaan pertama, klik kanan ikon **My Digit → Buka → Buka**.
   Langkah ini diperlukan karena aplikasi belum dinotarisasi Apple.

### Lokasi data
Semua data tersimpan di satu berkas database:

| Sistem  | Lokasi                                                        |
| ------- | ------------------------------------------------------------- |
| Windows | `%APPDATA%\SIDIGIT-23\My Digit\data\my-digit.sqlite`           |
| macOS   | `~/Library/Application Support/id.SIDIGIT-23.My-Digit/my-digit.sqlite` |

Di folder yang sama ada folder `cadangan-otomatis/` berisi salinan pengaman
yang dibuat aplikasi sendiri: sebelum data lama diperbarui ke format baru, dan
sebelum data diganti lewat **Pulihkan dari Cadangan**. Salinan ini tidak dihapus
otomatis; hapus sendiri yang sudah tidak diperlukan.

Gunakan menu **Tampilan & Data → Cadangkan Data** secara berkala. Berkas
cadangan dari Windows dapat dipulihkan di macOS, dan sebaliknya. Simpan
cadangan di luar folder data (mis. Dokumen atau flashdisk).

### Memperbarui dari My Digit 0.2
Saat dibuka pertama kali, My Digit 0.3 memperbarui format data secara otomatis
setelah membuat salinan pengaman. Semua transaksi, kategori, dan profil tetap,
dan laporan tetap tercetak sama. Data yang sudah dipakai langsung masuk sebagai
Sekolah (atau Organisasi/Lembaga bila jabatan pimpinannya "Ketua"/"Ketua
Yayasan"); jenisnya bisa diubah di **Profil → Ubah…**. Berkas yang sudah
diperbarui tetap dapat dibuka oleh My Digit 0.2.

## Untuk pengembang

### Menjalankan dari kode sumber
```bash
cargo run
```

### Membuat installer
Installer dibuat dengan [cargo-packager](https://github.com/crabnebula-dev/cargo-packager)
dan harus dibuat **di sistem operasi tujuannya** (installer Windows di
Windows, `.dmg` di Mac).

```bash
cargo install cargo-packager --locked
cargo packager --release
```

Hasilnya ada di folder `dist/`:
- Windows: `my-digit_x.y.z_x64-setup.exe` (NSIS)
- macOS: `My Digit.app` dan `My Digit_x.y.z_<arsitektur>.dmg`

Pengaturan pemaketan ada di bagian `[package.metadata.packager]` pada
`Cargo.toml`. Ikon diambil dari `assets/icon.icns` (macOS) dan
`assets/icon.ico` (Windows). Kedua ikon dibuat dari `assets/logo.png`.

### Otomatis lewat GitHub Actions
Workflow `.github/workflows/rilis.yml` membuat installer Windows, macOS
Apple Silicon, dan macOS Intel sekaligus. Workflow berjalan saat tag versi
di-push, atau dapat dijalankan manual dari tab **Actions**.

```bash
git tag v0.2.0
```

```bash
git push origin v0.2.0
```

Hasil installer dapat diunduh dari halaman workflow tersebut (bagian *Artifacts*).

### Tanda tangan digital (opsional)
Agar peringatan SmartScreen/Gatekeeper tidak muncul:
- **Windows:** beli sertifikat code signing, lalu isi
  `[package.metadata.packager.windows]` (`certificate-thumbprint` atau
  `sign-command`).
- **macOS:** daftar Apple Developer Program, lalu isi `signing-identity` di
  `[package.metadata.packager.macos]` dan kredensial notarisasi.

### Alat bantu pengembangan
- Data contoh:
  `MY_DIGIT_DATA_DIR=target/demo cargo test buat_data_demo -- --ignored`
- Jalankan dengan data contoh, langsung ke halaman tertentu:
  `MY_DIGIT_DATA_DIR=target/demo MY_DIGIT_HALAMAN=laporan cargo run`
- Tangkapan layar otomatis (aplikasi menutup sendiri setelah menyimpan):
  tambahkan `MY_DIGIT_TANGKAP=target/layar.png`
- Contoh PDF dari data contoh:
  `MY_DIGIT_DATA_DIR=target/demo cargo test contoh_pdf -- --ignored`
  (hasil di `target/shots/`)

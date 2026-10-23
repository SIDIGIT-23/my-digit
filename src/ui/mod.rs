//! Antarmuka My Digit.

mod akun;
mod anggaran;
pub mod ikon;
mod kas;
mod laporan;
mod penyiapan;
mod profil;
mod ringkasan;
pub mod tema;
mod transaksi;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke, TextureHandle, Vec2};

use crate::berkas;
use crate::branding;
use crate::db::{Db, FOLDER_CADANGAN_OTOMATIS, InfoCadangan, folder_data};
use crate::format::{BULAN, hari_ini};
use crate::laporan::{Periode, Sumber};
use crate::model::*;
use crate::pdf::Gambar;
use tema::palet;

const LAMA_SPLASH: Duration = Duration::from_millis(2200);
const LAMA_PESAN: Duration = Duration::from_secs(4);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Halaman {
    Ringkasan,
    Transaksi,
    Anggaran,
    Laporan,
    PemeriksaanKas,
    Akun,
    Kategori,
    Profil,
    Pengaturan,
    Tentang,
    /// Ponsel: daftar menu yang tidak ada di bar tab.
    Lainnya,
}

/// Bar tab ponsel: (halaman, ikon, label).
const TAB_PONSEL: [ItemMenu; 4] = [
    (Halaman::Ringkasan, ikon::HOUSE, "Ringkasan"),
    (Halaman::Transaksi, ikon::RECEIPT, "Transaksi"),
    (Halaman::Laporan, ikon::FILE_TEXT, "Laporan"),
    (Halaman::Lainnya, ikon::DOTS_THREE_CIRCLE, "Lainnya"),
];

/// (halaman, ikon, label)
type ItemMenu = (Halaman, &'static str, &'static str);

const MENU: &[(&str, &[ItemMenu])] = &[
    (
        "Keuangan",
        &[
            (Halaman::Ringkasan, ikon::HOUSE, "Ringkasan"),
            (Halaman::Transaksi, ikon::RECEIPT, "Transaksi"),
            (Halaman::Anggaran, ikon::CHART_PIE_SLICE, "Anggaran"),
            (Halaman::Laporan, ikon::FILE_TEXT, "Laporan"),
            (Halaman::PemeriksaanKas, ikon::MONEY, "Pemeriksaan Kas"),
        ],
    ),
    (
        "Data Master",
        &[
            (Halaman::Akun, ikon::BANK, "Akun Kas & Bank"),
            (Halaman::Kategori, ikon::TAG, "Kategori"),
        ],
    ),
    (
        "Pengaturan",
        &[
            // Label dan ikon Profil mengikuti jenis ("Profil Sekolah", ...).
            (Halaman::Profil, ikon::BUILDINGS, "Profil"),
            (Halaman::Pengaturan, ikon::GEAR_SIX, "Tampilan & Data"),
            (Halaman::Tentang, ikon::INFO, "Tentang Aplikasi"),
        ],
    ),
];

/// Berkas cadangan yang menunggu konfirmasi pemulihan.
pub struct Pulih {
    path: PathBuf,
    info: Result<InfoCadangan, String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PilihanTema {
    Sistem,
    Terang,
    Gelap,
}

impl PilihanTema {
    fn kode(self) -> &'static str {
        match self {
            Self::Sistem => "sistem",
            Self::Terang => "terang",
            Self::Gelap => "gelap",
        }
    }
    fn dari_kode(s: &str) -> Self {
        match s {
            "terang" => Self::Terang,
            "gelap" => Self::Gelap,
            _ => Self::Sistem,
        }
    }
    fn terapkan(self, ctx: &egui::Context) {
        ctx.set_theme(match self {
            Self::Sistem => egui::ThemePreference::System,
            Self::Terang => egui::ThemePreference::Light,
            Self::Gelap => egui::ThemePreference::Dark,
        });
    }
}

pub struct Pesan {
    teks: String,
    galat: bool,
    sampai: Instant,
}

/// Pemilih periode bulanan yang dipakai beberapa halaman.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Bulan {
    pub tahun: i16,
    /// 0 = setahun penuh.
    pub bulan: i8,
}

impl Bulan {
    pub fn sekarang() -> Self {
        let d = hari_ini();
        Self {
            tahun: d.year(),
            bulan: d.month(),
        }
    }
    pub fn periode(self) -> Periode {
        if self.bulan == 0 {
            Periode::tahun(self.tahun)
        } else {
            Periode::bulan(self.tahun, self.bulan)
        }
    }
}

pub struct MyDigitApp {
    pub db: Option<Db>,
    pub akun: Vec<Akun>,
    pub kategori: Vec<Kategori>,
    pub transaksi: Vec<Transaksi>,
    pub profil: ProfilLembaga,
    pub instansi: PengaturanInstansi,
    pub halaman: Halaman,
    pub tema: PilihanTema,

    pub logo_app: TextureHandle,
    pub logo_lembaga: Option<TextureHandle>,
    splash_sampai: Option<Instant>,
    pesan: Option<Pesan>,
    galat_db: Option<String>,
    /// Pengembangan: simpan tangkapan layar ke berkas ini lalu keluar.
    tangkap: Option<(PathBuf, u32)>,
    pulih: Option<Pulih>,

    /// Layar pertama "Masuk sebagai"; `Some` selama penyiapan belum selesai.
    pub ui_penyiapan: Option<penyiapan::State>,

    pub ui_transaksi: transaksi::State,
    pub ui_akun: akun::State,
    pub ui_anggaran: anggaran::State,
    pub ui_laporan: laporan::State,
    pub ui_kas: kas::State,
    pub ui_profil: profil::State,
}

impl MyDigitApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        tema::pasang(&cc.egui_ctx);
        let logo_app = muat_tekstur(&cc.egui_ctx, "logo-app", branding::LOGO_PNG)
            .expect("logo aplikasi valid");

        let (db, galat_db) = match Db::buka_bawaan() {
            Ok(db) => (Some(db), None),
            Err(e) => (None, Some(e)),
        };
        let mut app = Self {
            db,
            akun: Vec::new(),
            kategori: Vec::new(),
            transaksi: Vec::new(),
            profil: ProfilLembaga::default(),
            instansi: PengaturanInstansi::default(),
            halaman: Halaman::Ringkasan,
            tema: PilihanTema::Sistem,
            logo_app,
            logo_lembaga: None,
            splash_sampai: Some(Instant::now() + LAMA_SPLASH),
            pesan: None,
            galat_db,
            tangkap: std::env::var_os("MY_DIGIT_TANGKAP").map(|p| (PathBuf::from(p), 0)),
            pulih: None,
            ui_penyiapan: None,
            ui_transaksi: transaksi::State::default(),
            ui_akun: akun::State::default(),
            ui_anggaran: anggaran::State::default(),
            ui_laporan: laporan::State::default(),
            ui_kas: kas::State::default(),
            ui_profil: profil::State::default(),
        };
        if let Some(db) = &app.db {
            app.tema = PilihanTema::dari_kode(&db.ambil("tema").unwrap_or_default());
        }
        app.tema.terapkan(&cc.egui_ctx);
        app.muat_ulang(&cc.egui_ctx);
        app.cek_penyiapan();
        if app
            .db
            .as_ref()
            .is_some_and(|db| db.salinan_migrasi.is_some())
        {
            app.info("Data diperbarui ke format baru. Salinan pengaman tersimpan di folder Cadangan Otomatis.");
        }
        if let Ok(h) = std::env::var("MY_DIGIT_HALAMAN") {
            app.buka_halaman_dev(&h);
        }
        app
    }

    /// Pengembangan: buka halaman tertentu langsung (melewati splash).
    fn buka_halaman_dev(&mut self, nama: &str) {
        self.splash_sampai = None;
        self.halaman = match nama {
            "transaksi" | "form" => Halaman::Transaksi,
            "anggaran" => Halaman::Anggaran,
            "laporan" => Halaman::Laporan,
            "kas" => Halaman::PemeriksaanKas,
            "akun" => Halaman::Akun,
            "kategori" => Halaman::Kategori,
            "profil" => Halaman::Profil,
            "pengaturan" => Halaman::Pengaturan,
            "tentang" => Halaman::Tentang,
            "lainnya" => Halaman::Lainnya,
            "splash" => {
                self.splash_sampai = Some(Instant::now() + Duration::from_secs(60));
                Halaman::Ringkasan
            }
            "penyiapan" | "penyiapan-data" => {
                let mut st = penyiapan::State::default();
                if nama == "penyiapan-data" {
                    st.pilih(crate::instansi::JenisInstansi::Instansi);
                    st.langkah = penyiapan::Langkah::Data;
                }
                self.ui_penyiapan = Some(st);
                Halaman::Ringkasan
            }
            "akun-baru" => {
                akun::buka_akun_baru(self);
                Halaman::Akun
            }
            "kategori-baru" => {
                akun::buka_kategori_baru(self);
                Halaman::Kategori
            }
            "hapus-akun" => {
                akun::buka_hapus_akun_dev(self);
                Halaman::Akun
            }
            "form-masuk" => {
                transaksi::buka_baru(self, JenisTransaksi::Masuk);
                Halaman::Transaksi
            }
            "form-transfer" => {
                transaksi::buka_baru(self, JenisTransaksi::Transfer);
                Halaman::Transaksi
            }
            "ubah-jenis" => {
                profil::State::buka_ubah_jenis(self, crate::instansi::JenisInstansi::Organisasi);
                Halaman::Profil
            }
            _ => Halaman::Ringkasan,
        };
        if nama == "form" {
            transaksi::buka_baru(self, JenisTransaksi::Keluar);
        }
    }

    fn tangkap_layar(&mut self, ctx: &egui::Context) {
        let Some((path, frame)) = &mut self.tangkap else {
            return;
        };
        *frame += 1;
        ctx.request_repaint();
        if *frame == 30 {
            let path = path.clone();
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                egui::ScreenshotCallback::new(move |img| {
                    let [w, h] = img.size;
                    let rgba: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_array()).collect();
                    if let Some(buf) = image::RgbaImage::from_raw(w as u32, h as u32, rgba) {
                        let _ = buf.save(&path);
                    }
                    std::process::exit(0);
                }),
            ));
        }
    }

    pub fn db(&self) -> &Db {
        self.db.as_ref().expect("database terbuka")
    }

    /// Baca ulang semua data dari database.
    pub fn muat_ulang(&mut self, ctx: &egui::Context) {
        let Some(db) = &self.db else { return };
        let hasil = (|| -> Result<_, String> {
            Ok((
                db.daftar_akun()?,
                db.daftar_kategori()?,
                db.daftar_transaksi()?,
            ))
        })();
        self.profil = db.profil();
        self.instansi = db.instansi();
        match hasil {
            Ok((a, k, t)) => {
                self.akun = a;
                self.kategori = k;
                self.transaksi = t;
            }
            Err(e) => self.galat(e),
        }
        self.ui_profil.draf = None;
        self.ui_profil.draf_instansi = None;
        self.logo_lembaga = self
            .path_logo_lembaga()
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|b| muat_tekstur(ctx, "logo-lembaga", &b));
    }

    /// Tampilkan layar "Masuk sebagai" bila data belum disiapkan.
    pub fn cek_penyiapan(&mut self) {
        self.ui_penyiapan = self
            .db
            .as_ref()
            .filter(|db| db.perlu_penyiapan())
            .map(|db| penyiapan::State::dari(db.profil(), db.instansi()));
    }

    /// Kosongkan semua state halaman (draf, pilihan, dialog) yang bisa
    /// merujuk id lama — dipanggil setelah data diganti.
    pub fn reset_ui(&mut self) {
        self.ui_transaksi = transaksi::State::default();
        self.ui_akun = akun::State::default();
        self.ui_anggaran = anggaran::State::default();
        self.ui_laporan = laporan::State::default();
        self.ui_kas = kas::State::default();
        self.ui_profil = profil::State::default();
        self.pulih = None;
    }

    /// Bagian pajak tampil bila fiturnya aktif atau sudah ada data pajak.
    pub fn pajak_tampil(&self) -> bool {
        self.instansi.pajak || self.ada_data_pajak()
    }

    pub fn ada_data_pajak(&self) -> bool {
        self.transaksi
            .iter()
            .any(|t| t.pungutan.total() > 0 || t.setor_pajak.is_some())
    }

    /// "Profil Sekolah", "Profil Organisasi", ...
    pub fn judul_profil(&self) -> String {
        format!("Profil {}", self.instansi.jenis.label())
    }

    pub fn sumber(&self) -> Sumber<'_> {
        Sumber {
            akun: &self.akun,
            kategori: &self.kategori,
            transaksi: &self.transaksi,
        }
    }

    pub fn path_logo_lembaga(&self) -> Option<PathBuf> {
        self.profil.logo.as_ref().map(|f| folder_data().join(f))
    }

    pub fn gambar_pdf(&self) -> Option<Gambar> {
        self.path_logo_lembaga()
            .and_then(|p| Gambar::dari_berkas(&p))
    }

    pub fn nama_akun(&self, id: i64) -> &str {
        self.akun
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.nama.as_str())
            .unwrap_or("—")
    }

    pub fn kategori(&self, id: Option<i64>) -> Option<&Kategori> {
        id.and_then(|id| self.kategori.iter().find(|k| k.id == id))
    }

    pub fn info(&mut self, teks: impl Into<String>) {
        self.pesan = Some(Pesan {
            teks: teks.into(),
            galat: false,
            sampai: Instant::now() + LAMA_PESAN,
        });
    }

    pub fn galat(&mut self, teks: impl Into<String>) {
        self.pesan = Some(Pesan {
            teks: teks.into(),
            galat: true,
            sampai: Instant::now() + LAMA_PESAN * 2,
        });
    }

    /// Jalankan perubahan data, muat ulang, dan tampilkan hasilnya.
    pub fn ubah(
        &mut self,
        ctx: &egui::Context,
        sukses: &str,
        f: impl FnOnce(&Db) -> Result<(), String>,
    ) -> bool {
        match f(self.db()) {
            Ok(()) => {
                self.muat_ulang(ctx);
                self.info(sukses);
                true
            }
            Err(e) => {
                self.galat(e);
                false
            }
        }
    }

    /// Simpan berkas hasil ekspor lewat dialog "Simpan".
    pub fn simpan_berkas(&mut self, nama: &str, ekstensi: &str, isi: &[u8]) {
        let Some(path) = berkas::pilih_simpan(nama, &ekstensi.to_uppercase(), &[ekstensi]) else {
            return;
        };
        match std::fs::write(&path, isi) {
            Ok(()) => {
                self.info(format!("Tersimpan: {}", path.display()));
                if berkas::BISA_MEMILIH {
                    buka_berkas(&path);
                }
            }
            Err(e) => self.galat(format!("Gagal menyimpan: {e}")),
        }
    }

    // ---------- Tampilan ----------

    fn identitas(&self, ui: &mut egui::Ui, ukuran_logo: f32) {
        let p = palet(ui);
        ui.vertical_centered(|ui| {
            ui.add(egui::Image::new(&self.logo_app).fit_to_exact_size(Vec2::splat(ukuran_logo)));
            ui.add_space(14.0);
            ui.label(RichText::new(branding::APP_NAME).size(34.0).strong());
            ui.label(
                RichText::new(branding::APP_TAGLINE)
                    .size(15.0)
                    .color(p.teks2),
            );
            ui.add_space(10.0);
            ui.label(RichText::new(branding::CREDIT).italics().color(p.teks2));
        });
    }

    fn splash(&mut self, ui: &mut egui::Ui, sampai: Instant) {
        let ctx = ui.ctx().clone();
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space((ui.available_height() - 400.0).max(0.0) / 2.0);
            self.identitas(ui, 200.0);
            ui.add_space(18.0);
            ui.vertical_centered(|ui| ui.spinner());
        });
        let now = Instant::now();
        let lewati = ctx.input(|i| i.pointer.any_click() || i.key_pressed(egui::Key::Escape));
        if now >= sampai || lewati {
            self.splash_sampai = None;
        } else {
            ctx.request_repaint_after(sampai - now);
        }
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        let p = palet(ui);
        let frame = egui::Frame::new().fill(p.sidebar).inner_margin(Margin {
            left: 10,
            right: 10,
            top: 0,
            bottom: 10,
        });
        egui::Panel::left("sidebar")
            .exact_size(224.0)
            .resizable(false)
            .frame(frame)
            .show(ui, |ui| {
                // Ruang untuk tombol jendela macOS (judul jendela transparan).
                ui.add_space(if cfg!(target_os = "macos") {
                    40.0
                } else {
                    14.0
                });
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.add(egui::Image::new(&self.logo_app).fit_to_exact_size(Vec2::splat(30.0)));
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.label(RichText::new(branding::APP_NAME).size(15.0).strong());
                        let nama = if !self.profil.nama.is_empty() {
                            self.profil.nama.clone()
                        } else if !self.instansi.subjenis.is_empty() {
                            format!(
                                "{} · {}",
                                self.instansi.jenis.label(),
                                self.instansi.subjenis
                            )
                        } else {
                            self.instansi.jenis.label().to_string()
                        };
                        ui.add(
                            egui::Label::new(RichText::new(nama).size(11.0).color(p.teks2))
                                .truncate(),
                        );
                    });
                });
                ui.add_space(10.0);
                for (bagian, item) in MENU {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.label(RichText::new(*bagian).size(11.0).strong().color(p.teks2));
                    });
                    ui.spacing_mut().item_spacing.y = 2.0;
                    for (h, simbol, label) in *item {
                        let (simbol, label) = if *h == Halaman::Profil {
                            (self.instansi.jenis.ikon(), self.judul_profil())
                        } else {
                            (*simbol, label.to_string())
                        };
                        if item_sidebar(ui, simbol, &label, self.halaman == *h).clicked() {
                            self.halaman = *h;
                        }
                    }
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!(
                                "v{} · {}",
                                branding::APP_VERSION,
                                branding::MAKER
                            ))
                            .size(10.5)
                            .color(p.teks2),
                        );
                    });
                });
            });
    }

    // ---------- Tablet (kelas sedang) ----------

    /// Rel ikon di kiri: semua menu sebagai ikon + label pendek.
    fn rel(&mut self, ui: &mut egui::Ui) {
        let p = palet(ui);
        let frame = egui::Frame::new().fill(p.sidebar).inner_margin(Margin {
            left: 6,
            right: 6,
            top: if cfg!(target_os = "macos") { 38 } else { 12 },
            bottom: 8,
        });
        let mut pilih = None;
        egui::Panel::left("rel")
            .exact_size(84.0)
            .resizable(false)
            .frame(frame)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add(
                                egui::Image::new(&self.logo_app)
                                    .fit_to_exact_size(Vec2::splat(30.0)),
                            );
                        });
                        for (_, item) in MENU {
                            ui.add_space(6.0);
                            for (h, simbol, label) in *item {
                                let (simbol, label, penuh) = if *h == Halaman::Profil {
                                    (self.instansi.jenis.ikon(), "Profil", self.judul_profil())
                                } else {
                                    (
                                        *simbol,
                                        label_pendek(*h).unwrap_or(label),
                                        label.to_string(),
                                    )
                                };
                                let (rect, r) = ui.allocate_exact_size(
                                    Vec2::new(ui.available_width(), 50.0),
                                    egui::Sense::click(),
                                );
                                let aktif = self.halaman == *h;
                                if aktif {
                                    ui.painter().rect_filled(
                                        rect.shrink2(Vec2::new(4.0, 2.0)),
                                        CornerRadius::same(10),
                                        p.pilih,
                                    );
                                } else if r.hovered() {
                                    ui.painter().rect_filled(
                                        rect.shrink2(Vec2::new(4.0, 2.0)),
                                        CornerRadius::same(10),
                                        p.garis.gamma_multiply(0.6),
                                    );
                                }
                                let warna = if aktif {
                                    p.aksen
                                } else {
                                    p.aksen.gamma_multiply(0.85)
                                };
                                let atas = rect.center_top();
                                ui.painter().text(
                                    atas + Vec2::new(0.0, 18.0),
                                    egui::Align2::CENTER_CENTER,
                                    simbol,
                                    ikon::font(20.0),
                                    warna,
                                );
                                ui.painter().text(
                                    atas + Vec2::new(0.0, 39.0),
                                    egui::Align2::CENTER_CENTER,
                                    label,
                                    egui::FontId::proportional(10.5),
                                    p.teks,
                                );
                                if r.on_hover_text(penuh)
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    pilih = Some(*h);
                                }
                            }
                        }
                    });
            });
        if let Some(h) = pilih {
            self.halaman = h;
        }
    }

    // ---------- Ponsel ----------

    /// Tab yang menyala: halaman di luar bar tab termasuk "Lainnya".
    fn tab_aktif(&self) -> Halaman {
        match self.halaman {
            Halaman::Ringkasan | Halaman::Transaksi | Halaman::Laporan => self.halaman,
            _ => Halaman::Lainnya,
        }
    }

    /// Bar atas ponsel: identitas, atau tombol kembali ke "Lainnya".
    fn kepala_ponsel(&mut self, ui: &mut egui::Ui) {
        let p = palet(ui);
        let frame = egui::Frame::new().fill(p.latar).inner_margin(Margin {
            left: 14,
            right: 14,
            top: 10,
            bottom: 4,
        });
        let mut kembali = false;
        egui::Panel::top("kepala-ponsel")
            .frame(frame)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.tab_aktif() == Halaman::Lainnya && self.halaman != Halaman::Lainnya {
                        let r = ui.add(
                            egui::Button::new((
                                ikon::rt(ikon::CARET_LEFT).color(p.aksen),
                                RichText::new("Lainnya").color(p.aksen),
                            ))
                            .frame(false),
                        );
                        kembali = r.clicked();
                    } else {
                        ui.add(
                            egui::Image::new(&self.logo_app).fit_to_exact_size(Vec2::splat(24.0)),
                        );
                        ui.label(RichText::new(branding::APP_NAME).strong());
                        if !self.profil.nama.is_empty() {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(format!("· {}", self.profil.nama)).color(p.teks2),
                                )
                                .truncate(),
                            );
                        }
                    }
                });
            });
        if kembali {
            self.halaman = Halaman::Lainnya;
        }
    }

    /// Bar tab bawah ponsel.
    fn tab_ponsel(&mut self, ui: &mut egui::Ui) {
        let p = palet(ui);
        let frame = egui::Frame::new()
            .fill(p.sidebar)
            .stroke(Stroke::new(0.5, p.garis))
            .inner_margin(Margin {
                left: 4,
                right: 4,
                top: 4,
                bottom: 10,
            });
        let aktif = self.tab_aktif();
        let mut pilih = None;
        egui::Panel::bottom("tab-ponsel")
            .frame(frame)
            .resizable(false)
            .show(ui, |ui| {
                ui.columns(TAB_PONSEL.len(), |c| {
                    for (ui, (h, simbol, label)) in c.iter_mut().zip(TAB_PONSEL) {
                        let (rect, r) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), 46.0),
                            egui::Sense::click(),
                        );
                        let warna = if h == aktif { p.aksen } else { p.teks2 };
                        let atas = rect.center_top();
                        ui.painter().text(
                            atas + Vec2::new(0.0, 14.0),
                            egui::Align2::CENTER_CENTER,
                            simbol,
                            ikon::font(22.0),
                            warna,
                        );
                        ui.painter().text(
                            atas + Vec2::new(0.0, 36.0),
                            egui::Align2::CENTER_CENTER,
                            label,
                            egui::FontId::proportional(10.5),
                            warna,
                        );
                        if r.clicked() {
                            pilih = Some(h);
                        }
                    }
                });
            });
        if let Some(h) = pilih {
            self.halaman = h;
        }
    }

    /// Halaman "Lainnya" (ponsel): menu bergaya daftar Pengaturan iOS.
    fn lainnya(&mut self, ui: &mut egui::Ui) {
        let p = palet(ui);
        tema::judul(ui, "Lainnya", "");
        let mut pilih = None;
        for (bagian, item) in MENU {
            let item: Vec<&ItemMenu> = item
                .iter()
                .filter(|(h, _, _)| !TAB_PONSEL.iter().any(|t| t.0 == *h))
                .collect();
            if item.is_empty() {
                continue;
            }
            ui.label(
                RichText::new(bagian.to_uppercase())
                    .size(11.0)
                    .color(p.teks2),
            );
            tema::kartu(ui, |ui| {
                for (i, (h, simbol, label)) in item.iter().enumerate() {
                    if i > 0 {
                        pemisah(ui);
                    }
                    let (simbol, label) = if *h == Halaman::Profil {
                        (self.instansi.jenis.ikon(), self.judul_profil())
                    } else {
                        (*simbol, label.to_string())
                    };
                    let (rect, r) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), 40.0),
                        egui::Sense::click(),
                    );
                    let kiri = rect.left_center();
                    ui.painter().text(
                        kiri + Vec2::new(4.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        simbol,
                        ikon::font(20.0),
                        p.aksen,
                    );
                    ui.painter().text(
                        kiri + Vec2::new(36.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        label,
                        egui::FontId::proportional(15.0),
                        p.teks,
                    );
                    ui.painter().text(
                        rect.right_center(),
                        egui::Align2::RIGHT_CENTER,
                        ikon::CARET_RIGHT,
                        ikon::font(15.0),
                        p.teks2,
                    );
                    if r.clicked() {
                        pilih = Some(*h);
                    }
                }
            });
            ui.add_space(12.0);
        }
        tema::label_redup(
            ui,
            format!(
                "{} {} · {}",
                branding::APP_NAME,
                branding::APP_VERSION,
                branding::CREDIT
            ),
        );
        if let Some(h) = pilih {
            self.halaman = h;
        }
    }

    fn toast(&mut self, ctx: &egui::Context) {
        let Some(pesan) = &self.pesan else { return };
        let sisa = pesan.sampai.saturating_duration_since(Instant::now());
        if sisa.is_zero() {
            self.pesan = None;
            return;
        }
        ctx.request_repaint_after(sisa);
        let (teks, galat) = (pesan.teks.clone(), pesan.galat);
        egui::Area::new(egui::Id::unique("toast"))
            .anchor(egui::Align2::CENTER_BOTTOM, Vec2::new(112.0, -24.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let p = palet(ui);
                egui::Frame::new()
                    .fill(if galat {
                        p.merah
                    } else {
                        Color32::from_rgb(44, 44, 46)
                    })
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::symmetric(16, 10))
                    .shadow(egui::Shadow {
                        offset: [0, 4],
                        blur: 16,
                        spread: 0,
                        color: Color32::from_black_alpha(40),
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let simbol = if galat {
                                ikon::WARNING_CIRCLE
                            } else {
                                ikon::CHECK_CIRCLE
                            };
                            ui.label(ikon::rt(simbol).size(16.0).color(Color32::WHITE));
                            ui.label(RichText::new(teks).color(Color32::WHITE));
                        });
                    });
            });
    }

    fn halaman_galat_db(&mut self, ui: &mut egui::Ui) {
        let galat = self.galat_db.clone().unwrap_or_default();
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(80.0);
            self.identitas(ui, 120.0);
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("Database tidak dapat dibuka")
                        .size(18.0)
                        .strong(),
                );
                ui.set_max_width(ui.available_width().min(560.0));
                ui.label(&galat);
                ui.label(format!("Lokasi: {}", folder_data().display()));
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if tema::tombol_utama(ui, ikon::ARROW_COUNTER_CLOCKWISE, "Coba Lagi").clicked()
                    {
                        match Db::buka_bawaan() {
                            Ok(db) => {
                                self.db = Some(db);
                                self.galat_db = None;
                                let ctx = ui.ctx().clone();
                                self.muat_ulang(&ctx);
                                self.cek_penyiapan();
                            }
                            Err(e) => self.galat_db = Some(e),
                        }
                    }
                    if tema::tombol(ui, ikon::FOLDER_OPEN, "Buka Folder Data").clicked() {
                        buka_berkas(&folder_data());
                    }
                });
            });
        });
    }

    fn tentang(&self, ui: &mut egui::Ui) {
        let p = palet(ui);
        ui.add_space(30.0);
        self.identitas(ui, 170.0);
        ui.vertical_centered(|ui| {
            ui.add_space(12.0);
            ui.label(RichText::new(format!("Versi {}", branding::APP_VERSION)).color(p.teks2));
            ui.label(RichText::new(format!("© {}", branding::MAKER)).color(p.teks2));
            ui.add_space(24.0);
        });
        ui.vertical_centered(|ui| {
            ui.set_max_width(ui.available_width().min(520.0));
            tema::kartu(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    tema::subjudul(ui, "Fitur");
                    for f in [
                        "Buku Kas Umum, Buku Pembantu Kas, Bank, dan Pajak",
                        "Anggaran tahunan dan laporan realisasi",
                        "Kwitansi dan Berita Acara Pemeriksaan Kas siap cetak",
                        "Ekspor PDF dan Excel (.xlsx) siap cetak",
                        "Cadangan dan pemulihan data",
                    ] {
                        ui.label(format!("•  {f}"));
                    }
                });
            });
        });
    }

    fn pengaturan(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        tema::judul(ui, "Tampilan & Data", "Tema aplikasi dan keamanan data.");
        tema::kartu(ui, |ui| {
            tema::subjudul(ui, "Tampilan");
            tema::baris_form(ui, "Tema", |ui| {
                let mut t = self.tema;
                if tema::segmen(
                    ui,
                    &mut t,
                    &[
                        (PilihanTema::Sistem, "Ikuti Sistem"),
                        (PilihanTema::Terang, "Terang"),
                        (PilihanTema::Gelap, "Gelap"),
                    ],
                ) {
                    self.tema = t;
                    t.terapkan(&ctx);
                    let _ = self.db().atur("tema", t.kode());
                }
            });
        });
        ui.add_space(12.0);
        tema::kartu(ui, |ui| {
            tema::subjudul(ui, "Data");
            let lokasi = self.db().path().display().to_string();
            tema::baris_form(ui, "Lokasi database", |ui| {
                ui.add(egui::Label::new(RichText::new(&lokasi).monospace()).truncate());
            });
            tema::baris_form(ui, "Salinan otomatis", |ui| {
                if tema::tombol(ui, ikon::FOLDER_OPEN, "Tampilkan Salinan Otomatis").clicked() {
                    let dir = self
                        .db()
                        .path()
                        .parent()
                        .map(|p| p.join(FOLDER_CADANGAN_OTOMATIS))
                        .unwrap_or_else(folder_data);
                    buka_berkas(if dir.exists() {
                        &dir
                    } else {
                        dir.parent().unwrap_or(&dir)
                    });
                }
            });
            tema::label_redup(
                ui,
                "Cadangkan data secara berkala, misalnya setiap akhir bulan, ke flashdisk atau penyimpanan awan.",
            );
            ui.horizontal_wrapped(|ui| {
                if tema::tombol_utama(ui, ikon::ARCHIVE, "Cadangkan Data…").clicked() {
                    self.cadangkan();
                }
                if tema::tombol(ui, ikon::ARROW_COUNTER_CLOCKWISE, "Pulihkan dari Cadangan…")
                    .clicked()
                {
                    self.mulai_pulihkan();
                }
                if tema::tombol(ui, ikon::FOLDER_OPEN, "Buka Folder Data").clicked() {
                    buka_berkas(&folder_data());
                }
            });
        });
    }

    /// Pilih berkas cadangan lalu tampilkan lembar konfirmasi.
    pub fn mulai_pulihkan(&mut self) {
        if !berkas::BISA_MEMILIH {
            self.galat("Memilih berkas cadangan belum tersedia di Android.");
            return;
        }
        let Some(path) = berkas::pilih_buka("Cadangan My Digit", &["sqlite", "db"]) else {
            return;
        };
        let info = Db::periksa_cadangan(&path);
        self.pulih = Some(Pulih { path, info });
    }

    /// Lembar "Pulihkan data?": bandingkan data saat ini dengan isi cadangan.
    fn lembar_pulihkan(&mut self, ctx: &egui::Context) {
        let Some(pulih) = &self.pulih else { return };
        let sekarang = self.db.as_ref().map(|db| db.info());
        let mut tutup = false;
        let mut lanjut = false;
        let r = egui::Modal::new(egui::Id::unique("konfirmasi-pulihkan")).show(ctx, |ui| {
            let p = palet(ui);
            ui.set_width(tema::lebar_modal(ui, 460.0));
            tema::subjudul(ui, "Pulihkan data?");
            ui.add(
                egui::Label::new(
                    RichText::new(pulih.path.display().to_string())
                        .size(11.0)
                        .color(p.teks2),
                )
                .truncate(),
            );
            ui.add_space(6.0);
            match &pulih.info {
                Err(e) => {
                    ui.label(RichText::new(e).color(p.merah));
                }
                Ok(c) => {
                    let ringkas = |i: &InfoCadangan| {
                        [
                            if i.nama.is_empty() {
                                "(tanpa nama)".to_string()
                            } else {
                                i.nama.clone()
                            },
                            i.jenis
                                .map(|j| j.label().to_string())
                                .unwrap_or_else(|| "—".into()),
                            format!("{} transaksi", i.transaksi),
                            i.terakhir
                                .map(crate::format::tanggal_panjang)
                                .unwrap_or_else(|| "—".into()),
                        ]
                    };
                    let kiri = sekarang.as_ref().and_then(|s| s.as_ref().ok()).map(ringkas);
                    egui::Grid::new("banding-pulih")
                        .num_columns(3)
                        .spacing([16.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("");
                            ui.label(RichText::new("Data saat ini").strong());
                            ui.label(RichText::new("Isi cadangan").strong());
                            ui.end_row();
                            let kanan = ringkas(c);
                            for (i, label) in ["Nama", "Jenis", "Transaksi", "Transaksi terakhir"]
                                .iter()
                                .enumerate()
                            {
                                ui.label(RichText::new(*label).color(p.teks2));
                                ui.label(
                                    kiri.as_ref()
                                        .map(|k| k[i].clone())
                                        .unwrap_or_else(|| "—".into()),
                                );
                                ui.label(&kanan[i]);
                                ui.end_row();
                            }
                        });
                    if let Some(Ok(s)) = &sekarang {
                        let beda_id = matches!((&s.id, &c.id), (Some(a), Some(b)) if a != b);
                        let beda_nama =
                            !s.nama.is_empty() && !c.nama.is_empty() && s.nama != c.nama;
                        if beda_id || beda_nama {
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(format!(
                                    "Cadangan ini milik ‘{}’, bukan ‘{}’.",
                                    if c.nama.is_empty() {
                                        "lembaga lain"
                                    } else {
                                        &c.nama
                                    },
                                    if s.nama.is_empty() {
                                        "data saat ini"
                                    } else {
                                        &s.nama
                                    }
                                ))
                                .color(p.oranye),
                            );
                        }
                        if c.transaksi < s.transaksi {
                            ui.label(
                                RichText::new(format!(
                                    "Cadangan berisi lebih sedikit transaksi ({} dari {}).",
                                    c.transaksi, s.transaksi
                                ))
                                .color(p.oranye),
                            );
                        }
                    }
                    ui.add_space(4.0);
                    tema::label_redup(
                        ui,
                        "Salinan data saat ini disimpan otomatis di folder Cadangan Otomatis.",
                    );
                }
            }
            ui.add_space(8.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if pulih.info.is_ok() && tema::tombol_bahaya(ui, "", "Ganti Data").clicked() {
                    lanjut = true;
                }
                if tema::tombol(ui, "", "Batal").clicked() {
                    tutup = true;
                }
            });
        });
        if r.should_close() {
            tutup = true;
        }
        if lanjut {
            let path = pulih.path.clone();
            self.pulih = None;
            self.pulihkan(ctx, &path);
        } else if tutup {
            self.pulih = None;
        }
    }

    fn pulihkan(&mut self, ctx: &egui::Context, sumber: &std::path::Path) {
        let Some(db) = self.db.take() else { return };
        match db.pulihkan(sumber) {
            Ok((db, _darurat)) => {
                self.db = Some(db);
                self.reset_ui();
                self.muat_ulang(ctx);
                self.cek_penyiapan();
                self.info(
                    "Data berhasil dipulihkan. Data sebelumnya disimpan di Cadangan Otomatis.",
                );
            }
            Err((db, pesan)) => {
                self.db = db.or_else(|| Db::buka_bawaan().ok());
                if self.db.is_none() {
                    self.galat_db = Some(pesan.clone());
                }
                self.reset_ui();
                self.muat_ulang(ctx);
                self.galat(pesan);
            }
        }
    }

    fn cadangkan(&mut self) {
        let nama = format!("{}-cadangan-{}.sqlite", slug(&self.profil.nama), hari_ini());
        let Some(path) = berkas::pilih_simpan(&nama, "SQLite", &["sqlite"]) else {
            return;
        };
        match self.db().cadangkan(&path) {
            Ok(()) => self.info(format!("Cadangan tersimpan: {}", path.display())),
            Err(e) => self.galat(format!("Gagal mencadangkan: {e}")),
        }
    }
}

/// Nama berkas aman: huruf kecil ASCII, angka, dan `-`, maks. 40 karakter.
fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let out: String = out.trim_end_matches('-').chars().take(40).collect();
    let out = out.trim_end_matches('-').to_string();
    if out.is_empty() {
        "my-digit".into()
    } else {
        out
    }
}

impl eframe::App for MyDigitApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.tangkap_layar(&ui.ctx().clone());
        if let Some(sampai) = self.splash_sampai {
            self.splash(ui, sampai);
            return;
        }
        if self.db.is_none() {
            self.halaman_galat_db(ui);
            return;
        }
        let ctx = ui.ctx().clone();
        if self.ui_penyiapan.is_some() {
            penyiapan::tampil(self, ui);
            self.lembar_pulihkan(&ctx);
            self.toast(&ctx);
            return;
        }
        let p = palet(ui);
        let kelas = tema::kelas(&ctx);
        if kelas != tema::Kelas::Ringkas && self.halaman == Halaman::Lainnya {
            // "Lainnya" hanya ada di bar tab ponsel.
            self.halaman = Halaman::Ringkasan;
        }
        let margin = if kelas == tema::Kelas::Ringkas {
            self.kepala_ponsel(ui);
            self.tab_ponsel(ui);
            Margin {
                left: 14,
                right: 14,
                top: 4,
                bottom: 8,
            }
        } else if kelas == tema::Kelas::Sedang {
            self.rel(ui);
            Margin {
                left: 20,
                right: 20,
                top: 20,
                bottom: 12,
            }
        } else {
            self.sidebar(ui);
            Margin {
                left: 28,
                right: 28,
                top: 24,
                bottom: 16,
            }
        };
        let frame = egui::Frame::new().fill(p.latar).inner_margin(margin);
        egui::CentralPanel::default().frame(frame).show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| match self.halaman {
                    Halaman::Ringkasan => ringkasan::tampil(self, ui),
                    Halaman::Transaksi => transaksi::tampil(self, ui),
                    Halaman::Anggaran => anggaran::tampil(self, ui),
                    Halaman::Laporan => laporan::tampil(self, ui),
                    Halaman::PemeriksaanKas => kas::tampil(self, ui),
                    Halaman::Akun => akun::tampil_akun(self, ui),
                    Halaman::Kategori => akun::tampil_kategori(self, ui),
                    Halaman::Profil => profil::tampil(self, ui),
                    Halaman::Pengaturan => self.pengaturan(ui),
                    Halaman::Tentang => self.tentang(ui),
                    Halaman::Lainnya => self.lainnya(ui),
                });
        });
        transaksi::formulir(self, &ctx);
        self.lembar_pulihkan(&ctx);
        self.toast(&ctx);
    }
}

// ---------- Komponen bersama ----------

/// Label singkat untuk rel ikon yang sempit.
fn label_pendek(h: Halaman) -> Option<&'static str> {
    match h {
        Halaman::PemeriksaanKas => Some("Kas"),
        Halaman::Akun => Some("Akun"),
        Halaman::Pengaturan => Some("Tampilan"),
        Halaman::Tentang => Some("Tentang"),
        _ => None,
    }
}

fn item_sidebar(ui: &mut egui::Ui, simbol: &str, label: &str, aktif: bool) -> egui::Response {
    let p = palet(ui);
    let (rect, resp) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.0), egui::Sense::click());
    if aktif {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(7), p.pilih);
    } else if resp.hovered() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(7), p.garis.gamma_multiply(0.6));
    }
    let warna_ikon = if aktif {
        p.aksen
    } else {
        p.aksen.gamma_multiply(0.85)
    };
    ui.painter().text(
        rect.left_center() + Vec2::new(12.0, 0.0),
        egui::Align2::LEFT_CENTER,
        simbol,
        ikon::font(17.0),
        warna_ikon,
    );
    ui.painter().text(
        rect.left_center() + Vec2::new(36.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(13.5),
        p.teks,
    );
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Pemilih bulan dan tahun. `boleh_setahun` menambah pilihan "Setahun".
pub fn pilih_bulan(ui: &mut egui::Ui, id: &str, b: &mut Bulan, boleh_setahun: bool) -> bool {
    let awal = *b;
    let label = if b.bulan == 0 {
        "Setahun".to_string()
    } else {
        BULAN[b.bulan as usize - 1].to_string()
    };
    egui::ComboBox::from_id_salt(id)
        .icon(tema::ikon_dropdown)
        .selected_text(label)
        .width(120.0)
        .show_ui(ui, |ui| {
            if boleh_setahun {
                ui.selectable_value(&mut b.bulan, 0, "Setahun");
            }
            for (i, nama) in BULAN.iter().enumerate() {
                ui.selectable_value(&mut b.bulan, i as i8 + 1, *nama);
            }
        });
    pilih_tahun(ui, &mut b.tahun);
    *b != awal
}

pub fn pilih_tahun(ui: &mut egui::Ui, tahun: &mut i16) -> bool {
    let awal = *tahun;
    // Di dalam tata letak kanan-ke-kiri, tombol "berikutnya" harus digambar lebih dulu.
    let mut tombol = [
        (ikon::CARET_LEFT, -1, "Tahun sebelumnya"),
        (ikon::CARET_RIGHT, 1, "Tahun berikutnya"),
    ];
    if ui.layout().prefer_right_to_left() {
        tombol.reverse();
    }
    let p = palet(ui);
    if tema::tombol_ikon(ui, tombol[0].0, tombol[0].2, p.aksen).clicked() {
        *tahun += tombol[0].1;
    }
    ui.label(RichText::new(awal.to_string()).strong());
    if tema::tombol_ikon(ui, tombol[1].0, tombol[1].2, p.aksen).clicked() {
        *tahun += tombol[1].1;
    }
    *tahun != awal
}

/// Garis pemisah tipis.
pub fn pemisah(ui: &mut egui::Ui) {
    let p = palet(ui);
    let rect = ui.available_rect_before_wrap();
    let y = ui.cursor().top();
    ui.painter()
        .hline(rect.x_range(), y, Stroke::new(1.0, p.garis));
    ui.add_space(1.0);
}

pub fn muat_tekstur(ctx: &egui::Context, nama: &str, bytes: &[u8]) -> Option<TextureHandle> {
    let img = image::load_from_memory(bytes).ok()?.into_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
    Some(ctx.load_texture(nama, color, egui::TextureOptions::LINEAR))
}

/// Buka berkas atau folder dengan aplikasi bawaan sistem.
pub fn buka_berkas(path: &std::path::Path) {
    berkas::buka(path);
}

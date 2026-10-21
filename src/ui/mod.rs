//! Antarmuka My Digit.

mod akun;
mod anggaran;
mod kas;
mod laporan;
mod profil;
mod ringkasan;
pub mod tema;
mod transaksi;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke, TextureHandle, Vec2};

use crate::branding;
use crate::db::{Db, folder_data};
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
}

/// (halaman, ikon, label)
type ItemMenu = (Halaman, &'static str, &'static str);

const MENU: &[(&str, &[ItemMenu])] = &[
    (
        "Keuangan",
        &[
            (Halaman::Ringkasan, "🏠", "Ringkasan"),
            (Halaman::Transaksi, "🧾", "Transaksi"),
            (Halaman::Anggaran, "🎯", "Anggaran"),
            (Halaman::Laporan, "📊", "Laporan"),
            (Halaman::PemeriksaanKas, "💵", "Pemeriksaan Kas"),
        ],
    ),
    (
        "Data Master",
        &[
            (Halaman::Akun, "🏦", "Akun Kas & Bank"),
            (Halaman::Kategori, "🏷", "Kategori"),
        ],
    ),
    (
        "Lembaga",
        &[
            (Halaman::Profil, "🏫", "Profil Lembaga"),
            (Halaman::Pengaturan, "⚙", "Pengaturan"),
            (Halaman::Tentang, "ℹ", "Tentang Aplikasi"),
        ],
    ),
];

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
    pub halaman: Halaman,
    pub tema: PilihanTema,

    logo_app: TextureHandle,
    pub logo_lembaga: Option<TextureHandle>,
    splash_sampai: Option<Instant>,
    pesan: Option<Pesan>,
    galat_db: Option<String>,
    /// Pengembangan: simpan tangkapan layar ke berkas ini lalu keluar.
    tangkap: Option<(PathBuf, u32)>,

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
            halaman: Halaman::Ringkasan,
            tema: PilihanTema::Sistem,
            logo_app,
            logo_lembaga: None,
            splash_sampai: Some(Instant::now() + LAMA_SPLASH),
            pesan: None,
            galat_db,
            tangkap: std::env::var_os("MY_DIGIT_TANGKAP").map(|p| (PathBuf::from(p), 0)),
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
            "splash" => {
                self.splash_sampai = Some(Instant::now() + Duration::from_secs(60));
                Halaman::Ringkasan
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
        match hasil {
            Ok((a, k, t)) => {
                self.akun = a;
                self.kategori = k;
                self.transaksi = t;
            }
            Err(e) => self.galat(e),
        }
        self.ui_profil.draf = None;
        self.logo_lembaga = self
            .path_logo_lembaga()
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|b| muat_tekstur(ctx, "logo-lembaga", &b));
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
        let Some(path) = rfd::FileDialog::new()
            .set_file_name(nama)
            .add_filter(ekstensi.to_uppercase(), &[ekstensi])
            .save_file()
        else {
            return;
        };
        match std::fs::write(&path, isi) {
            Ok(()) => {
                self.info(format!("Tersimpan: {}", path.display()));
                buka_berkas(&path);
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
                        let nama = if self.profil.nama.is_empty() {
                            "Bendahara Lembaga"
                        } else {
                            &self.profil.nama
                        };
                        ui.label(RichText::new(nama).size(11.0).color(p.teks2));
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
                    for (h, ikon, label) in *item {
                        if item_sidebar(ui, ikon, label, self.halaman == *h).clicked() {
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
                        ui.label(RichText::new(teks).color(Color32::WHITE));
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
                ui.label(&galat);
                ui.label(format!("Lokasi: {}", folder_data().display()));
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
            ui.set_max_width(520.0);
            tema::kartu(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    tema::subjudul(ui, "Fitur");
                    for f in [
                        "Buku Kas Umum, Buku Pembantu Kas, Bank, dan Pajak",
                        "Anggaran tahunan dan laporan realisasi",
                        "Kwitansi dan Berita Acara Pemeriksaan Kas siap cetak",
                        "Ekspor PDF dan CSV (Excel)",
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
        tema::judul(ui, "Pengaturan", "Tampilan dan keamanan data.");
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
                ui.label(RichText::new(&lokasi).monospace());
            });
            tema::label_redup(
                ui,
                "Cadangkan data secara berkala, misalnya setiap akhir bulan, ke flashdisk atau penyimpanan awan.",
            );
            ui.horizontal(|ui| {
                if tema::tombol_utama(ui, "Cadangkan Data…").clicked() {
                    self.cadangkan();
                }
                if tema::tombol(ui, "Pulihkan dari Cadangan…").clicked() {
                    self.ui_profil.pulihkan = rfd::FileDialog::new()
                        .add_filter("Cadangan My Digit", &["sqlite", "db"])
                        .pick_file();
                }
                if tema::tombol(ui, "Buka Folder Data").clicked() {
                    buka_berkas(&folder_data());
                }
            });
        });

        if let Some(path) = self.ui_profil.pulihkan.clone() {
            let mut tutup = false;
            let mut lanjut = false;
            egui::Modal::new(egui::Id::unique("konfirmasi-pulihkan")).show(&ctx, |ui| {
                ui.set_width(380.0);
                tema::subjudul(ui, "Pulihkan data?");
                ui.label(format!(
                    "Semua data saat ini akan diganti dengan isi cadangan:\n{}",
                    path.display()
                ));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if tema::tombol_bahaya(ui, "Ganti Data").clicked() {
                        lanjut = true;
                    }
                    if tema::tombol(ui, "Batal").clicked() {
                        tutup = true;
                    }
                });
            });
            if lanjut {
                let db = self.db.take().expect("database terbuka");
                let cadangan_darurat = db.path().with_extension("sebelum-pulih.sqlite");
                let _ = db.cadangkan(&cadangan_darurat);
                let path_db = db.path().to_path_buf();
                match db.pulihkan(&path) {
                    Ok(db) => {
                        self.db = Some(db);
                        self.muat_ulang(&ctx);
                        self.info("Data berhasil dipulihkan.");
                    }
                    Err(e) => {
                        self.db = Db::buka(&path_db).ok();
                        self.muat_ulang(&ctx);
                        self.galat(e);
                    }
                }
            }
            if lanjut || tutup {
                self.ui_profil.pulihkan = None;
            }
        }
    }

    fn cadangkan(&mut self) {
        let nama = format!("my-digit-cadangan-{}.sqlite", hari_ini());
        let Some(path) = rfd::FileDialog::new()
            .set_file_name(&nama)
            .add_filter("SQLite", &["sqlite"])
            .save_file()
        else {
            return;
        };
        match self.db().cadangkan(&path) {
            Ok(()) => self.info(format!("Cadangan tersimpan: {}", path.display())),
            Err(e) => self.galat(format!("Gagal mencadangkan: {e}")),
        }
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
        self.sidebar(ui);
        let p = palet(ui);
        let frame = egui::Frame::new().fill(p.latar).inner_margin(Margin {
            left: 28,
            right: 28,
            top: 24,
            bottom: 16,
        });
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
                });
        });
        transaksi::formulir(self, &ctx);
        self.toast(&ctx);
    }
}

// ---------- Komponen bersama ----------

fn item_sidebar(ui: &mut egui::Ui, ikon: &str, label: &str, aktif: bool) -> egui::Response {
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
        ikon,
        egui::FontId::proportional(14.0),
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
    // Di dalam tata letak kanan-ke-kiri, tombol "▶" harus digambar lebih dulu.
    let mut tombol = [("◀", -1), ("▶", 1)];
    if ui.layout().prefer_right_to_left() {
        tombol.reverse();
    }
    if ui.small_button(tombol[0].0).clicked() {
        *tahun += tombol[0].1;
    }
    ui.label(RichText::new(awal.to_string()).strong());
    if ui.small_button(tombol[1].0).clicked() {
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
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(path).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("explorer").arg(path).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let _ = std::process::Command::new("xdg-open").arg(path).spawn();
}

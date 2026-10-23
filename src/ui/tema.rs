//! Tampilan bergaya aplikasi desktop Apple: sidebar abu-abu lembut, kartu
//! bersudut bulat, warna aksen biru sistem, dan huruf sistem
//! (SF Pro di macOS, Segoe UI di Windows).

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Response,
    RichText, Stroke, TextStyle, Theme, Ui, Vec2,
};
use egui_extras::DatePickerButton;
use jiff::civil::Date;

use super::ikon;

#[derive(Clone, Copy)]
pub struct Palet {
    pub aksen: Color32,
    pub sidebar: Color32,
    pub latar: Color32,
    pub kartu: Color32,
    pub garis: Color32,
    pub teks: Color32,
    pub teks2: Color32,
    pub hijau: Color32,
    pub merah: Color32,
    pub oranye: Color32,
    pub pilih: Color32,
}

pub const TERANG: Palet = Palet {
    aksen: Color32::from_rgb(0, 122, 255),
    sidebar: Color32::from_rgb(236, 236, 241),
    latar: Color32::from_rgb(250, 250, 252),
    kartu: Color32::WHITE,
    garis: Color32::from_rgb(226, 226, 232),
    teks: Color32::from_rgb(29, 29, 31),
    teks2: Color32::from_rgb(110, 110, 115),
    hijau: Color32::from_rgb(40, 167, 69),
    merah: Color32::from_rgb(230, 55, 45),
    oranye: Color32::from_rgb(240, 140, 0),
    pilih: Color32::from_rgb(214, 228, 252),
};

pub const GELAP: Palet = Palet {
    aksen: Color32::from_rgb(10, 132, 255),
    sidebar: Color32::from_rgb(38, 38, 41),
    latar: Color32::from_rgb(28, 28, 30),
    kartu: Color32::from_rgb(44, 44, 46),
    garis: Color32::from_rgb(58, 58, 60),
    teks: Color32::from_rgb(245, 245, 247),
    teks2: Color32::from_rgb(152, 152, 157),
    hijau: Color32::from_rgb(48, 209, 88),
    merah: Color32::from_rgb(255, 69, 58),
    oranye: Color32::from_rgb(255, 159, 10),
    pilih: Color32::from_rgb(34, 64, 110),
};

pub fn palet(ui: &Ui) -> Palet {
    if ui.visuals().dark_mode {
        GELAP
    } else {
        TERANG
    }
}

/// Huruf sistem sesuai OS. Bila tidak ditemukan, huruf bawaan egui dipakai.
fn huruf_sistem() -> Option<Vec<u8>> {
    let kandidat: &[&str] = if cfg!(target_os = "macos") {
        &[
            "/System/Library/Fonts/SFNS.ttf",
            "/System/Library/Fonts/HelveticaNeue.ttc",
        ]
    } else if cfg!(target_os = "windows") {
        &[
            "C:\\Windows\\Fonts\\segoeui.ttf",
            "C:\\Windows\\Fonts\\arial.ttf",
        ]
    } else if cfg!(target_os = "android") {
        &[
            "/system/fonts/Roboto-Regular.ttf",
            "/system/fonts/DroidSans.ttf",
        ]
    } else {
        &["/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"]
    };
    kandidat.iter().find_map(|p| std::fs::read(p).ok())
}

pub fn pasang(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    if let Some(data) = huruf_sistem() {
        fonts
            .font_data
            .insert("sistem".into(), FontData::from_owned(data).into());
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "sistem".into());
    }
    ikon::pasang(&mut fonts);
    ctx.set_fonts(fonts);

    for (theme, p) in [(Theme::Light, TERANG), (Theme::Dark, GELAP)] {
        let mut style = (*ctx.style_of(theme)).clone();
        style.text_styles = [
            (TextStyle::Small, FontId::proportional(11.0)),
            (TextStyle::Body, FontId::proportional(13.5)),
            (TextStyle::Button, FontId::proportional(13.5)),
            (TextStyle::Heading, FontId::proportional(26.0)),
            (TextStyle::Monospace, FontId::monospace(12.5)),
        ]
        .into();
        style.spacing.item_spacing = Vec2::new(8.0, 8.0);
        style.spacing.button_padding = Vec2::new(12.0, 5.0);
        style.spacing.interact_size.y = 28.0;
        style.spacing.combo_width = 180.0;

        let v = &mut style.visuals;
        v.panel_fill = p.latar;
        v.window_fill = p.kartu;
        v.window_stroke = Stroke::new(1.0, p.garis);
        v.window_corner_radius = CornerRadius::same(12);
        v.menu_corner_radius = CornerRadius::same(8);
        v.extreme_bg_color = p.kartu;
        v.text_edit_bg_color = Some(p.kartu);
        v.faint_bg_color = if theme == Theme::Dark {
            Color32::from_rgb(50, 50, 53)
        } else {
            Color32::from_rgb(246, 246, 249)
        };
        v.selection.bg_fill = p.aksen.gamma_multiply(0.35);
        v.selection.stroke = Stroke::new(1.0, p.aksen);
        v.hyperlink_color = p.aksen;
        v.override_text_color = Some(p.teks);
        v.striped = true;

        let sudut = CornerRadius::same(7);
        let w = &mut v.widgets;
        w.noninteractive.bg_stroke = Stroke::new(1.0, p.garis);
        w.noninteractive.corner_radius = sudut;
        for (wv, isi) in [
            (&mut w.inactive, p.kartu),
            (&mut w.hovered, p.kartu),
            (&mut w.active, p.pilih),
            (&mut w.open, p.kartu),
        ] {
            wv.corner_radius = sudut;
            wv.bg_fill = isi;
            wv.weak_bg_fill = isi;
            wv.expansion = 0.0;
            wv.fg_stroke = Stroke::new(1.0, p.teks);
        }
        w.inactive.bg_stroke = Stroke::new(1.0, p.garis);
        w.hovered.bg_stroke = Stroke::new(1.0, p.aksen.gamma_multiply(0.6));
        w.active.bg_stroke = Stroke::new(1.0, p.aksen);
        w.open.bg_stroke = Stroke::new(1.0, p.aksen);
        ctx.set_style_of(theme, style);
    }
}

// ---------- Komponen ----------

/// Layar sempit (ponsel): tata letak tab bawah dan isian bertumpuk.
pub fn sempit(ui: &Ui) -> bool {
    ui.ctx().content_rect().width() < 700.0
}

/// Baris kepala halaman (judul + tombol kanan). Di layar sempit tombol
/// pindah ke bawah judul agar tidak menumpuk.
pub fn baris_kepala<R>(ui: &mut Ui, isi: impl FnOnce(&mut Ui) -> R) -> R {
    if sempit(ui) {
        ui.vertical(isi).inner
    } else {
        ui.horizontal(isi).inner
    }
}

/// Kelompok tombol rata kanan di baris kepala, dibatasi satu baris agar
/// tidak melebar ke seluruh tinggi halaman saat kepala ditumpuk.
pub fn kanan<R>(ui: &mut Ui, isi: impl FnOnce(&mut Ui) -> R) -> R {
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), isi)
            .inner
    })
    .inner
}

/// Lebar dialog yang muat di layar, maksimal `ideal`.
pub fn lebar_modal(ui: &Ui, ideal: f32) -> f32 {
    ideal.min(ui.ctx().content_rect().width() - 56.0).max(240.0)
}

/// Judul besar halaman, seperti di aplikasi bawaan macOS.
pub fn judul(ui: &mut Ui, judul: &str, sub: &str) {
    let p = palet(ui);
    ui.add_space(4.0);
    let ukuran = if sempit(ui) { 22.0 } else { 26.0 };
    ui.label(RichText::new(judul).size(ukuran).strong().color(p.teks));
    if !sub.is_empty() {
        ui.label(RichText::new(sub).color(p.teks2));
    }
    ui.add_space(10.0);
}

/// Kartu putih bersudut bulat dengan garis tipis.
pub fn kartu<R>(ui: &mut Ui, isi: impl FnOnce(&mut Ui) -> R) -> R {
    let p = palet(ui);
    egui::Frame::new()
        .fill(p.kartu)
        .stroke(Stroke::new(1.0, p.garis))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            isi(ui)
        })
        .inner
}

/// Judul kecil di dalam kartu.
pub fn subjudul(ui: &mut Ui, s: &str) {
    ui.label(RichText::new(s).size(15.0).strong());
}

pub fn label_redup(ui: &mut Ui, s: impl Into<String>) -> Response {
    let p = palet(ui);
    ui.label(RichText::new(s.into()).color(p.teks2))
}

/// Tombol berisi ikon (boleh `""`) dan teks dengan warna yang sama.
fn isi_tombol<'a>(ikon: &str, teks: RichText, warna: Color32) -> egui::Button<'a> {
    let teks = teks.color(warna);
    if ikon.is_empty() {
        egui::Button::new(teks)
    } else {
        egui::Button::new((ikon::rt(ikon).color(warna), teks))
    }
}

/// Tombol utama berwarna aksen. `ikon` boleh `""`.
pub fn tombol_utama(ui: &mut Ui, ikon: &str, teks: &str) -> Response {
    let p = palet(ui);
    ui.add(
        isi_tombol(ikon, RichText::new(teks).strong(), Color32::WHITE)
            .fill(p.aksen)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(7))
            .min_size(Vec2::new(0.0, 28.0)),
    )
}

/// Tombol sekunder dengan garis tipis. `ikon` boleh `""`.
pub fn tombol(ui: &mut Ui, ikon: &str, teks: &str) -> Response {
    let p = palet(ui);
    ui.add(isi_tombol(ikon, RichText::new(teks), p.teks).min_size(Vec2::new(0.0, 28.0)))
}

/// Tombol aksi berbahaya (hapus). `ikon` boleh `""`.
pub fn tombol_bahaya(ui: &mut Ui, ikon: &str, teks: &str) -> Response {
    let p = palet(ui);
    ui.add(isi_tombol(ikon, RichText::new(teks), p.merah).min_size(Vec2::new(0.0, 28.0)))
}

/// Tombol ikon kecil tanpa bingkai, dengan keterangan saat disorot.
pub fn tombol_ikon(ui: &mut Ui, ikon: &str, keterangan: &str, warna: Color32) -> Response {
    ui.add(egui::Button::new(ikon::rt(ikon).size(15.0).color(warna)).frame(false))
        .on_hover_text(keterangan)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Ikon dropdown ala tombol pop-up macOS (chevron atas-bawah),
/// untuk `ComboBox::icon`.
pub fn ikon_dropdown(
    ui: &Ui,
    rect: egui::Rect,
    visual: &egui::style::WidgetVisuals,
    _terbuka: bool,
) {
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        ikon::CARET_UP_DOWN,
        ikon::font(rect.height().max(12.0) * 1.15),
        visual.fg_stroke.color,
    );
}

/// Chevron kanan/bawah untuk `CollapsingHeader::icon`.
pub fn ikon_lipat(ui: &mut Ui, keterbukaan: f32, respons: &Response) {
    let simbol = if keterbukaan > 0.5 {
        ikon::CARET_DOWN
    } else {
        ikon::CARET_RIGHT
    };
    let warna = palet(ui).teks2;
    ui.painter().text(
        respons.rect.center(),
        egui::Align2::CENTER_CENTER,
        simbol,
        ikon::font(14.0),
        warna,
    );
}

/// Teks petunjuk isian yang diawali ikon, mis. kaca pembesar pada kolom cari.
pub fn petunjuk_ikon(ui: &Ui, simbol: &str, teks: &str) -> egui::text::LayoutJob {
    let warna = ui.visuals().weak_text_color();
    let format = |font_id| egui::TextFormat {
        font_id,
        color: warna,
        valign: egui::Align::Center,
        ..Default::default()
    };
    let mut job = egui::text::LayoutJob::default();
    job.append(simbol, 0.0, format(ikon::font(14.0)));
    job.append(
        &format!("  {teks}"),
        0.0,
        format(FontId::proportional(13.5)),
    );
    job
}

/// Pemilih tanggal dengan ikon kalender (menggantikan emoji bawaan egui).
pub fn pilih_tanggal(ui: &mut Ui, id: &str, tanggal: &mut Date) -> Response {
    let p = palet(ui);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.label(ikon::rt(ikon::CALENDAR_BLANK).size(15.0).color(p.teks2));
        ui.add(
            DatePickerButton::new(tanggal)
                .id_salt(id)
                .format("%d/%m/%Y")
                .show_icon(false),
        )
    })
    .inner
}

/// Kontrol segmen ala macOS.
pub fn segmen<T: PartialEq + Copy>(ui: &mut Ui, nilai: &mut T, pilihan: &[(T, &str)]) -> bool {
    let p = palet(ui);
    let mut berubah = false;
    egui::Frame::new()
        .fill(if ui.visuals().dark_mode {
            Color32::from_rgb(58, 58, 60)
        } else {
            Color32::from_rgb(228, 228, 233)
        })
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(2))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            ui.horizontal(|ui| {
                // Di dalam tata letak kanan-ke-kiri urutan dibalik agar tetap terbaca kiri-ke-kanan.
                let urutan: Vec<&(T, &str)> = if ui.layout().prefer_right_to_left() {
                    pilihan.iter().rev().collect()
                } else {
                    pilihan.iter().collect()
                };
                for (v, label) in urutan {
                    let aktif = *nilai == *v;
                    let tombol = egui::Button::new(RichText::new(*label).color(p.teks))
                        .fill(if aktif { p.kartu } else { Color32::TRANSPARENT })
                        .stroke(if aktif {
                            Stroke::new(0.5, p.garis)
                        } else {
                            Stroke::NONE
                        })
                        .corner_radius(CornerRadius::same(6))
                        .min_size(Vec2::new(0.0, 24.0));
                    if ui.add(tombol).clicked() && !aktif {
                        *nilai = *v;
                        berubah = true;
                    }
                }
            });
        });
    berubah
}

/// Ubin statistik untuk ringkasan.
pub fn ubin(ui: &mut Ui, ikon: &str, label: &str, nilai: &str, warna: Color32) {
    kartu(ui, |ui| {
        let p = palet(ui);
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(ikon::rt(ikon).size(15.0).color(warna));
                ui.add(egui::Label::new(RichText::new(label).color(p.teks2)).truncate());
            });
            let ukuran = if sempit(ui) { 16.0 } else { 21.0 };
            ui.add(egui::Label::new(RichText::new(nilai).size(ukuran).strong()).truncate());
        });
    });
}

/// Bilah kemajuan tipis ala macOS dengan label di kanannya.
pub fn bilah(ui: &mut Ui, rasio: f32, warna: Color32, label: &str) {
    let p = palet(ui);
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(120.0, 6.0), egui::Sense::hover());
        ui.painter()
            .rect_filled(rect, CornerRadius::same(3), p.garis);
        if rasio > 0.0 {
            let mut isi = rect;
            isi.set_width((rect.width() * rasio.min(1.0)).max(6.0));
            ui.painter().rect_filled(isi, CornerRadius::same(3), warna);
        }
        ui.label(RichText::new(label).size(12.0).color(p.teks2));
    });
}

/// Baris isian formulir: label di kiri, kontrol di kanan.
/// Di layar sempit label ditaruh di atas isian.
pub fn baris_form<R>(ui: &mut Ui, label: &str, isi: impl FnOnce(&mut Ui) -> R) -> R {
    let p = palet(ui);
    if sempit(ui) {
        return ui
            .vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.label(RichText::new(label).size(12.0).color(p.teks2));
                isi(ui)
            })
            .inner;
    }
    ui.horizontal(|ui| {
        ui.add_sized(
            Vec2::new(150.0, 28.0),
            egui::Label::new(RichText::new(label).color(p.teks2)),
        );
        isi(ui)
    })
    .inner
}

/// Isian teks bersaran: sampai 3 saran tampil sebagai tombol kecil; lebih
/// dari itu menjadi menu pop-up di sebelah isian (seperti combo box macOS).
pub fn isian_saran(
    ui: &mut Ui,
    teks: &mut String,
    petunjuk: &str,
    lebar: f32,
    saran: &[&str],
) -> bool {
    let awal = teks.clone();
    ui.horizontal_wrapped(|ui| {
        ui.add(
            egui::TextEdit::singleline(teks)
                .hint_text(petunjuk)
                .desired_width(lebar)
                .margin(Margin::symmetric(8, 5)),
        );
        if saran.len() > 3 {
            ui.menu_button(ikon::rt(ikon::CARET_UP_DOWN), |ui| {
                for s in saran {
                    if ui.selectable_label(*s == teks.as_str(), *s).clicked() {
                        *teks = s.to_string();
                        ui.close();
                    }
                }
            })
            .response
            .on_hover_text("Pilih dari saran");
        } else {
            for s in saran {
                if *s != teks.as_str() && ui.small_button(*s).clicked() {
                    *teks = s.to_string();
                }
            }
        }
    });
    *teks != awal
}

/// Isian teks satu baris dengan lebar penuh.
pub fn isian(ui: &mut Ui, teks: &mut String, petunjuk: &str) -> Response {
    ui.add(
        egui::TextEdit::singleline(teks)
            .hint_text(petunjuk)
            .desired_width(f32::INFINITY)
            .margin(Margin::symmetric(8, 5)),
    )
}

/// Isian nominal rupiah, rata kanan.
pub fn isian_rupiah(ui: &mut Ui, teks: &mut String, lebar: f32) -> Response {
    let r = ui.add(
        egui::TextEdit::singleline(teks)
            .hint_text("0")
            .align(egui::Align2::RIGHT_CENTER)
            .desired_width(lebar)
            .margin(Margin::symmetric(8, 5)),
    );
    // Rapikan pemisah ribuan saat isian ditinggalkan.
    if r.lost_focus()
        && let Some(n) = crate::format::parse_rupiah(teks)
    {
        *teks = if n == 0 {
            String::new()
        } else {
            crate::format::ribuan(n)
        };
    }
    r
}

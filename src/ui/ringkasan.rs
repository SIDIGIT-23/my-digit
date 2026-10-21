//! Halaman Ringkasan: posisi kas dan arus terkini.

use eframe::egui::{self, Align2, CornerRadius, FontId, RichText, Sense, Stroke, Vec2};

use super::{Halaman, MyDigitApp, tema, transaksi};
use crate::format::{BULAN, hari_ini, rupiah, tanggal_panjang, tanggal_pendek};
use crate::laporan::Periode;
use crate::model::*;
use tema::palet;

pub fn tampil(app: &mut MyDigitApp, ui: &mut egui::Ui) {
    let p = palet(ui);
    let hari = hari_ini();
    let sub = if app.profil.nama.is_empty() {
        format!("Hari ini {}", tanggal_panjang(hari))
    } else {
        format!("{} · {}", app.profil.nama, tanggal_panjang(hari))
    };

    ui.horizontal(|ui| {
        ui.vertical(|ui| tema::judul(ui, "Ringkasan", &sub));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if tema::tombol(ui, "⇄  Transfer").clicked() {
                transaksi::buka_baru(app, JenisTransaksi::Transfer);
            }
            if tema::tombol(ui, "−  Pengeluaran").clicked() {
                transaksi::buka_baru(app, JenisTransaksi::Keluar);
            }
            if tema::tombol_utama(ui, "+  Pemasukan").clicked() {
                transaksi::buka_baru(app, JenisTransaksi::Masuk);
            }
        });
    });

    let s = app.sumber();
    let bulan_ini = Periode::bulan(hari.year(), hari.month());
    let (masuk, keluar) = s.arus(bulan_ini);
    let saldo = s.saldo_total();
    let pajak_tertahan = {
        let t = s.pembantu_pajak(Periode {
            mulai: jiff::civil::Date::constant(1900, 1, 1),
            sampai: hari,
        });
        t.total.map(|r| r[6].clone()).unwrap_or_default()
    };

    let ubin = [
        ("💰", "Saldo Kas & Bank", rupiah(saldo), p.aksen),
        ("⬇", "Pemasukan bulan ini", rupiah(masuk), p.hijau),
        ("⬆", "Pengeluaran bulan ini", rupiah(keluar), p.merah),
        (
            "🧾",
            "Pajak belum disetor",
            format!("Rp {pajak_tertahan}"),
            p.oranye,
        ),
    ];
    let kolom = if ui.available_width() > 900.0 { 4 } else { 2 };
    for baris in ubin.chunks(kolom) {
        ui.columns(kolom, |c| {
            for (ui, (ikon, label, nilai, warna)) in c.iter_mut().zip(baris) {
                tema::ubin(ui, ikon, label, nilai, *warna);
            }
        });
        ui.add_space(4.0);
    }
    ui.add_space(8.0);

    let lebar = ui.available_width();
    let berdampingan = lebar > 820.0;
    let akun_dan_grafik = |app: &mut MyDigitApp, ui: &mut egui::Ui| {
        tema::kartu(ui, |ui| {
            ui.horizontal(|ui| {
                tema::subjudul(ui, "Saldo per Akun");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.link("Kelola").clicked() {
                        app.halaman = Halaman::Akun;
                    }
                });
            });
            let s = app.sumber();
            for a in &app.akun {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(if a.jenis == JenisAkun::Bank {
                            "🏦"
                        } else {
                            "💵"
                        })
                        .color(p.aksen),
                    );
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.label(&a.nama);
                        ui.label(RichText::new(a.jenis.label()).size(11.0).color(p.teks2));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(rupiah(s.saldo_akun(a.id))).strong());
                    });
                });
            }
        });
        ui.add_space(12.0);
        tema::kartu(ui, |ui| {
            tema::subjudul(ui, "Arus Kas 6 Bulan Terakhir");
            grafik_arus(app, ui);
        });
    };
    let terbaru = |app: &mut MyDigitApp, ui: &mut egui::Ui| {
        tema::kartu(ui, |ui| {
            ui.horizontal(|ui| {
                tema::subjudul(ui, "Transaksi Terbaru");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.link("Lihat semua").clicked() {
                        app.halaman = Halaman::Transaksi;
                    }
                });
            });
            if app.transaksi.is_empty() {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    tema::label_redup(ui, "Belum ada transaksi.");
                    tema::label_redup(ui, "Mulai dengan mencatat pemasukan pertama.");
                });
                ui.add_space(20.0);
            }
            let daftar: Vec<Transaksi> = app.transaksi.iter().rev().take(8).cloned().collect();
            for t in daftar {
                let r = baris_transaksi(app, ui, &t);
                if r.double_clicked() {
                    transaksi::buka_edit(app, &t);
                }
            }
        });
    };

    if berdampingan {
        ui.columns(2, |c| {
            akun_dan_grafik(app, &mut c[0]);
            terbaru(app, &mut c[1]);
        });
    } else {
        akun_dan_grafik(app, ui);
        ui.add_space(12.0);
        terbaru(app, ui);
    }
}

fn baris_transaksi(app: &MyDigitApp, ui: &mut egui::Ui, t: &Transaksi) -> egui::Response {
    let p = palet(ui);
    let (ikon, warna, tanda) = match t.jenis {
        JenisTransaksi::Masuk => ("⬇", p.hijau, "+"),
        JenisTransaksi::Keluar => ("⬆", p.merah, "−"),
        JenisTransaksi::Transfer => ("⇄", p.aksen, ""),
    };
    let r = ui
        .horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::hover());
            ui.painter()
                .rect_filled(rect, CornerRadius::same(14), warna.gamma_multiply(0.15));
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                ikon,
                FontId::proportional(13.0),
                warna,
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let warna_n = if t.jenis == JenisTransaksi::Transfer {
                    p.teks
                } else {
                    warna
                };
                ui.label(
                    RichText::new(format!("{tanda}{}", rupiah(t.jumlah)))
                        .strong()
                        .color(warna_n),
                );
                // Sisa lebar untuk uraian; teks panjang dipotong dengan "…".
                let ukuran = Vec2::new(ui.available_width(), 34.0);
                ui.allocate_ui_with_layout(
                    ukuran,
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.add(egui::Label::new(&t.uraian).truncate());
                        let kat = app
                            .kategori(t.kategori_id)
                            .map(|k| k.nama.as_str())
                            .unwrap_or(t.jenis.label());
                        let info = format!(
                            "{} · {} · {}",
                            tanggal_pendek(t.tanggal),
                            kat,
                            app.nama_akun(t.akun_id)
                        );
                        ui.add(
                            egui::Label::new(RichText::new(info).size(11.0).color(p.teks2))
                                .truncate(),
                        );
                    },
                );
            });
        })
        .response;
    r.interact(Sense::click())
}

/// Grafik batang pemasukan (hijau) dan pengeluaran (merah) per bulan.
fn grafik_arus(app: &MyDigitApp, ui: &mut egui::Ui) {
    let p = palet(ui);
    let s = app.sumber();
    let hari = hari_ini();
    let mut data = Vec::new();
    for i in (0..6).rev() {
        let total = hari.year() as i32 * 12 + hari.month() as i32 - 1 - i;
        let (th, bl) = ((total / 12) as i16, (total % 12 + 1) as i8);
        let (m, k) = s.arus(Periode::bulan(th, bl));
        data.push((BULAN[bl as usize - 1][..3].to_string(), m, k));
    }
    let maks = data
        .iter()
        .map(|(_, m, k)| (*m).max(*k))
        .max()
        .unwrap_or(0)
        .max(1) as f32;

    let (rect, resp) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 170.0), Sense::hover());
    let painter = ui.painter_at(rect);
    let area = rect.shrink2(Vec2::new(4.0, 0.0));
    let bawah = area.bottom() - 20.0;
    let tinggi = bawah - area.top() - 6.0;
    for i in 0..=3 {
        let y = bawah - tinggi * i as f32 / 3.0;
        painter.hline(area.x_range(), y, Stroke::new(0.5, p.garis));
    }
    let slot = area.width() / data.len() as f32;
    let lebar_batang = (slot * 0.26).min(18.0);
    let mut hover = None;
    for (i, (label, m, k)) in data.iter().enumerate() {
        let cx = area.left() + slot * (i as f32 + 0.5);
        for (j, (nilai, warna)) in [(*m, p.hijau), (*k, p.merah)].into_iter().enumerate() {
            let h = (nilai as f32 / maks * tinggi).max(if nilai > 0 { 2.0 } else { 0.0 });
            let x = cx + (j as f32 - 1.0) * (lebar_batang + 2.0) + 1.0;
            let batang = egui::Rect::from_min_max(
                egui::pos2(x, bawah - h),
                egui::pos2(x + lebar_batang, bawah),
            );
            painter.rect_filled(
                batang,
                CornerRadius {
                    nw: 4,
                    ne: 4,
                    sw: 0,
                    se: 0,
                },
                warna,
            );
        }
        painter.text(
            egui::pos2(cx, bawah + 4.0),
            Align2::CENTER_TOP,
            label,
            FontId::proportional(11.0),
            p.teks2,
        );
        let kolom = egui::Rect::from_center_size(
            egui::pos2(cx, area.center().y),
            Vec2::new(slot, area.height()),
        );
        if resp.hovered() && resp.hover_pos().is_some_and(|pos| kolom.contains(pos)) {
            hover = Some(i);
        }
    }
    if let Some(i) = hover {
        let (label, m, k) = &data[i];
        resp.on_hover_text(format!(
            "{label}\nPemasukan: {}\nPengeluaran: {}",
            rupiah(*m),
            rupiah(*k)
        ));
    }
    ui.horizontal(|ui| {
        for (warna, label) in [(p.hijau, "Pemasukan"), (p.merah, "Pengeluaran")] {
            let (r, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
            ui.painter().rect_filled(r, CornerRadius::same(3), warna);
            ui.label(RichText::new(label).size(11.0).color(p.teks2));
            ui.add_space(8.0);
        }
    });
}

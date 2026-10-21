//! Ekspor laporan ke Excel (.xlsx) yang sudah diformat dan siap cetak:
//! kop lembaga dengan logo, judul, tabel bergaris dengan header berwarna,
//! nilai bertipe (tanggal, rupiah, persen), baris total, blok tanda tangan,
//! dan pengaturan cetak A4 dengan footer My Digit.

use std::path::Path;

use jiff::civil::Date;
use rust_xlsxwriter::{
    Color, DocProperties, ExcelDateTime, Format, FormatAlign, FormatBorder, FormatUnderline, Image,
    Workbook, Worksheet, XlsxError,
};

use crate::branding;
use crate::format::tanggal_panjang;
use crate::laporan::{JenisKolom, Tabel};
use crate::model::ProfilLembaga;
use crate::pdf::{baris_bagian, baris_kontak};

const BIRU_KEPALA: u32 = 0xE8EEF7;
const GARIS_KEPALA: u32 = 0xB7C3D6;
const GARIS: u32 = 0xD9DEE5;
const SELANG: u32 = 0xF7F9FC;
const BAGIAN: u32 = 0xF0F2F5;
const ABU: u32 = 0x6E6E73;
const TEKS: u32 = 0x1D1D1F;

const FORMAT_UANG: &str = "#,##0;-#,##0";
const FORMAT_PERSEN: &str = "0.0%";
const FORMAT_TANGGAL: &str = "dd/mm/yyyy";

/// Susun laporan menjadi berkas .xlsx.
pub fn laporan(
    profil: &ProfilLembaga,
    logo: Option<&Path>,
    tabel: &Tabel,
    dicetak: Date,
) -> Result<Vec<u8>, String> {
    buat(profil, logo, tabel, dicetak).map_err(|e| format!("Gagal membuat berkas Excel: {e}"))
}

fn buat(
    profil: &ProfilLembaga,
    logo: Option<&Path>,
    tabel: &Tabel,
    dicetak: Date,
) -> Result<Vec<u8>, XlsxError> {
    let mut wb = Workbook::new();
    wb.set_properties(
        &DocProperties::new()
            .set_title(format!("{} — {}", tabel.judul, tabel.subjudul))
            .set_author(format!("{} — {}", branding::APP_NAME, branding::MAKER))
            .set_company(profil.nama.clone()),
    );
    let ws = wb.add_worksheet();
    ws.set_name(nama_sheet(&tabel.judul))?;
    ws.set_screen_gridlines(false);

    let n = tabel.kolom.len() as u16;
    let akhir = n - 1;

    let gambar = logo.and_then(|p| Image::new(p).ok());

    // Lebar kolom menyesuaikan isi terpanjang (dibatasi), dan kolom pertama
    // cukup lebar untuk logo bila ada.
    for (i, k) in tabel.kolom.iter().enumerate() {
        let terpanjang = tabel
            .baris
            .iter()
            .chain(tabel.total.iter())
            .filter_map(|r| r.get(i))
            .map(|s| s.chars().count())
            .chain([k.nama.chars().count()])
            .max()
            .unwrap_or(0);
        let mut lebar = (terpanjang as f64 * 1.1 + 3.0).clamp(8.0, 62.0);
        if i == 0 && gambar.is_some() {
            lebar = lebar.max(11.0);
        }
        ws.set_column_width(i as u16, lebar)?;
    }

    // ---------- Kop ----------
    let dasar = Format::new()
        .set_font_name("Calibri")
        .set_font_color(Color::RGB(TEKS));
    let mut kolom_kop = 0;
    if let Some(img) = gambar {
        let img = img.set_scale_to_size(54, 54, true);
        ws.insert_image_with_offset(0, 0, &img, 4, 4)?;
        kolom_kop = 1;
    }
    let nama = if profil.nama.is_empty() {
        "Nama Lembaga".to_string()
    } else {
        profil.nama.to_uppercase()
    };
    let mut alamat = profil.alamat.clone();
    if !profil.kota.is_empty() {
        if !alamat.is_empty() {
            alamat += ", ";
        }
        alamat += &profil.kota;
    }
    let kontak = baris_kontak(profil);

    // Baris instansi induk (maks. 2) menggeser seluruh isi ke bawah;
    // tanpa induk, tata letaknya sama dengan sebelumnya.
    let induk = profil.baris_induk();
    let o = induk.len() as u32;
    for (i, baris) in induk.iter().enumerate() {
        ws.set_row_height(i as u32, 16)?;
        tulis_gabung(
            ws,
            i as u32,
            kolom_kop,
            akhir,
            baris,
            &dasar.clone().set_bold().set_font_size(11),
        )?;
    }
    ws.set_row_height(o, 22)?;
    ws.set_row_height(o + 1, 16)?;
    ws.set_row_height(o + 2, if kontak.is_empty() { 6 } else { 16 })?;
    tulis_gabung(
        ws,
        o,
        kolom_kop,
        akhir,
        &nama,
        &dasar.clone().set_bold().set_font_size(14),
    )?;
    let kecil = dasar
        .clone()
        .set_font_size(9)
        .set_font_color(Color::RGB(ABU));
    tulis_gabung(ws, o + 1, kolom_kop, akhir, &alamat, &kecil)?;
    tulis_gabung(ws, o + 2, kolom_kop, akhir, &kontak, &kecil)?;
    // Garis ganda di bawah kop, seperti kop surat.
    ws.set_row_height(o + 3, 6)?;
    let garis_kop = dasar.clone().set_border_bottom(FormatBorder::Double);
    for c in 0..n {
        ws.write_blank(o + 3, c, &garis_kop)?;
    }

    // ---------- Judul ----------
    let tengah = dasar.clone().set_align(FormatAlign::Center);
    ws.set_row_height(o + 5, 20)?;
    tulis_gabung(
        ws,
        o + 5,
        0,
        akhir,
        &tabel.judul.to_uppercase(),
        &tengah.clone().set_bold().set_font_size(13),
    )?;
    tulis_gabung(
        ws,
        o + 6,
        0,
        akhir,
        &tabel.subjudul,
        &tengah
            .clone()
            .set_font_size(10)
            .set_font_color(Color::RGB(ABU)),
    )?;

    // ---------- Tabel ----------
    let baris_kepala: u32 = o + 8;
    let sel = |isi: Option<u32>, tebal: bool, atas_tebal: bool| {
        let mut f = dasar
            .clone()
            .set_font_size(10)
            .set_border(FormatBorder::Thin)
            .set_border_color(Color::RGB(GARIS))
            .set_align(FormatAlign::VerticalCenter);
        if let Some(c) = isi {
            f = f.set_background_color(Color::RGB(c));
        }
        if tebal {
            f = f.set_bold();
        }
        if atas_tebal {
            f = f.set_border_top(FormatBorder::Medium);
        }
        f
    };

    ws.set_row_height(baris_kepala, 22)?;
    for (i, k) in tabel.kolom.iter().enumerate() {
        let f = sel(Some(BIRU_KEPALA), true, false)
            .set_border_color(Color::RGB(GARIS_KEPALA))
            .set_align(if k.kanan {
                FormatAlign::Right
            } else {
                FormatAlign::Left
            });
        ws.write_string_with_format(baris_kepala, i as u16, k.nama, &f)?;
    }

    let mut r = baris_kepala + 1;
    for (i, isi) in tabel.baris.iter().enumerate() {
        let bagian = baris_bagian(isi);
        let latar = if bagian {
            Some(BAGIAN)
        } else if i % 2 == 1 {
            Some(SELANG)
        } else {
            None
        };
        tulis_baris(ws, r, tabel, isi, &sel(latar, bagian, false))?;
        ws.set_row_height(r, 18)?;
        r += 1;
    }
    if let Some(total) = &tabel.total {
        tulis_baris(ws, r, tabel, total, &sel(Some(BIRU_KEPALA), true, true))?;
        ws.set_row_height(r, 20)?;
        r += 1;
    }

    // ---------- Tanda tangan ----------
    r += 2;
    let tengah_ttd = dasar
        .clone()
        .set_font_size(10)
        .set_align(FormatAlign::Center);
    let tengah_nama = tengah_ttd
        .clone()
        .set_bold()
        .set_underline(FormatUnderline::Single);
    let tengah_nip = tengah_ttd.clone().set_font_size(9);
    // Blok kiri mendapat separuh kolom pertama (dibulatkan ke atas).
    let bagi = n.div_ceil(2);
    let (kiri, kanan) = ((0, bagi.saturating_sub(1)), (bagi, akhir));
    let kota = if profil.kota.is_empty() {
        String::new()
    } else {
        format!("{}, ", profil.kota)
    };
    let blok = [
        (
            kiri,
            "Mengetahui,".to_string(),
            profil.jabatan_pimpinan.as_str(),
            profil.nama_pimpinan.as_str(),
            ProfilLembaga::baris_nomor(&profil.label_nomor_pimpinan, &profil.nip_pimpinan),
        ),
        (
            kanan,
            format!("{kota}{}", tanggal_panjang(dicetak)),
            profil.jabatan_bendahara.as_str(),
            profil.nama_bendahara.as_str(),
            ProfilLembaga::baris_nomor(&profil.label_nomor_bendahara, &profil.nip_bendahara),
        ),
    ];
    for ((c0, c1), atas, jabatan, nama, nip) in blok {
        tulis_gabung(ws, r, c0, c1, &atas, &tengah_ttd)?;
        tulis_gabung(ws, r + 1, c0, c1, jabatan, &tengah_ttd)?;
        let nama = if nama.is_empty() {
            "(...............................)"
        } else {
            nama
        };
        tulis_gabung(ws, r + 5, c0, c1, nama, &tengah_nama)?;
        if !nip.is_empty() {
            tulis_gabung(ws, r + 6, c0, c1, &nip, &tengah_nip)?;
        }
    }
    r += 8;
    tulis_gabung(
        ws,
        r,
        0,
        akhir,
        branding::PDF_FOOTER,
        &dasar
            .clone()
            .set_font_size(8)
            .set_italic()
            .set_font_color(Color::RGB(ABU)),
    )?;

    // ---------- Cetak ----------
    ws.set_paper_size(9); // A4
    if tabel.mendatar {
        ws.set_landscape();
    }
    ws.set_print_fit_to_pages(1, 0);
    ws.set_print_center_horizontally(true);
    ws.set_margins(0.5, 0.5, 0.6, 0.7, 0.3, 0.3);
    ws.set_repeat_rows(baris_kepala, baris_kepala)?;
    ws.set_footer(format!(
        "&L&8{}&R&8Halaman &P dari &N",
        branding::PDF_FOOTER.replace('&', "&&")
    ));

    wb.save_to_buffer()
}

/// Tulis satu baris tabel; isi kolom diubah ke tipe Excel sesuai jenisnya.
fn tulis_baris(
    ws: &mut Worksheet,
    r: u32,
    tabel: &Tabel,
    isi: &[String],
    f: &Format,
) -> Result<(), XlsxError> {
    for (c, (k, s)) in tabel.kolom.iter().zip(isi).enumerate() {
        let c = c as u16;
        let f_kanan = f.clone().set_align(FormatAlign::Right);
        match (k.jenis, s.as_str()) {
            (_, "") => {
                ws.write_blank(r, c, f)?;
            }
            (JenisKolom::Uang, _) if baca_uang(s).is_some() => {
                ws.write_number_with_format(
                    r,
                    c,
                    baca_uang(s).unwrap() as f64,
                    &f_kanan.set_num_format(FORMAT_UANG),
                )?;
            }
            (JenisKolom::Persen, _) if baca_persen(s).is_some() => {
                ws.write_number_with_format(
                    r,
                    c,
                    baca_persen(s).unwrap(),
                    &f_kanan.set_num_format(FORMAT_PERSEN),
                )?;
            }
            (JenisKolom::Tanggal, _) if baca_tanggal(s).is_some() => {
                let d = baca_tanggal(s).unwrap();
                ws.write_datetime_with_format(
                    r,
                    c,
                    &d,
                    &f.clone()
                        .set_num_format(FORMAT_TANGGAL)
                        .set_align(FormatAlign::Left),
                )?;
            }
            _ => {
                let f = if k.kanan { f_kanan } else { f.clone() };
                ws.write_string_with_format(r, c, s, &f)?;
            }
        }
    }
    Ok(())
}

/// Tulis teks pada rentang sel yang digabung (atau satu sel bila c0 == c1).
fn tulis_gabung(
    ws: &mut Worksheet,
    r: u32,
    c0: u16,
    c1: u16,
    s: &str,
    f: &Format,
) -> Result<(), XlsxError> {
    if c0 >= c1 {
        ws.write_string_with_format(r, c0, s, f)?;
    } else {
        ws.merge_range(r, c0, r, c1, s, f)?;
    }
    Ok(())
}

/// `"1.250.000"` / `"-5.000"` → angka.
fn baca_uang(s: &str) -> Option<i64> {
    let negatif = s.starts_with('-');
    let angka: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if angka.is_empty()
        || s.chars()
            .any(|c| !(c.is_ascii_digit() || c == '.' || c == '-'))
    {
        return None;
    }
    let n: i64 = angka.parse().ok()?;
    Some(if negatif { -n } else { n })
}

/// `"50,0%"` → `0.5`.
fn baca_persen(s: &str) -> Option<f64> {
    let t = s.strip_suffix('%')?.replace(',', ".");
    t.parse::<f64>().ok().map(|v| v / 100.0)
}

/// `"21/10/2026"` → tanggal Excel.
fn baca_tanggal(s: &str) -> Option<ExcelDateTime> {
    let mut it = s.split('/');
    let (d, m, y) = (
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
    );
    ExcelDateTime::from_ymd(y, m, d).ok()
}

/// Nama sheet Excel: maks. 31 karakter, tanpa `[]:*?/\`.
fn nama_sheet(judul: &str) -> String {
    let bersih: String = judul.chars().filter(|c| !"[]:*?/\\".contains(*c)).collect();
    bersih.chars().take(31).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::laporan::{Periode, Sumber};
    use crate::model::*;

    #[test]
    fn baca_nilai() {
        assert_eq!(baca_uang("1.250.000"), Some(1_250_000));
        assert_eq!(baca_uang("-5.000"), Some(-5_000));
        assert_eq!(baca_uang("Saldo awal"), None);
        assert_eq!(baca_persen("50,0%"), Some(0.5));
        assert_eq!(baca_persen("—"), None);
        assert!(baca_tanggal("21/10/2026").is_some());
        assert!(baca_tanggal("4.1").is_none());
        assert_eq!(nama_sheet("Buku Kas Umum"), "Buku Kas Umum");
    }

    #[test]
    fn berkas_xlsx_valid() {
        let akun = vec![Akun {
            id: 1,
            nama: "Kas".into(),
            jenis: JenisAkun::Tunai,
            no_rekening: String::new(),
            saldo_awal: 0,
        }];
        let kategori = vec![Kategori {
            id: 1,
            kode: "5.5".into(),
            nama: "Listrik".into(),
            arus: Arus::Keluar,
        }];
        let transaksi = vec![Transaksi {
            id: 1,
            tanggal: Date::constant(2026, 10, 2),
            no_bukti: "BKK-2026-0001".into(),
            jenis: JenisTransaksi::Keluar,
            akun_id: 1,
            akun_tujuan_id: None,
            kategori_id: Some(1),
            uraian: "Tagihan listrik & internet".into(),
            jumlah: 1_480_000,
            pihak: String::new(),
            sumber_dana: String::new(),
            pungutan: Pungutan::default(),
            setor_pajak: None,
        }];
        let s = Sumber {
            akun: &akun,
            kategori: &kategori,
            transaksi: &transaksi,
        };
        let profil = ProfilLembaga {
            nama: "SMP Negeri 23 Dumai".into(),
            ..Default::default()
        };
        let bytes = laporan(
            &profil,
            None,
            &s.bku(Periode::bulan(2026, 10)),
            Date::constant(2026, 10, 31),
        )
        .unwrap();
        assert_eq!(&bytes[..2], b"PK", "xlsx adalah arsip zip");
        let dengan_induk = ProfilLembaga {
            induk: "Pemerintah Kota Dumai\nDinas Pendidikan".into(),
            ..profil
        };
        let bytes = laporan(
            &dengan_induk,
            None,
            &s.bku(Periode::bulan(2026, 10)),
            Date::constant(2026, 10, 31),
        )
        .unwrap();
        assert_eq!(&bytes[..2], b"PK");
    }
}

/// Buat contoh .xlsx dari database demo untuk diperiksa:
/// `MY_DIGIT_DATA_DIR=target/demo cargo test contoh_excel -- --ignored`
#[cfg(test)]
#[test]
#[ignore]
fn contoh_excel() {
    use crate::laporan::{Periode, Sumber};
    let db = crate::db::Db::buka(&crate::db::folder_data().join(crate::db::NAMA_DB)).unwrap();
    let (akun, kategori, transaksi) = (
        db.daftar_akun().unwrap(),
        db.daftar_kategori().unwrap(),
        db.daftar_transaksi().unwrap(),
    );
    let s = Sumber {
        akun: &akun,
        kategori: &kategori,
        transaksi: &transaksi,
    };
    let hari = crate::format::hari_ini();
    let profil = db.profil();
    let logo = Some(Path::new("assets/logo.png"));
    let out = Path::new("target/shots");
    std::fs::create_dir_all(out).unwrap();
    let tahun = Periode::tahun(hari.year());
    for (nama, t) in [
        ("bku", s.bku(tahun)),
        ("rekap", s.rekap_kategori(tahun)),
        (
            "realisasi",
            s.realisasi_anggaran(hari.year(), &db.daftar_anggaran(hari.year()).unwrap()),
        ),
    ] {
        std::fs::write(
            out.join(format!("{nama}.xlsx")),
            laporan(&profil, logo, &t, hari).unwrap(),
        )
        .unwrap();
    }
}

//! Ekspor dokumen PDF: laporan tabel, kwitansi, dan berita acara kas.
//!
//! Setiap halaman mendapat kop lembaga (halaman pertama) dan footer
//! identitas aplikasi ([`branding::PDF_FOOTER`]) beserta nomor halaman.

use std::path::Path;

use jiff::civil::{Date, Weekday};
use pdf_writer::{Content, Filter, Finish, Name, Pdf, Rect, Ref, Str};

use crate::branding;
use crate::format::{judul_kata, ribuan, rupiah, tanggal_panjang, terbilang};
use crate::laporan::Tabel;
use crate::model::*;

const A4: (f32, f32) = (595.0, 842.0);
const MARGIN: f32 = 42.0;
const FOOTER_Y: f32 = 26.0;
const BATAS_BAWAH: f32 = 52.0;

#[derive(Clone, Copy)]
enum Huruf {
    Biasa,
    Tebal,
    Miring,
}

impl Huruf {
    fn nama(self) -> Name<'static> {
        match self {
            Self::Biasa => Name(b"F1"),
            Self::Tebal => Name(b"F2"),
            Self::Miring => Name(b"F3"),
        }
    }
}

/// Logo lembaga yang siap ditanam ke PDF.
pub struct Gambar {
    rgb: Vec<u8>,
    alpha: Vec<u8>,
    lebar: u32,
    tinggi: u32,
}

impl Gambar {
    pub fn dari_berkas(path: &Path) -> Option<Self> {
        let img = image::open(path).ok()?;
        // Perkecil agar ukuran PDF tetap ringan.
        let img = img.thumbnail(256, 256).into_rgba8();
        let (lebar, tinggi) = img.dimensions();
        let mut rgb = Vec::with_capacity((lebar * tinggi * 3) as usize);
        let mut alpha = Vec::with_capacity((lebar * tinggi) as usize);
        for p in img.pixels() {
            rgb.extend_from_slice(&p.0[..3]);
            alpha.push(p.0[3]);
        }
        Some(Self {
            rgb: miniz_oxide::deflate::compress_to_vec_zlib(&rgb, 6),
            alpha: miniz_oxide::deflate::compress_to_vec_zlib(&alpha, 6),
            lebar,
            tinggi,
        })
    }
}

/// Kanvas multi-halaman sederhana yang mengalir dari atas ke bawah.
struct Kanvas {
    halaman: Vec<Content>,
    w: f32,
    h: f32,
    y: f32,
    ada_logo: bool,
}

impl Kanvas {
    fn new(mendatar: bool, ada_logo: bool) -> Self {
        let (w, h) = if mendatar { (A4.1, A4.0) } else { A4 };
        Self {
            halaman: vec![Content::new()],
            w,
            h,
            y: h - MARGIN,
            ada_logo,
        }
    }

    fn c(&mut self) -> &mut Content {
        self.halaman.last_mut().unwrap()
    }

    fn halaman_baru(&mut self) {
        self.halaman.push(Content::new());
        self.y = self.h - MARGIN;
    }

    /// Pindah halaman bila sisa ruang kurang dari `tinggi`.
    fn pastikan(&mut self, tinggi: f32) -> bool {
        if self.y - tinggi < BATAS_BAWAH {
            self.halaman_baru();
            true
        } else {
            false
        }
    }

    fn teks(&mut self, huruf: Huruf, ukuran: f32, x: f32, y: f32, s: &str) {
        let bytes = win_ansi(s);
        let c = self.c();
        c.begin_text();
        c.set_font(huruf.nama(), ukuran);
        c.next_line(x, y);
        c.show(Str(&bytes));
        c.end_text();
    }

    fn teks_kanan(&mut self, huruf: Huruf, ukuran: f32, x_kanan: f32, y: f32, s: &str) {
        let w = lebar_teks(s, ukuran, huruf);
        self.teks(huruf, ukuran, x_kanan - w, y, s);
    }

    fn teks_tengah(&mut self, huruf: Huruf, ukuran: f32, x_tengah: f32, y: f32, s: &str) {
        let w = lebar_teks(s, ukuran, huruf);
        self.teks(huruf, ukuran, x_tengah - w / 2.0, y, s);
    }

    fn garis(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, tebal: f32) {
        let c = self.c();
        c.set_line_width(tebal);
        c.move_to(x1, y1);
        c.line_to(x2, y2);
        c.stroke();
    }

    fn garis_tipis(&mut self, lebar: f32) {
        let y = self.y;
        self.c().set_stroke_gray(0.82);
        self.garis(MARGIN, y, MARGIN + lebar, y, 0.3);
        self.c().set_stroke_gray(0.0);
    }

    fn kotak_isi(&mut self, x: f32, y: f32, w: f32, h: f32, abu: f32) {
        let c = self.c();
        c.set_fill_gray(abu);
        c.rect(x, y, w, h);
        c.fill_nonzero();
        c.set_fill_gray(0.0);
    }

    fn kotak(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let c = self.c();
        c.set_line_width(0.8);
        c.rect(x, y, w, h);
        c.stroke();
    }

    /// Paragraf dengan pemenggalan baris otomatis.
    fn paragraf(&mut self, huruf: Huruf, ukuran: f32, x: f32, lebar: f32, s: &str) {
        for baris in bungkus(s, lebar, ukuran, huruf) {
            self.pastikan(ukuran * 1.5);
            self.teks(huruf, ukuran, x, self.y - ukuran, &baris);
            self.y -= ukuran * 1.45;
        }
    }

    /// Kop lembaga: logo, nama, alamat, dan garis ganda.
    /// Kop lembaga: logo, instansi induk (maks. 2 baris), nama, alamat,
    /// identitas/kontak, dan garis ganda. Tanpa induk dan identitas,
    /// tata letaknya sama persis dengan My Digit v0.2.
    fn kop(&mut self, p: &ProfilLembaga) {
        let tinggi_logo = 48.0;
        let induk = p.baris_induk();
        let geser = 12.0 * induk.len() as f32;
        let tinggi = tinggi_kop(p) - 8.0;
        let mut x = MARGIN;
        if self.ada_logo {
            let y = self.y - (tinggi - tinggi_logo) / 2.0;
            let c = self.c();
            c.save_state();
            c.transform([tinggi_logo, 0.0, 0.0, tinggi_logo, MARGIN, y - tinggi_logo]);
            c.x_object(Name(b"Im1"));
            c.restore_state();
            x += tinggi_logo + 12.0;
        }
        let lebar = self.w - MARGIN - x;
        let y = self.y;
        for (i, baris) in induk.iter().enumerate() {
            let baris = potong(baris, lebar, 10.0, Huruf::Tebal);
            self.teks(Huruf::Tebal, 10.0, x, y - 10.0 - 12.0 * i as f32, &baris);
        }
        let nama = if p.nama.is_empty() {
            "Nama Lembaga"
        } else {
            &p.nama
        };
        let nama = potong(&nama.to_uppercase(), lebar, 15.0, Huruf::Tebal);
        self.teks(Huruf::Tebal, 15.0, x, y - 16.0 - geser, &nama);
        let mut info = p.alamat.clone();
        if !p.kota.is_empty() {
            if !info.is_empty() {
                info += ", ";
            }
            info += &p.kota;
        }
        self.teks(
            Huruf::Biasa,
            9.0,
            x,
            y - 30.0 - geser,
            &potong(&info, lebar, 9.0, Huruf::Biasa),
        );
        let kontak = baris_kontak(p);
        if !kontak.is_empty() {
            self.teks(
                Huruf::Biasa,
                9.0,
                x,
                y - 42.0 - geser,
                &potong(&kontak, lebar, 9.0, Huruf::Biasa),
            );
        }
        self.y -= tinggi + 8.0;
        let (w, y) = (self.w, self.y);
        self.garis(MARGIN, y, w - MARGIN, y, 1.6);
        self.garis(MARGIN, y - 2.5, w - MARGIN, y - 2.5, 0.5);
        self.y -= 18.0;
    }

    fn judul(&mut self, judul: &str, subjudul: &str) {
        let (tengah, y) = (self.w / 2.0, self.y);
        self.teks_tengah(Huruf::Tebal, 13.0, tengah, y - 13.0, &judul.to_uppercase());
        self.y -= 20.0;
        if !subjudul.is_empty() {
            let y = self.y;
            self.teks_tengah(Huruf::Biasa, 9.5, tengah, y - 9.5, subjudul);
            self.y -= 16.0;
        }
        self.y -= 6.0;
    }

    /// Blok dua tanda tangan. Kolom kanan selalu diawali kota dan tanggal.
    /// Tiap pihak: (baris atas, jabatan, nama, baris nomor induk yang sudah jadi).
    fn tanda_tangan(
        &mut self,
        p: &ProfilLembaga,
        tanggal: Date,
        kiri: [&str; 4],
        kanan: [&str; 4],
    ) {
        self.pastikan(115.0);
        self.y -= 16.0;
        let lebar = (self.w - 2.0 * MARGIN) / 2.0;
        let kota = if p.kota.is_empty() {
            String::new()
        } else {
            format!("{}, ", p.kota)
        };
        let y = self.y;
        for (i, [atas, jabatan, nama, nip]) in [kiri, kanan].into_iter().enumerate() {
            let cx = MARGIN + lebar * i as f32 + lebar / 2.0;
            let atas = if i == 1 {
                format!("{kota}{}", tanggal_panjang(tanggal))
            } else {
                atas.to_string()
            };
            self.teks_tengah(Huruf::Biasa, 10.0, cx, y - 10.0, &atas);
            let jabatan = potong(jabatan, lebar - 10.0, 10.0, Huruf::Biasa);
            self.teks_tengah(Huruf::Biasa, 10.0, cx, y - 24.0, &jabatan);
            let nama = if nama.is_empty() {
                "(...............................)"
            } else {
                nama
            };
            self.teks_tengah(Huruf::Tebal, 10.0, cx, y - 86.0, nama);
            let nw = lebar_teks(nama, 10.0, Huruf::Tebal);
            self.garis(cx - nw / 2.0, y - 88.0, cx + nw / 2.0, y - 88.0, 0.5);
            if !nip.is_empty() {
                self.teks_tengah(Huruf::Biasa, 9.0, cx, y - 99.0, nip);
            }
        }
        self.y -= 108.0;
    }

    /// Susun PDF akhir, menambahkan footer di setiap halaman.
    fn selesai(mut self, logo: Option<&Gambar>) -> Vec<u8> {
        let total = self.halaman.len();
        let (w, h) = (self.w, self.h);
        let kiri = win_ansi(branding::PDF_FOOTER);
        for (i, c) in self.halaman.iter_mut().enumerate() {
            c.set_line_width(0.4);
            c.set_stroke_gray(0.6);
            c.move_to(MARGIN, FOOTER_Y + 10.0);
            c.line_to(w - MARGIN, FOOTER_Y + 10.0);
            c.stroke();
            c.set_stroke_gray(0.0);
            c.set_fill_gray(0.35);
            c.begin_text();
            c.set_font(Huruf::Biasa.nama(), 7.5);
            c.next_line(MARGIN, FOOTER_Y);
            c.show(Str(&kiri));
            c.end_text();
            let nomor = format!("Halaman {} dari {}", i + 1, total);
            let nw = lebar_teks(&nomor, 7.5, Huruf::Biasa);
            c.begin_text();
            c.set_font(Huruf::Biasa.nama(), 7.5);
            c.next_line(w - MARGIN - nw, FOOTER_Y);
            c.show(Str(nomor.as_bytes()));
            c.end_text();
            c.set_fill_gray(0.0);
        }

        let catalog_id = Ref::new(1);
        let tree_id = Ref::new(2);
        let fonts = [Ref::new(3), Ref::new(4), Ref::new(5)];
        let img_id = Ref::new(6);
        let mask_id = Ref::new(7);
        let page_ids: Vec<Ref> = (0..total).map(|i| Ref::new(10 + 2 * i as i32)).collect();

        let mut pdf = Pdf::new();
        pdf.catalog(catalog_id).pages(tree_id);
        pdf.pages(tree_id)
            .kids(page_ids.iter().copied())
            .count(total as i32);
        for (id, base) in
            fonts
                .iter()
                .zip([&b"Helvetica"[..], b"Helvetica-Bold", b"Helvetica-Oblique"])
        {
            pdf.type1_font(*id)
                .base_font(Name(base))
                .encoding_predefined(Name(b"WinAnsiEncoding"));
        }
        if let Some(g) = logo {
            let mut im = pdf.image_xobject(img_id, &g.rgb);
            im.filter(Filter::FlateDecode);
            im.width(g.lebar as i32);
            im.height(g.tinggi as i32);
            im.color_space().device_rgb();
            im.bits_per_component(8);
            im.s_mask(mask_id);
            im.finish();
            let mut m = pdf.image_xobject(mask_id, &g.alpha);
            m.filter(Filter::FlateDecode);
            m.width(g.lebar as i32);
            m.height(g.tinggi as i32);
            m.color_space().device_gray();
            m.bits_per_component(8);
            m.finish();
        }
        for (i, content) in self.halaman.into_iter().enumerate() {
            let page_id = page_ids[i];
            let content_id = Ref::new(page_id.get() + 1);
            let mut page = pdf.page(page_id);
            page.media_box(Rect::new(0.0, 0.0, w, h));
            page.parent(tree_id);
            page.contents(content_id);
            {
                let mut res = page.resources();
                {
                    let mut f = res.fonts();
                    f.pair(Name(b"F1"), fonts[0]);
                    f.pair(Name(b"F2"), fonts[1]);
                    f.pair(Name(b"F3"), fonts[2]);
                }
                if logo.is_some() {
                    res.x_objects().pair(Name(b"Im1"), img_id);
                }
            }
            page.finish();
            pdf.stream(content_id, &content.finish());
        }
        pdf.finish()
    }
}

// ---------- Dokumen ----------

/// Laporan tabel (BKU, buku pembantu, rekap, realisasi).
pub fn laporan(
    profil: &ProfilLembaga,
    logo: Option<&Gambar>,
    tabel: &Tabel,
    dicetak: Date,
) -> Vec<u8> {
    let mut k = Kanvas::new(tabel.mendatar, logo.is_some());
    k.kop(profil);
    k.judul(&tabel.judul, &tabel.subjudul);

    let lebar_total = k.w - 2.0 * MARGIN;
    let satuan: f32 = tabel.kolom.iter().map(|c| c.lebar).sum();
    let lebar: Vec<f32> = tabel
        .kolom
        .iter()
        .map(|c| c.lebar / satuan * lebar_total)
        .collect();
    let ukuran = 8.5;
    let tinggi_baris = 15.0;
    let pad = 4.0;

    let kepala = |k: &mut Kanvas| {
        k.kotak_isi(MARGIN, k.y - tinggi_baris, lebar_total, tinggi_baris, 0.88);
        let mut x = MARGIN;
        let y = k.y - tinggi_baris + 4.5;
        for (c, w) in tabel.kolom.iter().zip(&lebar) {
            if c.kanan {
                k.teks_kanan(Huruf::Tebal, ukuran, x + w - pad, y, c.nama);
            } else {
                k.teks(Huruf::Tebal, ukuran, x + pad, y, c.nama);
            }
            x += w;
        }
        k.y -= tinggi_baris;
    };
    let baris = |k: &mut Kanvas, isi: &[String], huruf: Huruf| {
        let mut x = MARGIN;
        let y = k.y - tinggi_baris + 4.5;
        for ((c, w), s) in tabel.kolom.iter().zip(&lebar).zip(isi) {
            let s = potong(s, w - 2.0 * pad, ukuran, huruf);
            if c.kanan {
                k.teks_kanan(huruf, ukuran, x + w - pad, y, &s);
            } else {
                k.teks(huruf, ukuran, x + pad, y, &s);
            }
            x += w;
        }
        k.y -= tinggi_baris;
        k.garis_tipis(lebar_total);
    };

    kepala(&mut k);
    for isi in &tabel.baris {
        if k.pastikan(tinggi_baris) {
            kepala(&mut k);
        }
        baris(
            &mut k,
            isi,
            if baris_bagian(isi) {
                Huruf::Tebal
            } else {
                Huruf::Biasa
            },
        );
    }
    if let Some(total) = &tabel.total {
        if k.pastikan(tinggi_baris) {
            kepala(&mut k);
        }
        k.kotak_isi(MARGIN, k.y - tinggi_baris, lebar_total, tinggi_baris, 0.94);
        baris(&mut k, total, Huruf::Tebal);
    }
    let (nomor_kiri, nomor_kanan) = nomor_penanda_tangan(profil);
    k.tanda_tangan(
        profil,
        dicetak,
        [
            "Mengetahui,",
            &profil.jabatan_pimpinan,
            &profil.nama_pimpinan,
            &nomor_kiri,
        ],
        [
            "",
            &profil.jabatan_bendahara,
            &profil.nama_bendahara,
            &nomor_kanan,
        ],
    );
    k.selesai(logo)
}

/// Baris nomor induk pimpinan dan bendahara, mis. `"NIP. 1970…"`.
fn nomor_penanda_tangan(p: &ProfilLembaga) -> (String, String) {
    (
        ProfilLembaga::baris_nomor(&p.label_nomor_pimpinan, &p.nip_pimpinan),
        ProfilLembaga::baris_nomor(&p.label_nomor_bendahara, &p.nip_bendahara),
    )
}

/// Baris identitas dan kontak di kop, mis. `"NPSN 10404123 · Telp/Email: …"`.
pub fn baris_kontak(p: &ProfilLembaga) -> String {
    let kontak: Vec<&str> = [p.telepon.as_str(), p.email.as_str()]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();
    let kontak = if kontak.is_empty() {
        String::new()
    } else {
        format!("Telp/Email: {}", kontak.join(" · "))
    };
    [p.baris_identitas(), kontak]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

/// Tinggi kop (termasuk jarak ke garis): 56 pt, +12 pt per baris induk.
fn tinggi_kop(p: &ProfilLembaga) -> f32 {
    48.0 + 12.0 * p.baris_induk().len() as f32 + 8.0
}

/// Baris judul bagian (mis. "PENERIMAAN"): satu sel huruf kapital semua.
pub fn baris_bagian(isi: &[String]) -> bool {
    let terisi: Vec<&String> = isi.iter().filter(|s| !s.is_empty()).collect();
    terisi.len() == 1
        && terisi[0].chars().any(|c| c.is_alphabetic())
        && !terisi[0].chars().any(|c| c.is_lowercase())
}

/// Kwitansi untuk satu transaksi pemasukan atau pengeluaran.
pub fn kwitansi(
    profil: &ProfilLembaga,
    logo: Option<&Gambar>,
    t: &Transaksi,
    kategori: Option<&Kategori>,
) -> Vec<u8> {
    let mut k = Kanvas::new(false, logo.is_some());
    k.kop(profil);
    k.judul("Kwitansi", &format!("No. {}", t.no_bukti));

    let jabatan = profil.jabatan_bendahara.as_str();
    let bendahara = if profil.nama.is_empty() {
        jabatan.to_string()
    } else {
        format!("{jabatan} {}", profil.nama)
    };
    let (_, nomor) = nomor_penanda_tangan(profil);
    // (sudah terima dari, [kiri: penyerah], [kanan: penerima])
    let (dari, kiri, kanan) = match t.jenis {
        JenisTransaksi::Masuk => (
            or_titik(&t.pihak).to_string(),
            ["", "Penyetor", t.pihak.as_str(), ""],
            ["", jabatan, profil.nama_bendahara.as_str(), nomor.as_str()],
        ),
        _ => (
            bendahara,
            [
                "Lunas dibayar,",
                jabatan,
                profil.nama_bendahara.as_str(),
                nomor.as_str(),
            ],
            ["", "Penerima", t.pihak.as_str(), ""],
        ),
    };

    let x_label = MARGIN + 6.0;
    let x_isi = MARGIN + 130.0;
    let lebar_isi = k.w - MARGIN - x_isi;
    let baris = |k: &mut Kanvas, label: &str, isi: &str, huruf: Huruf| {
        let y = k.y;
        k.teks(Huruf::Biasa, 10.5, x_label, y - 10.5, label);
        k.teks(Huruf::Biasa, 10.5, x_isi - 10.0, y - 10.5, ":");
        k.paragraf(huruf, 10.5, x_isi, lebar_isi, isi);
        k.y -= 6.0;
    };
    baris(&mut k, "Sudah terima dari", &dari, Huruf::Tebal);
    baris(
        &mut k,
        "Uang sejumlah",
        &format!("{} Rupiah", judul_kata(&terbilang(t.jumlah))),
        Huruf::Miring,
    );
    let mut untuk = t.uraian.clone();
    if let Some(kat) = kategori {
        untuk += &format!(" ({} {})", kat.kode, kat.nama);
    }
    baris(&mut k, "Untuk pembayaran", &untuk, Huruf::Biasa);
    if !t.sumber_dana.is_empty() {
        baris(&mut k, "Sumber dana", &t.sumber_dana, Huruf::Biasa);
    }
    if t.pungutan.total() > 0 {
        let rinci: Vec<String> = JenisPajak::SEMUA
            .iter()
            .filter(|p| t.pungutan.get(**p) > 0)
            .map(|p| format!("{} {}", p.label(), rupiah(t.pungutan.get(*p))))
            .collect();
        baris(
            &mut k,
            "Potongan pajak",
            &format!(
                "{} — diterima bersih {}",
                rinci.join(", "),
                rupiah(t.jumlah - t.pungutan.total())
            ),
            Huruf::Biasa,
        );
    }

    k.y -= 10.0;
    let teks = format!("{},-", rupiah(t.jumlah));
    let w = lebar_teks(&teks, 14.0, Huruf::Tebal) + 28.0;
    let y = k.y;
    k.kotak_isi(MARGIN, y - 30.0, w, 30.0, 0.92);
    k.kotak(MARGIN, y - 30.0, w, 30.0);
    k.teks(Huruf::Tebal, 14.0, MARGIN + 14.0, y - 20.0, &teks);
    k.y -= 30.0;

    k.tanda_tangan(profil, t.tanggal, kiri, kanan);
    k.selesai(logo)
}

/// Pecahan uang untuk pemeriksaan kas fisik.
pub const PECAHAN: [i64; 11] = [
    100_000, 50_000, 20_000, 10_000, 5_000, 2_000, 1_000, 500, 200, 100, 50,
];

pub struct DataOpname<'a> {
    pub tanggal: Date,
    pub akun: &'a str,
    /// Jumlah lembar/keping untuk tiap [`PECAHAN`].
    pub jumlah: &'a [u32; PECAHAN.len()],
    pub saldo_buku: i64,
    pub keterangan: &'a str,
}

impl DataOpname<'_> {
    pub fn total_fisik(&self) -> i64 {
        PECAHAN
            .iter()
            .zip(self.jumlah)
            .map(|(p, n)| p * *n as i64)
            .sum()
    }
}

/// Berita Acara Pemeriksaan Kas (cash opname).
pub fn berita_acara_kas(profil: &ProfilLembaga, logo: Option<&Gambar>, d: &DataOpname) -> Vec<u8> {
    let mut k = Kanvas::new(false, logo.is_some());
    k.kop(profil);
    k.judul("Berita Acara Pemeriksaan Kas", "");

    let lebar = k.w - 2.0 * MARGIN;
    let lembaga = if profil.nama.is_empty() {
        "lembaga".to_string()
    } else {
        profil.nama.clone()
    };
    k.paragraf(
        Huruf::Biasa,
        10.5,
        MARGIN,
        lebar,
        &format!(
            "Pada hari ini, {}, tanggal {}, telah dilakukan pemeriksaan {} pada {} \
             yang dikelola oleh {}, dengan hasil sebagai berikut:",
            nama_hari(d.tanggal.weekday()),
            tanggal_panjang(d.tanggal),
            d.akun,
            lembaga,
            profil.jabatan_bendahara
        ),
    );
    k.y -= 8.0;

    let kol = [MARGIN, MARGIN + 300.0, MARGIN + lebar];
    let tinggi = 15.0;
    let y = k.y;
    k.kotak_isi(MARGIN, y - tinggi, lebar, tinggi, 0.88);
    k.teks(Huruf::Tebal, 9.0, kol[0] + 6.0, y - 10.5, "Pecahan");
    k.teks_kanan(Huruf::Tebal, 9.0, kol[1] - 6.0, y - 10.5, "Lembar / Keping");
    k.teks_kanan(Huruf::Tebal, 9.0, kol[2] - 6.0, y - 10.5, "Nilai");
    k.y -= tinggi;
    for (p, n) in PECAHAN.iter().zip(d.jumlah) {
        let jenis = if *p >= 1_000 { "kertas" } else { "logam" };
        let y = k.y;
        k.teks(
            Huruf::Biasa,
            9.5,
            kol[0] + 6.0,
            y - 10.5,
            &format!("{} ({jenis})", rupiah(*p)),
        );
        k.teks_kanan(Huruf::Biasa, 9.5, kol[1] - 6.0, y - 10.5, &n.to_string());
        k.teks_kanan(
            Huruf::Biasa,
            9.5,
            kol[2] - 6.0,
            y - 10.5,
            &ribuan(p * *n as i64),
        );
        k.y -= tinggi;
        k.garis_tipis(lebar);
    }
    k.y -= 10.0;
    let fisik = d.total_fisik();
    let selisih = fisik - d.saldo_buku;
    for (label, nilai) in [
        ("Jumlah uang fisik (A)", fisik),
        ("Saldo menurut buku kas (B)", d.saldo_buku),
        ("Selisih (A − B)", selisih),
    ] {
        let y = k.y;
        k.teks(Huruf::Tebal, 10.0, kol[0] + 6.0, y - 11.0, label);
        k.teks_kanan(Huruf::Tebal, 10.0, kol[2] - 6.0, y - 11.0, &rupiah(nilai));
        k.y -= 17.0;
    }
    k.y -= 6.0;
    let kesimpulan = match selisih {
        0 => "Uang fisik sesuai dengan saldo buku kas.".to_string(),
        s if s > 0 => format!("Terdapat kelebihan kas sebesar {}.", rupiah(s)),
        s => format!("Terdapat kekurangan kas sebesar {}.", rupiah(-s)),
    };
    k.paragraf(Huruf::Biasa, 10.5, MARGIN, lebar, &kesimpulan);
    if !d.keterangan.trim().is_empty() {
        k.paragraf(
            Huruf::Biasa,
            10.5,
            MARGIN,
            lebar,
            &format!("Keterangan: {}", d.keterangan.trim()),
        );
    }
    k.paragraf(
        Huruf::Biasa,
        10.5,
        MARGIN,
        lebar,
        "Demikian berita acara ini dibuat dengan sebenarnya untuk dipergunakan sebagaimana mestinya.",
    );
    let (nomor_kiri, nomor_kanan) = nomor_penanda_tangan(profil);
    k.tanda_tangan(
        profil,
        d.tanggal,
        [
            "Yang memeriksa,",
            &profil.jabatan_pimpinan,
            &profil.nama_pimpinan,
            &nomor_kiri,
        ],
        [
            "",
            &format!("Yang diperiksa, {}", profil.jabatan_bendahara),
            &profil.nama_bendahara,
            &nomor_kanan,
        ],
    );
    k.selesai(logo)
}

// ---------- Utilitas teks ----------

fn or_titik(s: &str) -> &str {
    if s.trim().is_empty() {
        "................................"
    } else {
        s
    }
}

fn nama_hari(w: Weekday) -> &'static str {
    match w {
        Weekday::Monday => "Senin",
        Weekday::Tuesday => "Selasa",
        Weekday::Wednesday => "Rabu",
        Weekday::Thursday => "Kamis",
        Weekday::Friday => "Jumat",
        Weekday::Saturday => "Sabtu",
        Weekday::Sunday => "Minggu",
    }
}

/// Lebar glyph Helvetica (per 1000 unit) untuk karakter ASCII 32..=126.
const LEBAR_HELVETICA: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556,
    556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556, 1015, 667, 667, 722, 722, 667,
    611, 778, 722, 278, 500, 667, 556, 833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667,
    667, 611, 278, 278, 278, 469, 556, 333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500,
    222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584,
];

/// Lebar glyph Helvetica-Bold (per 1000 unit) untuk karakter ASCII 32..=126.
const LEBAR_HELVETICA_TEBAL: [u16; 95] = [
    278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556,
    556, 556, 556, 556, 556, 556, 556, 333, 333, 584, 584, 584, 611, 975, 722, 722, 722, 722, 667,
    611, 778, 722, 278, 556, 722, 611, 833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667,
    667, 611, 333, 278, 333, 584, 556, 333, 556, 611, 556, 611, 556, 333, 611, 611, 278, 278, 556,
    278, 889, 611, 611, 611, 611, 389, 556, 333, 611, 556, 778, 556, 556, 500, 389, 280, 389, 584,
];

fn lebar_teks(s: &str, ukuran: f32, huruf: Huruf) -> f32 {
    let tabel = if matches!(huruf, Huruf::Tebal) {
        &LEBAR_HELVETICA_TEBAL
    } else {
        &LEBAR_HELVETICA
    };
    let unit: u32 = s
        .chars()
        .map(|c| match c as u32 {
            32..=126 => tabel[c as usize - 32] as u32,
            _ => 556,
        })
        .sum();
    unit as f32 / 1000.0 * ukuran
}

/// Potong teks dengan "..." agar muat di `lebar`.
fn potong(s: &str, lebar: f32, ukuran: f32, huruf: Huruf) -> String {
    if lebar_teks(s, ukuran, huruf) <= lebar {
        return s.to_string();
    }
    let mut out: String = s.to_string();
    while !out.is_empty() && lebar_teks(&format!("{out}..."), ukuran, huruf) > lebar {
        out.pop();
    }
    format!("{}...", out.trim_end())
}

fn bungkus(s: &str, lebar: f32, ukuran: f32, huruf: Huruf) -> Vec<String> {
    let mut hasil = Vec::new();
    let mut baris = String::new();
    for kata in s.split_whitespace() {
        let calon = if baris.is_empty() {
            kata.to_string()
        } else {
            format!("{baris} {kata}")
        };
        if lebar_teks(&calon, ukuran, huruf) > lebar && !baris.is_empty() {
            hasil.push(std::mem::take(&mut baris));
            baris = kata.to_string();
        } else {
            baris = calon;
        }
    }
    if !baris.is_empty() {
        hasil.push(baris);
    }
    hasil
}

/// Ubah teks ke WinAnsiEncoding untuk font standar PDF.
/// Karakter di luar Latin-1 (selain beberapa tanda baca umum) diganti `?`.
fn win_ansi(s: &str) -> Vec<u8> {
    s.chars()
        .map(|ch| match ch {
            '—' => 0x97,
            '–' => 0x96,
            '−' => b'-',
            '‘' => 0x91,
            '’' => 0x92,
            '“' => 0x93,
            '”' => 0x94,
            '•' | '·' => 0x95,
            '…' => 0x85,
            '€' => 0x80,
            c if (c as u32) < 0x80 || (0xA0..=0xFF).contains(&(c as u32)) => c as u8,
            _ => b'?',
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::laporan::{Periode, Sumber};

    fn hitung(hay: &[u8], needle: &[u8]) -> usize {
        hay.windows(needle.len()).filter(|w| *w == needle).count()
    }

    /// pdf-writer menulis string non-ASCII (ada tanda —) sebagai string hex.
    fn footer_hex() -> Vec<u8> {
        win_ansi(branding::PDF_FOOTER)
            .iter()
            .flat_map(|b| format!("{b:02X}").into_bytes())
            .collect()
    }

    pub fn transaksi_contoh(n: i64) -> Vec<Transaksi> {
        (1..=n)
            .map(|i| Transaksi {
                id: i,
                tanggal: Date::constant(2026, 10, 1),
                no_bukti: format!("BKM-2026-{i:04}"),
                jenis: JenisTransaksi::Masuk,
                akun_id: 1,
                akun_tujuan_id: None,
                kategori_id: None,
                uraian: format!("Penerimaan iuran komite ke-{i}"),
                jumlah: 150_000,
                pihak: "Wali murid".into(),
                sumber_dana: "Komite".into(),
                pungutan: Pungutan::default(),
                setor_pajak: None,
            })
            .collect()
    }

    #[test]
    fn setiap_halaman_memuat_footer() {
        let profil = ProfilLembaga {
            nama: "SMP Negeri 23 Dumai".into(),
            ..Default::default()
        };
        let akun = vec![Akun {
            id: 1,
            nama: "Kas".into(),
            jenis: JenisAkun::Tunai,
            no_rekening: String::new(),
            saldo_awal: 0,
        }];
        let transaksi = transaksi_contoh(120);
        let s = Sumber {
            akun: &akun,
            kategori: &[],
            transaksi: &transaksi,
        };
        let tabel = s.bku(Periode::bulan(2026, 10));
        let bytes = laporan(&profil, None, &tabel, Date::constant(2026, 10, 31));

        let halaman = hitung(&bytes, b"Halaman ");
        assert!(
            halaman >= 2,
            "laporan panjang harus lebih dari satu halaman"
        );
        assert_eq!(hitung(&bytes, &footer_hex()), halaman);

        let t = &transaksi[0];
        let kw = kwitansi(&profil, None, t, None);
        assert_eq!(hitung(&kw, &footer_hex()), 1);
    }

    #[test]
    fn potong_dan_bungkus() {
        let s = "Pembelian alat tulis kantor untuk kegiatan ujian semester";
        assert!(potong(s, 60.0, 9.0, Huruf::Biasa).ends_with("..."));
        assert!(bungkus(s, 100.0, 10.0, Huruf::Biasa).len() > 1);
        assert!(baris_bagian(&["".into(), "PENERIMAAN".into(), "".into()]));
        assert!(!baris_bagian(&[
            "4.1".into(),
            "Dana BOS".into(),
            "0".into()
        ]));
    }
}

/// Buat contoh PDF dari database demo untuk diperiksa:
/// `MY_DIGIT_DATA_DIR=target/demo cargo test contoh_pdf -- --ignored`
#[cfg(test)]
#[test]
#[ignore]
fn contoh_pdf() {
    use crate::laporan::{Periode, Sumber};
    let db = crate::db::Db::buka(&crate::db::folder_data().join(crate::db::NAMA_DB)).unwrap();
    let (akun, kategori, transaksi) = (
        db.daftar_akun().unwrap(),
        db.daftar_kategori().unwrap(),
        db.daftar_transaksi().unwrap(),
    );
    let profil = db.profil();
    let logo = Gambar::dari_berkas(Path::new("assets/logo.png"));
    let s = Sumber {
        akun: &akun,
        kategori: &kategori,
        transaksi: &transaksi,
    };
    let hari = crate::format::hari_ini();
    let out = Path::new("target/shots");
    std::fs::create_dir_all(out).unwrap();
    let bku = s.bku(Periode::tahun(hari.year()));
    std::fs::write(
        out.join("bku.pdf"),
        laporan(&profil, logo.as_ref(), &bku, hari),
    )
    .unwrap();
    let pajak = s.pembantu_pajak(Periode::tahun(hari.year()));
    std::fs::write(
        out.join("pajak.pdf"),
        laporan(&profil, logo.as_ref(), &pajak, hari),
    )
    .unwrap();
    let t = transaksi.iter().find(|t| t.pungutan.total() > 0).unwrap();
    std::fs::write(
        out.join("kwitansi.pdf"),
        kwitansi(
            &profil,
            logo.as_ref(),
            t,
            kategori.iter().find(|k| Some(k.id) == t.kategori_id),
        ),
    )
    .unwrap();
    let mut jumlah = [0u32; PECAHAN.len()];
    jumlah[0] = 69;
    jumlah[1] = 1;
    jumlah[3] = 3;
    let d = DataOpname {
        tanggal: hari,
        akun: "Kas Tunai",
        jumlah: &jumlah,
        saldo_buku: s.saldo_akun(1),
        keterangan: "",
    };
    std::fs::write(
        out.join("opname.pdf"),
        berita_acara_kas(&profil, logo.as_ref(), &d),
    )
    .unwrap();
}

#[cfg(test)]
mod tests_penanda_tangan {
    use super::*;
    use crate::laporan::{Periode, Sumber};

    fn ada(hay: &[u8], s: &str) -> bool {
        hay.windows(s.len()).any(|w| w == s.as_bytes())
    }

    #[test]
    fn kop_tanpa_isian_baru_identik() {
        let mut p = ProfilLembaga::default();
        assert_eq!(tinggi_kop(&p), 56.0, "sama dengan v0.2");
        p.induk = "Pemerintah Kota Dumai\nDinas Pendidikan\nBaris ketiga diabaikan".into();
        assert_eq!(
            p.baris_induk(),
            ["PEMERINTAH KOTA DUMAI", "DINAS PENDIDIKAN"]
        );
        assert_eq!(tinggi_kop(&p), 80.0);
    }

    #[test]
    fn ttd_label_nomor() {
        let akun = vec![Akun {
            id: 1,
            nama: "Kas".into(),
            jenis: JenisAkun::Tunai,
            no_rekening: String::new(),
            saldo_awal: 0,
        }];
        let s = Sumber {
            akun: &akun,
            kategori: &[],
            transaksi: &[],
        };
        let tabel = s.bku(Periode::bulan(2026, 10));
        let tgl = Date::constant(2026, 10, 31);
        let mut p = ProfilLembaga {
            nama: "Masjid Al-Ikhlas".into(),
            jabatan_bendahara: "Bendahara Masjid".into(),
            label_nomor_pimpinan: "NIY".into(),
            nip_pimpinan: "123".into(),
            label_nomor_bendahara: String::new(),
            nip_bendahara: "456789".into(),
            id1_label: "No. SK Kemenkumham".into(),
            id1_nomor: "AHU-01".into(),
            ..Default::default()
        };
        let lap = laporan(&p, None, &tabel, tgl);
        assert!(ada(&lap, "NIY. 123"));
        assert!(ada(&lap, "(456789)"), "label kosong → nomor saja");
        assert!(!ada(&lap, "NIP."));
        assert!(ada(&lap, "Bendahara Masjid"));
        assert!(ada(&lap, "No. SK Kemenkumham AHU-01"));

        let t = Transaksi {
            id: 1,
            tanggal: tgl,
            no_bukti: "BKK-2026-0001".into(),
            jenis: JenisTransaksi::Keluar,
            akun_id: 1,
            akun_tujuan_id: None,
            kategori_id: None,
            uraian: "Santunan".into(),
            jumlah: 100_000,
            pihak: "Panti".into(),
            sumber_dana: String::new(),
            pungutan: Pungutan::default(),
            setor_pajak: None,
        };
        assert!(ada(&kwitansi(&p, None, &t, None), "Bendahara Masjid"));
        let jumlah = [0u32; PECAHAN.len()];
        let d = DataOpname {
            tanggal: tgl,
            akun: "Kas",
            jumlah: &jumlah,
            saldo_buku: 0,
            keterangan: "",
        };
        assert!(ada(
            &berita_acara_kas(&p, None, &d),
            "Yang diperiksa, Bendahara Masjid"
        ));

        p.label_nomor_bendahara = "NIP".into();
        assert!(ada(&laporan(&p, None, &tabel, tgl), "NIP. 456789"));
    }
}

//! Penyimpanan data di SQLite.
//!
//! Database berada di folder data aplikasi sesuai sistem operasi:
//! - macOS: `~/Library/Application Support/id.SIDIGIT-23.My-Digit/`
//! - Windows: `%APPDATA%\SIDIGIT-23\My Digit\data\`
//!
//! Variabel lingkungan `MY_DIGIT_DATA_DIR` dapat menimpa lokasi ini
//! (berguna untuk pengujian).

use std::path::{Path, PathBuf};

use jiff::civil::Date;
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::model::*;

pub type Hasil<T> = Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

pub const NAMA_DB: &str = "my-digit.sqlite";

/// Folder data aplikasi.
pub fn folder_data() -> PathBuf {
    if let Some(dir) = std::env::var_os("MY_DIGIT_DATA_DIR") {
        return PathBuf::from(dir);
    }
    directories::ProjectDirs::from("id", "SIDIGIT-23", "My Digit")
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("my-digit-data"))
}

pub struct Db {
    conn: Connection,
    path: PathBuf,
}

const SKEMA: &str = r#"
CREATE TABLE IF NOT EXISTS akun (
    id INTEGER PRIMARY KEY,
    nama TEXT NOT NULL,
    jenis TEXT NOT NULL,
    no_rekening TEXT NOT NULL DEFAULT '',
    saldo_awal INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS kategori (
    id INTEGER PRIMARY KEY,
    kode TEXT NOT NULL,
    nama TEXT NOT NULL,
    arus TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS transaksi (
    id INTEGER PRIMARY KEY,
    tanggal TEXT NOT NULL,
    no_bukti TEXT NOT NULL,
    jenis TEXT NOT NULL,
    akun_id INTEGER NOT NULL REFERENCES akun(id),
    akun_tujuan_id INTEGER REFERENCES akun(id),
    kategori_id INTEGER REFERENCES kategori(id),
    uraian TEXT NOT NULL,
    jumlah INTEGER NOT NULL,
    pihak TEXT NOT NULL DEFAULT '',
    sumber_dana TEXT NOT NULL DEFAULT '',
    ppn INTEGER NOT NULL DEFAULT 0,
    pph21 INTEGER NOT NULL DEFAULT 0,
    pph22 INTEGER NOT NULL DEFAULT 0,
    pph23 INTEGER NOT NULL DEFAULT 0,
    setor_pajak TEXT
);
CREATE INDEX IF NOT EXISTS transaksi_tanggal ON transaksi(tanggal);
CREATE TABLE IF NOT EXISTS anggaran (
    id INTEGER PRIMARY KEY,
    tahun INTEGER NOT NULL,
    kategori_id INTEGER NOT NULL REFERENCES kategori(id),
    jumlah INTEGER NOT NULL,
    UNIQUE (tahun, kategori_id)
);
CREATE TABLE IF NOT EXISTS pengaturan (
    kunci TEXT PRIMARY KEY,
    nilai TEXT NOT NULL
);
"#;

const KATEGORI_AWAL: &[(&str, &str, Arus)] = &[
    ("4.1", "Dana BOS", Arus::Masuk),
    ("4.2", "Iuran / Dana Komite", Arus::Masuk),
    ("4.3", "Donasi & Sumbangan", Arus::Masuk),
    ("4.4", "Jasa Giro / Bunga Bank", Arus::Masuk),
    ("4.9", "Penerimaan Lain-lain", Arus::Masuk),
    ("5.1", "Honor & Belanja Pegawai", Arus::Keluar),
    ("5.2", "Alat Tulis & Bahan Habis Pakai", Arus::Keluar),
    ("5.3", "Kegiatan Pembelajaran", Arus::Keluar),
    ("5.4", "Pemeliharaan Sarana Prasarana", Arus::Keluar),
    ("5.5", "Langganan Daya & Jasa", Arus::Keluar),
    ("5.6", "Belanja Modal / Peralatan", Arus::Keluar),
    ("5.7", "Biaya Administrasi Bank", Arus::Keluar),
    ("5.8", "Setoran Pajak", Arus::Keluar),
    ("5.9", "Pengeluaran Lain-lain", Arus::Keluar),
];

impl Db {
    /// Buka database di folder data aplikasi, dibuat bila belum ada.
    pub fn buka_bawaan() -> Hasil<Self> {
        let dir = folder_data();
        std::fs::create_dir_all(&dir).map_err(err)?;
        Self::buka(&dir.join(NAMA_DB))
    }

    pub fn buka(path: &Path) -> Hasil<Self> {
        let conn = Connection::open(path).map_err(err)?;
        let db = Self {
            conn,
            path: path.to_path_buf(),
        };
        db.siapkan()?;
        Ok(db)
    }

    #[cfg(test)]
    pub fn memori() -> Self {
        let db = Self {
            conn: Connection::open_in_memory().unwrap(),
            path: PathBuf::new(),
        };
        db.siapkan().unwrap();
        db
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn siapkan(&self) -> Hasil<()> {
        self.conn
            .execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(err)?;
        self.conn.execute_batch(SKEMA).map_err(err)?;
        let kosong: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM akun", [], |r| r.get(0))
            .map_err(err)?;
        if kosong == 0 {
            self.conn
                .execute_batch(
                    "INSERT INTO akun (nama, jenis) VALUES ('Kas Tunai', 'tunai');
                     INSERT INTO akun (nama, jenis) VALUES ('Rekening Bank', 'bank');",
                )
                .map_err(err)?;
            for (kode, nama, arus) in KATEGORI_AWAL {
                self.conn
                    .execute(
                        "INSERT INTO kategori (kode, nama, arus) VALUES (?1, ?2, ?3)",
                        params![kode, nama, arus.kode()],
                    )
                    .map_err(err)?;
            }
        }
        Ok(())
    }

    // ---------- Akun ----------

    pub fn daftar_akun(&self) -> Hasil<Vec<Akun>> {
        let mut st = self
            .conn
            .prepare(
                "SELECT id, nama, jenis, no_rekening, saldo_awal FROM akun ORDER BY jenis DESC, id",
            )
            .map_err(err)?;
        st.query_map([], |r| {
            Ok(Akun {
                id: r.get(0)?,
                nama: r.get(1)?,
                jenis: JenisAkun::dari_kode(&r.get::<_, String>(2)?),
                no_rekening: r.get(3)?,
                saldo_awal: r.get(4)?,
            })
        })
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err)
    }

    pub fn simpan_akun(&self, a: &Akun) -> Hasil<()> {
        if a.id == 0 {
            self.conn.execute(
                "INSERT INTO akun (nama, jenis, no_rekening, saldo_awal) VALUES (?1, ?2, ?3, ?4)",
                params![a.nama, a.jenis.kode(), a.no_rekening, a.saldo_awal],
            )
        } else {
            self.conn.execute(
                "UPDATE akun SET nama = ?1, jenis = ?2, no_rekening = ?3, saldo_awal = ?4 WHERE id = ?5",
                params![a.nama, a.jenis.kode(), a.no_rekening, a.saldo_awal, a.id],
            )
        }
        .map(|_| ())
        .map_err(err)
    }

    pub fn hapus_akun(&self, id: i64) -> Hasil<()> {
        let dipakai: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM transaksi WHERE akun_id = ?1 OR akun_tujuan_id = ?1",
                [id],
                |r| r.get(0),
            )
            .map_err(err)?;
        if dipakai > 0 {
            return Err(format!("Akun masih dipakai oleh {dipakai} transaksi."));
        }
        self.conn
            .execute("DELETE FROM akun WHERE id = ?1", [id])
            .map(|_| ())
            .map_err(err)
    }

    // ---------- Kategori ----------

    pub fn daftar_kategori(&self) -> Hasil<Vec<Kategori>> {
        let mut st = self
            .conn
            .prepare("SELECT id, kode, nama, arus FROM kategori ORDER BY kode, nama")
            .map_err(err)?;
        st.query_map([], |r| {
            Ok(Kategori {
                id: r.get(0)?,
                kode: r.get(1)?,
                nama: r.get(2)?,
                arus: Arus::dari_kode(&r.get::<_, String>(3)?),
            })
        })
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err)
    }

    pub fn simpan_kategori(&self, k: &Kategori) -> Hasil<()> {
        if k.id == 0 {
            self.conn.execute(
                "INSERT INTO kategori (kode, nama, arus) VALUES (?1, ?2, ?3)",
                params![k.kode, k.nama, k.arus.kode()],
            )
        } else {
            self.conn.execute(
                "UPDATE kategori SET kode = ?1, nama = ?2, arus = ?3 WHERE id = ?4",
                params![k.kode, k.nama, k.arus.kode(), k.id],
            )
        }
        .map(|_| ())
        .map_err(err)
    }

    pub fn hapus_kategori(&self, id: i64) -> Hasil<()> {
        let dipakai: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM transaksi WHERE kategori_id = ?1",
                [id],
                |r| r.get(0),
            )
            .map_err(err)?;
        if dipakai > 0 {
            return Err(format!("Kategori masih dipakai oleh {dipakai} transaksi."));
        }
        self.conn
            .execute("DELETE FROM anggaran WHERE kategori_id = ?1", [id])
            .map_err(err)?;
        self.conn
            .execute("DELETE FROM kategori WHERE id = ?1", [id])
            .map(|_| ())
            .map_err(err)
    }

    // ---------- Transaksi ----------

    const KOLOM_TRANSAKSI: &'static str = "id, tanggal, no_bukti, jenis, akun_id, akun_tujuan_id, \
        kategori_id, uraian, jumlah, pihak, sumber_dana, ppn, pph21, pph22, pph23, setor_pajak";

    fn baris_transaksi(r: &Row) -> rusqlite::Result<Transaksi> {
        let tanggal: String = r.get(1)?;
        Ok(Transaksi {
            id: r.get(0)?,
            tanggal: tanggal
                .parse::<Date>()
                .unwrap_or(Date::constant(2000, 1, 1)),
            no_bukti: r.get(2)?,
            jenis: JenisTransaksi::dari_kode(&r.get::<_, String>(3)?),
            akun_id: r.get(4)?,
            akun_tujuan_id: r.get(5)?,
            kategori_id: r.get(6)?,
            uraian: r.get(7)?,
            jumlah: r.get(8)?,
            pihak: r.get(9)?,
            sumber_dana: r.get(10)?,
            pungutan: Pungutan {
                ppn: r.get(11)?,
                pph21: r.get(12)?,
                pph22: r.get(13)?,
                pph23: r.get(14)?,
            },
            setor_pajak: r
                .get::<_, Option<String>>(15)?
                .and_then(|s| JenisPajak::dari_kode(&s)),
        })
    }

    /// Semua transaksi, urut tanggal lalu urutan input.
    pub fn daftar_transaksi(&self) -> Hasil<Vec<Transaksi>> {
        let sql = format!(
            "SELECT {} FROM transaksi ORDER BY tanggal, id",
            Self::KOLOM_TRANSAKSI
        );
        let mut st = self.conn.prepare(&sql).map_err(err)?;
        st.query_map([], Self::baris_transaksi)
            .map_err(err)?
            .collect::<Result<_, _>>()
            .map_err(err)
    }

    pub fn simpan_transaksi(&self, t: &Transaksi) -> Hasil<i64> {
        let p = &t.pungutan;
        let setor = t.setor_pajak.map(|s| s.kode());
        let tanggal = t.tanggal.to_string();
        if t.id == 0 {
            self.conn
                .execute(
                    "INSERT INTO transaksi (tanggal, no_bukti, jenis, akun_id, akun_tujuan_id, kategori_id,
                        uraian, jumlah, pihak, sumber_dana, ppn, pph21, pph22, pph23, setor_pajak)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                    params![
                        tanggal, t.no_bukti, t.jenis.kode(), t.akun_id, t.akun_tujuan_id, t.kategori_id,
                        t.uraian, t.jumlah, t.pihak, t.sumber_dana, p.ppn, p.pph21, p.pph22, p.pph23, setor
                    ],
                )
                .map_err(err)?;
            Ok(self.conn.last_insert_rowid())
        } else {
            self.conn
                .execute(
                    "UPDATE transaksi SET tanggal = ?1, no_bukti = ?2, jenis = ?3, akun_id = ?4,
                        akun_tujuan_id = ?5, kategori_id = ?6, uraian = ?7, jumlah = ?8, pihak = ?9,
                        sumber_dana = ?10, ppn = ?11, pph21 = ?12, pph22 = ?13, pph23 = ?14,
                        setor_pajak = ?15
                     WHERE id = ?16",
                    params![
                        tanggal,
                        t.no_bukti,
                        t.jenis.kode(),
                        t.akun_id,
                        t.akun_tujuan_id,
                        t.kategori_id,
                        t.uraian,
                        t.jumlah,
                        t.pihak,
                        t.sumber_dana,
                        p.ppn,
                        p.pph21,
                        p.pph22,
                        p.pph23,
                        setor,
                        t.id
                    ],
                )
                .map_err(err)?;
            Ok(t.id)
        }
    }

    pub fn hapus_transaksi(&self, id: i64) -> Hasil<()> {
        self.conn
            .execute("DELETE FROM transaksi WHERE id = ?1", [id])
            .map(|_| ())
            .map_err(err)
    }

    /// Nomor bukti berikutnya, mis. `BKK-2026-0007`.
    pub fn nomor_bukti_berikutnya(&self, jenis: JenisTransaksi, tahun: i16) -> Hasil<String> {
        let awalan = format!("{}-{}-", jenis.awalan_bukti(), tahun);
        let terakhir: Option<String> = self
            .conn
            .query_row(
                "SELECT no_bukti FROM transaksi WHERE no_bukti LIKE ?1 || '%'
                 ORDER BY length(no_bukti) DESC, no_bukti DESC LIMIT 1",
                [&awalan],
                |r| r.get(0),
            )
            .optional()
            .map_err(err)?;
        let n = terakhir
            .and_then(|s| s[awalan.len()..].parse::<u32>().ok())
            .unwrap_or(0);
        Ok(format!("{awalan}{:04}", n + 1))
    }

    // ---------- Anggaran ----------

    pub fn daftar_anggaran(&self, tahun: i16) -> Hasil<Vec<Anggaran>> {
        let mut st = self
            .conn
            .prepare("SELECT kategori_id, jumlah FROM anggaran WHERE tahun = ?1")
            .map_err(err)?;
        st.query_map([tahun], |r| {
            Ok(Anggaran {
                kategori_id: r.get(0)?,
                jumlah: r.get(1)?,
            })
        })
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err)
    }

    pub fn atur_anggaran(&self, tahun: i16, kategori_id: i64, jumlah: i64) -> Hasil<()> {
        if jumlah == 0 {
            self.conn.execute(
                "DELETE FROM anggaran WHERE tahun = ?1 AND kategori_id = ?2",
                params![tahun, kategori_id],
            )
        } else {
            self.conn.execute(
                "INSERT INTO anggaran (tahun, kategori_id, jumlah) VALUES (?1, ?2, ?3)
                 ON CONFLICT (tahun, kategori_id) DO UPDATE SET jumlah = excluded.jumlah",
                params![tahun, kategori_id, jumlah],
            )
        }
        .map(|_| ())
        .map_err(err)
    }

    // ---------- Pengaturan & profil ----------

    pub fn ambil(&self, kunci: &str) -> Option<String> {
        self.conn
            .query_row(
                "SELECT nilai FROM pengaturan WHERE kunci = ?1",
                [kunci],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten()
    }

    pub fn atur(&self, kunci: &str, nilai: &str) -> Hasil<()> {
        self.conn
            .execute(
                "INSERT INTO pengaturan (kunci, nilai) VALUES (?1, ?2)
                 ON CONFLICT (kunci) DO UPDATE SET nilai = excluded.nilai",
                params![kunci, nilai],
            )
            .map(|_| ())
            .map_err(err)
    }

    pub fn profil(&self) -> ProfilLembaga {
        let g = |k: &str| self.ambil(&format!("profil.{k}")).unwrap_or_default();
        ProfilLembaga {
            nama: g("nama"),
            alamat: g("alamat"),
            kota: g("kota"),
            telepon: g("telepon"),
            email: g("email"),
            jabatan_pimpinan: self
                .ambil("profil.jabatan_pimpinan")
                .unwrap_or_else(|| "Kepala Sekolah".into()),
            nama_pimpinan: g("nama_pimpinan"),
            nip_pimpinan: g("nip_pimpinan"),
            nama_bendahara: g("nama_bendahara"),
            nip_bendahara: g("nip_bendahara"),
            logo: self.ambil("profil.logo").filter(|s| !s.is_empty()),
        }
    }

    pub fn simpan_profil(&self, p: &ProfilLembaga) -> Hasil<()> {
        let pasangan = [
            ("nama", &p.nama),
            ("alamat", &p.alamat),
            ("kota", &p.kota),
            ("telepon", &p.telepon),
            ("email", &p.email),
            ("jabatan_pimpinan", &p.jabatan_pimpinan),
            ("nama_pimpinan", &p.nama_pimpinan),
            ("nip_pimpinan", &p.nip_pimpinan),
            ("nama_bendahara", &p.nama_bendahara),
            ("nip_bendahara", &p.nip_bendahara),
        ];
        for (k, v) in pasangan {
            self.atur(&format!("profil.{k}"), v)?;
        }
        self.atur("profil.logo", p.logo.as_deref().unwrap_or(""))
    }

    // ---------- Cadangan ----------

    /// Salin seluruh database ke `tujuan` (aman walau aplikasi sedang berjalan).
    pub fn cadangkan(&self, tujuan: &Path) -> Hasil<()> {
        if tujuan.exists() {
            std::fs::remove_file(tujuan).map_err(err)?;
        }
        self.conn
            .execute("VACUUM INTO ?1", [tujuan.to_string_lossy()])
            .map(|_| ())
            .map_err(err)
    }

    /// Ganti database aktif dengan berkas cadangan `sumber`.
    pub fn pulihkan(self, sumber: &Path) -> Hasil<Self> {
        // Pastikan berkas cadangan memang database My Digit.
        {
            let uji = Connection::open(sumber).map_err(err)?;
            uji.query_row("SELECT COUNT(*) FROM transaksi", [], |r| r.get::<_, i64>(0))
                .map_err(|_| "Berkas ini bukan cadangan My Digit.".to_string())?;
        }
        let path = self.path.clone();
        drop(self);
        std::fs::copy(sumber, &path).map_err(err)?;
        Self::buka(&path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contoh(db: &Db, jenis: JenisTransaksi, jumlah: i64) -> Transaksi {
        Transaksi {
            id: 0,
            tanggal: Date::constant(2026, 10, 1),
            no_bukti: db.nomor_bukti_berikutnya(jenis, 2026).unwrap(),
            jenis,
            akun_id: 1,
            akun_tujuan_id: None,
            kategori_id: None,
            uraian: "Uji".into(),
            jumlah,
            pihak: String::new(),
            sumber_dana: String::new(),
            pungutan: Pungutan::default(),
            setor_pajak: None,
        }
    }

    #[test]
    fn data_awal_dan_nomor_bukti() {
        let db = Db::memori();
        assert_eq!(db.daftar_akun().unwrap().len(), 2);
        assert!(!db.daftar_kategori().unwrap().is_empty());

        db.simpan_transaksi(&contoh(&db, JenisTransaksi::Masuk, 1_000))
            .unwrap();
        db.simpan_transaksi(&contoh(&db, JenisTransaksi::Masuk, 2_000))
            .unwrap();
        assert_eq!(
            db.nomor_bukti_berikutnya(JenisTransaksi::Masuk, 2026)
                .unwrap(),
            "BKM-2026-0003"
        );
        assert_eq!(
            db.nomor_bukti_berikutnya(JenisTransaksi::Keluar, 2026)
                .unwrap(),
            "BKK-2026-0001"
        );
        assert!(
            db.hapus_akun(1).is_err(),
            "akun yang dipakai tidak boleh dihapus"
        );
    }

    #[test]
    fn profil_tersimpan() {
        let db = Db::memori();
        let mut p = db.profil();
        assert_eq!(p.jabatan_pimpinan, "Kepala Sekolah");
        p.nama = "SMP Negeri 23 Dumai".into();
        db.simpan_profil(&p).unwrap();
        assert_eq!(db.profil(), p);
    }

    #[test]
    fn anggaran_upsert() {
        let db = Db::memori();
        db.atur_anggaran(2026, 6, 5_000_000).unwrap();
        db.atur_anggaran(2026, 6, 7_000_000).unwrap();
        let a = db.daftar_anggaran(2026).unwrap();
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].jumlah, 7_000_000);
        db.atur_anggaran(2026, 6, 0).unwrap();
        assert!(db.daftar_anggaran(2026).unwrap().is_empty());
    }
}

/// Buat database contoh untuk demo dan tangkapan layar:
/// `MY_DIGIT_DATA_DIR=target/demo cargo test buat_data_demo -- --ignored`
#[cfg(test)]
#[test]
#[ignore]
#[allow(clippy::type_complexity)]
fn buat_data_demo() {
    let dir = folder_data();
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(NAMA_DB);
    let _ = std::fs::remove_file(&path);
    let db = Db::buka(&path).unwrap();
    let mut p = db.profil();
    p.nama = "SMP Negeri 23 Dumai".into();
    p.alamat = "Jl. Contoh No. 23".into();
    p.kota = "Dumai".into();
    p.nama_pimpinan = "Nama Kepala Sekolah".into();
    p.nip_pimpinan = "197001012000031001".into();
    p.nama_bendahara = "Nama Bendahara".into();
    p.nip_bendahara = "198502022010012002".into();
    db.simpan_profil(&p).unwrap();
    db.simpan_akun(&Akun {
        id: 1,
        nama: "Kas Tunai".into(),
        jenis: JenisAkun::Tunai,
        no_rekening: String::new(),
        saldo_awal: 1_250_000,
    })
    .unwrap();
    db.simpan_akun(&Akun {
        id: 2,
        nama: "Bank Riau Kepri — BOS".into(),
        jenis: JenisAkun::Bank,
        no_rekening: "123-45-67890".into(),
        saldo_awal: 8_500_000,
    })
    .unwrap();

    let kat = |kode: &str| {
        db.daftar_kategori()
            .unwrap()
            .into_iter()
            .find(|k| k.kode == kode)
            .map(|k| k.id)
    };
    let hari = crate::format::hari_ini();
    let bulan = |mundur: i32, tgl: i8| {
        let total = hari.year() as i32 * 12 + hari.month() as i32 - 1 - mundur;
        Date::new((total / 12) as i16, (total % 12 + 1) as i8, tgl).unwrap()
    };
    let data: Vec<(
        Date,
        JenisTransaksi,
        i64,
        Option<i64>,
        &str,
        i64,
        &str,
        Pungutan,
        Option<JenisPajak>,
    )> = vec![
        (
            bulan(5, 6),
            JenisTransaksi::Masuk,
            2,
            kat("4.1"),
            "Penerimaan Dana BOS Tahap I",
            42_000_000,
            "Kemendikdasmen",
            Pungutan::default(),
            None,
        ),
        (
            bulan(5, 12),
            JenisTransaksi::Keluar,
            2,
            kat("5.3"),
            "Kegiatan MPLS siswa baru",
            4_800_000,
            "Panitia MPLS",
            Pungutan::default(),
            None,
        ),
        (
            bulan(4, 3),
            JenisTransaksi::Keluar,
            2,
            kat("5.1"),
            "Honor guru tidak tetap Juli",
            9_000_000,
            "GTT",
            Pungutan {
                pph21: 450_000,
                ..Default::default()
            },
            None,
        ),
        (
            bulan(4, 20),
            JenisTransaksi::Masuk,
            1,
            kat("4.2"),
            "Iuran komite kelas VII",
            3_600_000,
            "Wali murid",
            Pungutan::default(),
            None,
        ),
        (
            bulan(3, 5),
            JenisTransaksi::Keluar,
            2,
            kat("5.6"),
            "Pembelian 2 unit printer",
            6_660_000,
            "CV Sinar Komputer",
            Pungutan {
                ppn: 660_000,
                pph22: 90_000,
                ..Default::default()
            },
            None,
        ),
        (
            bulan(3, 10),
            JenisTransaksi::Keluar,
            2,
            kat("5.8"),
            "Setor PPh 21 honor Juli",
            450_000,
            "Kas Negara",
            Pungutan::default(),
            Some(JenisPajak::Pph21),
        ),
        (
            bulan(2, 2),
            JenisTransaksi::Transfer,
            2,
            None,
            "Penarikan tunai untuk belanja kegiatan",
            5_000_000,
            "",
            Pungutan::default(),
            None,
        ),
        (
            bulan(2, 14),
            JenisTransaksi::Keluar,
            1,
            kat("5.2"),
            "Kertas HVS dan tinta ujian tengah semester",
            1_870_000,
            "Toko Buku Pelita",
            Pungutan::default(),
            None,
        ),
        (
            bulan(1, 8),
            JenisTransaksi::Keluar,
            1,
            kat("5.4"),
            "Perbaikan atap ruang kelas VIII-B",
            2_350_000,
            "Pak Rahmat",
            Pungutan::default(),
            None,
        ),
        (
            bulan(1, 19),
            JenisTransaksi::Masuk,
            1,
            kat("4.3"),
            "Sumbangan alumni angkatan 2010",
            2_500_000,
            "Ikatan Alumni",
            Pungutan::default(),
            None,
        ),
        (
            bulan(0, 2),
            JenisTransaksi::Keluar,
            2,
            kat("5.5"),
            "Tagihan listrik dan internet",
            1_480_000,
            "PLN & ISP",
            Pungutan::default(),
            None,
        ),
        (
            bulan(0, 5),
            JenisTransaksi::Masuk,
            2,
            kat("4.4"),
            "Jasa giro rekening BOS",
            38_500,
            "Bank",
            Pungutan::default(),
            None,
        ),
        (
            bulan(0, 9),
            JenisTransaksi::Keluar,
            1,
            kat("5.3"),
            "Konsumsi rapat pleno komite",
            1_150_000,
            "Katering Ibu Sari",
            Pungutan::default(),
            None,
        ),
        (
            bulan(0, 12),
            JenisTransaksi::Keluar,
            2,
            kat("5.7"),
            "Biaya administrasi bank",
            12_500,
            "Bank",
            Pungutan::default(),
            None,
        ),
    ];
    for (tanggal, jenis, akun_id, kategori_id, uraian, jumlah, pihak, pungutan, setor_pajak) in data
    {
        let no_bukti = db.nomor_bukti_berikutnya(jenis, tanggal.year()).unwrap();
        db.simpan_transaksi(&Transaksi {
            id: 0,
            tanggal,
            no_bukti,
            jenis,
            akun_id,
            akun_tujuan_id: (jenis == JenisTransaksi::Transfer).then_some(1),
            kategori_id,
            uraian: uraian.into(),
            jumlah,
            pihak: pihak.into(),
            sumber_dana: if akun_id == 2 {
                "BOS Reguler".into()
            } else {
                "Komite".into()
            },
            pungutan,
            setor_pajak,
        })
        .unwrap();
    }
    for (kode, n) in [
        ("5.1", 36_000_000),
        ("5.2", 6_000_000),
        ("5.3", 12_000_000),
        ("5.4", 8_000_000),
        ("5.5", 6_000_000),
        ("5.6", 10_000_000),
        ("4.1", 84_000_000),
    ] {
        db.atur_anggaran(hari.year(), kat(kode).unwrap(), n)
            .unwrap();
    }
}

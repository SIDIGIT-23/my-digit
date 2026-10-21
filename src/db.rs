//! Penyimpanan data di SQLite.
//!
//! Database berada di folder data aplikasi sesuai sistem operasi:
//! - macOS: `~/Library/Application Support/id.SIDIGIT-23.My-Digit/`
//! - Windows: `%APPDATA%\SIDIGIT-23\My Digit\data\`
//!
//! Variabel lingkungan `MY_DIGIT_DATA_DIR` dapat menimpa lokasi ini
//! (berguna untuk pengujian).
//!
//! Versi format data disimpan di `PRAGMA user_version` ([`VERSI_SKEMA`]).
//! Sebelum data lama diperbarui, salinan pengaman dibuat lebih dulu di
//! folder `cadangan-otomatis/`; bila salinan gagal, pembaruan dibatalkan.

use std::path::{Path, PathBuf};
use std::time::Duration;

use jiff::civil::Date;
use rusqlite::{Connection, OpenFlags, OptionalExtension, Row, params};

use crate::instansi::{KATEGORI_SEKOLAH, awalan_valid, kunci_kode};
use crate::model::*;

pub type Hasil<T> = Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

pub const NAMA_DB: &str = "my-digit.sqlite";
/// Versi format data. v0 = My Digit ≤ 0.2, v1 = jenis instansi.
pub const VERSI_SKEMA: i32 = 1;
pub const FOLDER_CADANGAN_OTOMATIS: &str = "cadangan-otomatis";

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
    /// Salinan pengaman yang dibuat saat data lama diperbarui (sekali).
    pub salinan_migrasi: Option<PathBuf>,
}

/// Ringkasan isi sebuah database (aktif atau berkas cadangan).
#[derive(Debug, Clone)]
pub struct InfoCadangan {
    pub nama: String,
    pub id: Option<String>,
    pub jenis: Option<JenisInstansi>,
    pub transaksi: i64,
    pub terakhir: Option<Date>,
    pub versi: i32,
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

/// Cap waktu untuk nama berkas, mis. `20261021-141530`.
fn cap_waktu() -> String {
    jiff::Zoned::now().strftime("%Y%m%d-%H%M%S").to_string()
}

fn id_baru() -> String {
    format!("i{:x}", jiff::Timestamp::now().as_nanosecond())
}

/// `path` + akhiran, mis. `my-digit.sqlite` → `my-digit.sqlite.baru`.
fn dengan_akhiran(path: &Path, akhiran: &str) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(akhiran);
    PathBuf::from(s)
}

/// Periksa integritas berkas SQLite tanpa mengubahnya.
fn periksa_utuh(path: &Path) -> Hasil<Connection> {
    let c = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(err)?;
    let hasil: String = c
        .query_row("PRAGMA quick_check", [], |r| r.get(0))
        .map_err(err)?;
    if hasil != "ok" {
        return Err(format!("Berkas rusak ({hasil})."));
    }
    Ok(c)
}

fn hitung_transaksi(c: &Connection) -> Hasil<i64> {
    c.query_row("SELECT COUNT(*) FROM transaksi", [], |r| r.get(0))
        .map_err(err)
}

/// Baca ringkasan isi database.
fn baca_info(c: &Connection) -> Hasil<InfoCadangan> {
    let ada_tabel = |nama: &str| -> Hasil<bool> {
        c.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
            [nama],
            |r| r.get(0),
        )
        .map_err(err)
    };
    if !ada_tabel("transaksi")? {
        return Err("Berkas ini bukan data My Digit.".into());
    }
    let versi: i32 = c
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(err)?;
    let ambil = |kunci: &str| -> Option<String> {
        c.query_row(
            "SELECT nilai FROM pengaturan WHERE kunci = ?1",
            [kunci],
            |r| r.get(0),
        )
        .optional()
        .ok()
        .flatten()
    };
    let (nama, id, jenis) = if ada_tabel("pengaturan")? {
        (
            ambil("profil.nama").unwrap_or_default(),
            ambil("instansi.id"),
            ambil("instansi.jenis").map(|s| JenisInstansi::dari_kode(&s)),
        )
    } else {
        (String::new(), None, None)
    };
    let terakhir: Option<String> = c
        .query_row("SELECT MAX(tanggal) FROM transaksi", [], |r| r.get(0))
        .map_err(err)?;
    Ok(InfoCadangan {
        nama,
        id,
        jenis,
        transaksi: hitung_transaksi(c)?,
        terakhir: terakhir.and_then(|s| s.parse().ok()),
        versi,
    })
}

impl Db {
    /// Buka database di folder data aplikasi, dibuat bila belum ada.
    pub fn buka_bawaan() -> Hasil<Self> {
        let dir = folder_data();
        std::fs::create_dir_all(&dir).map_err(err)?;
        Self::buka(&dir.join(NAMA_DB))
    }

    pub fn buka(path: &Path) -> Hasil<Self> {
        let conn = Connection::open(path).map_err(err)?;
        let mut db = Self {
            conn,
            path: path.to_path_buf(),
            salinan_migrasi: None,
        };
        db.siapkan()?;
        Ok(db)
    }

    #[cfg(test)]
    pub fn memori() -> Self {
        let mut db = Self {
            conn: Connection::open_in_memory().unwrap(),
            path: PathBuf::new(),
            salinan_migrasi: None,
        };
        db.siapkan().unwrap();
        db
    }

    /// Database di memori yang sudah melewati penyiapan untuk `jenis`.
    #[cfg(test)]
    pub fn memori_siap(jenis: JenisInstansi) -> Self {
        let db = Self::memori();
        let mut p = ProfilLembaga {
            nama: "Lembaga Uji".into(),
            ..Default::default()
        };
        let mut i = PengaturanInstansi {
            jenis,
            ..Default::default()
        };
        jenis.templat().isi(&mut p, &mut i);
        db.selesaikan_penyiapan(jenis, &p, &i).unwrap();
        db
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn folder_cadangan_otomatis(&self) -> PathBuf {
        self.path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default()
            .join(FOLDER_CADANGAN_OTOMATIS)
    }

    /// Jalankan `f` dalam satu transaksi (bila belum berada di dalam transaksi).
    ///
    /// Bila `f` atau COMMIT gagal, transaksi selalu di-ROLLBACK: COMMIT yang
    /// gagal karena SQLITE_BUSY membiarkan transaksi tetap terbuka, dan
    /// penulisan berikutnya akan ikut masuk lalu hilang saat koneksi ditutup.
    fn dalam_transaksi<T>(&self, segera: bool, f: impl FnOnce() -> Hasil<T>) -> Hasil<T> {
        if !self.conn.is_autocommit() {
            return f();
        }
        self.conn
            .execute_batch(if segera { "BEGIN IMMEDIATE" } else { "BEGIN" })
            .map_err(err)?;
        let hasil = f().and_then(|v| self.conn.execute_batch("COMMIT").map(|_| v).map_err(err));
        if hasil.is_err() && !self.conn.is_autocommit() {
            let _ = self.conn.execute_batch("ROLLBACK");
        }
        hasil
    }

    fn versi(&self) -> Hasil<i32> {
        self.conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(err)
    }

    /// Siapkan skema dan perbarui data lama ke [`VERSI_SKEMA`].
    fn siapkan(&mut self) -> Hasil<()> {
        self.conn
            .busy_timeout(Duration::from_secs(5))
            .map_err(err)?;
        self.conn
            .execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(err)?;
        let versi = self.versi()?;
        if versi > VERSI_SKEMA {
            return Err("Data ini dibuat oleh My Digit versi yang lebih baru. \
                        Perbarui My Digit untuk membukanya. Data tidak diubah."
                .into());
        }
        let lama: bool = self
            .conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'akun')",
                [],
                |r| r.get(0),
            )
            .map_err(err)?;
        if versi == 0 && lama && !self.path.as_os_str().is_empty() {
            let salinan = self.cadangkan_otomatis("sebelum-v1").map_err(|e| {
                format!(
                    "Pembaruan data dibatalkan karena salinan pengaman gagal dibuat. \
                     Data Anda tidak diubah. ({e})"
                )
            })?;
            self.salinan_migrasi = Some(salinan);
        }
        self.conn.execute_batch(SKEMA).map_err(err)?;
        if versi == 0 {
            self.dalam_transaksi(true, || {
                // Dibaca ulang di dalam transaksi: jendela lain mungkin sudah memperbarui.
                if self.versi()? != 0 {
                    return Ok(());
                }
                if !lama {
                    // Berkas baru: kategori diisi saat penyiapan "Masuk sebagai".
                    self.conn
                        .execute_batch(
                            "INSERT INTO akun (nama, jenis) VALUES ('Kas Tunai', 'tunai');
                             INSERT INTO akun (nama, jenis) VALUES ('Rekening Bank', 'bank');",
                        )
                        .map_err(err)?;
                } else if !self.masih_bawaan_v02()? {
                    // Data v0.2 yang sudah dipakai: langsung siap, jenis ditebak.
                    self.isi_bila_belum("instansi.jenis", self.tebak_jenis().kode())?;
                    self.isi_bila_belum("instansi.siap", "1")?;
                }
                self.isi_bila_belum("instansi.id", &id_baru())?;
                self.conn
                    .execute_batch(&format!("PRAGMA user_version = {VERSI_SKEMA}"))
                    .map_err(err)
            })?;
        }
        Ok(())
    }

    fn isi_bila_belum(&self, kunci: &str, nilai: &str) -> Hasil<()> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO pengaturan (kunci, nilai) VALUES (?1, ?2)",
                [kunci, nilai],
            )
            .map(|_| ())
            .map_err(err)
    }

    fn tebak_jenis(&self) -> JenisInstansi {
        match self.ambil("profil.jabatan_pimpinan").as_deref() {
            Some("Ketua Yayasan") => JenisInstansi::Lembaga,
            Some("Ketua") => JenisInstansi::Organisasi,
            _ => JenisInstansi::Sekolah,
        }
    }

    /// Data masih persis seperti baru dipasang di v0.2: tanpa transaksi,
    /// anggaran, dan nama, dengan 14 kategori bawaan yang belum diubah.
    pub fn masih_bawaan_v02(&self) -> Hasil<bool> {
        let hitung =
            |sql: &str| -> Hasil<i64> { self.conn.query_row(sql, [], |r| r.get(0)).map_err(err) };
        if hitung("SELECT COUNT(*) FROM transaksi")? > 0
            || hitung("SELECT COUNT(*) FROM anggaran")? > 0
        {
            return Ok(false);
        }
        if self
            .ambil("profil.nama")
            .is_some_and(|n| !n.trim().is_empty())
        {
            return Ok(false);
        }
        let mut ada: Vec<(String, String, String)> = self
            .daftar_kategori()?
            .into_iter()
            .map(|k| (k.kode, k.nama, k.arus.kode().to_string()))
            .collect();
        let mut bawaan: Vec<(String, String, String)> = KATEGORI_SEKOLAH
            .iter()
            .map(|(k, n, a)| (k.to_string(), n.to_string(), a.kode().to_string()))
            .collect();
        ada.sort();
        bawaan.sort();
        Ok(ada == bawaan)
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

    /// Semua kategori, urut kode secara angka (4.2 sebelum 4.10), lalu nama.
    pub fn daftar_kategori(&self) -> Hasil<Vec<Kategori>> {
        let mut st = self
            .conn
            .prepare("SELECT id, kode, nama, arus FROM kategori")
            .map_err(err)?;
        let mut daftar: Vec<Kategori> = st
            .query_map([], |r| {
                Ok(Kategori {
                    id: r.get(0)?,
                    kode: r.get(1)?,
                    nama: r.get(2)?,
                    arus: Arus::dari_kode(&r.get::<_, String>(3)?),
                })
            })
            .map_err(err)?
            .collect::<Result<_, _>>()
            .map_err(err)?;
        daftar.sort_by(|a, b| {
            kunci_kode(&a.kode)
                .cmp(&kunci_kode(&b.kode))
                .then_with(|| a.nama.cmp(&b.nama))
        });
        Ok(daftar)
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

    /// Nomor bukti berikutnya, mis. `BKK-2026-0007`. `awalan` harus sudah
    /// valid ([`awalan_valid`]) sehingga tidak memuat wildcard LIKE.
    pub fn nomor_bukti_berikutnya(&self, awalan: &str, tahun: i16) -> Hasil<String> {
        let awalan = format!("{awalan}-{tahun}-");
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
            .and_then(|s| s.get(awalan.len()..).and_then(|x| x.parse::<u32>().ok()))
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

    // ---------- Pengaturan, profil & instansi ----------

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

    /// Profil lembaga. Kunci yang belum ada memakai bawaan v0.2, sehingga
    /// keluaran data lama tidak berubah; kunci yang ada tapi kosong tetap kosong.
    pub fn profil(&self) -> ProfilLembaga {
        let g = |k: &str| self.ambil(&format!("profil.{k}")).unwrap_or_default();
        let atau = |k: &str, bawaan: &str| {
            self.ambil(&format!("profil.{k}"))
                .unwrap_or_else(|| bawaan.into())
        };
        ProfilLembaga {
            nama: g("nama"),
            alamat: g("alamat"),
            kota: g("kota"),
            telepon: g("telepon"),
            email: g("email"),
            jabatan_pimpinan: atau("jabatan_pimpinan", "Kepala Sekolah"),
            nama_pimpinan: g("nama_pimpinan"),
            nip_pimpinan: g("nip_pimpinan"),
            nama_bendahara: g("nama_bendahara"),
            nip_bendahara: g("nip_bendahara"),
            logo: self.ambil("profil.logo").filter(|s| !s.is_empty()),
            induk: g("induk"),
            jabatan_bendahara: atau("jabatan_bendahara", "Bendahara"),
            label_nomor_pimpinan: atau("label_nomor_pimpinan", "NIP"),
            label_nomor_bendahara: atau("label_nomor_bendahara", "NIP"),
            id1_label: g("id1_label"),
            id1_nomor: g("id1_nomor"),
            id2_label: g("id2_label"),
            id2_nomor: g("id2_nomor"),
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
            ("induk", &p.induk),
            ("jabatan_bendahara", &p.jabatan_bendahara),
            ("label_nomor_pimpinan", &p.label_nomor_pimpinan),
            ("label_nomor_bendahara", &p.label_nomor_bendahara),
            ("id1_label", &p.id1_label),
            ("id1_nomor", &p.id1_nomor),
            ("id2_label", &p.id2_label),
            ("id2_nomor", &p.id2_nomor),
        ];
        self.dalam_transaksi(false, || {
            for (k, v) in pasangan {
                self.atur(&format!("profil.{k}"), v)?;
            }
            self.atur("profil.logo", p.logo.as_deref().unwrap_or(""))
        })
    }

    /// Pengaturan instansi; awalan tersimpan yang tidak valid jatuh ke bawaan.
    pub fn instansi(&self) -> PengaturanInstansi {
        let bawaan = PengaturanInstansi::default();
        let awalan = |k: &str, b: &str| {
            self.ambil(&format!("nomor.awalan.{k}"))
                .filter(|s| awalan_valid(s))
                .unwrap_or_else(|| b.to_string())
        };
        PengaturanInstansi {
            jenis: self
                .ambil("instansi.jenis")
                .map(|s| JenisInstansi::dari_kode(&s))
                .unwrap_or(bawaan.jenis),
            subjenis: self.ambil("instansi.subjenis").unwrap_or_default(),
            pajak: self
                .ambil("instansi.pajak")
                .map(|s| s == "1")
                .unwrap_or(bawaan.pajak),
            istilah_anggaran: self
                .ambil("instansi.istilah_anggaran")
                .unwrap_or(bawaan.istilah_anggaran),
            sumber_dana: self
                .ambil("instansi.sumber_dana")
                .map(|s| {
                    s.split('|')
                        .map(str::trim)
                        .filter(|x| !x.is_empty())
                        .map(String::from)
                        .collect()
                })
                .unwrap_or(bawaan.sumber_dana),
            awalan_masuk: awalan("masuk", &bawaan.awalan_masuk),
            awalan_keluar: awalan("keluar", &bawaan.awalan_keluar),
            awalan_transfer: awalan("transfer", &bawaan.awalan_transfer),
        }
    }

    pub fn simpan_instansi(&self, i: &PengaturanInstansi) -> Hasil<()> {
        for (j, a) in [
            ("masuk", &i.awalan_masuk),
            ("keluar", &i.awalan_keluar),
            ("transfer", &i.awalan_transfer),
        ] {
            if !awalan_valid(a) {
                return Err(format!(
                    "Awalan nomor bukti {j} \"{a}\" tidak valid (1–8 huruf besar/angka)."
                ));
            }
        }
        let sumber = i
            .sumber_dana
            .iter()
            .map(|s| s.replace('|', "").trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("|");
        self.dalam_transaksi(false, || {
            self.atur("instansi.jenis", i.jenis.kode())?;
            self.atur("instansi.subjenis", i.subjenis.trim())?;
            self.atur("instansi.pajak", if i.pajak { "1" } else { "0" })?;
            self.atur("instansi.istilah_anggaran", i.istilah_anggaran.trim())?;
            self.atur("instansi.sumber_dana", &sumber)?;
            self.atur("nomor.awalan.masuk", &i.awalan_masuk)?;
            self.atur("nomor.awalan.keluar", &i.awalan_keluar)?;
            self.atur("nomor.awalan.transfer", &i.awalan_transfer)
        })
    }

    /// Layar "Masuk sebagai" perlu ditampilkan.
    pub fn perlu_penyiapan(&self) -> bool {
        self.ambil("instansi.siap").as_deref() != Some("1")
    }

    /// Simpan hasil layar penyiapan dan isi kategori templat.
    pub fn selesaikan_penyiapan(
        &self,
        jenis: JenisInstansi,
        p: &ProfilLembaga,
        i: &PengaturanInstansi,
    ) -> Hasil<()> {
        // Diperiksa sebelum profil (yang berisi nama) disimpan.
        let bawaan = self.masih_bawaan_v02()?;
        let i = PengaturanInstansi { jenis, ..i.clone() };
        self.dalam_transaksi(true, || {
            self.simpan_profil(p)?;
            self.simpan_instansi(&i)?;
            let ada: i64 = self
                .conn
                .query_row("SELECT COUNT(*) FROM kategori", [], |r| r.get(0))
                .map_err(err)?;
            let ganti_bawaan = bawaan && jenis != JenisInstansi::Sekolah;
            if ganti_bawaan {
                // 14 kategori bawaan v0.2 belum dirujuk transaksi atau anggaran.
                self.conn.execute("DELETE FROM kategori", []).map_err(err)?;
            }
            if ada == 0 || ganti_bawaan {
                for (kode, nama, arus) in jenis.templat().kategori {
                    self.conn
                        .execute(
                            "INSERT INTO kategori (kode, nama, arus) VALUES (?1, ?2, ?3)",
                            params![kode, nama, arus.kode()],
                        )
                        .map_err(err)?;
                }
            }
            self.atur("instansi.siap", "1")
        })
    }

    /// Kategori templat `jenis` yang belum ada (id = 0), untuk ditambahkan
    /// saat jenis diubah. Nama yang sudah ada dilewati; kode yang bentrok
    /// diganti kode berikutnya dalam kelompok yang sama (mis. 4.10).
    pub fn rencana_kategori(&self, jenis: JenisInstansi) -> Hasil<Vec<Kategori>> {
        let ada = self.daftar_kategori()?;
        let sama = |a: &str, b: &str| a.trim().to_lowercase() == b.trim().to_lowercase();
        let mut kode_terpakai: Vec<String> =
            ada.iter().map(|k| k.kode.trim().to_string()).collect();
        let mut rencana = Vec::new();
        for (kode, nama, arus) in jenis.templat().kategori {
            if ada.iter().any(|k| k.arus == *arus && sama(&k.nama, nama)) {
                continue;
            }
            let kode = if kode_terpakai.iter().any(|k| k == kode) {
                let kelompok = kode.split('.').next().unwrap_or("9");
                let maks = kode_terpakai
                    .iter()
                    .filter_map(|k| {
                        let (a, b) = k.split_once('.')?;
                        (a == kelompok && !b.contains('.'))
                            .then(|| b.parse::<u32>().ok())
                            .flatten()
                    })
                    .max()
                    .unwrap_or(0);
                format!("{kelompok}.{}", maks + 1)
            } else {
                kode.to_string()
            };
            kode_terpakai.push(kode.clone());
            rencana.push(Kategori {
                id: 0,
                kode,
                nama: nama.to_string(),
                arus: *arus,
            });
        }
        Ok(rencana)
    }

    /// Ubah jenis: simpan pengaturan & profil, tambahkan kategori. Tidak pernah menghapus.
    pub fn terapkan_jenis(
        &self,
        i: &PengaturanInstansi,
        p: &ProfilLembaga,
        kategori: &[Kategori],
    ) -> Hasil<usize> {
        self.dalam_transaksi(true, || {
            self.simpan_instansi(i)?;
            self.simpan_profil(p)?;
            for k in kategori {
                self.simpan_kategori(&Kategori { id: 0, ..k.clone() })?;
            }
            Ok(kategori.len())
        })
    }

    // ---------- Cadangan ----------

    pub fn info(&self) -> Hasil<InfoCadangan> {
        baca_info(&self.conn)
    }

    /// Periksa berkas cadangan tanpa mengubahnya.
    pub fn periksa_cadangan(path: &Path) -> Hasil<InfoCadangan> {
        let c = periksa_utuh(path)
            .map_err(|e| format!("Berkas tidak dapat dibaca sebagai data My Digit: {e}"))?;
        let info = baca_info(&c)?;
        if info.versi > VERSI_SKEMA {
            return Err("Cadangan ini dibuat oleh My Digit versi yang lebih baru. Perbarui My Digit lebih dulu.".into());
        }
        Ok(info)
    }

    /// Salinan pengaman buatan aplikasi di `cadangan-otomatis/`, diverifikasi.
    pub fn cadangkan_otomatis(&self, alasan: &str) -> Hasil<PathBuf> {
        let dir = self.folder_cadangan_otomatis();
        std::fs::create_dir_all(&dir).map_err(err)?;
        let dasar = format!("my-digit-{alasan}-{}-{}", cap_waktu(), std::process::id());
        let mut tujuan = dir.join(format!("{dasar}.sqlite"));
        let mut n = 1;
        while tujuan.exists() {
            n += 1;
            tujuan = dir.join(format!("{dasar}-{n}.sqlite"));
        }
        let hasil = (|| -> Hasil<()> {
            self.conn
                .execute("VACUUM INTO ?1", [tujuan.to_string_lossy()])
                .map_err(err)?;
            let salinan = periksa_utuh(&tujuan)?;
            if hitung_transaksi(&salinan)? != hitung_transaksi(&self.conn)? {
                return Err("Jumlah transaksi pada salinan tidak sama.".into());
            }
            Ok(())
        })();
        match hasil {
            Ok(()) => Ok(tujuan),
            Err(e) => {
                // Hanya berkas yang baru dibuat di atas yang dihapus.
                let _ = std::fs::remove_file(&tujuan);
                Err(e)
            }
        }
    }

    /// Salin seluruh database ke `tujuan` (aman walau aplikasi sedang berjalan).
    /// Ditulis ke berkas sementara dulu, sehingga cadangan lama di `tujuan`
    /// tetap utuh bila penyalinan gagal.
    pub fn cadangkan(&self, tujuan: &Path) -> Hasil<()> {
        if let (Some(folder), Some(induk)) = (self.path.parent(), tujuan.parent())
            && let (Ok(folder), Ok(induk)) = (folder.canonicalize(), induk.canonicalize())
            && !folder.as_os_str().is_empty()
            && induk.starts_with(&folder)
        {
            return Err(
                "Simpan cadangan di luar folder data aplikasi, misalnya Dokumen atau flashdisk."
                    .into(),
            );
        }
        let sementara = dengan_akhiran(tujuan, ".tmp");
        let _ = std::fs::remove_file(&sementara);
        let hasil = self
            .conn
            .execute("VACUUM INTO ?1", [sementara.to_string_lossy()])
            .map_err(err)
            .and_then(|_| std::fs::rename(&sementara, tujuan).map_err(err));
        if hasil.is_err() {
            let _ = std::fs::remove_file(&sementara);
        }
        hasil
    }

    /// Ganti database aktif dengan isi berkas cadangan `sumber`.
    ///
    /// Data saat ini disalin lebih dulu ke `cadangan-otomatis/`; bila gagal,
    /// pemulihan dibatalkan. Berhasil → `(db, path salinan)`. Gagal →
    /// `(db yang tetap terbuka bila bisa, pesan)`.
    // Saat gagal, Db yang masih terbuka dikembalikan agar aplikasi tidak kehilangan datanya.
    #[allow(clippy::result_large_err)]
    pub fn pulihkan(self, sumber: &Path) -> Result<(Db, PathBuf), (Option<Db>, String)> {
        if let (Ok(a), Ok(b)) = (sumber.canonicalize(), self.path.canonicalize())
            && a == b
        {
            return Err((
                Some(self),
                "Berkas itu adalah data yang sedang dipakai.".into(),
            ));
        }
        if let Err(e) = Self::periksa_cadangan(sumber) {
            return Err((Some(self), e));
        }
        let darurat = match self.cadangkan_otomatis("sebelum-pulih") {
            Ok(p) => p,
            Err(e) => {
                return Err((
                    Some(self),
                    format!("Pemulihan dibatalkan: salinan pengaman gagal dibuat. ({e})"),
                ));
            }
        };
        let path = self.path.clone();
        let baru = dengan_akhiran(&path, ".baru");
        let _ = std::fs::remove_file(&baru);
        if let Err(e) = std::fs::copy(sumber, &baru)
            .map_err(err)
            .and_then(|_| periksa_utuh(&baru).map(|_| ()))
        {
            let _ = std::fs::remove_file(&baru);
            return Err((Some(self), format!("Pemulihan dibatalkan: {e}")));
        }
        drop(self);
        let buka_lagi = |pesan: String| {
            let _ = std::fs::remove_file(&baru);
            (Db::buka(&path).ok(), pesan)
        };
        if dengan_akhiran(&path, "-journal").exists() {
            return Err(buka_lagi(
                "Pemulihan dibatalkan: data sedang dipakai proses lain.".into(),
            ));
        }
        if let Err(e) = std::fs::rename(&baru, &path) {
            return Err(buka_lagi(format!(
                "Pemulihan gagal, data tidak diubah: {e}"
            )));
        }
        match Db::buka(&path) {
            Ok(db) => Ok((db, darurat)),
            Err(e) => {
                // Kembalikan data sebelumnya dari salinan pengaman.
                let kembali = std::fs::copy(&darurat, &path).is_ok();
                let pesan = format!(
                    "Data hasil pemulihan tidak dapat dibuka: {e}. {}",
                    if kembali {
                        "Data sebelumnya sudah dikembalikan."
                    } else {
                        "Salinan data sebelumnya ada di folder Cadangan Otomatis."
                    }
                );
                Err((Db::buka(&path).ok(), pesan))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instansi::JenisInstansi as J;

    /// Folder uji di dalam `target/` proyek (bukan folder temp sistem).
    fn folder_uji(nama: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("uji")
            .join(format!("{nama}-{}", jiff::Timestamp::now().as_nanosecond()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn salinan_di(dir: &Path) -> Vec<PathBuf> {
        std::fs::read_dir(dir.join(FOLDER_CADANGAN_OTOMATIS))
            .map(|r| r.filter_map(|e| e.ok().map(|e| e.path())).collect())
            .unwrap_or_default()
    }

    /// Database seperti buatan My Digit v0.2 (user_version 0).
    fn buat_db_v02(path: &Path, dipakai: bool) {
        let c = Connection::open(path).unwrap();
        c.execute_batch(SKEMA).unwrap();
        c.execute_batch(
            "INSERT INTO akun (nama, jenis) VALUES ('Kas Tunai', 'tunai');
             INSERT INTO akun (nama, jenis) VALUES ('Rekening Bank', 'bank');",
        )
        .unwrap();
        for (kode, nama, arus) in KATEGORI_SEKOLAH {
            c.execute(
                "INSERT INTO kategori (kode, nama, arus) VALUES (?1, ?2, ?3)",
                params![kode, nama, arus.kode()],
            )
            .unwrap();
        }
        if dipakai {
            c.execute_batch(
                "INSERT INTO pengaturan VALUES ('profil.nama', 'SMP Negeri 23 Dumai');
                 INSERT INTO pengaturan VALUES ('profil.kota', 'Dumai');
                 INSERT INTO pengaturan VALUES ('profil.jabatan_pimpinan', 'Kepala Sekolah');
                 INSERT INTO pengaturan VALUES ('profil.nip_bendahara', '198502022010012002');
                 INSERT INTO pengaturan VALUES ('tema', 'gelap');
                 INSERT INTO transaksi (tanggal, no_bukti, jenis, akun_id, kategori_id, uraian, jumlah)
                     VALUES ('2026-10-01', 'BKM-2026-0001', 'masuk', 2, 1, 'BOS', 1000000);
                 INSERT INTO transaksi (tanggal, no_bukti, jenis, akun_id, kategori_id, uraian, jumlah)
                     VALUES ('2026-10-02', 'BKK-2026-0001', 'keluar', 1, 7, 'ATK', 50000);
                 INSERT INTO transaksi (tanggal, no_bukti, jenis, akun_id, akun_tujuan_id, uraian, jumlah)
                     VALUES ('2026-10-03', 'TRF-2026-0001', 'transfer', 2, 1, 'Tarik tunai', 200000);
                 INSERT INTO anggaran (tahun, kategori_id, jumlah) VALUES (2026, 7, 600000);",
            )
            .unwrap();
        }
    }

    fn hitung(db: &Db, tabel: &str) -> i64 {
        db.conn
            .query_row(&format!("SELECT COUNT(*) FROM {tabel}"), [], |r| r.get(0))
            .unwrap()
    }

    fn contoh(db: &Db, jenis: JenisTransaksi, jumlah: i64) -> Transaksi {
        Transaksi {
            id: 0,
            tanggal: Date::constant(2026, 10, 1),
            no_bukti: db
                .nomor_bukti_berikutnya(jenis.awalan_bukti(), 2026)
                .unwrap(),
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
        let db = Db::memori_siap(J::Sekolah);
        assert_eq!(db.daftar_akun().unwrap().len(), 2);
        assert_eq!(db.daftar_kategori().unwrap().len(), KATEGORI_SEKOLAH.len());

        db.simpan_transaksi(&contoh(&db, JenisTransaksi::Masuk, 1_000))
            .unwrap();
        db.simpan_transaksi(&contoh(&db, JenisTransaksi::Masuk, 2_000))
            .unwrap();
        assert_eq!(
            db.nomor_bukti_berikutnya("BKM", 2026).unwrap(),
            "BKM-2026-0003"
        );
        assert_eq!(
            db.nomor_bukti_berikutnya("BKK", 2026).unwrap(),
            "BKK-2026-0001"
        );
        assert_eq!(
            db.nomor_bukti_berikutnya("KM", 2026).unwrap(),
            "KM-2026-0001"
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
        assert_eq!(p.jabatan_bendahara, "Bendahara");
        assert_eq!(p.label_nomor_pimpinan, "NIP");
        p.nama = "SMP Negeri 23 Dumai".into();
        p.induk = "PEMERINTAH KOTA DUMAI\nDINAS PENDIDIKAN".into();
        p.label_nomor_bendahara = String::new();
        p.id1_label = "NPSN".into();
        p.id1_nomor = "10404123".into();
        db.simpan_profil(&p).unwrap();
        let baca = db.profil();
        assert_eq!(baca, p);
        assert_eq!(baca.label_nomor_bendahara, "", "kunci kosong tetap kosong");
    }

    #[test]
    fn anggaran_upsert() {
        let db = Db::memori_siap(J::Sekolah);
        let id = db.daftar_kategori().unwrap()[6].id;
        db.atur_anggaran(2026, id, 5_000_000).unwrap();
        db.atur_anggaran(2026, id, 7_000_000).unwrap();
        let a = db.daftar_anggaran(2026).unwrap();
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].jumlah, 7_000_000);
        db.atur_anggaran(2026, id, 0).unwrap();
        assert!(db.daftar_anggaran(2026).unwrap().is_empty());
    }

    #[test]
    fn migrasi_db_lama_tanpa_kehilangan() {
        let dir = folder_uji("migrasi");
        let path = dir.join(NAMA_DB);
        buat_db_v02(&path, true);
        let profil_lama = {
            let c = Connection::open(&path).unwrap();
            let n: i64 = c
                .query_row("SELECT COUNT(*) FROM pengaturan", [], |r| r.get(0))
                .unwrap();
            n
        };
        assert_eq!(profil_lama, 5);

        let db = Db::buka(&path).unwrap();
        assert_eq!(hitung(&db, "akun"), 2);
        assert_eq!(hitung(&db, "kategori"), 14);
        assert_eq!(hitung(&db, "transaksi"), 3);
        assert_eq!(hitung(&db, "anggaran"), 1);
        assert_eq!(db.versi().unwrap(), VERSI_SKEMA);
        assert!(!db.perlu_penyiapan());
        let p = db.profil();
        assert_eq!(p.nama, "SMP Negeri 23 Dumai");
        assert_eq!(p.jabatan_bendahara, "Bendahara");
        assert_eq!(p.label_nomor_bendahara, "NIP");
        assert_eq!(db.instansi(), PengaturanInstansi::default());
        assert_eq!(db.ambil("tema").as_deref(), Some("gelap"));
        assert!(db.ambil("instansi.id").is_some());

        let salinan = salinan_di(&dir);
        assert_eq!(salinan.len(), 1);
        assert_eq!(db.salinan_migrasi.as_ref(), Some(&salinan[0]));
        let c = Connection::open(&salinan[0]).unwrap();
        assert_eq!(hitung_transaksi(&c).unwrap(), 3);
        let v: i32 = c
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, 0, "salinan adalah data sebelum diperbarui");
    }

    #[test]
    fn migrasi_idempoten() {
        let dir = folder_uji("idempoten");
        let path = dir.join(NAMA_DB);
        buat_db_v02(&path, true);
        drop(Db::buka(&path).unwrap());
        let db = Db::buka(&path).unwrap();
        assert!(db.salinan_migrasi.is_none());
        assert_eq!(salinan_di(&dir).len(), 1);
        assert_eq!(hitung(&db, "kategori"), 14);
        assert_eq!(hitung(&db, "akun"), 2);
    }

    #[test]
    fn migrasi_batal_bila_salinan_gagal() {
        let dir = folder_uji("batal");
        let path = dir.join(NAMA_DB);
        buat_db_v02(&path, true);
        // Berkas (bukan folder) bernama cadangan-otomatis membuat salinan gagal.
        std::fs::write(dir.join(FOLDER_CADANGAN_OTOMATIS), b"x").unwrap();
        assert!(Db::buka(&path).is_err());
        let c = Connection::open(&path).unwrap();
        let v: i32 = c
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, 0);
        let baru: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM pengaturan WHERE kunci LIKE 'instansi.%'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(baru, 0);
    }

    #[test]
    fn tolak_versi_lebih_baru() {
        let dir = folder_uji("versi");
        let path = dir.join(NAMA_DB);
        buat_db_v02(&path, true);
        Connection::open(&path)
            .unwrap()
            .execute_batch("PRAGMA user_version = 2")
            .unwrap();
        let sebelum = std::fs::read(&path).unwrap();
        assert!(Db::buka(&path).is_err());
        assert!(salinan_di(&dir).is_empty());
        assert_eq!(std::fs::read(&path).unwrap(), sebelum);
    }

    #[test]
    fn db_baru_tanpa_kategori() {
        let dir = folder_uji("baru");
        let db = Db::buka(&dir.join(NAMA_DB)).unwrap();
        assert_eq!(hitung(&db, "akun"), 2);
        assert_eq!(hitung(&db, "kategori"), 0);
        assert!(db.perlu_penyiapan());
        assert!(db.salinan_migrasi.is_none());
        let mut p = ProfilLembaga {
            nama: "OSIS SMPN 23".into(),
            ..Default::default()
        };
        let mut i = PengaturanInstansi::default();
        J::Organisasi.templat().isi(&mut p, &mut i);
        db.selesaikan_penyiapan(J::Organisasi, &p, &i).unwrap();
        assert!(!db.perlu_penyiapan());
        assert_eq!(db.instansi().jenis, J::Organisasi);
        assert!(!db.instansi().pajak);
        assert_eq!(db.profil().jabatan_pimpinan, "Ketua");
        assert_eq!(
            hitung(&db, "kategori") as usize,
            J::Organisasi.templat().kategori.len()
        );
    }

    #[test]
    fn db_v02_bawaan_ikut_penyiapan() {
        let dir = folder_uji("bawaan");
        let path = dir.join(NAMA_DB);
        buat_db_v02(&path, false);
        let db = Db::buka(&path).unwrap();
        assert!(db.perlu_penyiapan());
        let p = ProfilLembaga {
            nama: "Masjid Al-Ikhlas".into(),
            ..Default::default()
        };
        db.selesaikan_penyiapan(J::Lembaga, &p, &PengaturanInstansi::default())
            .unwrap();
        let nama: Vec<String> = db
            .daftar_kategori()
            .unwrap()
            .into_iter()
            .map(|k| k.nama)
            .collect();
        let templat: Vec<String> = J::Lembaga
            .templat()
            .kategori
            .iter()
            .map(|k| k.1.to_string())
            .collect();
        assert_eq!(nama, templat);

        // Satu transaksi saja sudah membuatnya bukan data bawaan.
        let dir = folder_uji("bawaan-dipakai");
        let path = dir.join(NAMA_DB);
        buat_db_v02(&path, false);
        Connection::open(&path)
            .unwrap()
            .execute_batch(
                "INSERT INTO transaksi (tanggal, no_bukti, jenis, akun_id, uraian, jumlah)
                 VALUES ('2026-10-01', 'BKM-2026-0001', 'masuk', 1, 'x', 1)",
            )
            .unwrap();
        let db = Db::buka(&path).unwrap();
        assert!(!db.perlu_penyiapan());
    }

    #[test]
    fn hapus_semua_akun_tidak_mengisi_ulang() {
        let dir = folder_uji("akun");
        let path = dir.join(NAMA_DB);
        let db = Db::buka(&path).unwrap();
        db.selesaikan_penyiapan(
            J::Sekolah,
            &ProfilLembaga::default(),
            &PengaturanInstansi::default(),
        )
        .unwrap();
        for a in db.daftar_akun().unwrap() {
            db.hapus_akun(a.id).unwrap();
        }
        drop(db);
        let db = Db::buka(&path).unwrap();
        assert_eq!(hitung(&db, "akun"), 0);
        assert_eq!(hitung(&db, "kategori"), 14);
    }

    #[test]
    fn ganti_jenis_tidak_menghapus() {
        let db = Db::memori_siap(J::Sekolah);
        db.simpan_transaksi(&Transaksi {
            kategori_id: Some(db.daftar_kategori().unwrap()[0].id),
            ..contoh(&db, JenisTransaksi::Masuk, 5)
        })
        .unwrap();
        let sebelum = db.daftar_kategori().unwrap();
        let rencana = db.rencana_kategori(J::Organisasi).unwrap();
        assert!(!rencana.is_empty());
        // "Penerimaan Lain-lain" dkk. sudah ada → dilewati.
        assert!(rencana.iter().all(|k| k.nama != "Penerimaan Lain-lain"));
        let i = PengaturanInstansi {
            jenis: J::Organisasi,
            ..Default::default()
        };
        let n = db.terapkan_jenis(&i, &db.profil(), &rencana).unwrap();
        let sesudah = db.daftar_kategori().unwrap();
        assert_eq!(sesudah.len(), sebelum.len() + n);
        assert!(
            sebelum
                .iter()
                .all(|k| sesudah.iter().any(|s| s.id == k.id && s.nama == k.nama))
        );
        let mut kode: Vec<&str> = sesudah.iter().map(|k| k.kode.as_str()).collect();
        let total = kode.len();
        kode.sort();
        kode.dedup();
        assert_eq!(kode.len(), total, "tidak ada kode ganda");
        assert!(
            sesudah.iter().any(|k| k.kode == "4.10"),
            "kode bentrok dialokasikan setelah 4.9"
        );
        assert_eq!(hitung(&db, "transaksi"), 1);
        assert!(
            db.rencana_kategori(J::Organisasi).unwrap().is_empty(),
            "panggilan kedua menambah 0"
        );
    }

    #[test]
    fn commit_gagal_tidak_menggantung() {
        let dir = folder_uji("commit");
        let path = dir.join(NAMA_DB);
        let db = Db::buka(&path).unwrap();
        db.conn.busy_timeout(Duration::from_millis(20)).unwrap();
        // Jendela lain sedang membaca: kunci SHARED membuat COMMIT kita BUSY.
        let lain = Connection::open(&path).unwrap();
        lain.execute_batch("BEGIN; SELECT COUNT(*) FROM akun;")
            .unwrap();
        let p = ProfilLembaga {
            nama: "Uji".into(),
            ..Default::default()
        };
        assert!(db.simpan_profil(&p).is_err());
        assert!(
            db.conn.is_autocommit(),
            "transaksi yang gagal harus di-rollback"
        );
        lain.execute_batch("COMMIT").unwrap();
        db.simpan_profil(&p).unwrap();
        drop(db);
        assert_eq!(
            Db::buka(&path).unwrap().profil().nama,
            "Uji",
            "simpan berikutnya benar-benar tersimpan"
        );
    }

    #[test]
    fn awalan_tidak_valid_ditolak() {
        let db = Db::memori_siap(J::Sekolah);
        let i = PengaturanInstansi {
            awalan_masuk: "B%".into(),
            ..Default::default()
        };
        assert!(db.simpan_instansi(&i).is_err());
        db.atur("nomor.awalan.masuk", "b k").unwrap();
        assert_eq!(
            db.instansi().awalan_masuk,
            "BKM",
            "nilai tersimpan tidak valid → bawaan"
        );
    }

    #[test]
    fn cadangkan_dan_pulihkan_aman() {
        let dir = folder_uji("pulih");
        let path = dir.join(NAMA_DB);
        buat_db_v02(&path, true);
        let db = Db::buka(&path).unwrap();
        assert!(
            db.cadangkan(&dir.join("di-dalam.sqlite")).is_err(),
            "tujuan di folder data ditolak"
        );

        let luar = folder_uji("pulih-luar");
        let cadangan = luar.join("cadangan.sqlite");
        db.cadangkan(&cadangan).unwrap();
        db.cadangkan(&cadangan).unwrap();
        assert!(!dengan_akhiran(&cadangan, ".tmp").exists());
        let info = Db::periksa_cadangan(&cadangan).unwrap();
        assert_eq!(info.nama, "SMP Negeri 23 Dumai");
        assert_eq!(info.transaksi, 3);

        db.hapus_transaksi(1).unwrap();
        let db = match db.pulihkan(&path) {
            Err((Some(db), _)) => db,
            _ => panic!("pulih dari berkas aktif harus ditolak"),
        };
        let jumlah_salinan = salinan_di(&dir).len();
        let (db, darurat) = db.pulihkan(&cadangan).map_err(|e| e.1).unwrap();
        assert_eq!(hitung(&db, "transaksi"), 3);
        assert!(darurat.exists());
        assert_eq!(salinan_di(&dir).len(), jumlah_salinan + 1);
        let c = Connection::open(&darurat).unwrap();
        assert_eq!(
            hitung_transaksi(&c).unwrap(),
            2,
            "salinan berisi data sebelum pulih"
        );

        let bukan = luar.join("bukan.sqlite");
        std::fs::write(&bukan, b"bukan database").unwrap();
        assert!(Db::periksa_cadangan(&bukan).is_err());
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
    let mut p = ProfilLembaga::default();
    let mut i = PengaturanInstansi {
        subjenis: "SMP Negeri".into(),
        ..Default::default()
    };
    JenisInstansi::Sekolah.templat().isi(&mut p, &mut i);
    p.nama = "SMP Negeri 23 Dumai".into();
    p.induk = "PEMERINTAH KOTA DUMAI\nDINAS PENDIDIKAN".into();
    p.alamat = "Jl. Contoh No. 23".into();
    p.kota = "Dumai".into();
    p.id1_nomor = "10404123".into();
    p.nama_pimpinan = "Nama Kepala Sekolah".into();
    p.nip_pimpinan = "197001012000031001".into();
    p.nama_bendahara = "Nama Bendahara".into();
    p.nip_bendahara = "198502022010012002".into();
    db.selesaikan_penyiapan(JenisInstansi::Sekolah, &p, &i)
        .unwrap();
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
    let tanpa = Pungutan::default;
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
            tanpa(),
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
            tanpa(),
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
                ..tanpa()
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
            tanpa(),
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
                ..tanpa()
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
            tanpa(),
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
            tanpa(),
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
            tanpa(),
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
            tanpa(),
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
            tanpa(),
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
            tanpa(),
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
            tanpa(),
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
            tanpa(),
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
            tanpa(),
            None,
        ),
    ];
    for (tanggal, jenis, akun_id, kategori_id, uraian, jumlah, pihak, pungutan, setor_pajak) in data
    {
        let no_bukti = db
            .nomor_bukti_berikutnya(jenis.awalan_bukti(), tanggal.year())
            .unwrap();
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

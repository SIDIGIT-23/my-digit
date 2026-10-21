# Spesifikasi: "Masuk sebagai" dan Jenis Instansi, My Digit v0.3 (satu putaran implementasi)

Semua klaim di bawah sudah dicek ke kode sumber. Nomor baris mengacu ke berkas saat ini.

## 0. Keputusan atas tiga usulan

| Usulan | Keputusan | Alasan (hasil cek kode) |
|---|---|---|
| Lensa 1: 7 templat dan asisten 6 langkah | Diganti 4 jenis milik pemilik (Sekolah, Organisasi, Instansi, Lembaga). Asisten dipangkas jadi 2 layar. Opsi "Atur nanti" dihapus. | Permintaan pemilik mengalahkan usulan. 6 langkah tidak muat sehari. |
| Lensa 1: bug pemicu isi awal | **Diterima** | `siapkan()` (db.rs:140-159) mengisi akun dan kategori ulang tiap kali `COUNT(*) FROM akun == 0`. Bila semua akun dihapus, kategori jadi dobel. |
| Lensa 1: rujukan "db.rs:715" | Dikoreksi | Fallback "Kepala Sekolah" ada di db.rs:468. |
| Lensa 1 dan 3: "NIP." dan "Bendahara" tertulis mati di pdf.rs | **Diterima, diperluas** | Kedua lensa melewatkan excel.rs: kop di :83-137, "Bendahara" di :245, `"NIP. {nip}"` di :260. |
| Lensa 1: `ORDER BY kode` mengurutkan teks | **Diperbaiki sekarang** (urut angka di Rust), tidak dihindari | Perlu supaya kategori templat yang ditambahkan saat ganti jenis bisa memakai kode 4.10 dan seterusnya. |
| Lensa 2: banyak instansi (indeks dan satu berkas per instansi) | **Ditunda** | Butuh sekitar 1 hari sendiri. Putaran ini hanya menyiapkan `instansi.id`, folder `cadangan-otomatis/`, `VERSI_SKEMA`, dan `reset_ui()`. |
| Lensa 2: celah cadangkan/pulihkan | **Diterima (bagian murah)** | `let _ = db.cadangkan(..)` (ui/mod.rs:622) mengabaikan kegagalan. Cadangan darurat cuma satu slot. `fs::copy` langsung ke berkas aktif (db.rs:519) tidak atomik. `cadangkan` menghapus tujuan dulu (db.rs:500-502). `ui_anggaran.isian` tidak di-reset setelah pulih, sehingga anggaran lama bisa tersimpan ke data hasil pulihan. |
| Lensa 2 dan 3: logo di dalam DB | Ditunda (skema v2) | Masalah lama, tidak menghalangi fitur jenis. |
| Lensa 3: tahun buku, format nomor berpola, tanggal/tempat TTD, penanda tangan ke-3 | Ditunda. Putaran ini hanya awalan nomor bukti. | Mengubah arti `anggaran.tahun` dan algoritma nomor. Risiko regresi tinggi. |
| Lensa 3: bawaan baca = perilaku sekarang | **Diterima sebagai prinsip** | Fallback saat kunci tidak ada tetap konstan. Templat hanya ditulis saat penyiapan atau saat pengguna menerapkannya. DB lama tidak berubah keluarannya. |
| Nama halaman "Pengaturan Instansi" | Diganti **"Profil {Jenis}"** (mis. "Profil Sekolah") | "Instansi" dan "Lembaga" sudah jadi nama jenis, jadi tidak bisa dipakai sebagai kata payung. |

## 1. Cakupan

### MASUK (urut pengerjaan; total sekitar 9-10 jam)

| # | Pekerjaan | Perkiraan |
|---|---|---|
| 1 | db.rs: `VERSI_SKEMA`, migrasi v0→v1 dengan salinan pengaman, pemicu isi awal diganti, `busy_timeout`, urut kode numerik | 2 j |
| 2 | src/instansi.rs: 4 jenis, templat, `periksa_nomor`, tes | 1 j |
| 3 | model.rs/db.rs: field baru `ProfilLembaga`, `PengaturanInstansi`, kunci baru | 0,75 j |
| 4 | pdf.rs dan excel.rs: baris induk dan identitas di kop, label nomor induk, jabatan bendahara | 1,25 j |
| 5 | ui/penyiapan.rs: layar pertama "Masuk sebagai" (2 langkah) | 2 j |
| 6 | ui/profil.rs → "Profil {Jenis}": kartu Masuk sebagai, lembar Ubah jenis, kartu Penanda Tangan dan Pembukuan | 1,5 j |
| 7 | Istilah, sumber dana, sakelar pajak, awalan bukti di transaksi/laporan/ringkasan/anggaran/akun | 0,75 j |
| 8 | Pengerasan cadangkan/pulihkan dan `reset_ui()` | 0,75 j |
| — | **Garis potong.** Di bawah ini boleh bergeser ke putaran berikut. | |
| 9 | Opsional: "Mulai dari Awal…" (arsipkan berkas, lalu penyiapan baru) | 0,75 j |

### DITUNDA

| Item | Alasan |
|---|---|
| Banyak instansi per instalasi (pemilih di sidebar, impor, arsip per instansi) | Sekitar 1 hari. Butuh logo di DB dan reset seluruh state UI lebih dulu. Rancangannya ada di §5.7. |
| Logo di dalam database | Butuh DDL (v2) dan gambar dari bytes di pdf/excel. Tidak menghalangi fitur jenis. |
| Tahun buku selain Januari | Mengubah arti `anggaran.tahun`, `Periode`, dan pemilih tahun. Keempat jenis memakai Januari. Sementara pakai mode "Rentang Tanggal". |
| Nomor bukti berpola ({BULAN_ROMAWI}, {KODE}, reset per bulan) | Butuh pengurai dan pencocok baru plus regresi. Putaran ini cukup awalan. |
| Tanggal TTD = akhir periode; kota TTD terpisah | Kecil, tapi bukan inti permintaan. |
| Penanda tangan ketiga | `tanda_tangan()` perlu tata letak 3 blok. |
| Istilah anggaran di judul/subjudul PDF | Menjaga keluaran lama. Putaran ini istilah hanya dipakai di teks UI. |
| Kategori per sub-jenis (SPP swasta, infak Jumat) dan kode rekening resmi (BOSP, Permendagri 90, BAS) | Perlu verifikasi juknis. Satu templat per jenis dulu. |
| Pratinjau kop grafis / "Lihat Contoh Kop" | Putaran ini cukup pratinjau teks tanda tangan. |
| Kunci satu jendela, cadangan otomatis harian, laporan Kas Jumat, Hijriah, multi pengguna | Di luar inti; masing-masing terpisah. |
| Huruf non-Latin di PDF | Perlu font TrueType tertanam. Putaran ini hanya peringatan di UI. |

## 2. Model data dan penyimpanan

### 2.1 Tata letak berkas (di `folder_data()`, tetap bisa ditimpa `MY_DIGIT_DATA_DIR`)

```
my-digit.sqlite                       data (tetap di tempat, nama tetap)
logo-lembaga.<ext>                    tetap seperti sekarang
cadangan-otomatis/                    BARU: salinan buatan aplikasi
  my-digit-sebelum-v1-YYYYMMDD-HHMMSS-<pid>.sqlite
  my-digit-sebelum-pulih-YYYYMMDD-HHMMSS-<pid>.sqlite
arsip/                                BARU, hanya bila item 9 dikerjakan
```

- Folder `cadangan-otomatis/` dihitung dari `path.parent()` berkas DB, bukan dari `folder_data()`, supaya tes bisa memakai folder sementara.
- Jangan ubah `ProjectDirs::from("id","SIDIGIT-23","My Digit")`, `NAMA_DB`, atau `identifier`. Kalau berubah, data tampak hilang.
- Skema tabel **tidak berubah** (tanpa CREATE/ALTER baru). v0.2 tetap bisa membuka berkas v1.

### 2.2 Kunci baru di tabel `pengaturan`

Fallback dipakai bila kunci **tidak ada**. Kunci yang ada tapi kosong tetap kosong.

| Kunci | Isi | Fallback |
|---|---|---|
| `instansi.id` | `format!("i{:x}", Timestamp::now().as_nanosecond())` | — (ditulis saat migrasi) |
| `instansi.jenis` | `sekolah`/`organisasi`/`instansi`/`lembaga` | Sekolah |
| `instansi.subjenis` | teks bebas, mis. "SMP Negeri" | "" |
| `instansi.siap` | "1" bila penyiapan selesai | tidak ada → layar penyiapan tampil |
| `instansi.pajak` | "1"/"0" | true |
| `instansi.istilah_anggaran` | mis. "RKAS", "DPA" | "RKAS" |
| `instansi.sumber_dana` | item dipisah `\|` (item dibersihkan dari `\|`, yang kosong dibuang) | "BOS\|Komite\|Swadaya" |
| `nomor.awalan.masuk` / `.keluar` / `.transfer` | `[A-Z0-9]{1,8}` | BKM / BKK / TRF (juga bila nilainya tidak valid) |
| `profil.induk` | maksimal 2 baris dipisah `\n` | "" |
| `profil.jabatan_bendahara` | | "Bendahara" |
| `profil.label_nomor_pimpinan` / `profil.label_nomor_bendahara` | mis. NIP, NIY, "" | "NIP" |
| `profil.id1_label`, `profil.id1_nomor`, `profil.id2_label`, `profil.id2_nomor` | mis. NPSN 10404123 | "" |

- `profil.nip_pimpinan` dan `profil.nip_bendahara` tidak diganti namanya. Maknanya kini "nomor induk".
- `profil.jabatan_pimpinan` tetap berfallback "Kepala Sekolah" (db.rs:468).

### 2.3 Struct

```rust
// src/instansi.rs (BARU; `mod instansi;` di main.rs)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JenisInstansi { Sekolah, Organisasi, Instansi, Lembaga }
impl JenisInstansi {
    pub const SEMUA: [Self; 4];                 // urutan: Sekolah, Organisasi, Instansi, Lembaga
    pub fn kode(self) -> &'static str;          // "sekolah" | "organisasi" | "instansi" | "lembaga"
    pub fn dari_kode(s: &str) -> Self;          // tak dikenal → Sekolah
    pub fn label(self) -> &'static str;         // "Sekolah" …
    pub fn kata(self) -> &'static str;          // "sekolah" … ("Nama sekolah")
    pub fn ikon(self) -> &'static str;
    pub fn keterangan(self) -> &'static str;    // satu baris untuk kartu
    pub fn templat(self) -> &'static Templat;
}
pub struct Templat {
    pub contoh_nama: &'static str, pub contoh_induk: &'static str,
    pub subjenis: &'static [&'static str],
    pub pimpinan: &'static str,    pub pilihan_pimpinan: &'static [&'static str],
    pub bendahara: &'static str,   pub pilihan_bendahara: &'static [&'static str],
    pub label_nomor: &'static str, pub pilihan_label_nomor: &'static [&'static str],
    pub id1: &'static str, pub id2: &'static str, pub pilihan_id: &'static [&'static str],
    pub istilah: &'static str,     pub pilihan_istilah: &'static [&'static str],
    pub sumber_dana: &'static [&'static str],
    pub pajak: bool,
    pub kategori: &'static [(&'static str, &'static str, Arus)],
}
impl Templat {
    /// Isi field profil/instansi dari templat (dipakai saat penyiapan).
    pub fn isi(&self, p: &mut ProfilLembaga, i: &mut PengaturanInstansi);
}
/// Peringatan non-blokir: NPSN 8 digit, NSM 12 digit, NPWP 15/16 digit (titik/strip dibuang).
pub fn periksa_nomor(label: &str, nomor: &str) -> Option<&'static str>;
/// "4.10" → [(4,""),(10,"")]; segmen non-angka → (u32::MAX, teks).
pub fn kunci_kode(kode: &str) -> Vec<(u32, String)>;
```

```rust
// model.rs: ProfilLembaga (tetap Default + PartialEq), field baru:
pub induk: String,
pub jabatan_bendahara: String,
pub label_nomor_pimpinan: String,
pub label_nomor_bendahara: String,
pub id1_label: String, pub id1_nomor: String,
pub id2_label: String, pub id2_nomor: String,
impl ProfilLembaga {
    /// "NIP. 1970…" | "1970…" bila label kosong | "" bila nomor kosong.
    pub fn baris_nomor(label: &str, nomor: &str) -> String;
    /// "NPSN 10404123 · NSM 1212…" (pasangan bernomor kosong dilewati).
    pub fn baris_identitas(&self) -> String;
    /// ≤ 2 baris induk yang tidak kosong, di-trim, huruf kapital.
    pub fn baris_induk(&self) -> Vec<String>;
}

// model.rs: BARU
#[derive(Debug, Clone, PartialEq)]
pub struct PengaturanInstansi {
    pub jenis: JenisInstansi, pub subjenis: String,
    pub pajak: bool, pub istilah_anggaran: String, pub sumber_dana: Vec<String>,
    pub awalan_masuk: String, pub awalan_keluar: String, pub awalan_transfer: String,
}
impl PengaturanInstansi { pub fn awalan(&self, j: JenisTransaksi) -> &str; }
```

### 2.4 Perubahan fungsi di db.rs

| Fungsi | Perubahan |
|---|---|
| `KATEGORI_AWAL` (:86-101) | Pindah ke instansi.rs sebagai `KATEGORI_SEKOLAH`, **isi identik**. db.rs merujuknya untuk cek "bawaan v0.2". |
| `buka` (:111) | `let mut db`; panggil `db.siapkan()`. Field baru `salinan_migrasi: Option<PathBuf>`. `Connection::open` tetap dipakai (berkas bawaan memang dibuat saat pertama kali). |
| `siapkan` (:135-161) | Ditulis ulang jadi `&mut self` sesuai §2.5. Pemicu `COUNT(*) FROM akun == 0` **dihapus**. |
| BARU `cadangkan_otomatis(&self, alasan) -> Hasil<PathBuf>` | Buat folder bila belum ada, lalu `VACUUM INTO` (di luar transaksi). Verifikasi: buka salinan `SQLITE_OPEN_READ_ONLY`, `PRAGMA quick_check = 'ok'`, `COUNT(*) transaksi` sama dengan sumber. Bila gagal, hapus **hanya** berkas parsial bernama unik yang baru dibuatnya, lalu kembalikan Err. |
| BARU `masih_bawaan_v02(&self) -> Hasil<bool>` | 0 transaksi, 0 anggaran, `profil.nama` kosong atau tidak ada, dan himpunan (kode, nama, arus) kategori == `KATEGORI_SEKOLAH` (14 baris persis). |
| `daftar_kategori` (:222) | Setelah query, urutkan di Rust dengan `kunci_kode`, lalu nama. |
| `nomor_bukti_berikutnya(jenis, tahun)` (:380) | Jadi `(awalan: &str, tahun: i16)`. Logika LIKE tidak berubah; awalan sudah divalidasi `[A-Z0-9]`, jadi tidak ada wildcard `%`/`_`. |
| `profil` / `simpan_profil` (:458-494) | Kunci baru dan fallback sesuai §2.2. Simpan dibungkus `unchecked_transaction()`. |
| BARU `instansi()`, `simpan_instansi(&PengaturanInstansi)` | Kunci `instansi.*` dan `nomor.awalan.*`. |
| BARU `perlu_penyiapan() -> bool` | `ambil("instansi.siap") != Some("1")`. |
| BARU `selesaikan_penyiapan(&self, jenis, &ProfilLembaga, &PengaturanInstansi)` | Satu transaksi: `simpan_profil`, `simpan_instansi`, lalu kategori. Bila kategori kosong, sisipkan templat. Bila `masih_bawaan_v02()` dan jenis ≠ Sekolah, hapus 14 baris bawaan (tak dirujuk apa pun) lalu sisipkan templat. Terakhir `instansi.siap='1'`. |
| BARU `rencana_kategori(jenis) -> Hasil<Vec<Kategori>>` | Pratinjau (id = 0). Butir templat dilewati bila nama yang sama (trim, tanpa beda huruf besar/kecil, arus sama) sudah ada. Kode templat dipakai bila belum terpakai. Bila terpakai, dialokasikan `"{segmen1}.{max+1}"` dari kode 2-segmen yang ada plus yang sudah dialokasikan. |
| BARU `terapkan_jenis(&self, &PengaturanInstansi, &ProfilLembaga, &[Kategori]) -> Hasil<usize>` | Satu transaksi: tulis instansi dan profil, sisipkan kategori baru. **Tidak pernah DELETE.** |
| `cadangkan` (:499) | Tolak bila induk tujuan (canonical) berada di dalam folder DB: "Simpan cadangan di luar folder data aplikasi, misalnya Dokumen atau flashdisk." Tulis `VACUUM INTO <tujuan>.tmp`, lalu `rename` menimpa tujuan. Hanya `.tmp` milik sendiri yang boleh dihapus. |
| BARU `Db::periksa_cadangan(path) -> Hasil<InfoCadangan>` dan `info(&self) -> InfoCadangan` | `InfoCadangan { nama, id: Option<String>, jenis: Option<JenisInstansi>, transaksi: i64, terakhir: Option<Date>, versi: i32 }`. Sumber dibuka READ_ONLY. Wajib: tabel transaksi ada, `quick_check` ok, `versi <= VERSI_SKEMA`. |
| `pulihkan(self, sumber) -> Hasil<(Db, PathBuf)>` (:510) | Urutan pada §2.7. Mengembalikan path salinan darurat. |
| BARU `#[cfg(test)] memori_siap(jenis)` | `memori()` lalu `selesaikan_penyiapan` dengan profil contoh. |

### 2.5 Migrasi my-digit.sqlite yang sudah ada (v0 → v1)

```
siapkan(&mut self):
 1. conn.busy_timeout(5 s); PRAGMA foreign_keys = ON
 2. versi = PRAGMA user_version
 3. versi > VERSI_SKEMA (=1) → Err("Data ini dibuat oleh My Digit versi yang lebih baru.
                                 Perbarui My Digit untuk membukanya. Data tidak diubah.")
 4. lama = EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='akun')
 5. if versi == 0 && lama && path tidak kosong:
        self.salinan_migrasi = Some(cadangkan_otomatis("sebelum-v1")?)   // Err → berhenti
 6. execute_batch(SKEMA)              // IF NOT EXISTS: no-op untuk DB lama
 7. if versi == 0:
      tx = transaction_with_behavior(Immediate)
      if PRAGMA user_version (dibaca ulang di dalam tx) == 0:
         if !lama:  INSERT akun ('Kas Tunai','tunai'), ('Rekening Bank','bank')   // tanpa kategori
         else if !masih_bawaan_v02():
             INSERT OR IGNORE instansi.jenis = tebak_jenis()
             INSERT OR IGNORE instansi.siap  = '1'
         // DB lama yang masih bawaan: tidak ditulis apa-apa, jadi layar penyiapan tampil
         INSERT OR IGNORE instansi.id = i<hex>
         PRAGMA user_version = 1
      tx.commit()
```

- `tebak_jenis()`: `profil.jabatan_pimpinan` = "Ketua Yayasan" → Lembaga; "Ketua" → Organisasi; selain itu (termasuk tidak ada) → Sekolah.
- Tidak ada kunci `profil.*` yang ditulis. Fallback (§2.2) membuat PDF/Excel SMP Negeri 23 Dumai identik dengan sekarang.
- Tabel akun, kategori, transaksi, dan anggaran tidak disentuh, kecuali penggantian 14 kategori bawaan yang belum dipakai, dan itu hanya terjadi saat penyiapan (§2.4).
- Toast sekali di `MyDigitApp::new` bila `salinan_migrasi` ada: "Data diperbarui ke format baru. Salinan pengaman tersimpan di folder Cadangan Otomatis."

**Bila gagal di tengah jalan**

| Titik gagal | Keadaan berkas | Yang terjadi |
|---|---|---|
| Langkah 3 (versi lebih baru) | tidak disentuh | Halaman galat dengan pesan versi |
| Langkah 5 (disk penuh, izin, verifikasi gagal) | DB lama utuh; salinan parsial dihapus | Halaman galat: "Pembaruan data dibatalkan karena salinan pengaman gagal dibuat. Data Anda tidak diubah." disertai galat dan lokasi. Tombol "Coba Lagi" dan "Buka Folder Data". |
| Aplikasi mati antara langkah 5 dan commit | SQLite me-rollback lewat journal; versi tetap 0 | Start berikutnya mengulang migrasi (salinan baru bernama unik) |
| Dua jendela membuka bersamaan | jendela kedua menunggu (busy_timeout), lalu melihat versi 1 di dalam tx | Tidak ada migrasi ganda |
| Setelah commit | v1 | Normal. v0.2 tetap bisa membuka berkas ini (skema sama, kunci baru diabaikan). |

### 2.6 Berkas baru (instalasi pertama)

- Isinya 2 akun dan 0 kategori, `user_version=1`. `perlu_penyiapan()` bernilai true.
- Kategori baru disisipkan saat penyiapan selesai.
- Bila aplikasi ditutup di tengah penyiapan, tidak ada yang tertulis. Layar penyiapan tampil lagi saat dibuka.

### 2.7 Pulihkan (satu instansi, dengan pengaman)

1. `Db::periksa_cadangan(sumber)`. Tolak bila canonical(sumber) == canonical(berkas aktif).
2. `cadangkan_otomatis("sebelum-pulih")`. Bila **gagal, batalkan**: "Pemulihan dibatalkan: salinan pengaman gagal dibuat." Ini menggantikan `let _ =` di ui/mod.rs:622.
3. `fs::copy(sumber, <db>.baru)`, lalu buka READ_ONLY dan jalankan `quick_check`.
4. `drop(self)`. Bila `<db>-journal` ada, hentikan dan buka kembali berkas asli.
5. `fs::rename(<db>.baru, <db>)`. Menimpa secara atomik di macOS dan Windows. Bila gagal, berkas asli tetap utuh, dibuka kembali, dan galat ditampilkan.
6. `Db::buka(<db>)`: cadangan v0 ikut dimigrasi di sini.
7. Di UI: `reset_ui()`, `muat_ulang()`, `ui_penyiapan = perlu_penyiapan().then(..)`. Toast: "Data berhasil dipulihkan. Data sebelumnya disimpan di Cadangan Otomatis."

## 3. Jenis instansi dan templat

Hanya 4 jenis tingkat atas, dengan urutan tetap: **Sekolah, Organisasi, Instansi, Lembaga**. Sub-jenis berupa teks bebas plus tombol saran. Sub-jenis hanya label (tampil di sidebar dan Profil) dan **tidak mengubah bawaan**.

Ikon sudah dicek di `assets/fonts/Phosphor.ttf` lewat tabel ligatur GSUB. Konstanta baru ditambahkan ke `ui/ikon.rs`.

| | Sekolah | Organisasi | Instansi | Lembaga |
|---|---|---|---|---|
| kode | `sekolah` | `organisasi` | `instansi` | `lembaga` |
| Ikon | GRADUATION_CAP `\u{E62C}` | USERS_THREE `\u{E68E}` | BUILDING_OFFICE `\u{E0FF}` | HAND_HEART `\u{E810}` |
| Keterangan kartu | SD, SMP, SMA/SMK, madrasah — negeri atau swasta. Dana BOS, komite, RKAS. | OSIS, karang taruna, komunitas, ormas, paguyuban. Iuran anggota dan kegiatan. | Kantor desa/kelurahan, dinas, puskesmas, UPT. APBD/APBN, DPA, pajak. | Yayasan, masjid/musala, LSM, panti asuhan, TPQ. Donasi, infak, hibah. |
| Sub-jenis (saran) | SD Negeri, SMP Negeri, SMA Negeri, SMK Negeri, Sekolah Swasta, Madrasah | OSIS, Karang Taruna, Komunitas, Ormas, Paguyuban, RT/RW | Kantor Desa, Kelurahan, Dinas/Badan, Puskesmas, UPT | Yayasan, Masjid/Musala, LSM, Panti Asuhan, TPQ |
| Contoh nama | mis. SMP Negeri 23 Dumai | mis. OSIS SMP Negeri 23 Dumai | mis. Kantor Kelurahan Bukit Datuk | mis. Masjid Al-Ikhlas |
| Contoh induk | PEMERINTAH KOTA DUMAI / DINAS PENDIDIKAN | SMP NEGERI 23 DUMAI | PEMERINTAH KOTA DUMAI / KECAMATAN DUMAI TIMUR | YAYASAN … |
| Pimpinan (bawaan) | Kepala Sekolah | Ketua | Kepala Kantor | Ketua |
| Saran pimpinan | Kepala Madrasah, Plt. Kepala Sekolah, Ketua Yayasan | Ketua Umum, Ketua OSIS, Pembina, Ketua RT, Ketua RW | Kepala Dinas, Kepala Desa, Lurah, Camat, Kepala Puskesmas, Kepala UPT, Pengguna Anggaran, Kuasa Pengguna Anggaran | Ketua Yayasan, Ketua Pengurus, Ketua DKM, Ketua Takmir, Direktur, Kepala TPQ, Pimpinan Panti |
| Bendahara (bawaan) | Bendahara | Bendahara | Bendahara Pengeluaran | Bendahara |
| Saran bendahara | Bendahara BOS, Bendahara Komite | Bendahara Umum, Bendahara OSIS | Bendahara Penerimaan, Kaur Keuangan, Bendahara | Bendahara Yayasan, Bendahara Masjid |
| Label nomor induk (keduanya) | NIP (saran: NIY, NUPTK) | "" (saran: NIP, NIY, NRA) | NIP (saran: NRP) | "" (saran: NIY) |
| Identitas 1 / 2 (label) | NPSN / "" | No. SK / "" | Kode Satker / NPWP | No. SK Kemenkumham / NPWP |
| Saran label identitas | NPSN, NSM, NPWP | No. SK Pengurus, No. SKT, No. AHU, NPWP | Kode Satker, Kode SKPD, Kode Desa, Kode Puskesmas, NPWP | No. SK Kemenkumham, No. Akta Pendirian, ID Masjid, No. Izin Operasional, NPWP |
| Istilah anggaran | RKAS (RKAM, RAPBS) | Rencana Anggaran (RAB, Program Kerja) | DPA (DIPA, APBDes, RBA) | Rencana Anggaran (RKAT, RAB) |
| Sumber dana (tombol cepat) | BOS, Komite, Swadaya | Kas, Iuran, Donasi, Sponsor | APBD, APBN, Dana Desa, BLUD | Donatur, Infak, Hibah, Zakat |
| **Fitur pajak** (form pajak, Buku Pembantu Pajak, ubin pajak) | **aktif** | nonaktif | **aktif** | nonaktif (catatan: "Yayasan yang memotong PPh karyawan sebaiknya mengaktifkan.") |
| Anggaran, Realisasi, Pemeriksaan Kas, Kwitansi | aktif | aktif | aktif | aktif |

Akun awal untuk semua jenis tetap "Kas Tunai" dan "Rekening Bank". Kode kategori adalah kode internal sederhana (x.1–x.9), bukan kode rekening resmi. Prefiks 4 = Masuk, 5 = Keluar.

**Sekolah** (identik dengan `KATEGORI_AWAL`, 14)

| Kode | Penerimaan | Kode | Pengeluaran |
|---|---|---|---|
| 4.1 | Dana BOS | 5.1 | Honor & Belanja Pegawai |
| 4.2 | Iuran / Dana Komite | 5.2 | Alat Tulis & Bahan Habis Pakai |
| 4.3 | Donasi & Sumbangan | 5.3 | Kegiatan Pembelajaran |
| 4.4 | Jasa Giro / Bunga Bank | 5.4 | Pemeliharaan Sarana Prasarana |
| 4.9 | Penerimaan Lain-lain | 5.5 | Langganan Daya & Jasa |
| | | 5.6 | Belanja Modal / Peralatan |
| | | 5.7 | Biaya Administrasi Bank |
| | | 5.8 | Setoran Pajak |
| | | 5.9 | Pengeluaran Lain-lain |

**Organisasi** (16)

| Kode | Penerimaan | Kode | Pengeluaran |
|---|---|---|---|
| 4.1 | Iuran Anggota | 5.1 | Program Kerja & Kegiatan |
| 4.2 | Iuran / Kontribusi Kegiatan | 5.2 | Konsumsi Rapat & Pertemuan |
| 4.3 | Donasi & Sponsor | 5.3 | Dana Sosial & Santunan |
| 4.4 | Hibah / Bantuan | 5.4 | ATK, Cetak & Administrasi |
| 4.5 | Hasil Usaha & Dana Usaha | 5.5 | Transportasi & Akomodasi |
| 4.6 | Jasa Giro / Bunga Bank | 5.6 | Perlengkapan & Inventaris |
| 4.9 | Penerimaan Lain-lain | 5.7 | Honor / Uang Saku Petugas |
| | | 5.8 | Biaya Administrasi Bank |
| | | 5.9 | Pengeluaran Lain-lain |

**Instansi** (17)

| Kode | Penerimaan | Kode | Pengeluaran |
|---|---|---|---|
| 4.1 | Uang Persediaan / Ganti Uang (UP/GU) | 5.1 | Belanja Pegawai / Honorarium |
| 4.2 | Tambahan Uang (TU) | 5.2 | Belanja Barang Pakai Habis & ATK |
| 4.3 | Pembayaran Langsung (LS) | 5.3 | Belanja Jasa (Listrik, Air, Telepon, Internet) |
| 4.4 | Dana Transfer (Dana Desa, ADD, DAK) | 5.4 | Belanja Perjalanan Dinas |
| 4.5 | Pendapatan Asli / Retribusi / Jasa Layanan | 5.5 | Belanja Pemeliharaan |
| 4.6 | Hibah / Bantuan Keuangan | 5.6 | Belanja Modal |
| 4.7 | Jasa Giro / Bunga Bank | 5.7 | Belanja Hibah / Bantuan Sosial |
| 4.9 | Penerimaan Lain-lain | 5.8 | Setoran Pajak |
| | | 5.9 | Setoran Sisa Kas / Pengeluaran Lain-lain |

**Lembaga** (18; memakai "Bagi Hasil / Jasa Giro", bukan "Bunga")

| Kode | Penerimaan | Kode | Pengeluaran |
|---|---|---|---|
| 4.1 | Donatur Tetap | 5.1 | Honor Pengurus, Pengajar & Petugas |
| 4.2 | Infak & Sedekah | 5.2 | Program & Kegiatan |
| 4.3 | Donasi Tidak Tetap / Kotak Amal | 5.3 | Santunan & Penyaluran Bantuan |
| 4.4 | Zakat, Fidyah & Dana Titipan | 5.4 | Penyaluran Zakat & Dana Titipan |
| 4.5 | Hibah / Bantuan Pemerintah | 5.5 | Operasional Kantor & ATK |
| 4.6 | Hibah / CSR Lembaga Lain | 5.6 | Listrik, Air & Langganan |
| 4.7 | Hasil Usaha & Sewa Fasilitas | 5.7 | Pemeliharaan & Pembangunan |
| 4.8 | Bagi Hasil / Jasa Giro Bank | 5.8 | Biaya Administrasi Bank |
| 4.9 | Penerimaan Lain-lain | 5.9 | Pengeluaran Lain-lain |

## 4. Pengaturan yang memengaruhi keluaran

| Pengaturan | Putaran ini | Bawaan untuk DB lama |
|---|---|---|
| Awal tahun buku | **Ditunda**. `Periode::tahun()` tetap 1 Jan – 31 Des. Rancangan berikutnya: `instansi.bulan_mulai`, `Periode::tahun_buku(tahun, bm)` (bm=1 identik), label "2026/2027", modal konfirmasi bila anggaran sudah ada. | — |
| Awalan nomor bukti | `nomor.awalan.*`; pola tetap `{AWALAN}-{TAHUN}-{NNNN}` | BKM/BKK/TRF |
| Penanda tangan | jabatan bendahara, label nomor induk per pihak | "Bendahara", "NIP" |
| Kop | baris induk (≤ 2 baris) dan baris identitas | kosong → kop identik |

**laporan.rs: tidak ada fungsi yang berubah.** Periode dan realisasi tetap. Penyaringan Buku Pembantu Pajak dilakukan di UI.

**pdf.rs**

| Fungsi | Perubahan |
|---|---|
| `Kanvas::kop` (:167-212) | `n = p.baris_induk().len()`. Baris induk Tebal 10 pt huruf kapital, baseline `y-10-12i`. Nama, info, dan kontak bergeser `12n` (`y-16-12n`, `y-30-12n`, `y-42-12n`). Baris kontak jadi `"{identitas} · Telp/Email: …"`, hanya identitas, atau tetap seperti sekarang. Tinggi blok `max(48, 48+12n)`, lalu `self.y -= tinggi + 8`. Logo dipusatkan vertikal (`y - 48 - (tinggi-48)/2`). Semua baris kop melewati `potong(.., w - MARGIN - x, ..)`. **Bila n = 0 dan identitas kosong, koordinat identik.** Ekstrak `fn tinggi_kop(p) -> f32` untuk tes. |
| `tanda_tangan` (:228-266) | Elemen ke-4 kini baris jadi. `format!("NIP. {nip}")` di :262 diganti teks apa adanya. Jabatan dipotong `potong(.., lebar-10, 10.0, Biasa)`. |
| `laporan` (:436-451) | Kiri: `baris_nomor(&p.label_nomor_pimpinan, &p.nip_pimpinan)`. Kanan: jabatan `&p.jabatan_bendahara`, `baris_nomor(&p.label_nomor_bendahara, &p.nip_bendahara)`. |
| `kwitansi` (:474-499) | `bendahara` jadi `jabatan_bendahara` atau `"{jabatan_bendahara} {nama}"`. "Bendahara" di :486 dan :495 diganti `jabatan_bendahara`. Nomor lewat `baris_nomor`. |
| `berita_acara_kas` (:601-602, :680-690) | "dikelola oleh Bendahara" diganti `{jabatan_bendahara}`. "Yang diperiksa, Bendahara" diganti `format!("Yang diperiksa, {}", jabatan_bendahara)`. Nomor lewat `baris_nomor`. |

**excel.rs**

| Bagian | Perubahan |
|---|---|
| Kop (:83-137) | `o = baris_induk().len() as u32`. Baris 0..o berisi induk (tebal, 11 pt). Nama, alamat, kontak, garis, judul (5, 6), dan `baris_kepala` (8) semuanya `+o`. Kontak ikut identitas seperti di PDF. o = 0 berarti tata letak identik. |
| Tanda tangan (:234-261) | `"Bendahara"` (:245) diganti `jabatan_bendahara`. `format!("NIP. {nip}")` (:260) diganti `baris_nomor(..)`. |

**db.rs** (§2.4): `nomor_bukti_berikutnya(awalan, tahun)`, profil dan instansi, migrasi, cadangan.

## 5. UI (gaya desktop Apple, teks Bahasa Indonesia)

### 5.1 Shell (ui/mod.rs)

- `MyDigitApp` mendapat field baru:
  - `pub instansi: PengaturanInstansi`, dimuat di `muat_ulang()` di sebelah `profil`;
  - `ui_penyiapan: Option<penyiapan::State>`, diisi di `new()` bila `db.perlu_penyiapan()`.
- Fungsi baru:
  - `fn reset_ui(&mut self)`: semua `ui_*` = `Default`. Dipanggil setelah pulih, setelah penyiapan selesai, setelah ganti jenis, dan setelah mulai baru.
  - `pub fn pajak_tampil(&self) -> bool`: `instansi.pajak || transaksi.iter().any(|t| t.pungutan.total() > 0 || t.setor_pajak.is_some())`.
- `App::ui`, urutannya: splash → `db.is_none()` → `if ui_penyiapan.is_some() { penyiapan::tampil(self, ui); toast; return; }` → normal.
- `buka_halaman_dev`: tambah `"penyiapan"`.
- `halaman_galat_db`: tambah tombol "Coba Lagi" (panggil ulang `Db::buka_bawaan`) dan "Buka Folder Data".
- **Sidebar**:
  - Kelompok "Lembaga" diganti nama jadi **"Pengaturan"**. Isinya: **"Profil {Jenis}"** (ikon jenis; label dinamis di loop `sidebar`), **"Tampilan & Data"** (halaman `Pengaturan` lama, ikon GEAR_SIX), "Tentang Aplikasi".
  - Subjudul di bawah "My Digit": nama, atau `"{Jenis} · {subjenis}"` bila nama kosong.

### 5.2 Layar pertama "Masuk sebagai" (src/ui/penyiapan.rs, BARU)

`State { langkah: Langkah {Jenis, Data}, jenis: Option<JenisInstansi>, profil: ProfilLembaga, instansi: PengaturanInstansi, galat: Option<String> }`

**Tata letak umum**
- Jendela penuh tanpa sidebar, `CentralPanel` berlatar `p.latar`.
- Di macOS ada ruang atas 40 px.
- Kolom tengah maksimal 640 px di dalam `ScrollArea`, supaya muat di jendela minimum 860×560.

**Langkah 1 — Masuk sebagai**
- Logo aplikasi 64 px, judul "Selamat datang di My Digit" (26 pt), tagline redup, lalu judul kecil "Masuk sebagai" (15 pt tebal).
- Kisi kartu 2×2 dengan urutan Sekolah, Organisasi, Instansi, Lembaga.
  - Kartu: tinggi minimal 112, sudut 12, margin 16, ikon 30 pt `p.aksen`, label 16 pt tebal, keterangan 12 pt `p.teks2` (maksimal 2 baris).
  - Kartu terpilih: isi `p.pilih`, garis 1,5 `p.aksen`, CHECK_CIRCLE di pojok kanan atas. Hover: garis aksen pudar dan kursor tangan.
- Bilah bawah: tautan kiri "Pulihkan dari Cadangan…" (memakai modal pulih bersama, §5.5). Kanan: tombol utama "Lanjutkan", nonaktif sampai ada pilihan. Return = Lanjutkan. Klik ganda kartu = Lanjutkan.

**Langkah 2 — Data {Jenis}**
- Indikator "Langkah 2 dari 2" dengan titik progres.
- Kartu form (`tema::baris_form`):
  - Jenis (sub-jenis): isian dengan tombol saran;
  - Nama {kata}*: petunjuk berupa `contoh_nama`;
  - Kota / kabupaten: "dipakai pada baris tanggal tanda tangan";
  - Jabatan pimpinan: isian dengan tombol saran;
  - Nama pimpinan;
  - Jabatan bendahara: isian dengan tombol saran;
  - Nama bendahara.
- Lipatan "Yang akan disiapkan" berisi:
  - "Kategori: N penerimaan · M pengeluaran", lalu daftar kode (monospace) dan nama;
  - "Fitur pajak: Aktif/Nonaktif";
  - "Istilah anggaran: DPA".
- Catatan redup: "Alamat, logo, nomor identitas, dan nomor induk dapat dilengkapi nanti di Profil {Jenis}."
- Tombol: "Kembali" (Esc) dan "Mulai Gunakan My Digit" (utama, nonaktif bila nama kosong).
- Bila jenis diganti lewat Kembali, isian yang masih sama dengan bawaan templat lama atau masih kosong diganti dengan bawaan templat baru. Isian yang sudah diketik pengguna dibiarkan.

**Selesai**
- `db.selesaikan_penyiapan(..)`. Bila Err: galat tampil di form dan layar tetap terbuka.
- Bila Ok: `ui_penyiapan = None`, `reset_ui()`, `muat_ulang()`, halaman = Ringkasan. Toast: "Selamat datang! Data {kata} siap digunakan."

### 5.3 Halaman "Profil {Jenis}" (ui/profil.rs, `Halaman::Profil`)

- Judul "Profil {Jenis}", subjudul "Dipakai untuk kop, tanda tangan laporan, dan kwitansi."
- Draf: `draf: Option<ProfilLembaga>`, `draf_instansi: Option<PengaturanInstansi>`.
- Tombol "Simpan" dan "Batalkan" tetap di kanan atas. Simpan menulis keduanya dalam satu transaksi.

| Kartu | Isi |
|---|---|
| **Masuk sebagai** | Ikon 28 pt, label tebal, baris redup "{subjenis} · {keterangan}", tombol "Ubah…" (ikon SWAP `\u{E83C}`). Tombol nonaktif bila draf berubah, dengan tooltip "Simpan atau batalkan perubahan terlebih dahulu." |
| **Identitas** | Nama {kata}; Jenis (sub-jenis) dengan tombol saran; Instansi induk (TextEdit 2 baris, petunjuk `contoh_induk`, catatan "Tampil di atas nama pada kop, huruf kapital."); Alamat; Kota / kabupaten; Nomor identitas 1 dan 2 (ComboBox label yang bisa diketik bebas, isian nomor, peringatan oranye dari `periksa_nomor`); Telepon; Email. Peringatan oranye bila nama/induk memuat huruf di luar Latin-1: "Huruf ini tidak dapat dicetak di PDF." |
| **Penanda Tangan** | Jabatan pimpinan (isian dan tombol saran dari templat, menggantikan daftar tetap di :78); Nama pimpinan; Nomor induk pimpinan (pop-up label: saran templat dan "Tanpa label", lalu isian nomor); Jabatan bendahara dengan tombol saran; Nama bendahara; Nomor induk bendahara. Di bawahnya pratinjau teks dua kolom: "Mengetahui," / jabatan / nama / `baris_nomor`, dan "{Kota}, {tanggal}" / jabatan bendahara / nama / nomor. |
| **Pembukuan** | Kotak centang "Gunakan fitur pajak (PPN, PPh 21/22/23)". Bila dimatikan tapi ada data pajak, tampil catatan "Bagian pajak tetap tampil karena sudah ada transaksi berpajak." Istilah anggaran: isian dengan tombol saran. Sumber dana: deretan chip dengan ×, isian, dan "+ Tambah". Awalan nomor bukti: Pemasukan / Pengeluaran / Transfer (isian 80 px, diubah ke huruf besar, divalidasi `[A-Z0-9]{1,8}`, Simpan nonaktif bila tidak valid), pratinjau "Contoh: BKM-2026-0001", catatan "Berlaku untuk transaksi baru. Nomor yang sudah ada tidak diubah. Awalan yang sama berarti satu urutan nomor." |
| **Logo** | Tidak berubah. |

### 5.4 Lembar "Ubah jenis" (egui::Modal, lebar 520)

- Judul "Masuk sebagai…". Empat kartu kecil berjajar (ikon dan label).
- **"Yang akan berubah"**: satu baris per field yang nilainya berbeda, berbentuk kotak centang "Jabatan pimpinan: Kepala Sekolah → Ketua".
  - Field yang tercakup: jabatan pimpinan, jabatan bendahara, label nomor induk pimpinan dan bendahara, label identitas 1 dan 2, fitur pajak, istilah anggaran, sumber dana.
  - Tercentang otomatis bila nilai sekarang == bawaan templat lama. Untuk label identitas, juga syaratnya nomornya masih kosong.
- **Kategori**: kotak centang (bawaan tercentang) "Tambahkan N kategori {Jenis} yang belum ada", lipatan pratinjau dari `rencana_kategori()` (kode akhir dan nama).
- Catatan redup: "Kategori, akun, dan transaksi yang ada tidak diubah atau dihapus."
- Nama, nomor identitas, nama dan nomor penanda tangan tidak pernah disentuh.
- Tombol rata kanan: "Batal" (Esc) lalu "Terapkan" (utama).
- Terapkan: `db.terapkan_jenis(..)` → `reset_ui()` → `muat_ulang()`. Toast: "Jenis diubah menjadi Organisasi. 12 kategori ditambahkan."

### 5.5 Halaman "Tampilan & Data" (ui/mod.rs `pengaturan`)

- Kartu Tampilan tidak berubah.
- Kartu Data:
  - Lokasi database; baris baru "Salinan otomatis" dengan tombol "Tampilkan Salinan Otomatis";
  - tombol "Cadangkan Data…" (nama bawaan `{slug-nama}-cadangan-{YYYY-MM-DD}.sqlite`; slug ASCII huruf kecil, angka, dan `-`, maksimal 40; kosong → `my-digit`), "Pulihkan dari Cadangan…", "Buka Folder Data".
- **Lembar "Pulihkan data?"** (lebar 460):
  - perbandingan dua kolom "Data saat ini" dan "Isi cadangan": nama, jenis, jumlah transaksi, transaksi terakhir;
  - pita peringatan bila `instansi.id` keduanya ada dan berbeda, atau nama keduanya terisi dan berbeda: "Cadangan ini milik ‘MI Al-Hikmah’, bukan ‘SMP Negeri 23 Dumai’.";
  - info bila cadangan berisi lebih sedikit transaksi;
  - catatan "Salinan data saat ini disimpan otomatis di folder Cadangan Otomatis.";
  - tombol: "Batal" (bawaan, Esc) lalu "Ganti Data" (merah). Urutan ini membalik urutan sekarang di :611-616.

### 5.6 Halaman lain

| Berkas | Perubahan |
|---|---|
| ui/transaksi.rs:58-61, :416 | `nomor_bukti_berikutnya(app.instansi.awalan(jenis), tahun)`. Petunjuk di :422 = `"{awalan}-{tahun}-0001"`. |
| ui/transaksi.rs:452-453 | Petunjuk `"mis. {sumber_dana[0]}"`, tombol cepat dari `instansi.sumber_dana`. |
| ui/transaksi.rs:463 | Bagian lipat "Pajak" hanya tampil bila `pajak_tampil()`, atau form sedang memuat setoran atau nominal pajak > 0. Nilai `pajak_tampil()` dihitung sebelum meminjam `f`. |
| ui/laporan.rs:54 | Kisi dari `SEMUA` yang disaring (PembantuPajak dibuang bila `!pajak_tampil()`). Bila jenis terpilih tersembunyi, pakai Bku. |
| ui/ringkasan.rs:62-67 | Bila `!pajak_tampil()`, ubin ke-4 jadi "Transaksi bulan ini" (ikon RECEIPT, jumlah, `p.aksen`). |
| ui/anggaran.rs:36 | `"Susun anggaran tahunan ({istilah}), lalu pantau realisasinya."` |
| ui/akun.rs:133, :193 | Petunjuk `"mis. Bank Riau Kepri — {sumber_dana[0]}"`. Teks "…sesuaikan dengan kode rekening {istilah}." |
| docs/PANDUAN-INSTALASI.md | Tambahkan folder `cadangan-otomatis/` dan catatan kompatibilitas v0.2. |

### 5.7 Mengelola instansi

| Aksi | Putaran ini |
|---|---|
| Ganti nama | Isian Nama di Profil {Jenis}. Sidebar dan Ringkasan langsung ikut. |
| Ganti jenis | §5.4 (tanpa menghapus apa pun). |
| Cadangkan / pulihkan | §5.5 (satu instansi, dengan pengaman). |
| Hapus dengan konfirmasi | **Opsional, item 9.** Tombol merah "Mulai dari Awal…" di kartu Data. Lembarnya menjelaskan: "Data ‘{nama}’ ({n} transaksi) dipindahkan ke folder Arsip, lalu My Digit dibuka kosong. Arsip dapat dibuka kembali lewat Pulihkan dari Cadangan." Ada isian "Ketik nama untuk melanjutkan" (bila nama kosong: ketik MULAI). Tombol "Batal" dan "Arsipkan & Mulai Baru" (merah). Implementasi: `drop(db)` → cek `-journal` → `rename` ke `arsip/my-digit-{slug}-{YYYYMMDD-HHMMSS}.sqlite` → `Db::buka(path)` (berkas baru) → penyiapan. Bila rename gagal, buka kembali berkas asli dan tampilkan galat. **Tanpa `remove_file`.** |
| Berpindah antarinstansi, tambah instansi baru | **Ditunda.** Rancangan berikutnya (lensa 2, disetujui sebagai arah): indeks `aplikasi.sqlite` (daftar, aktif, tema); `instansi/i<hex>.sqlite` per instansi; `my-digit.sqlite` didaftarkan di tempatnya; pemilih di atas sidebar; `ganti_instansi` memanggil `reset_ui()`; arsip ganti hapus; impor dari cadangan; logo di DB lebih dulu. Putaran ini sudah menyiapkan `instansi.id`, `VERSI_SKEMA`, `reset_ui()`, dan `cadangan-otomatis/`. |

## 6. Kasus tepi dan tes

### Kasus tepi

- **DB v0.2 yang dipakai** (ada transaksi atau nama): langsung masuk Ringkasan. Jenis ditebak, keluaran PDF/Excel identik, toast muncul sekali.
- **DB v0.2 yang masih bawaan**: layar penyiapan tampil. Bila memilih selain Sekolah, 14 kategori bawaan diganti. Ini aman karena tidak ada yang merujuknya dan salinan sebelum-v1 sudah ada.
- **Semua akun dihapus**: tidak ada pengisian ulang, tidak ada kategori dobel.
- **Pajak dimatikan tapi ada data pajak**: UI pajak tetap tampil. Perhitungan di laporan.rs dan pdf.rs tidak pernah disaring.
- **Ganti jenis saat nama templat sudah ada**: butir dilewati. Bila kode bentrok, dialokasikan 4.10 dan seterusnya; urutan numerik membuatnya tampil setelah 4.9.
- **Awalan berisi `%`, `_`, `-`, atau spasi**: ditolak di UI. `instansi()` jatuh ke bawaan bila nilai tersimpan tidak valid.
- **Kop**: induk 3 baris atau lebih dipotong jadi 2. Teks panjang dipotong dengan `potong`. Huruf non-Latin-1 jadi `?` di PDF dan sudah diperingatkan di UI.
- **Pulihkan**: berkas sama dengan berkas aktif ditolak. Cadangan versi 2 ditolak sebelum menyentuh apa pun. Cadangan v0 dimigrasi. Ada `-journal` → dibatalkan. Bila rename gagal di Windows, berkas asli dibuka kembali.
- **Setelah pulih atau ganti jenis**: id lama yang tertahan (`ui_anggaran.isian`, `ui_transaksi.hapus`/`form`, `ui_akun.hapus_*`) di-reset.
- **Cadangkan**: tujuan di dalam folder data ditolak. Bila `VACUUM` gagal, cadangan lama di tujuan tetap utuh (lewat `.tmp`).
- **Batasan yang diketahui**:
  - logo tetap berkas, jadi cadangan yang dipulihkan di laptop lain kehilangan logo;
  - arsip "Mulai dari Awal" bisa merujuk `logo-lembaga.*` yang sudah ditimpa;
  - keduanya baru tuntas dengan logo di DB.

### Tes baru dan yang diubah

Tes memakai folder `std::env::temp_dir()/mydigit-uji-<nama>-<nanos>` dan tidak bergantung pada env.

| Tes | Isi |
|---|---|
| `migrasi_db_lama_tanpa_kehilangan` | Fixture v0 dibuat dengan SKEMA dan SQL mentah: 14 kategori, profil SMPN 23, 3 transaksi, 1 anggaran, `tema`. `Db::buka` → jumlah baris semua tabel sama, `profil()` sama dengan sebelumnya, `user_version=1`, `jenis=sekolah`, `siap=1`, satu salinan di `cadangan-otomatis/` dengan jumlah transaksi yang sama. |
| `migrasi_idempoten` | Dibuka dua kali → tetap satu salinan, tidak ada baris dobel. |
| `migrasi_batal_bila_salinan_gagal` | `cadangan-otomatis` dibuat sebagai berkas (bukan folder) → Err, `user_version` tetap 0, tidak ada kunci baru. |
| `tolak_versi_lebih_baru` | `user_version=2` → Err, tidak ada salinan, berkas tidak berubah. |
| `db_baru_tanpa_kategori` | 2 akun, 0 kategori, `perlu_penyiapan()`. Setelah `selesaikan_penyiapan(Organisasi)`, kategori == templat dan siap. |
| `db_v02_bawaan_ikut_penyiapan` | Hanya data bawaan → perlu penyiapan. Setelah `selesaikan(Lembaga)`, kategori == templat Lembaga. Varian dengan 1 transaksi → tidak dianggap bawaan, `siap=1`. |
| `hapus_semua_akun_tidak_mengisi_ulang` | Regresi pemicu lama. |
| `ganti_jenis_tidak_menghapus` | Sekolah dengan transaksi, lalu `terapkan_jenis(Organisasi)` → semua kategori lama tetap, kategori baru tanpa kode ganda. Panggilan kedua menambah 0. |
| `urutan_kode_numerik` | 4.1, 4.10, 4.2, 5.2.1, 5.2 → 4.1, 4.2, 4.10, 5.2, 5.2.1. |
| `templat_valid` | Tiap templat: kode unik, x.1–x.9, prefiks 4 ⇔ Masuk dan 5 ⇔ Keluar, maksimal 9 per arus. `KATEGORI_SEKOLAH` == salinan `KATEGORI_AWAL` lama di tes. |
| `nomor_bukti_awalan_kustom` | Awalan "KM" → "KM-2026-0001". Tes lama "BKM-2026-0003" dan "BKK-2026-0001" tetap lulus lewat `awalan_bukti()`. |
| `profil_baru_tersimpan` | Field baru bolak-balik tersimpan. Kunci tidak ada → "Bendahara"/"NIP". Kunci ada tapi kosong tetap kosong. |
| `cadangkan_dan_pulihkan_aman` | Tujuan di folder data ditolak. Pulih dari berkas yang sama ditolak. Salinan sebelum-pulih ada. `periksa_cadangan` membaca nama dan jumlah transaksi. |
| pdf `kop_tanpa_isian_baru_identik` | `tinggi_kop(default) == 56`; 2 baris induk == 80. |
| pdf `ttd_label_nomor` | Label "NIY" → memuat "NIY. 123". Label kosong → nomor tanpa "NIP.". Bawaan → "NIP. ". `jabatan_bendahara` khusus muncul di laporan, kwitansi, dan berita acara. `setiap_halaman_memuat_footer` tetap lulus. |
| excel | Tes "PK" yang ada ditambah varian dengan induk 2 baris. Isi Excel dicek manual lewat `contoh_excel`. |
| Diubah | `data_awal_dan_nomor_bukti` dan `anggaran_upsert` memakai `Db::memori_siap(Sekolah)` (tanpa kategori, FK `anggaran.kategori_id` akan gagal). `contoh()` dan `buat_data_demo` (db.rs:532, :820) memakai `nomor_bukti_berikutnya(jenis.awalan_bukti(), ..)`, dan `buat_data_demo` memanggil penyiapan Sekolah lebih dulu. `profil_tersimpan` tidak berubah. |

**Uji manual sebelum rilis**
- Windows (Segoe UI) dan macOS (SF Pro), mode terang dan gelap, jendela 860×560.
- Layar penyiapan untuk keempat jenis.
- Salinan `my-digit.sqlite` asli dari SMP Negeri 23 Dumai: PDF BKU, kwitansi, dan berita acara dibandingkan dengan v0.2.
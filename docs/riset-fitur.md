# Riset Fitur — My Digit

Catatan riset kebutuhan aplikasi bendahara keuangan lembaga (sekolah, yayasan,
masjid, organisasi), dan bagaimana tiap kebutuhan diwujudkan di My Digit.

## Temuan

1. **Buku Kas Umum (BKU) adalah inti pembukuan.** Pada pedoman dana BOS, BKU
   mencatat semua transaksi dengan pihak luar dan dicatat segera saat terjadi.
   Kolom yang lazim: tanggal, kode, nomor bukti, uraian, penerimaan,
   pengeluaran, saldo.
2. **Setiap transaksi BKU juga masuk buku pembantu:** Buku Pembantu Kas
   (tunai), Buku Pembantu Bank, dan Buku Pembantu Pajak (PPN, PPh 21, PPh 22,
   PPh 23).
3. **Pajak dicatat dua kali.** Pungutan pajak saat membayar belanja dicatat
   sebagai penerimaan, lalu disetor sebagai pengeluaran.
4. **Wajib cetak dan tanda tangan.** Bila pembukuan memakai komputer, BKU dan
   buku pembantu dicetak minimal sebulan sekali dan ditandatangani bendahara
   dan kepala lembaga.
5. **Anggaran dulu, lalu realisasi.** Lembaga menyusun anggaran (mis. RKAS),
   lalu kepala dan bendahara memantau realisasinya.
6. **Setiap transaksi butuh bukti** berupa nota atau kwitansi.
7. **Pemeriksaan kas (cash opname)** membandingkan uang fisik dengan saldo
   buku, lalu hasilnya dituangkan dalam berita acara yang ditandatangani
   bendahara dan pemeriksa.
8. **Masalah umum di lapangan:** banyak lembaga masih memakai Excel yang
   terpisah-pisah, sehingga laporan bulanan lambat dibuat.

## Fitur My Digit

| Kebutuhan                 | Fitur                                                                 |
| ------------------------- | --------------------------------------------------------------------- |
| Ringkasan cepat           | **Ringkasan**: saldo total, per akun, arus 6 bulan, transaksi terbaru |
| Pencatatan (temuan 1, 6)  | **Transaksi**: masuk, keluar, transfer antar akun; nomor bukti otomatis (BKM/BKK/TRF) |
| Pajak (temuan 3)          | Pungutan PPN/PPh 21/22/23 pada pengeluaran, dan transaksi setoran pajak |
| Buku (temuan 1, 2)        | **Laporan**: BKU, Buku Pembantu Kas, Bank, dan Pajak                   |
| Anggaran (temuan 5)       | **Anggaran** per kategori per tahun, dan laporan Realisasi Anggaran    |
| Rekap                     | Laporan Rekap per Kategori                                            |
| Cetak & TTD (temuan 4)    | Ekspor PDF dengan kop lembaga, blok tanda tangan, dan footer My Digit; ekspor CSV untuk Excel |
| Bukti (temuan 6)          | Cetak **kwitansi** per transaksi, lengkap dengan terbilang            |
| Cash opname (temuan 7)    | **Pemeriksaan Kas**: hitung pecahan uang, selisih, cetak berita acara |
| Data master               | **Akun Kas & Bank** (saldo awal, no. rekening), **Kategori** berkode   |
| Identitas lembaga         | **Profil Lembaga**: kop, logo lembaga, kepala & bendahara beserta NIP  |
| Keamanan data (temuan 8)  | **Pengaturan**: cadangkan dan pulihkan database, pilihan tema          |

## Rencana berikutnya

- Kunci periode (tutup buku bulanan) agar data lama tidak berubah.
- Lampiran foto nota per transaksi.
- Multi pengguna dengan peran (bendahara, kepala, pemeriksa).
- Impor data dari Excel/CSV.

## Sumber

- Lampiran Perwal 73/2016 (format BKU & buku pembantu BOS): https://peraturan.bpk.go.id/Download/131220/PERWAL732016_lampiran.pdf
- Perbup Kubu Raya 10/2011 (format Buku Pembantu Kas & Pajak): https://pasal.id/peraturan/perbup/perbup-kab-kubu-raya-10-2011
- Repositori UIN SATU, aplikasi keuangan sekolah: https://repo.uinsatu.ac.id/36374/
- Prosedur cash opname (Mekari Jurnal): https://www.jurnal.id/id/blog/prosedur-cash-opname-dan-contoh/

# SipilCAD

Aplikasi CAD untuk Arsitektur dan Teknik Sipil yang berjalan di peramban, bagian dari
[SipilStock](https://sipilstock.com). SipilCAD adalah **turunan [Open CAD Studio](https://github.com/HakanSeven12/OpenCADStudio)**
(HakanSeven12 dan kontributor) dan dilisensikan **GPL-3.0**, sama seperti induknya.

| | |
|---|---|
| Berdasarkan | Open CAD Studio, GPL-3.0 (hak cipta penulis dan kontributor aslinya) |
| Lisensi | GPL-3.0, lihat [LICENSE](LICENSE) |
| Perubahan | [MODIFICATIONS.md](MODIFICATIONS.md): daftar yang dihitung otomatis terhadap upstream murni |
| Cara build | [BUILD.md](BUILD.md) |

## Tentang repositori ini

Repositori ini berisi **kode sumber lengkap** dari setiap versi SipilCAD yang tayang. Setiap build web
menghasilkan satu commit (snapshot kode sumber persis seperti yang dibangun) dan satu tag `build-<upstream>-<kit>`.
`SOURCE.txt` yang disajikan di samping aplikasi menaut ke tag build yang sedang berjalan.

Kode dihasilkan otomatis: tree upstream pada commit tertentu, ditambah serangkaian patch rebrand yang kecil.
Karena itu kode di sini tidak diedit langsung; perubahan upstream masuk lewat build berikutnya.

## Perubahan dari upstream

Ringkasnya: nama dan logo SipilCAD, tautan ke proyek upstream disembunyikan, dan data upstream
dikosongkan. Semua titik ubah di kode Rust bertanda `// SIPIL:` (`git grep "SIPIL:"`), dan tambahan kita
dikumpulkan di `src/sipil.rs`. Daftar lengkapnya ada di [MODIFICATIONS.md](MODIFICATIONS.md).

## Panggilan keluar ke pihak ketiga

Tombol, panel, dan perintah yang mengarah ke proyek upstream (situs, Patreon, GitHub, Reddit, open-aec,
YouTube) disembunyikan atau diblokir lewat `HIDE_UPSTREAM_LINKS` dan `is_blocked_url` di `src/sipil.rs`.
`supporters.json`, `videos.json`, dan `discussions.json` berisi daftar kosong, sehingga aplikasi tidak
memuat data atau gambar dari layanan pihak ketiga saat dibuka. Host yang sudah ditinjau tercatat di
`tests/upstream_links.rs` (`KNOWN_HOSTS`); uji itu gagal bila kode sumber memuat host atau titik pemanggil
`open_url` yang baru.

## Pengembangan lokal

Desktop (butuh Rust stable):

```bash
cargo run --release
```

Web (lihat [BUILD.md](BUILD.md) untuk resep yang dipakai build resmi):

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
bash scripts/sipil-tools.sh   # wasm-bindgen CLI pada versi yang sama dengan Cargo.lock
trunk serve                   # dev server, path /app/ (lihat Trunk.toml)
```

## Lisensi komponen pihak ketiga

Daftar pustaka Rust yang dikompilasi ke wasm beserta teks lisensinya dibuat otomatis oleh `cargo-about` dan
disajikan sebagai `THIRD_PARTY_LICENSES.txt` di samping aplikasi. Font yang dibundel (Noto, SIL OFL 1.1)
dijelaskan di `SOURCE.txt`.

## Lisensi

GNU General Public License v3.0. Lihat [LICENSE](LICENSE).

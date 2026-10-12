# Cara build

Dokumen ini adalah resep build web SipilCAD. Perintah di bawah dijalankan dari akar repositori ini, pada
commit/tag build yang ingin direproduksi. Hasil build resmi dibuat persis dengan urutan ini oleh pipeline
otomatis.

## Alat

| Alat | Versi |
|---|---|
| Rust | stable, dengan target `wasm32-unknown-unknown` |
| Trunk | 0.21.x (build resmi: 0.21.14) |
| wasm-bindgen CLI | **sama dengan crate `wasm-bindgen` di `Cargo.lock`**: `bash scripts/sipil-tools.sh` |
| Python 3, curl, tar | untuk skrip pasca-build dan lisensi |

## Langkah

```bash
# 1. wasm-bindgen CLI (dipakai hook post_build di Trunk.toml)
bash scripts/sipil-tools.sh

# 2. build web; --public-url ./ membuat semua URL relatif, sehingga hasilnya bisa dilayani dari folder mana pun
trunk build --locked --release --public-url ./ --dist dist/sipilcad --html-output index.html web-app.html

# 3. pasca-build: bersihkan index.html, kosongkan data upstream, tulis LICENSE.txt dan SOURCE.txt
python3 scripts/sipil-postbuild.py dist/sipilcad --tag <tag build> --commit <sha commit> --upstream-sha <sha upstream>

# 4. THIRD_PARTY_LICENSES.txt (cargo-about, diverifikasi SHA-256)
bash scripts/sipil-licenses.sh dist/sipilcad
```

`<sha upstream>` dan `<tag build>` tercantum di pesan commit dan di `MODIFICATIONS.md`. Folder `dist/sipilcad`
adalah isi yang dilayani di `https://sipilstock.com/sipilcad/`.

## Dari mana kode di repositori ini berasal

Setiap build: checkout Open CAD Studio pada commit yang dikunci, patch rebrand diterapkan, hasilnya di-commit
ke repositori ini, lalu dibangun dari commit itu. Karena build dijalankan di tree yang sudah di-commit,
"Revision" di dialog About sama dengan commit publik di sini. Patch itu sendiri tidak perlu untuk
membangun ulang: seluruh hasilnya sudah ada di commit.

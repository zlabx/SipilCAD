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

# 3. pasca-build: bersihkan index.html, kosongkan data upstream, tulis LICENSE.txt dan SOURCE.txt, cap CACHE_VERSION di sw.js
python3 scripts/sipil-postbuild.py dist/sipilcad --tag <tag build> --commit <sha commit> --upstream-sha <sha upstream> --upstream-ref <tag/ref upstream>

# 4. THIRD_PARTY_LICENSES.txt (cargo-about, diverifikasi SHA-256)
bash scripts/sipil-licenses.sh dist/sipilcad
```

`<sha upstream>` dan `<tag build>` tercantum di pesan commit dan di `MODIFICATIONS.md`. Folder `dist/sipilcad`
adalah isi yang dilayani di `https://sipilstock.com/sipilcad/`.

## Build desktop (Windows)

Exe dan installer MSI dibangun dari commit yang sama (tag `build-...`), bukan dari tree lain. Dijalankan di `windows-latest`
(Visual Studio Build Tools, WiX 3, ImageMagick sudah ada di runner); dari akar repositori ini:

```bash
# 1. ikon (ImageMagick): aplikasi dari logo SipilCAD, berkas DWG/DXF dari SVG mimetype
magick assets/logo.svg -define icon:auto-resize=16,24,32,48,64,128,256 packaging/windows/AppIcon.ico
magick -background none -density 1536 assets/mimetypes/image-vnd.dwg.svg -define icon:auto-resize=16,24,32,48,64,128,256 packaging/windows/dwg.ico
magick -background none -density 1536 assets/mimetypes/image-vnd.dxf.svg -define icon:auto-resize=16,24,32,48,64,128,256 packaging/windows/dxf.ico

# 2. build (Rust stable, target x86_64-pc-windows-msvc)
cargo build --locked --release
cargo build --locked --release -p dwg-thumbnailer-win

# 3. SipilCAD.exe = target/release/OpenCADStudio.exe diganti nama (nama target Cargo tetap OpenCADStudio karena build web memakainya)
# 4. dokumen pendamping: LICENSE.txt (dari LICENSE), SOURCE.txt, OFL-PTSans.txt, THIRD_PARTY_LICENSES.txt
#    (cargo-about 0.9.2 versi Windows, SHA-256 dicek; about-windows.toml disalin menjadi about.toml, target x86_64-pc-windows-msvc)
# 5. MSI: candle -arch x64 -dVersion=<tahun-2000>.<nomor>.0 -dSource=SipilCAD.exe -dIcon=AppIcon.ico -dLicense=License.rtf -dDocs=<folder dokumen> main.wxs ui.wxs
#         light main.wixobj ui.wixobj -ext WixUIExtension -cultures:en-us -loc strings.en-US.wxl
```

Versi MSI dari `APP_VERSION` di `src/sipil.rs` (mis. `2027.0` menjadi `27.0.0`; MSI membatasi angka pertama sampai 255).
Urutan resmi, uji pasang/copot MSI, dan penerbitan Release ada di workflow `sipilcad-desktop.yml` pada repositori `zlabx/Apps`.
Yang tidak ada di v1: tanda tangan kode, plugin, build macOS/Linux.

## Dari mana kode di repositori ini berasal

Setiap build: checkout Open CAD Studio pada commit yang dikunci, patch rebrand diterapkan, hasilnya di-commit
ke repositori ini, lalu dibangun dari commit itu. Karena build dijalankan di tree yang sudah di-commit,
"Revision" di dialog About sama dengan commit publik di sini. Patch itu sendiri tidak perlu untuk
membangun ulang: seluruh hasilnya sudah ada di commit.

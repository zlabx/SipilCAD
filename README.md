# SipilCAD

**Gambar teknik 2D dan pemodelan 3D dengan format DWG/DXF native, untuk desktop dan web** — bagian dari ekosistem [SipilStock](https://sipilstock.com).

SipilCAD adalah turunan (derivative work) dari **[Open CAD Studio](https://github.com/HakanSeven12/OpenCADStudio)** karya **HakanSeven12**, dan dirilis di bawah lisensi yang sama, **GPL-3.0**.

> ## Atribusi
>
> | | |
> |---|---|
> | Proyek asli | **Open CAD Studio** — <https://github.com/HakanSeven12/OpenCADStudio> |
> | Penulis asli | HakanSeven12 dan kontributor Open CAD Studio |
> | Aplikasi asli | <https://www.opencadstudio.com> |
> | Lisensi | GPL-3.0 (lihat [`LICENSE`](LICENSE)) |
>
> Seluruh riwayat commit Open CAD Studio dipertahankan di repo ini. Semua kredit atas editor, kernel, dan antarmuka dasar
> menjadi milik penulis asli. Perubahan khusus SipilCAD dicatat di bagian [Perubahan dari upstream](#perubahan-dari-upstream).

---

## Status

- Basis saat ini: **Open CAD Studio 2026.39.0**, upstream commit [`07f25f4b`](https://github.com/HakanSeven12/OpenCADStudio/commit/07f25f4b) (29 Sep 2026).
- Perubahan dari upstream sebatas konfigurasi CI dan rilis (lihat [Perubahan dari upstream](#perubahan-dari-upstream)): dua workflow upstream dinonaktifkan, ditambah workflow ukur dan rilis web. Kode aplikasi, branding, dan antarmuka **belum diubah**.
- **Tayang di SipilStock** (<https://sipilstock.com/sipilcad/>) sejak rilis `web-v0.1.0` (commit `4dc5a4b9`). Teruji di Chrome dan Edge desktop: buka DWG dan DXF, gambar, simpan, dan 3D. `.wasm` utama (53,14 MiB, di atas batas 25 MiB per berkas Cloudflare Pages) dilayani dari R2 (`sipilcad-cdn.sipilstock.com`) dan terkompresi saat diunduh (sekitar 17 MB). Lihat [Keterbatasan yang diketahui](#keterbatasan-yang-diketahui).
- Repo ini dulunya bernama **SipilXCAD** dan sudah di-rename menjadi **SipilCAD**; riwayat commit tetap utuh. Repo ini menggantikan SipilCAD lama, sebuah viewer berbasis TypeScript (Vite) yang repo-nya kini sudah dihapus.

## Keterbatasan yang diketahui

Versi pertama hanya menargetkan "jalan"; yang berikut diketahui dan sengaja belum ditangani.

- **Firefox:** membuka DWG/DXF gagal dengan galat `Web parser worker: initialize worker: out of memory`. Galat yang sama muncul di aplikasi web resmi Open CAD Studio pada Firefox yang sama, jadi bukan akibat build ini. Baru teruji di satu mesin Windows.
- **Ponsel:** masalah WebGL pernah dilaporkan; belum diselidiki. Praktis hanya Chrome dan Edge desktop yang didukung.
- **Muat pertama berat:** sekitar 17 MB terkompresi (sekitar 12 detik pada koneksi 11 Mbps). Setelahnya dari cache browser, karena `.wasm` dilayani dengan `Cache-Control: public, max-age=31536000, immutable`.
- **Galat 404 di Console** untuk `supporters.json` dan `video_thumbs/*.jpg`: berkas itu dibuat oleh pipeline deploy upstream dan tidak ada di paket ini. Aplikasi tetap jalan (daftar pendukung kosong, thumbnail video memakai cadangan). Kosmetik.
- **Peringatan `integrity` di Console** dari Chrome/Edge untuk preload `.wasm`: atribut itu diabaikan pada preload jenis `fetch`. Tidak berdampak; `.wasm` hanya diunduh sekali.

## Fitur (dari Open CAD Studio)

- Membuka, mengedit, memulihkan, dan menyimpan **DWG dan DXF** secara native, target simpan versi R14 sampai 2018. Impor OBJ dan LandXML (titik survei), ekspor STL, STEP AP203 (mesh), PDF (desktop), dan CSV.
- Drafting 2D: garis, polyline, kurva, spline, hatch, object snap, tracking, layer, blok, dan xref.
- Dokumentasi: teks, dimensi, leader, tabel, model space, paper space, viewport, dan plot style (CTB/STB).
- Pemodelan 3D berbasis kernel: primitif solid, extrude, revolve, sweep, loft, dan operasi Boolean.
- Render GPU (`wgpu`) dan editor yang sama untuk desktop dan browser.
- Khusus desktop: plugin proses terpisah, skrip perintah, konversi headless (`--export`), server otomasi (`--serve`), dan endpoint MCP (`--mcp`).
- 21 bahasa antarmuka. **Bahasa Indonesia belum ada.**

Stack: Rust, [iced](https://github.com/iced-rs/iced), `wgpu`. Versi web dibuild ke WebAssembly dengan [Trunk](https://trunkrs.dev). Kernel, codec, dan grafnya ada di repo terpisah milik upstream (`opencadkernel`, `opencadcodec`, `opencadgraph`, berlisensi MPL-2.0) dan diambil lewat Cargo dari GitHub pada commit tertentu.

## Pengembangan lokal

Butuh Git dan Rust stable. Di Ubuntu/Debian, pustaka native yang dibutuhkan (menurut README upstream):

```bash
sudo apt install libgl1-mesa-dev libx11-dev libxcursor-dev libxi-dev \
  libxrandr-dev libxkbcommon-dev libwayland-dev libfontconfig1-dev \
  libfreetype6-dev
```

Desktop:

```bash
git clone https://github.com/zlabx/SipilCAD.git
cd SipilCAD
cargo build --release --bin OpenCADStudio   # hasil: target/release/OpenCADStudio
```

Nama binary masih `OpenCADStudio` (belum di-rebrand).

Web:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk wasm-bindgen-cli   # versi wasm-bindgen harus sama dengan Cargo.lock
trunk serve                            # dev server, path /app/ (lihat Trunk.toml)
```

Build web produksi mengikuti `.github/workflows/pages.yml` upstream, dengan `--public-url` diganti ke subfolder tujuan:

```bash
trunk build --locked --release --public-url /sipilcad/ --dist dist/sipilcad --html-output index.html web-app.html
```

Catatan dari file upstream:

- Hook `post_build` di `Trunk.toml` menjalankan `scripts/build-web-worker.sh` (membuild `ocs_web_worker`), jadi `wasm-bindgen` harus ada di PATH.
- `build.rs` menghitung jarak commit dari tag rilis. Build dari clone dangkal (shallow) akan berlabel `+g<hash>`; gunakan clone penuh (`fetch-depth: 0`) bila label versi penting.
- Workflow upstream juga merakit landing page dan membuat `supporters.json`/`discussions.json`. Bagian itu **tidak** dipakai di sini.

## Struktur branch dan sinkron dengan upstream

| Branch | Fungsi |
|---|---|
| `upstream` | **Cermin murni** `HakanSeven12/OpenCADStudio` (`main`). Jangan commit apa pun di sini. |
| `main` | Versi SipilCAD. Ini yang dipakai untuk build/deploy. |

Remote `upstream` menunjuk ke repo Open CAD Studio. Cara mengambil update:

```bash
git remote add upstream https://github.com/HakanSeven12/OpenCADStudio.git   # sekali saja
git fetch upstream

git checkout upstream
git merge --ff-only upstream/main     # cermin harus selalu fast-forward
git push origin upstream

git checkout main
git merge upstream                    # tinjau hasilnya, jalankan cargo test, lalu push
```

Kalau `README.md` konflik saat merge (upstream ikut mengubahnya), pertahankan versi SipilCAD:
`git checkout --ours README.md && git add README.md`.

### Aturan supaya update tetap murah

Upstream sangat aktif (rilis mingguan, ribuan commit), jadi selisih yang besar akan cepat mahal.

- Batasi perubahan di `main`: branding, konfigurasi, dan tautan ke SipilStock. Hindari refactor.
- Jangan menghapus file upstream yang tidak dipakai (mis. `docs/`, `packaging/`, `plugins/`); menghapusnya memicu konflik merge. Cukup jangan dipakai.
- Tambahan besar (mis. terjemahan Indonesia di `locales/*/opencadstudio.ftl`) sebaiknya dikirim juga sebagai PR ke upstream agar tidak perlu dirawat sendiri.
- Untuk deploy, kunci ke tag atau commit `main` tertentu, jangan ke `HEAD`, supaya merge upstream yang bermasalah tidak langsung naik ke produksi.

## Perubahan dari upstream

Catat setiap perubahan di sini agar mudah ditinjau saat merge.

| File | Perubahan | Alasan |
|---|---|---|
| `.github/workflows/weekly-release.yml` | Pemicu `schedule` (cron Minggu 12:00 UTC) di-comment; `workflow_dispatch` manual tetap ada | Di repo ini jadwal itu akan mempublikasikan rilis setiap minggu dan memicu build Windows/Linux/macOS serta Pages, dengan secret dan identitas upstream yang tidak kita punya |
| `.github/workflows/issue-welcome.yml` | Job diberi `if: github.repository == 'HakanSeven12/OpenCADStudio'` | Komentar otomatisnya mengarahkan pengguna ke Patreon upstream dan seolah ditulis pemilik repo |
| `.github/workflows/wasm-size.yml` (baru) | Workflow ukur saja: `trunk build --release` lalu melaporkan ukuran berkas (asli, gzip, brotli) dan baris pemuat wasm/js di `index.html` (untuk merancang hosting R2) di ringkasan job dan anotasi, lalu mengunggah hasil build sebagai artifact (retensi 1 hari) untuk uji lokal di browser. Tidak ada deploy, rilis, atau secret | Menentukan apakah hasil build web muat di batas 25 MiB per berkas Cloudflare Pages |
| `.github/workflows/release-web.yml` (baru) | Build web, unggah `.wasm` utama ke R2, ganti URL-nya di `index.html`, paketkan sisanya, dan terbitkan GitHub Release. Mode rilis (tag `web-v*`) dan dry-run | `.wasm` melebihi batas 25 MiB Cloudflare Pages dan build Rust terlalu lama untuk batas build Pages; jadi dibangun dan dirilis di sini, lalu SipilStock hanya mengunduh paketnya |
| `.github/scripts/release_web.py` (baru) | Logika rilis: patch `index.html`, unggah ke R2, verifikasi dari URL publik, paket, pembersihan versi lama, dan pengosongan `_dryrun/` | Dipanggil oleh `release-web.yml`; hanya memproses hasil build |
| `README.md` | Diganti | Atribusi dan panduan SipilCAD |

Workflow `ci.yml` dan `web-check.yml` sengaja dibiarkan: keduanya berjalan pada push ke `main` dan pull request, tidak memakai secret, dan berguna sebagai pemeriksaan build. Hasilnya belum pernah dilihat di repo ini.

## Rilis build web

Dikerjakan oleh `.github/workflows/release-web.yml`. Prasyarat: bucket R2 publik dengan domain kustom dan CORS `GET`/`HEAD`; secret `R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `R2_BUCKET`; variabel `R2_PUBLIC_URL`.

- **Dry-run:** berjalan otomatis saat `release-web.yml` atau `release_web.py` diubah di `main`, atau manual lewat tab Actions. `.wasm` diunggah ke `_dryrun/` di R2 (isinya dikosongkan dulu tiap dry-run, karena nama berkas memuat hash yang berbeda tiap build dan tanpa itu berkas menumpuk), diverifikasi dari URL publik, dan paketnya hanya jadi artifact selama 1 hari. Tidak ada GitHub Release.
- **Rilis:** push tag `web-v*` (mis. `git tag web-v0.1.0 && git push origin web-v0.1.0`). `.wasm` diunggah ke `sipilcad/<tag>/` di R2, `index.html` di paket menunjuk ke sana, GitHub Release dibuat dengan `sipilcad-web-<tag>.tar.gz` dan checksum-nya, lalu R2 menyisakan 3 versi terbaru. Pembersihan memang dilakukan workflow ini, bukan aturan lifecycle berbasis umur, karena aturan itu bisa menghapus `.wasm` yang masih tayang.
- **Isi paket:** semua hasil `trunk build` kecuali `.wasm` utama, ditambah `LICENSE` dan `SOURCE.txt`. Tidak ada berkas di paket yang melebihi 25 MiB (diperiksa otomatis).

### Cara naik versi

1. Sinkron dengan upstream atau ubah, lalu pastikan CI hijau (`Tests`, `Web build check`).
2. Push tag baru `web-vX.Y.Z`; workflow `Release web` merilis. Periksa Summary-nya (ukuran, SHA-256, kompresi wasm dari URL publik).
3. Uji di Preview Cloudflare: di proyek Pages, isi `SIPILCAD_REF=web-vX.Y.Z` pada environment **Preview**, lalu push satu branch uji di `zlabx/zlabx`.
4. Pin: ubah `DEFAULT_REF` di `scripts/build-sipilcad.sh` di `zlabx/zlabx`, push ke `main` (deploy produksi otomatis), lalu hapus variabel Preview.
5. Rollback: isi `SIPILCAD_REF=web-v<lama>` di Production atau kembalikan `DEFAULT_REF`; `SIPILCAD_REF=none` mematikan SipilCAD sementara. Karena R2 hanya menyisakan 3 versi terbaru, rollback aman untuk dua versi ke belakang.

### Belum dirilis

Sudah di `main`, tetapi belum ada di rilis yang tayang (`web-v0.1.0`); baru berlaku di rilis berikutnya:

- Paket memuat `LICENSE.txt` (salinan `LICENSE` yang tampil di browser, karena berkas tanpa ekstensi terunduh).
- `SOURCE.txt` menyebut komponen pihak ketiga yang ikut tersaji (font OFL dan font LFF).

## Checklist sebelum deploy

Awalnya hasil audit membaca kode dan workflow upstream; diperbarui setelah SipilCAD tayang. Yang sudah dikerjakan atau terbukti ditandai centang. Sisanya sengaja ditunda, karena versi pertama hanya perlu jalan.

- [x] **Workflow otomatis upstream** (rilis mingguan terjadwal, komentar otomatis di issue) dinonaktifkan; lihat tabel di atas.
- [x] **Ukuran build web.** `.wasm` utama 53,14 MiB melebihi batas 25 MiB per berkas Cloudflare Pages, jadi dilayani dari R2 dengan domain kustom (lihat Rilis build web). Berkas lain di paket semuanya di bawah batas (terbesar sekitar 10 MiB, font). Mengecilkan `.wasm` (`opt-level`, LTO, `wasm-opt`) tidak dicoba.
- [x] **Toolchain build.** Build Rust memakan 11 sampai 18 menit (batas build Pages 20 menit), jadi dibangun di GitHub Actions; build Pages hanya mengunduh paket rilis.
- [x] **Header cross-origin isolation: tidak diperlukan.** Terbukti: aplikasi berjalan di Chrome dan Edge produksi tanpa `_headers` dan tanpa COOP/COEP. `src/par.rs` menunjukkan WASM berjalan tanpa thread.
- [ ] **Panggilan keluar ke pihak ketiga saat runtime.** Dari kode `src/`: feed dan halaman Discussions upstream di GitHub (`src/discussions.rs`), registry plugin dari `raw.githubusercontent.com/HakanSeven12/OpenCADStudio` serta rilis/README repo plugin lewat GitHub (`src/plugin/marketplace.rs`), thumbnail dan oEmbed YouTube untuk playlist upstream (`src/videos.rs`), dan `api.frankfurter.dev` serta Patreon (`src/patreon.rs`; kapan tepatnya dipanggil belum ditelusuri). Bila dipanggil dari browser pengguna, IP pengguna terkirim ke pihak-pihak itu. Teramati di tab Network produksi: thumbnail video `mqdefault.jpg` dimuat sebagai cadangan (kemungkinan dari YouTube). Putuskan mana yang dimatikan atau diarahkan ke SipilStock, dan perbarui Kebijakan Privasi. Saya tidak menemukan analitik atau telemetri pihak ketiga, tapi pencarian ini hanya mencakup `src/`, bukan dependensi.
- [ ] **Berkas web dari origin sendiri.** `supporters.json` dan `video_thumbs/*.jpg` tidak ada di paket (dibuat oleh pipeline deploy upstream), sehingga muncul 404 di Console; aplikasi tetap jalan. Bisa disenyapkan dengan menyertakan berkas kosong di paket rilis (belum dilakukan).
- [ ] **Branding dan tautan.** Judul halaman (`web-app.html`), logo, nama `OpenCADStudio` di ratusan berkas, serta tautan Patreon, open-aec.com, dan Reddit milik upstream di antarmuka. `site/CNAME` berisi `www.opencadstudio.com`; jangan dipakai. Tetap sertakan atribusi.
- [ ] **Halaman "Tentang/Lisensi"** di dalam aplikasi: sebut Open CAD Studio oleh HakanSeven12, lisensi GPL-3.0, dan tautan ke repo ini.
- [ ] **Dependensi git eksternal.** `Cargo.toml` mengambil `iced`, fork `iced_aw` (branch `agent/fix-iced-fonts`), dan tiga crate upstream langsung dari GitHub. Bila repo atau branch itu hilang atau di-force-push, build rusak. Pertimbangkan mirror ke akun zlabx atau `cargo vendor`.
- [ ] **`release.yml` dan `pages.yml` upstream** memakai secret milik upstream (Patreon, penandatanganan Windows/Azure, Snapcraft) dan repo/nama paket upstream. Jangan diaktifkan sebelum diadaptasi ke SipilCAD.
- [ ] **`.github/FUNDING.yml`** masih menunjuk ke sponsor upstream. Sengaja dibiarkan agar dukungan mengalir ke penulis asli; ganti bila tidak diinginkan.
- [x] **Lisensi font web dan font LFF** diperiksa: font web OFL 1.1 dengan `fonts/OFL.txt` di paket, font LFF public domain dan/atau GPL v2+ (lihat [Lisensi komponen pihak ketiga](#lisensi-komponen-pihak-ketiga)). Sisa: `ltypeshp.lff` tanpa baris lisensi.
- [ ] **Daftar lisensi crate Rust pihak ketiga** belum ada di paket (upstream juga tidak punya). Opsi: `cargo-about` di workflow rilis.
- [ ] **Bahasa Indonesia** (opsional): tambahkan locale baru dan kirim juga ke upstream.

## Integrasi dengan SipilStock

SipilStock ([`zlabx/zlabx`](https://github.com/zlabx/zlabx), privat) memasang SipilCAD saat deploy lewat `scripts/build-sipilcad.sh`: skrip mengunduh paket rilis `web-v*` dari repo ini (publik, tanpa token), memeriksa SHA-256, `LICENSE`, `SOURCE.txt`, dan ukuran berkas, memastikan `.wasm` di R2 terjangkau, lalu mengekstrak ke `sipilcad/` (di-gitignore). Versi yang tayang dipin lewat `DEFAULT_REF` di skrip itu (kini `web-v0.1.0`); env `SIPILCAD_REF` menimpanya (uji di Preview Cloudflare) dan `none` melewati SipilCAD. Build command Cloudflare Pages: `npm run generate:all && npm run build:sipilframe && npm run build:sipilcad`. Kartu SipilCAD di halaman utama, halaman Apps, dan dropdown Apps mengarah ke `/sipilcad/`.

## Lisensi komponen pihak ketiga

Diperiksa pada Tahap 1 (Oktober 2026). Ini catatan teknis, bukan nasihat hukum.

- **Font web** (`fonts/`, dimuat browser sesuai bahasa): 11 berkas subset **Noto Sans** (termasuk Noto Sans CJK dari Adobe) di bawah **SIL Open Font License 1.1**. Cara membuatnya ada di `web/fonts/generate.sh`. `fonts/OFL.txt` (pernyataan hak cipta dan teks lisensi) ikut di paket rilis yang tayang dan identik dengan upstream; metadata hak cipta dan lisensi juga tertanam di tiap berkas font. Nama font tetap "Noto Sans" dan "Noto Sans CJK", bukan nama yang dicadangkan ("Source").
- **Font garis LFF** (`assets/fonts/*.lff`, 27 berkas, 2,52 MiB, tertanam di `.wasm`): bawaan LibreCAD. Header berkas mencantumkan lisensi **public domain** (font Hershey) dan/atau **GPL v2 atau lebih baru**, yang kompatibel dengan GPL-3.0 aplikasi ini. Pengecualian: `ltypeshp.lff` tidak punya baris lisensi di headernya; sumber aslinya belum ditelusuri.
- **Crate Rust pihak ketiga:** upstream tidak menyertakan daftar lisensinya, dan paket web ini pun belum. Pilihan ke depan: membuat daftarnya otomatis (mis. dengan `cargo-about`) dan menyertakannya di paket.

## Lisensi

[GNU General Public License v3.0](LICENSE). Copyright pada kode Open CAD Studio dimiliki penulis aslinya; modifikasi SipilCAD dirilis di bawah lisensi yang sama. Kode sumber lengkap tersedia di repo ini untuk siapa pun yang menggunakan SipilCAD. Crate `opencadkernel`, `opencadcodec`, dan `opencadgraph` yang dipakai sebagai dependensi berlisensi MPL-2.0 di repo masing-masing.

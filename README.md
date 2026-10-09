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
- Perubahan dari upstream sebatas konfigurasi CI dan rilis (lihat [Perubahan dari upstream](#perubahan-dari-upstream)): dua workflow upstream dinonaktifkan, ditambah workflow ukur, rilis web, dan latihan jalur darurat dependensi (mirror). Kode aplikasi, branding, dan antarmuka **belum diubah**.
- **Tayang di SipilStock** (<https://sipilstock.com/sipilcad/>) sejak rilis `web-v0.1.0` (commit `4dc5a4b9`). Teruji di Chrome dan Edge desktop: buka DWG dan DXF, gambar, simpan, dan 3D. `.wasm` utama (53,14 MiB, di atas batas 25 MiB per berkas Cloudflare Pages) dilayani dari R2 (`sipilcad-cdn.sipilstock.com`) dan terkompresi saat diunduh (sekitar 17 MB). Lihat [Keterbatasan yang diketahui](#keterbatasan-yang-diketahui).
- Repo ini dulunya bernama **SipilXCAD** dan sudah di-rename menjadi **SipilCAD**; riwayat commit tetap utuh. Repo ini menggantikan SipilCAD lama, sebuah viewer berbasis TypeScript (Vite) yang repo-nya kini sudah dihapus.

## Keterbatasan yang diketahui

Versi pertama hanya menargetkan "jalan"; yang berikut diketahui dan sengaja belum ditangani.

- **Firefox:** membuka DWG/DXF gagal dengan galat `Web parser worker: initialize worker: out of memory`. Galat yang sama muncul di aplikasi web resmi Open CAD Studio pada Firefox yang sama, jadi bukan akibat build ini. Baru teruji di satu mesin Windows.
- **Ponsel:** masalah WebGL pernah dilaporkan; belum diselidiki. Praktis hanya Chrome dan Edge desktop yang didukung.
- **Muat pertama berat:** sekitar 17 MB terkompresi (sekitar 12 detik pada koneksi 11 Mbps). Setelahnya dari cache browser, karena `.wasm` dilayani dengan `Cache-Control: public, max-age=31536000, immutable`. Cache Rule di edge Cloudflare sengaja **tidak dipasang** (keputusan pemilik, Oktober 2026): respons wasm saat ini `cf-cache-status: DYNAMIC`, jadi tiap pengunjung baru dibaca langsung dari R2 (egress R2 gratis dan kuota baca gratis 10 juta per bulan jauh dari habis). Waktu muat 12 detik untuk 17 MB berarti sekitar 11 Mbps, kemungkinan besar dibatasi koneksi pengguna sehingga manfaatnya belum pasti. Bila muat pertama dikeluhkan: ukur dulu dengan `curl.exe -s -o NUL -H "Accept-Encoding: br" -w "ttfb=%{time_starttransfer}s total=%{time_total}s kecepatan=%{speed_download}B/s\n" <URL wasm>` beberapa kali; bila `ttfb` besar (di atas sekitar 1 detik), pasang aturan di Cloudflare: Caching, Cache Rules, hostname `sipilcad-cdn.sipilstock.com`, Eligible for cache, Edge TTL "Use cache-control header if present, bypass cache if not", Browser TTL respect origin; verifikasi `cf-cache-status: HIT` pada permintaan kedua. Catatan: versi lama yang dihapus pembersihan R2 bisa tetap tersaji dari cache edge sampai tergusur.
- **Galat 404 di Console** untuk `supporters.json` dan `video_thumbs/*.jpg`: berkas itu dibuat oleh pipeline deploy upstream dan tidak ada di paket ini. Aplikasi tetap jalan (daftar pendukung kosong, thumbnail video memakai cadangan). Kosmetik. **Sudah ditangani di `main`, belum dirilis** (lihat [Belum dirilis](#belum-dirilis)): paket rilis berikutnya memuat `supporters.json`, `videos.json`, dan `discussions.json` berisi daftar kosong, sehingga 404 itu hilang dan thumbnail tidak lagi diminta ke `i.ytimg.com`.
- **Peringatan `integrity` di Console** dari Chrome/Edge untuk preload `.wasm`: atribut itu diabaikan pada preload jenis `fetch`. Tidak berdampak; `.wasm` hanya diunduh sekali. **Sudah ditangani di `main`, belum dirilis:** langkah patch menghapus atribut itu dari preload `.wasm`.

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
| `.github/scripts/release_web.py` (baru) | Logika rilis: patch `index.html` (URL wasm ke R2; atribut `integrity` dihapus hanya dari `<link rel="preload">` untuk `.wasm`; meta `robots` `noindex` dijaga tetap ada), unggah ke R2, verifikasi dari URL publik, paket (`supporters.json`, `videos.json`, dan `discussions.json` ditulis `[]`; catatan perubahan di `SOURCE.txt`), pembersihan versi lama, dan pengosongan `_dryrun/` | Dipanggil oleh `release-web.yml`; hanya memproses hasil build. Pengosongan data mencegah panggilan ke pihak ketiga (thumbnail YouTube) dan 404 di Console. Penjaga noindex: bila upstream menghapus atau mengubah meta `robots`, patch memasang `noindex, follow` dan memberi `::warning` |
| `.github/scripts/test_release_web.py` dan `.github/scripts/fixtures/index.trunk.html` (baru) | Uji `unittest` untuk `patch` dan `package`; fixture adalah `index.html` asli keluaran trunk (rilis `web-v0.1.0`) sebelum patch. Jalankan manual: `python3 -m unittest discover -s .github/scripts -p 'test_*.py' -v` | CI belum menjalankannya (workflow tidak diubah); jalankan setiap kali `release_web.py` berubah |
| `.github/FUNDING.yml` (dihapus) | Berkas dari upstream yang menampilkan tombol Sponsor (GitHub Sponsors dan Patreon penulis asli) di halaman repo | Hanya memengaruhi halaman repo GitHub; aplikasi dan rilis tidak terpengaruh. Upstream menyentuhnya 2 kali sejak Maret 2026; konflik modify/delete bila berubah lagi |
| `.github/mirrors.json` (baru) | Pemetaan 7 sumber git (upstream, mirror, commit pin, daftar `patch`) | Dasar jalur darurat bila repo upstream hilang atau di-force-push |
| `.github/scripts/use_mirrors.py` (baru) | `check`, `apply`, dan `verify-lock` untuk mengalihkan dependensi git ke mirror (hanya dipakai di jalur darurat) | Build normal tetap memakai upstream |
| `.github/workflows/mirror-drill.yml` (baru) | Latihan jalur darurat di runner (tidak mengubah repo): alihkan ke mirror, `cargo fetch`, verifikasi lock, `cargo check` wasm32 `--locked` | Membuktikan mirror benar-benar bisa menggantikan upstream, dan menjaganya tetap terbukti |
| `README.md` | Diganti | Atribusi dan panduan SipilCAD |

Workflow `ci.yml` dan `web-check.yml` sengaja dibiarkan: keduanya berjalan pada push ke `main` dan pull request, tidak memakai secret, dan berguna sebagai pemeriksaan build. Hasilnya hijau (`Tests` dan `Web build check`) pada commit `ce90df6b`.

## Rilis build web

Dikerjakan oleh `.github/workflows/release-web.yml`. Prasyarat: bucket R2 publik dengan domain kustom dan CORS `GET`/`HEAD`; secret `R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `R2_BUCKET`; variabel `R2_PUBLIC_URL`.

- **Dry-run:** berjalan otomatis saat `release-web.yml` atau `release_web.py` diubah di `main`, atau manual lewat tab Actions. `.wasm` diunggah ke `_dryrun/` di R2 (isinya dikosongkan dulu tiap dry-run, karena nama berkas memuat hash yang berbeda tiap build dan tanpa itu berkas menumpuk), diverifikasi dari URL publik, dan paketnya hanya jadi artifact selama 1 hari. Tidak ada GitHub Release.
- **Rilis:** push tag `web-v*` (mis. `git tag web-v0.1.0 && git push origin web-v0.1.0`). `.wasm` diunggah ke `sipilcad/<tag>/` di R2, `index.html` di paket menunjuk ke sana, GitHub Release dibuat dengan `sipilcad-web-<tag>.tar.gz` dan checksum-nya, lalu R2 menyisakan 3 versi terbaru. Pembersihan memang dilakukan workflow ini, bukan aturan lifecycle berbasis umur, karena aturan itu bisa menghapus `.wasm` yang masih tayang.
- **Isi paket:** semua hasil `trunk build` kecuali `.wasm` utama, ditambah `LICENSE`, `LICENSE.txt`, dan `SOURCE.txt` (rilis sebelum `web-v0.2.0` belum memuat `LICENSE.txt`). Khusus `supporters.json`, `videos.json`, dan `discussions.json`: isinya selalu `[]` (data upstream tidak ikut). Tidak ada berkas di paket yang melebihi 25 MiB (diperiksa otomatis).

### Cara naik versi

1. Sinkron dengan upstream atau ubah, lalu pastikan CI hijau (`Tests`, `Web build check`).
2. Push tag baru `web-vX.Y.Z`; workflow `Release web` merilis. Periksa Summary-nya (ukuran, SHA-256, kompresi wasm dari URL publik).
3. Uji di Preview Cloudflare: di proyek Pages, isi `SIPILCAD_REF=web-vX.Y.Z` pada environment **Preview**, lalu push satu branch uji di `zlabx/zlabx`.
4. Pin: ubah `DEFAULT_REF` di `scripts/build-sipilcad.sh` di `zlabx/zlabx`, push ke `main` (deploy produksi otomatis), lalu hapus variabel Preview.
5. Rollback: isi `SIPILCAD_REF=web-v<lama>` di Production atau kembalikan `DEFAULT_REF`; `SIPILCAD_REF=none` mematikan SipilCAD sementara. Karena R2 hanya menyisakan 3 versi terbaru, rollback aman untuk dua versi ke belakang.

### Belum dirilis

Sudah di `main`, tetapi belum ada di rilis yang tayang (`web-v0.1.0`); baru berlaku di rilis berikutnya:

- Paket memuat `LICENSE.txt` (salinan `LICENSE` yang tampil di browser, karena berkas tanpa ekstensi terunduh). Untuk rilis yang belum memuatnya (termasuk `web-v0.1.0`), skrip build di `zlabx` membuat `LICENSE.txt` dari `LICENSE` saat memasang.
- `SOURCE.txt` menyebut komponen pihak ketiga yang ikut tersaji (font OFL dan font LFF), dan sejak Tahap 4 juga mencatat perubahan pada hasil build.
- Tahap 4: atribut `integrity` dihapus dari `<link rel="preload">` untuk `.wasm` di `index.html` (peringatan Console hilang; `modulepreload` JS tetap ber-integrity). `supporters.json`, `videos.json`, dan `discussions.json` di paket berisi `[]`: 404 `supporters.json` dan thumbnail hilang, tidak ada lagi permintaan ke `i.ytimg.com`, dan daftar diskusi upstream tidak tampil. Diuji dengan harness lokal dan end-to-end (`patch` lalu `package`) memakai isi paket `web-v0.1.0`; verifikasi di dry-run CI dan Preview menyusul.
- Tahap 5 (noindex, keputusan pemilik): `/sipilcad/` tetap tidak diindeks. `web-app.html` upstream sudah memuat `<meta name="robots" content="noindex, follow">`; langkah patch kini memastikannya ada di `index.html` hasil build (menambah atau memperbaiki bila upstream mengubahnya, dengan `::warning` di Summary dan `noindex` di metrics). Tidak ada perubahan di zlabx: `sitemap.xml`, `llms.txt`, `robots.txt`, dan `_headers` tidak disentuh, sehingga halaman ini tidak didaftarkan.

## Ketahanan build: mirror dependensi git

Build bergantung pada **7 sumber git** (`Cargo.lock`, 22 paket), selain ratusan crate dari crates.io. Dua di antaranya (`winit`, `cryoglyph`) tidak tampak di manifest kita karena diambil oleh manifest `iced`. Bila salah satu repo itu dihapus atau di-force-push, rilis berikutnya gagal; aplikasi yang sudah tayang tidak terpengaruh.

Tiap sumber punya **mirror**: fork di akun `zlabx`, publik, **diarsipkan (hanya-baca)**, dengan semua cabang dan lisensi aslinya. Fork dibuat lewat API GitHub (token tanpa izin `workflow` tidak bisa mendorong repo yang memuat `.github/workflows`).

| Upstream | Mirror | Commit pin | Lisensi |
|---|---|---|---|
| `iced-rs/iced` | [`zlabx/iced-mirror`](https://github.com/zlabx/iced-mirror) | `23604ff2` | MIT |
| `iced-rs/winit` (tidak langsung, via iced) | [`zlabx/winit-mirror`](https://github.com/zlabx/winit-mirror) | `05b8ff17` | Apache-2.0 |
| `iced-rs/cryoglyph` (tidak langsung, via iced) | [`zlabx/cryoglyph-mirror`](https://github.com/zlabx/cryoglyph-mirror) | `53ba3e87` | Apache-2.0 |
| `HakanSeven12/iced_aw` (cabang `agent/fix-iced-fonts`) | [`zlabx/iced_aw-mirror`](https://github.com/zlabx/iced_aw-mirror) | `9bff59df` | MIT |
| `HakanSeven12/opencadcodec` | [`zlabx/opencadcodec-mirror`](https://github.com/zlabx/opencadcodec-mirror) | `cdf22778` | MPL-2.0 |
| `HakanSeven12/opencadkernel` | [`zlabx/opencadkernel-mirror`](https://github.com/zlabx/opencadkernel-mirror) | `52a9ebeb` | MPL-2.0 |
| `HakanSeven12/opencadgraph` | [`zlabx/opencadgraph-mirror`](https://github.com/zlabx/opencadgraph-mirror) | `597c8b81` | MPL-2.0 |

Tiap commit pin terbukti bisa diambil lewat git dari URL mirror-nya. Enam mirror punya tag `pin-<sha8>` pada commit itu. `winit-mirror` tidak (API menolak membuat tag di fork dari fork), tetapi commit-nya ada di cabang `v0.30.x` milik mirror itu sendiri.

**Mirror tidak aktif.** `main` tetap memakai upstream. Jalur darurat memakai `.github/mirrors.json` dan `.github/scripts/use_mirrors.py` (hanya pustaka standar Python; di Python di bawah 3.11 butuh `pip install tomli`):

1. `python3 .github/scripts/use_mirrors.py check`: pemetaan menutupi semua sumber git di `Cargo.lock`, pin sama dengan commit di `Cargo.lock`, semua commit terambil dari mirror, dan semua rujukan tidak langsung tercakup daftar `patch`.
2. `cp Cargo.lock /tmp/Cargo.lock.orig`, lalu `python3 .github/scripts/use_mirrors.py apply`: URL git di `Cargo.toml` (root dan `crates/*`) dialihkan ke mirror dengan commit pasti, dan `[patch]` ditambahkan untuk dependensi tidak langsung.
3. `cargo fetch`: Cargo menulis ulang `Cargo.lock` sendiri (tidak diedit tangan).
4. `python3 .github/scripts/use_mirrors.py verify-lock --base /tmp/Cargo.lock.orig`: versi paket dan commit git harus identik dengan lock asli, tanpa salinan kembar, dan tidak ada lagi sumber upstream.
5. Bangun dan uji, lalu commit `Cargo.toml`, `crates/*/Cargo.toml`, dan `Cargo.lock` ke `main` (rilis memakai `--locked`) dan catat di tabel perubahan. Setelah upstream pulih, revert commit itu.

**Mirror drill** (`mirror-drill.yml`) menjalankan langkah 1 sampai 4 di runner lalu `cargo check --locked` untuk wasm32 (`OpenCADStudio` dan `ocs_web_worker`), tanpa mengubah repo. Berjalan manual atau saat berkas ini, skrip, atau `mirrors.json` berubah. Hasil pertama (7 Okt 2026): sukses, `Cargo.lock` berubah 22 baris ditambah dan 22 dihapus (22 paket git berganti sumber, tanpa pergeseran versi atau commit). Latihan itu menemukan dan memperbaiki dua kelemahan: salinan kembar `opencadcodec` karena `opencadkernel` merujuk upstream di dalam manifest-nya, dan `tomllib` yang tidak ada di Python 3.10.

**Yang belum dibuktikan:** drill hanya `cargo check` wasm32, bukan `trunk build` penuh dan bukan `cargo test`. Mirror melindungi dari upstream yang hilang atau di-force-push, bukan dari gangguan GitHub itu sendiri.

**Merawat setelah sinkron upstream:** bila merge upstream mengubah commit di `Cargo.lock`, `check` (dan drill) gagal dengan pesan pin tidak sama. Perbaikannya: pastikan mirror memuat commit baru (buka arsip, sinkronkan fork atau buat fork baru, beri tag, arsipkan lagi), lalu perbarui `pin` di `mirrors.json`. Bila upstream menambah dependensi git baru, `check` menyebut sumber atau rujukan yang belum terpetakan.

**Catatan lain:** rollback rilis aman dua versi ke belakang (R2 menyisakan 3 versi), dan `ci.yml` serta `web-check.yml` upstream sengaja tidak diubah supaya selisih dengan upstream kecil.

## Peta jalan

Rencana final (Oktober 2026). Tahap awal (bucket R2, pipeline rilis, integrasi SipilStock, tayang) sudah selesai dan tidak diberi nomor di sini.

| Tahap | Isi | Status |
|---|---|---|
| 1 | Kepatuhan dan tampilan lisensi (`LICENSE.txt`, periksa font, tautan lisensi) | Selesai. `LICENSE.txt` disajikan `zlabx`; paket rilis berikutnya memuatnya. Tautan "Sumber & Lisensi" sengaja belum ada (lisensi seharusnya tampil di dalam aplikasi) |
| 2 | Ketahanan build: mirror 7 dependensi git, jalur darurat, `Mirror drill` | Selesai |
| 3 | Performa muat (Cache Rule untuk wasm) | Dilewati dengan sengaja |
| 4 | Kerapian Console dan kosmetik (`supporters.json`, peringatan `integrity`) | Kode selesai di `main` (belum dirilis; rilis bersama Tahap 5 dan 6 sebagai `web-v0.2.0`) |
| 5 | SEO dan indeks halaman. Keputusan pemilik: tetap **noindex** (sudah ada di `web-app.html`; patch build menjaganya) | Kode selesai di `main` (belum dirilis; rilis bersama Tahap 4 dan 6 sebagai `web-v0.2.0`) |
| 6 | Privasi dan panggilan keluar ke pihak ketiga, termasuk Kebijakan Privasi SipilStock | Belum |
| 7 | Menyembunyikan (bukan menghapus) semua tautan dan tombol yang mengarah ke Open CAD Studio (situs, Patreon, GitHub, Reddit, open-aec, YouTube), branding minimal, dialog About dengan dua tautan License dan Source ke domain SipilStock, daftar lisensi crate. Keputusan pemilik: tanpa locale Indonesia (Inggris cukup); tautan ke domain sendiri boleh | Belum |
| 8 | Operasional jangka panjang dan penutupan (Budget Alert R2, uptime check, rotasi token, README final, pencabutan token) | Belum |

Pernah diusulkan lalu dibuang atas keputusan pemilik: mempersempit akses token, dan teks "Chrome/Edge desktop" di kartu SipilStock. Firefox dan WebGL ponsel tidak ditangani (lihat [Keterbatasan yang diketahui](#keterbatasan-yang-diketahui)).

## Checklist sebelum deploy

Awalnya hasil audit membaca kode dan workflow upstream; diperbarui setelah SipilCAD tayang. Yang sudah dikerjakan atau terbukti ditandai centang. Sisanya sengaja ditunda, karena versi pertama hanya perlu jalan.

- [x] **Workflow otomatis upstream** (rilis mingguan terjadwal, komentar otomatis di issue) dinonaktifkan; lihat tabel di atas.
- [x] **Ukuran build web.** `.wasm` utama 53,14 MiB melebihi batas 25 MiB per berkas Cloudflare Pages, jadi dilayani dari R2 dengan domain kustom (lihat Rilis build web). Berkas lain di paket semuanya di bawah batas (terbesar sekitar 10 MiB, font). Mengecilkan `.wasm` (`opt-level`, LTO, `wasm-opt`) tidak dicoba.
- [x] **Toolchain build.** Build Rust memakan 11 sampai 18 menit (batas build Pages 20 menit), jadi dibangun di GitHub Actions; build Pages hanya mengunduh paket rilis.
- [x] **Header cross-origin isolation: tidak diperlukan.** Terbukti: aplikasi berjalan di Chrome dan Edge produksi tanpa `_headers` dan tanpa COOP/COEP. `src/par.rs` menunjukkan WASM berjalan tanpa thread.
- [ ] **Cache Rule untuk wasm di edge Cloudflare** dilewati dengan sengaja (keputusan pemilik); alasan dan cara memasangnya ada di [Keterbatasan yang diketahui](#keterbatasan-yang-diketahui).
- [ ] **Panggilan keluar ke pihak ketiga saat runtime.** Telaah kode `src/` (Oktober 2026) untuk target `wasm32`: `api.frankfurter.dev` dan API Patreon (`src/patreon.rs`), registry dan rilis plugin (`src/plugin/marketplace.rs`), feed Discussions dan oEmbed YouTube, pengecekan rilis, dan unduhan font dari repo upstream semuanya dikunci `cfg(not(target_arch = "wasm32"))`, jadi **tidak berjalan di build web**. Saat boot, web hanya mengambil `supporters.json`, `videos.json`, dan `discussions.json` dari origin sendiri; satu-satunya panggilan otomatis ke pihak ketiga adalah thumbnail cadangan `i.ytimg.com` untuk tiap entri `videos.json`, yang dimatikan dengan mengosongkan berkas itu (Tahap 4, belum dirilis). Tautan ke Patreon, Reddit, open-aec.com, GitHub, dan YouTube upstream di antarmuka hanya terbuka atas klik pengguna; Tahap 7 menyembunyikannya. Tidak ada analitik atau telemetri di `src/` (dependensi tidak diperiksa). Berkas disimpan di perangkat pengguna: `localStorage` (pengaturan) dan OPFS (salinan berkas terakhir); gambar diproses di browser dan tidak diunggah. Sisa: memperbarui Kebijakan Privasi SipilStock (Tahap 6).
- [ ] **Berkas web dari origin sendiri.** `supporters.json` dan `video_thumbs/*.jpg` tidak ada di paket (dibuat oleh pipeline deploy upstream), sehingga muncul 404 di Console; aplikasi tetap jalan. Ditangani di `main` dengan menulis `[]` ke `supporters.json`, `videos.json`, dan `discussions.json` saat `package`; baru tuntas setelah rilis `web-v0.2.0` tayang.
- [ ] **Branding dan tautan.** Judul halaman (`web-app.html`), logo, nama `OpenCADStudio` di ratusan berkas, serta tautan Patreon, open-aec.com, dan Reddit milik upstream di antarmuka. `site/CNAME` berisi `www.opencadstudio.com`; jangan dipakai. Tetap sertakan atribusi.
- [ ] **Halaman "Tentang/Lisensi"** di dalam aplikasi: sebut Open CAD Studio oleh HakanSeven12, lisensi GPL-3.0, dan tautan ke repo ini. Keputusan (Tahap 1, langkah 4): untuk sementara **tidak ada tautan "Sumber & Lisensi"** dari antarmuka SipilStock (kartu, halaman Apps, footer). Lisensi seharusnya tampil di dalam aplikasi itu sendiri; tautan ke `/sipilcad/SOURCE.txt` dan `/sipilcad/LICENSE.txt` ditambahkan setelah itu. Sementara ini keduanya hanya tersaji di alamat tersebut.
- [x] **Dependensi git eksternal** dimirror: 7 sumber git (termasuk `winit` dan `cryoglyph` yang tidak langsung) punya fork beku di akun `zlabx`, dengan jalur darurat yang dilatih oleh workflow `Mirror drill` (lihat [Ketahanan build](#ketahanan-build-mirror-dependensi-git)). Sisa: drill baru `cargo check` wasm32, belum `trunk build` penuh.
- [ ] **`release.yml` dan `pages.yml` upstream** memakai secret milik upstream (Patreon, penandatanganan Windows/Azure, Snapcraft) dan repo/nama paket upstream. Jangan diaktifkan sebelum diadaptasi ke SipilCAD.
- [x] **`.github/FUNDING.yml`** dihapus dari `main` (keputusan pemilik, Oktober 2026: tidak ada funding). Tombol Sponsor di halaman repo ini hilang. Berkas di branch `upstream` tidak disentuh (cermin murni; GitHub hanya membaca branch default). Bila upstream mengubah berkas itu lagi, sinkron berikutnya menghasilkan konflik modify/delete: selesaikan dengan `git rm .github/FUNDING.yml`.
- [x] **Lisensi font web dan font LFF** diperiksa: font web OFL 1.1 dengan `fonts/OFL.txt` di paket, font LFF public domain dan/atau GPL v2+ (lihat [Lisensi komponen pihak ketiga](#lisensi-komponen-pihak-ketiga)). Sisa: `ltypeshp.lff` tanpa baris lisensi.
- [ ] **Daftar lisensi crate Rust pihak ketiga** belum ada di paket (upstream juga tidak punya). Opsi: `cargo-about` di workflow rilis.
- [x] **Bahasa Indonesia:** tidak dikerjakan (keputusan pemilik, Oktober 2026: bahasa Inggris cukup). Locale baru juga berarti menerjemahkan ulang string baru upstream di tiap sinkron, karena uji `i18n` mewajibkan kelengkapan.

## Integrasi dengan SipilStock

SipilStock ([`zlabx/zlabx`](https://github.com/zlabx/zlabx), privat) memasang SipilCAD saat deploy lewat `scripts/build-sipilcad.sh`: skrip mengunduh paket rilis `web-v*` dari repo ini (publik, tanpa token), memeriksa SHA-256, `LICENSE`, `SOURCE.txt`, dan ukuran berkas, menyediakan `LICENSE.txt` bila paket belum memuatnya, memastikan `.wasm` di R2 terjangkau, lalu mengekstrak ke `sipilcad/` (di-gitignore). Versi yang tayang dipin lewat `DEFAULT_REF` di skrip itu (kini `web-v0.1.0`); env `SIPILCAD_REF` menimpanya (uji di Preview Cloudflare) dan `none` melewati SipilCAD. Build command Cloudflare Pages (berlaku untuk produksi dan Preview): `npm run generate:all && npm run build:sipilframe && npm run build:sipilcad && npm run build:sipildraw && npm run prune:public`. Kartu SipilCAD di halaman utama, halaman Apps, dan dropdown Apps mengarah ke `/sipilcad/`.

## Lisensi komponen pihak ketiga

Diperiksa pada Tahap 1 (Oktober 2026). Ini catatan teknis, bukan nasihat hukum.

- **Font web** (`fonts/`, dimuat browser sesuai bahasa): 11 berkas subset **Noto Sans** (termasuk Noto Sans CJK dari Adobe) di bawah **SIL Open Font License 1.1**. Cara membuatnya ada di `web/fonts/generate.sh`. `fonts/OFL.txt` (pernyataan hak cipta dan teks lisensi) ikut di paket rilis yang tayang dan identik dengan upstream; metadata hak cipta dan lisensi juga tertanam di tiap berkas font. Nama font tetap "Noto Sans" dan "Noto Sans CJK", bukan nama yang dicadangkan ("Source").
- **Font garis LFF** (`assets/fonts/*.lff`, 27 berkas, 2,52 MiB, tertanam di `.wasm`): bawaan LibreCAD. Header berkas mencantumkan lisensi **public domain** (font Hershey) dan/atau **GPL v2 atau lebih baru**, yang kompatibel dengan GPL-3.0 aplikasi ini. Pengecualian: `ltypeshp.lff` tidak punya baris lisensi di headernya; sumber aslinya belum ditelusuri.
- **Crate Rust pihak ketiga:** upstream tidak menyertakan daftar lisensinya, dan paket web ini pun belum. Pilihan ke depan: membuat daftarnya otomatis (mis. dengan `cargo-about`) dan menyertakannya di paket.

## Lisensi

[GNU General Public License v3.0](LICENSE). Copyright pada kode Open CAD Studio dimiliki penulis aslinya; modifikasi SipilCAD dirilis di bawah lisensi yang sama. Kode sumber lengkap tersedia di repo ini untuk siapa pun yang menggunakan SipilCAD. Crate `opencadkernel`, `opencadcodec`, dan `opencadgraph` yang dipakai sebagai dependensi berlisensi MPL-2.0 di repo masing-masing.

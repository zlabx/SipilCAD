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
- Perubahan baru sebatas **menonaktifkan dua workflow GitHub Actions upstream** yang akan berjalan otomatis di repo ini; lihat [Perubahan dari upstream](#perubahan-dari-upstream). Kode aplikasi, branding, dan antarmuka **belum diubah**.
- **Belum dideploy.** Di CI, `cargo test` dan `cargo check` wasm lolos (commit `a0c2f8a2`), dan `trunk build --release` sukses (workflow `wasm-size.yml`, commit `0bb4b469`), tetapi hasilnya belum diuji di browser. **Berkas `.wasm` utama 53,14 MiB, melebihi batas 25 MiB per berkas Cloudflare Pages.** Baca [Checklist sebelum deploy](#checklist-sebelum-deploy) dulu.
- Repo ini dulunya bernama **SipilXCAD** dan sudah di-rename menjadi **SipilCAD**; riwayat commit tetap utuh. Repo ini menggantikan SipilCAD lama (viewer berbasis JavaScript), yang kini diarsipkan di luar repo ini. Folder `/sipilcad/` di SipilStock sudah dikosongkan dan menunggu aplikasi ini.

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
| `.github/workflows/wasm-size.yml` (baru) | Workflow ukur saja: `trunk build --release` lalu melaporkan ukuran berkas (asli, gzip, brotli) dan baris pemuat wasm/js di `index.html` (untuk merancang hosting R2) di ringkasan job dan anotasi. Tidak ada deploy, rilis, artifact, atau secret | Menentukan apakah hasil build web muat di batas 25 MiB per berkas Cloudflare Pages |
| `README.md` | Diganti | Atribusi dan panduan SipilCAD |

Workflow `ci.yml` dan `web-check.yml` sengaja dibiarkan: keduanya berjalan pada push ke `main` dan pull request, tidak memakai secret, dan berguna sebagai pemeriksaan build. Hasilnya belum pernah dilihat di repo ini.

## Checklist sebelum deploy

Hasil audit awal (membaca kode dan workflow upstream; belum ada build). Yang sudah dikerjakan ditandai centang.

- [x] **Workflow otomatis upstream** (rilis mingguan terjadwal, komentar otomatis di issue) dinonaktifkan; lihat tabel di atas.
- [ ] **Ukuran build web: sudah diukur, melebihi batas.** Hasil `trunk build --release` di CI (`wasm-size.yml`, commit `0bb4b469`, `--public-url /sipilxcad/`): total 90,44 MiB dalam 26 berkas. `OpenCADStudio-*_bg.wasm` **53,14 MiB** (gzip 17,87 MiB, brotli 11,27 MiB), sedangkan Cloudflare Pages membatasi 25 MiB per berkas (batas yang sama berlaku untuk static asset di Workers). Berkas lain di bawah batas: font terbesar sekitar 10 MiB (`web/fonts`) dan `worker_pkg/ocs_web_worker_bg.wasm` 4,60 MiB. Belum diketahui apakah `wasm-opt` sudah dijalankan `trunk` (log tidak terbaca), dan `[profile.release]` upstream hanya berisi `strip = true`. Pilihan yang belum dicoba: (a) mengecilkan `.wasm` (`opt-level = "z"`, LTO, `codegen-units = 1`, `wasm-opt`), tetapi perlu turun lebih dari separuh; (b) menaruh `.wasm` di bucket R2 publik dengan domain kustom (jalur yang disarankan dokumentasi Cloudflare) dan mengarahkan loader ke sana, yang butuh CORS dan, bila COEP `require-corp` dipakai, header CORP; (c) meng-host aplikasi web di layanan lain (upstream memakai GitHub Pages) dan menautkannya dari SipilStock.
- [ ] **Toolchain build.** Pastikan Rust, `trunk`, dan `wasm-bindgen-cli` tersedia di lingkungan build Cloudflare Pages. Bila tidak, build di GitHub Actions dan publikasikan hasilnya.
- [ ] **Header cross-origin isolation.** `Trunk.toml` menyebut header `Cross-Origin-Opener-Policy: same-origin` dan `Cross-Origin-Embedder-Policy: require-corp` agar WASM bisa multi-thread; tanpanya aplikasi tetap jalan satu thread. Di Cloudflare Pages bisa lewat `_headers`. `require-corp` memblokir sumber lintas-origin tanpa header CORP, jadi uji thumbnail dan pemuatan lain setelah diaktifkan. Halaman juga sebaiknya dibuka langsung, bukan di-embed lewat iframe.
- [ ] **Panggilan keluar ke pihak ketiga saat runtime.** Dari kode `src/`: feed dan halaman Discussions upstream di GitHub (`src/discussions.rs`), registry plugin dari `raw.githubusercontent.com/HakanSeven12/OpenCADStudio` serta rilis/README repo plugin lewat GitHub (`src/plugin/marketplace.rs`), thumbnail dan oEmbed YouTube untuk playlist upstream (`src/videos.rs`), dan `api.frankfurter.dev` serta Patreon (`src/patreon.rs`; kapan tepatnya dipanggil belum ditelusuri). Bila dipanggil dari browser pengguna, IP pengguna terkirim ke pihak-pihak itu. Putuskan mana yang dimatikan atau diarahkan ke SipilStock, dan perbarui Kebijakan Privasi. Saya tidak menemukan analitik atau telemetri pihak ketiga, tapi pencarian ini hanya mencakup `src/`, bukan dependensi.
- [ ] **Berkas web yang diambil dari origin sendiri.** Build web meminta `discussions.json`, `videos.json`, dan `supporters.json` secara relatif. Dua yang pertama ada di `web/`; `supporters.json` dibuat oleh workflow upstream dengan token Patreon dan tidak ada di repo. Uji perilaku aplikasi bila berkas itu tidak ada.
- [ ] **Branding dan tautan.** Judul halaman (`web-app.html`), logo, nama `OpenCADStudio` di ratusan berkas, serta tautan Patreon, open-aec.com, dan Reddit milik upstream di antarmuka. `site/CNAME` berisi `www.opencadstudio.com`; jangan dipakai. Tetap sertakan atribusi.
- [ ] **Halaman "Tentang/Lisensi"** di dalam aplikasi: sebut Open CAD Studio oleh HakanSeven12, lisensi GPL-3.0, dan tautan ke repo ini.
- [ ] **Dependensi git eksternal.** `Cargo.toml` mengambil `iced`, fork `iced_aw` (branch `agent/fix-iced-fonts`), dan tiga crate upstream langsung dari GitHub. Bila repo atau branch itu hilang atau di-force-push, build rusak. Pertimbangkan mirror ke akun zlabx atau `cargo vendor`.
- [ ] **`release.yml` dan `pages.yml` upstream** memakai secret milik upstream (Patreon, penandatanganan Windows/Azure, Snapcraft) dan repo/nama paket upstream. Jangan diaktifkan sebelum diadaptasi ke SipilCAD.
- [ ] **`.github/FUNDING.yml`** masih menunjuk ke sponsor upstream. Sengaja dibiarkan agar dukungan mengalir ke penulis asli; ganti bila tidak diinginkan.
- [ ] **Bahasa Indonesia** (opsional): tambahkan locale baru dan kirim juga ke upstream.

## Integrasi dengan SipilStock

Repo ini (dulu SipilXCAD) sudah menggantikan SipilCAD lama di repo utama [`zlabx/zlabx`](https://github.com/zlabx/zlabx) (privat): folder `sipilcad/` dan skrip build lama sudah dihapus dari sana, sedangkan kartu SipilCAD di halaman utama, halaman Apps, dan dropdown Apps dipertahankan dan menunggu tautan `/sipilcad/` diisi. Cara publikasinya (build di GitHub Actions dan `.wasm` di R2, atau cara lain) belum diputuskan; lihat checklist di atas. Karena repo ini publik, tidak diperlukan token untuk clone.

## Lisensi

[GNU General Public License v3.0](LICENSE). Copyright pada kode Open CAD Studio dimiliki penulis aslinya; modifikasi SipilCAD dirilis di bawah lisensi yang sama. Kode sumber lengkap tersedia di repo ini untuk siapa pun yang menggunakan SipilCAD. Crate `opencadkernel`, `opencadcodec`, dan `opencadgraph` yang dipakai sebagai dependensi berlisensi MPL-2.0 di repo masing-masing.

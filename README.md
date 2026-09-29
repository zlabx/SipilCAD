# SipilXCAD

**Gambar teknik 2D dan pemodelan 3D dengan format DWG/DXF native, untuk desktop dan web** — bagian dari ekosistem [SipilStock](https://sipilstock.com).

SipilXCAD adalah turunan (derivative work) dari **[Open CAD Studio](https://github.com/HakanSeven12/OpenCADStudio)** karya **HakanSeven12**, dan dirilis di bawah lisensi yang sama, **GPL-3.0**.

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
> menjadi milik penulis asli. Perubahan khusus SipilXCAD dicatat di bagian [Perubahan dari upstream](#perubahan-dari-upstream).

---

## Status

- Basis saat ini: **Open CAD Studio 2026.39.0**, upstream commit [`07f25f4b`](https://github.com/HakanSeven12/OpenCADStudio/commit/07f25f4b) (29 Sep 2026).
- Perubahan baru sebatas **menonaktifkan workflow GitHub Actions upstream** yang akan berjalan otomatis di repo ini; lihat [Perubahan dari upstream](#perubahan-dari-upstream). Kode aplikasi, branding, dan antarmuka **belum diubah**.
- **Belum dibuild dan belum dideploy** dari repo ini. Perintah build di bawah diambil dari dokumentasi dan workflow upstream, belum dijalankan di sini. Baca [Checklist sebelum deploy](#checklist-sebelum-deploy) dulu.
- SipilXCAD saat ini produk terpisah dari SipilCAD. Nanti repo ini akan di-rename dan menggantikan SipilCAD di SipilStock.

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
git clone https://github.com/zlabx/SipilXCAD.git
cd SipilXCAD
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
trunk build --locked --release --public-url /sipilxcad/ --dist dist/sipilxcad --html-output index.html web-app.html
```

Catatan dari file upstream:

- Hook `post_build` di `Trunk.toml` menjalankan `scripts/build-web-worker.sh` (membuild `ocs_web_worker`), jadi `wasm-bindgen` harus ada di PATH.
- `build.rs` menghitung jarak commit dari tag rilis. Build dari clone dangkal (shallow) akan berlabel `+g<hash>`; gunakan clone penuh (`fetch-depth: 0`) bila label versi penting.
- Workflow upstream juga merakit landing page dan membuat `supporters.json`/`discussions.json`. Bagian itu **tidak** dipakai di sini.

## Struktur branch dan sinkron dengan upstream

| Branch | Fungsi |
|---|---|
| `upstream` | **Cermin murni** `HakanSeven12/OpenCADStudio` (`main`). Jangan commit apa pun di sini. |
| `main` | Versi SipilXCAD. Ini yang dipakai untuk build/deploy. |

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

Kalau `README.md` konflik saat merge (upstream ikut mengubahnya), pertahankan versi SipilXCAD:
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
| `README.md` | Diganti | Atribusi dan panduan SipilXCAD |

## Checklist sebelum deploy

Hasil audit awal (membaca kode dan workflow upstream; belum ada build). Yang sudah dikerjakan ditandai centang.

- [ ] **Workflow otomatis upstream** (rilis mingguan terjadwal, komentar otomatis di issue): dinonaktifkan lewat commit terpisah; lihat tabel di atas.
- [ ] **Build web belum pernah dijalankan.** Pastikan `trunk build` sukses dan ukur ukuran hasilnya. Cloudflare Pages membatasi ukuran per file (25 MiB); bila `.wasm` melewatinya, pertimbangkan `wasm-opt` atau menaruh berkas itu di tempat lain (mis. R2).
- [ ] **Toolchain build.** Pastikan Rust, `trunk`, dan `wasm-bindgen-cli` tersedia di lingkungan build Cloudflare Pages. Bila tidak, build di GitHub Actions dan publikasikan hasilnya.
- [ ] **Header cross-origin isolation.** `Trunk.toml` menyebut header `Cross-Origin-Opener-Policy: same-origin` dan `Cross-Origin-Embedder-Policy: require-corp` agar WASM bisa multi-thread; tanpanya aplikasi tetap jalan satu thread. Di Cloudflare Pages bisa lewat `_headers`. `require-corp` memblokir sumber lintas-origin tanpa header CORP, jadi uji thumbnail dan pemuatan lain setelah diaktifkan. Halaman juga sebaiknya dibuka langsung, bukan di-embed lewat iframe.
- [ ] **Panggilan keluar ke pihak ketiga saat runtime.** Dari kode `src/`: feed dan halaman Discussions upstream di GitHub (`src/discussions.rs`), registry plugin dari `raw.githubusercontent.com/HakanSeven12/OpenCADStudio` serta rilis/README repo plugin lewat GitHub (`src/plugin/marketplace.rs`), thumbnail dan oEmbed YouTube untuk playlist upstream (`src/videos.rs`), dan `api.frankfurter.dev` serta Patreon (`src/patreon.rs`; kapan tepatnya dipanggil belum ditelusuri). Bila dipanggil dari browser pengguna, IP pengguna terkirim ke pihak-pihak itu. Putuskan mana yang dimatikan atau diarahkan ke SipilStock, dan perbarui Kebijakan Privasi. Saya tidak menemukan analitik atau telemetri pihak ketiga, tapi pencarian ini hanya mencakup `src/`, bukan dependensi.
- [ ] **Berkas web yang diambil dari origin sendiri.** Build web meminta `discussions.json`, `videos.json`, dan `supporters.json` secara relatif. Dua yang pertama ada di `web/`; `supporters.json` dibuat oleh workflow upstream dengan token Patreon dan tidak ada di repo. Uji perilaku aplikasi bila berkas itu tidak ada.
- [ ] **Branding dan tautan.** Judul halaman (`web-app.html`), logo, nama `OpenCADStudio` di ratusan berkas, serta tautan Patreon, open-aec.com, dan Reddit milik upstream di antarmuka. `site/CNAME` berisi `www.opencadstudio.com`; jangan dipakai. Tetap sertakan atribusi.
- [ ] **Halaman "Tentang/Lisensi"** di dalam aplikasi: sebut Open CAD Studio oleh HakanSeven12, lisensi GPL-3.0, dan tautan ke repo ini.
- [ ] **Dependensi git eksternal.** `Cargo.toml` mengambil `iced`, fork `iced_aw` (branch `agent/fix-iced-fonts`), dan tiga crate upstream langsung dari GitHub. Bila repo atau branch itu hilang atau di-force-push, build rusak. Pertimbangkan mirror ke akun zlabx atau `cargo vendor`.
- [ ] **`release.yml` dan `pages.yml` upstream** memakai secret milik upstream (Patreon, penandatanganan Windows/Azure, Snapcraft) dan repo/nama paket upstream. Jangan diaktifkan sebelum diadaptasi ke SipilXCAD.
- [ ] **`.github/FUNDING.yml`** masih menunjuk ke sponsor upstream. Sengaja dibiarkan agar dukungan mengalir ke penulis asli; ganti bila tidak diinginkan.
- [ ] **Bahasa Indonesia** (opsional): tambahkan locale baru dan kirim juga ke upstream.

## Integrasi dengan SipilStock

Saat ini SipilXCAD berdiri sebagai produk terpisah. Rencananya repo ini akan di-rename dan menggantikan SipilCAD (`/sipilcad/`) di repo utama [`zlabx/zlabx`](https://github.com/zlabx/zlabx) (privat). Cara publikasinya (clone dan build saat deploy Cloudflare Pages, atau build di GitHub Actions) belum diputuskan; lihat checklist di atas. Karena repo ini publik, tidak diperlukan token untuk clone.

## Lisensi

[GNU General Public License v3.0](LICENSE). Copyright pada kode Open CAD Studio dimiliki penulis aslinya; modifikasi SipilXCAD dirilis di bawah lisensi yang sama. Kode sumber lengkap tersedia di repo ini untuk siapa pun yang menggunakan SipilXCAD. Crate `opencadkernel`, `opencadcodec`, dan `opencadgraph` yang dipakai sebagai dependensi berlisensi MPL-2.0 di repo masing-masing.

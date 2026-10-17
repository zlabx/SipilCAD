# Modifikasi terhadap Open CAD Studio

SipilCAD adalah turunan [Open CAD Studio](https://github.com/HakanSeven12/OpenCADStudio) (GPL-3.0) oleh HakanSeven12 dan
kontributor. Dokumen ini mencatat apa yang diubah pada kode sumber, sebagaimana disyaratkan GPL-3.0
bagian 5(a). Tanggal perubahan adalah tanggal commit build ini di riwayat git repositori.

**Dasar:** `HakanSeven12/OpenCADStudio` pada commit `cff240ef71b4d7d0e4832c8d260c5af7f2fe3b17`.

Daftar di bawah dihitung otomatis terhadap tree upstream murni pada commit tersebut, pada setiap build.

## Ringkasan perubahan

- **Nama dan identitas:** judul tab, layar pemuatan, dialog About, dan teks info yang disalin memakai nama
  SipilCAD; logo diganti.
- **Tautan ke proyek upstream disembunyikan atau diblokir** (situs, Patreon, GitHub, Reddit, open-aec,
  YouTube): tombol dan panel di Start page, dialog donasi saat start, baris Plugins di Options, perintah
  yang membuka situs upstream (DONATE, HELP/F1, REPORT, CHANGELOG, PLUGINS, dst.), dan jaring pengaman
  `open_url` untuk tautan upstream baru. Semuanya di balik satu flag, `HIDE_UPSTREAM_LINKS` di
  `src/sipil.rs`; setiap titik ubah bertanda `// SIPIL:` (`git grep "SIPIL:"`).
- **Atribusi dan lisensi:** dialog About menampilkan tautan License dan Source (berkas `LICENSE.txt` dan
  `SOURCE.txt` yang disajikan di samping aplikasi). `LICENSE` upstream dan header hak cipta tidak diubah.
- **Data upstream dikosongkan:** `web/videos.json` dan `web/discussions.json` berisi daftar kosong;
  `supporters.json` dikosongkan saat build (`scripts/sipil-postbuild.py`).
- **Penjaga sinkron:** `tests/upstream_links.rs` gagal bila upstream menambah host atau titik pemanggil
  `open_url` baru.
- **Tambahan build:** `about.toml` dan `about.hbs` (THIRD_PARTY_LICENSES.txt), `scripts/sipil-*`,
  `SOURCE.txt.in`, `BUILD.md`, dan `README.md` yang baru.

## Berkas yang diubah
- `README.md`
- `assets/logo.svg`
- `src/app/commands/view.rs`
- `src/app/startup.rs`
- `src/app/view/mod.rs`
- `src/lib.rs`
- `src/sys.rs`
- `src/ui/window/about.rs`
- `src/ui/window/options.rs`
- `src/ui/window/xref_help.rs`
- `web-app.html`
- `web/discussions.json`
- `web/locale-labels.json`
- `web/videos.json`

## Berkas yang ditambah
- `BUILD.md`
- `SOURCE.txt.in`
- `about.hbs`
- `about.toml`
- `scripts/sipil-licenses.sh`
- `scripts/sipil-postbuild.py`
- `scripts/sipil-tools.sh`
- `src/sipil.rs`
- `tests/upstream_links.rs`

## Berkas upstream yang dihapus
- (tidak ada)

## Berkas upstream yang tidak disertakan di repositori ini
- `.github/`
(workflow CI/rilis upstream; build SipilCAD memakai alur sendiri, lihat `BUILD.md`)

## Statistik perubahan (`git diff --stat`, tanpa `src/sipil.rs`)

```
 BUILD.md                   |  40 ++++++++++++
 README.md                  | 262 +--------------------------------------------------------------------------
 SOURCE.txt.in              |  33 ++++++++++
 about.hbs                  |  28 ++++++++
 about.toml                 |  32 ++++++++++
 assets/logo.svg            |  22 +------
 scripts/sipil-licenses.sh  |  30 +++++++++
 scripts/sipil-postbuild.py | 192 +++++++++++++++++++++++++++++++++++++++++++++++++++++++
 scripts/sipil-tools.sh     |  31 +++++++++
 src/app/commands/view.rs   |  17 +++++
 src/app/startup.rs         |   6 +-
 src/app/view/mod.rs        | 160 ++++++++++++++++++++++++++++++----------------
 src/lib.rs                 |   1 +
 src/sys.rs                 |   5 ++
 src/ui/window/about.rs     |  45 +++++++++++--
 src/ui/window/options.rs   |  32 ++++++----
 src/ui/window/xref_help.rs |  27 +++++---
 tests/upstream_links.rs    | 157 +++++++++++++++++++++++++++++++++++++++++++++
 web-app.html               |  67 +++++++++++++------
 web/discussions.json       |  43 +------------
 web/locale-labels.json     |  44 ++++++-------
 web/videos.json            |  23 +------
 22 files changed, 832 insertions(+), 465 deletions(-)
```

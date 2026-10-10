//! SIPIL: penjaga sinkron upstream.
//!
//! SipilCAD menyembunyikan semua tombol, panel, dan tautan yang mengarah ke proyek Open CAD
//! Studio (lihat `src/sipil.rs`). Upstream sangat aktif dan sewaktu-waktu menambah tautan baru;
//! tanpa penjaga, tautan itu muncul di aplikasi kita tanpa ada yang menyadarinya.
//!
//! Uji ini memindai kode sumber dan gagal bila ada:
//!   1. host URL baru (daftar `KNOWN_HOSTS` di bawah), atau
//!   2. titik pemanggil `open_url(` / `Message::OpenUrl(` lebih banyak dari `KNOWN_OPEN_URL_SITES`.
//!
//! Bila gagal setelah `git merge upstream`: periksa tautan barunya. Kalau mengarah ke proyek upstream,
//! sembunyikan di balik `crate::sipil::HIDE_UPSTREAM_LINKS` (cari penanda `// SIPIL:` sebagai contoh) dan
//! tambahkan polanya ke `BLOCKED_URL_PARTS` di `src/sipil.rs`. Kalau aman, tambahkan hostnya ke daftar ini.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Host yang sudah ditinjau (per Oktober 2026). Yang terkait upstream sudah disembunyikan atau tidak
/// berjalan di build web (lihat README bagian "Panggilan keluar ke pihak ketiga").
const KNOWN_HOSTS: &[&str] = &[
    "127.0.0.1",
    "3dconnexion.com",
    "api.frankfurter.dev",
    "api.github.com",
    "crates.io",
    "description.invalid",
    "example.com",
    "github.com",
    "hakanseven12.github.io",
    "i.ytimg.com",
    "intra",
    "localhost",
    "open-aec.com",
    "patreon.com",
    "raw.githubusercontent.com",
    "server",
    "sipilcad-cdn.sipilstock.com",
    "sipilstock.com",
    "www.freedesktop.org",
    "www.patreon.com",
    "www.reddit.com",
    "www.w3.org",
    "www.youtube.com",
    "youtu.be",
    "youtube.com",
];

/// Jumlah titik pemanggil `open_url(` dan `OpenUrl(` di `src/` yang sudah ditinjau.
const KNOWN_OPEN_URL_SITES: usize = 25;

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// Baris kode (bukan komentar) dari semua berkas `.rs` di bawah `dirs`.
fn code_lines(dirs: &[&str]) -> Vec<(PathBuf, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for dir in dirs {
        rust_files(&root.join(dir), &mut files);
    }
    files.sort();
    let mut lines = Vec::new();
    for file in files {
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        for line in text.lines() {
            if !line.trim_start().starts_with("//") {
                lines.push((file.clone(), line.to_string()));
            }
        }
    }
    lines
}

fn hosts_in(line: &str, hosts: &mut BTreeSet<String>) {
    for scheme in ["https://", "http://"] {
        let mut rest = line;
        while let Some(index) = rest.find(scheme) {
            let after = &rest[index + scheme.len()..];
            let end = after
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '-'))
                .unwrap_or(after.len());
            let host = after[..end].to_ascii_lowercase();
            if !host.is_empty() {
                hosts.insert(host);
            }
            rest = &after[end..];
        }
    }
}

#[test]
fn no_new_external_hosts_in_source() {
    let mut found = BTreeSet::new();
    for (_, line) in code_lines(&["src", "crates"]) {
        hosts_in(&line, &mut found);
    }
    let known: BTreeSet<String> = KNOWN_HOSTS.iter().map(|h| h.to_string()).collect();
    let new_hosts: Vec<&String> = found.difference(&known).collect();
    assert!(
        new_hosts.is_empty(),
        "host URL baru di kode sumber: {new_hosts:?}. Tautan baru dari upstream? Periksa, sembunyikan bila \
         mengarah ke proyek upstream (lihat komentar di atas berkas ini), lalu tambahkan ke KNOWN_HOSTS."
    );
}

#[test]
fn no_new_open_url_call_sites() {
    let mut sites = Vec::new();
    for (file, line) in code_lines(&["src"]) {
        let hits = line.matches("open_url(").count() + line.matches("OpenUrl(").count();
        if hits > 0 {
            sites.push((file, line.trim().to_string()));
        }
    }
    assert!(
        sites.len() <= KNOWN_OPEN_URL_SITES,
        "titik pemanggil open_url/OpenUrl bertambah ({} > {}). Tombol atau tautan baru dari upstream? \
         Periksa lalu perbarui KNOWN_OPEN_URL_SITES. Daftar sekarang:\n{}",
        sites.len(),
        KNOWN_OPEN_URL_SITES,
        sites
            .iter()
            .map(|(file, line)| format!("  {}: {line}", file.display()))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn guard_scanner_finds_hosts() {
    let mut hosts = BTreeSet::new();
    hosts_in(
        r#"let a = "https://Example.COM/x?y=1"; let b = "http://a-b.example.org:8080/z"; let c = "https://{repo}/x";"#,
        &mut hosts,
    );
    assert_eq!(
        hosts.into_iter().collect::<Vec<_>>(),
        vec!["a-b.example.org".to_string(), "example.com".to_string()]
    );
}

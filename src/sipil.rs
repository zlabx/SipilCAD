//! SIPIL: tambahan khusus SipilCAD.
//!
//! Semua kode tambahan kita dikumpulkan di berkas ini supaya perubahan pada berkas upstream
//! tinggal baris pengait kecil yang bertanda `// SIPIL:`. `git grep "SIPIL:"` menampilkan
//! semuanya, jadi sinkron dengan upstream tinggal memeriksa daftar itu.

/// Sembunyikan semua tombol, panel, perintah, dan tautan yang mengarah ke proyek Open CAD Studio
/// (situs, Patreon, GitHub, Reddit, open-aec, YouTube). Disembunyikan, bukan dihapus: diff kecil
/// sehingga merge upstream murah, dan semuanya bisa dikembalikan dengan satu flag ini.
pub const HIDE_UPSTREAM_LINKS: bool = true;

/// Dokumen lisensi dan sumber yang disajikan berdampingan dengan aplikasi (berkas di folder yang
/// sama dengan index.html). Sengaja path RELATIF, bukan URL absolut: peramban menyelesaikannya
/// terhadap halaman yang sedang dibuka, sehingga build yang sama berjalan di prefix mana pun
/// (mis. /sipilcad/ maupun /sipilcad-next/) dan tidak pernah menaut ke situs lain.
pub const LICENSE_PATH: &str = "LICENSE.txt";
pub const SOURCE_PATH: &str = "SOURCE.txt";
/// Teks lisensi font PT Sans (SIL OFL 1.1): splash web dan huruf label ikon. Berkas ini disalin ke samping index.html
/// oleh `<link data-trunk rel="copy-file" href="web/pt-sans/OFL-PTSans.txt">` di web-app.html (patch 0009).
pub const FONT_PATH: &str = "OFL-PTSans.txt";

/// Versi SipilCAD yang tampil di dialog About (judul kartu atas) dan di baris pertama "Copy Info". Sengaja terpisah dari versi
/// upstream (`version` di Cargo.toml, `OCS_APP_VERSION`), yang tetap mengikuti rilis Open CAD Studio dan dipakai bagian lain
/// (agen pengguna jaringan, penanda dialog donasi, API otomasi). Format `<tahun>.<nomor>` tanpa awalan "v" dan tanpa angka nol
/// di depan nomor; naikkan di sini bila SipilCAD merilis versi baru.
pub const APP_VERSION: &str = "2027.0";

/// Kalimat di bawah nama produk pada dialog About (pengganti "CAD application for Architecture & Engineering"). Satu kalimat;
/// bila lebih lebar dari kartu atas, teks membungkus ke baris kedua.
pub const ABOUT_TAGLINE: &str = "Professional 2D/3D CAD software for architecture & engineering";

/// Dialog About hanya menampilkan kartu atas (logo, nama, kalimat, versi) dan baris tombol bawah; kartu versi/platform/arch,
/// build/commit/profile, dan baris CPU/RAM/GPU disembunyikan sehingga jendelanya lebih pendek. Info yang sama tetap ada di teks
/// "Copy Info". Setel `false` untuk menampilkan dialog lengkap seperti upstream.
pub const COMPACT_ABOUT: bool = true;

/// Potongan URL (huruf kecil) milik proyek upstream. Cukup mencocokkan substring: pola dipilih
/// agar tidak mengenai domain kita atau hyperlink yang tertanam di gambar milik pengguna.
const BLOCKED_URL_PARTS: &[&str] = &[
    // github.com/HakanSeven12/..., hakanseven12.github.io, patreon.com/HakanSeven12,
    // api.github.com/repos/HakanSeven12/..., raw.githubusercontent.com/HakanSeven12/...
    "hakanseven12",
    "patreon.com",
    "reddit.com/r/opencadstudio",
    "open-aec.com",
    // Playlist tutorial YouTube milik upstream, dan thumbnail-nya.
    "plzq_tekifh9banoox1hicaunm3anzdbol",
    "i.ytimg.com",
];

/// True bila `url` mengarah ke proyek upstream dan tidak boleh dibuka dari aplikasi ini.
pub fn is_blocked_url(url: &str) -> bool {
    if !HIDE_UPSTREAM_LINKS {
        return false;
    }
    let url = url.to_ascii_lowercase();
    BLOCKED_URL_PARTS.iter().any(|part| url.contains(*part))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Setiap URL upstream yang ada di kode (daftar diambil dari `src/`) harus terblokir.
    #[test]
    fn blocks_every_known_upstream_url() {
        if !HIDE_UPSTREAM_LINKS {
            return;
        }
        for url in [
            "https://github.com/HakanSeven12/OpenCADStudio/issues/new?body=x",
            "https://github.com/HakanSeven12/OpenCADStudio/releases/latest",
            "https://github.com/HakanSeven12/OpenCADStudio/discussions/2",
            "https://api.github.com/repos/HakanSeven12/OpenCADStudio/releases/latest",
            "https://raw.githubusercontent.com/HakanSeven12/OpenCADStudio/main/plugins/registry.json",
            "https://hakanseven12.github.io/OpenCADStudio/",
            "https://patreon.com/HakanSeven12",
            "https://www.patreon.com/api/oauth2/v2/campaigns",
            "https://www.reddit.com/r/OpenCADStudio/",
            "https://open-aec.com/",
            "https://youtube.com/playlist?list=PLZq_TEkIFh9bAnoOX1HiCAunm3anZDBOl",
            "https://www.youtube.com/watch?v=abc&list=PLZq_TEkIFh9bAnoOX1HiCAunm3anZDBOl",
            "https://i.ytimg.com/vi/abc/mqdefault.jpg",
        ] {
            assert!(is_blocked_url(url), "harus terblokir: {url}");
        }
    }

    /// Domain kita sendiri dan hyperlink milik pengguna tidak boleh ikut terblokir.
    #[test]
    fn allows_own_domain_and_user_links() {
        for url in [
            LICENSE_PATH,
            SOURCE_PATH,
            FONT_PATH,
            "https://sipilstock.com/sipilcad/",
            "https://sipilstock.com/sipilcad-next/",
            "https://example.com/spesifikasi.pdf",
            "https://github.com/someone/their-plugin",
            "https://www.youtube.com/watch?v=abc",
            "mailto:orang@example.com",
        ] {
            assert!(!is_blocked_url(url), "tidak boleh terblokir: {url}");
        }
    }

    #[test]
    fn matching_ignores_case() {
        assert!(is_blocked_url("HTTPS://PATREON.COM/hakanseven12"));
    }

    /// `<tahun>.<nomor>` (mis. "2027.0"): empat angka, titik, lalu nomor tanpa nol di depan; tanpa awalan "v" (dialog
    /// menambahkannya sendiri) dan tanpa akhiran build.
    #[test]
    fn app_version_has_year_dot_number_shape() {
        let (year, number) = APP_VERSION.split_once('.').expect("harus <tahun>.<nomor>");
        assert!(year.len() == 4 && year.chars().all(|c| c.is_ascii_digit()), "tahun: {APP_VERSION}");
        assert!(
            !number.is_empty()
                && number.chars().all(|c| c.is_ascii_digit())
                && (number == "0" || !number.starts_with('0')),
            "nomor: {APP_VERSION}"
        );
    }

    /// Kalimat About: satu kalimat pendek (paling banyak dua baris di kartu atas), tanpa spasi di ujung, dan bukan teks upstream.
    #[test]
    fn about_tagline_is_one_short_sentence_and_not_upstream() {
        assert!(!ABOUT_TAGLINE.trim().is_empty() && ABOUT_TAGLINE == ABOUT_TAGLINE.trim(), "{ABOUT_TAGLINE:?}");
        assert!(ABOUT_TAGLINE.chars().count() <= 80, "{ABOUT_TAGLINE}");
        assert!(!ABOUT_TAGLINE.ends_with('.') && !ABOUT_TAGLINE.contains('\n'), "{ABOUT_TAGLINE}");
        assert!(!ABOUT_TAGLINE.eq_ignore_ascii_case("CAD application for Architecture & Engineering"));
        assert!(!ABOUT_TAGLINE.to_ascii_lowercase().contains("open cad studio"));
    }

    #[test]
    fn paths_point_at_the_served_documents() {
        // Relatif (tanpa skema dan tanpa "/" di depan) dan berupa berkas teks di samping index.html.
        for path in [LICENSE_PATH, SOURCE_PATH, FONT_PATH] {
            assert!(!path.starts_with('/') && !path.contains("://"), "harus relatif: {path}");
            assert!(path.ends_with(".txt") && !path.contains('/'), "harus berkas .txt di folder yang sama: {path}");
        }
    }
}

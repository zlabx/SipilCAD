//! SIPIL: tambahan khusus SipilCAD.
//!
//! Semua kode tambahan kita dikumpulkan di berkas ini supaya perubahan pada berkas upstream
//! tinggal baris pengait kecil yang bertanda `// SIPIL:`. `git grep "SIPIL:"` menampilkan
//! semuanya, jadi sinkron dengan upstream tinggal memeriksa daftar itu.

/// Sembunyikan semua tombol, panel, perintah, dan tautan yang mengarah ke proyek Open CAD Studio
/// (situs, Patreon, GitHub, Reddit, open-aec, YouTube). Disembunyikan, bukan dihapus: diff kecil
/// sehingga merge upstream murah, dan semuanya bisa dikembalikan dengan satu flag ini.
pub const HIDE_UPSTREAM_LINKS: bool = true;

/// Dokumen lisensi dan sumber yang disajikan SipilStock berdampingan dengan aplikasi. Build web
/// dipasang di `/sipilcad/` (lihat `--public-url` di workflow rilis); tautan ini sengaja berupa
/// path di domain sendiri, bukan URL ke situs lain.
pub const LICENSE_PATH: &str = "/sipilcad/LICENSE.txt";
pub const SOURCE_PATH: &str = "/sipilcad/SOURCE.txt";

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
            "https://sipilstock.com/sipilcad/",
            "https://sipilcad-cdn.sipilstock.com/sipilcad/web-v0.2.0/app_bg.wasm",
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

    #[test]
    fn paths_point_at_the_served_documents() {
        assert!(LICENSE_PATH.starts_with("/sipilcad/") && LICENSE_PATH.ends_with(".txt"));
        assert!(SOURCE_PATH.starts_with("/sipilcad/") && SOURCE_PATH.ends_with(".txt"));
    }
}

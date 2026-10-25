#!/usr/bin/env python3
"""SIPIL: langkah pasca-build web SipilCAD (dipanggil dari build_cmd di Apps/apps.json).

Tidak menyentuh kode aplikasi dan tidak mengubah berkas .wasm. Yang dikerjakan pada hasil
`trunk build` (folder DIST):
  1. hapus atribut `integrity` dari <link rel="preload"> berkas .wasm (peramban mengabaikannya dan
     memberi peringatan di Console; pemuat wasm memakai fetch() tanpa integrity). modulepreload JS
     tetap ber-integrity.
  2. tulis `[]` ke supporters.json, videos.json, dan discussions.json, supaya aplikasi tidak memuat
     data atau gambar dari layanan pihak ketiga saat dibuka.
  3. salin LICENSE menjadi LICENSE.txt dan isi SOURCE.txt dari SOURCE.txt.in.
  4. beri cap `CACHE_VERSION` di sw.js (service worker PWA upstream, sejak v2026.42) dengan hash seluruh
     bundle. Di upstream cap itu dipasang oleh workflow Pages mereka, yang tidak kita pakai. Tanpa cap
     nama cache selalu "v1": cache browser pengguna tidak pernah dibersihkan (wasm ~55 MiB tiap deploy
     menumpuk) dan berkas tanpa hash (ocs-parse-worker.js, locale-labels.json, ...) basi satu kunjungan
     setelah deploy. Langkah ini paling akhir, setelah semua berkas lain final.
Lalu memeriksa bahwa hasilnya sesuai harapan (gagal keras bila tidak): tepat satu *_bg.wasm, meta
robots noindex ada, judul tab dan splash sudah SipilCAD, tidak ada sisa nama upstream di
locale-labels.json, app.webmanifest, dan sw.js, serta cap sw.js terpasang tepat satu kali.
"""
import argparse
import hashlib
import json
import re
import sys
from pathlib import Path

UPSTREAM_NAME = "Open CAD Studio"
BLANK_DATA_FILES = ("supporters.json", "videos.json", "discussions.json")

LINK_TAG_RE = re.compile(r"<link\b[^>]*>", re.IGNORECASE)
REL_PRELOAD_RE = re.compile(r"""\brel\s*=\s*(["'])preload\1""", re.IGNORECASE)
REL_MODULEPRELOAD_RE = re.compile(r"""\brel\s*=\s*(["'])modulepreload\1""", re.IGNORECASE)
HREF_RE = re.compile(r"""\bhref\s*=\s*(["'])(.*?)\1""", re.IGNORECASE)
INTEGRITY_ATTR_RE = re.compile(r"""\s+integrity\s*=\s*(?:"[^"]*"|'[^']*')""", re.IGNORECASE)
ROBOTS_META_RE = re.compile(r"""<meta\b[^>]*\bname\s*=\s*(["'])robots\1[^>]*>""", re.IGNORECASE)
CONTENT_ATTR_RE = re.compile(r"""\bcontent\s*=\s*(["'])(.*?)\1""", re.IGNORECASE | re.DOTALL)
NOINDEX_RE = re.compile(r"\b(?:noindex|none)\b", re.IGNORECASE)
HTML_COMMENT_RE = re.compile(r"<!--.*?-->", re.DOTALL)
TITLE_RE = re.compile(r"<title>(.*?)</title>", re.IGNORECASE | re.DOTALL)
# The splash shows the product name in its own element (patch 0009): <div class="splash-product">SipilCAD</div>.
# sw.js (patch 0011 menjaga namanya, upstream menyimpan penanda): `const CACHE_VERSION = "v1"; // stamped by ...`.
SW_VERSION_RE = re.compile(r"""(\bconst\s+CACHE_VERSION\s*=\s*)(["'])([^"']*)\2""")
SW_STAMP_RE = re.compile(r"^[0-9a-f]{16}$")
SPLASH_OK_RE = re.compile(r"""class=(["'])splash-product\1\s*>\s*Sipil\s*CAD\s*</div>""", re.IGNORECASE)


def fail(msg):
    print(f"::error title=sipil-postbuild::{msg}", flush=True)
    raise SystemExit(msg)


def note(msg):
    print(f"::notice title=sipil-postbuild::{msg}", flush=True)


def find_main_wasm(dist):
    found = sorted(Path(dist).glob("*_bg.wasm"))
    if len(found) != 1:
        fail(f"Diharapkan tepat 1 berkas *_bg.wasm di {dist}, ditemukan {len(found)}: {[p.name for p in found]}")
    return found[0]


def is_wasm_preload(tag):
    if not REL_PRELOAD_RE.search(tag):
        return False
    href = HREF_RE.search(tag)
    return bool(href) and href.group(2).split("?")[0].endswith(".wasm")


def has_integrity(tag):
    return bool(INTEGRITY_ATTR_RE.search(tag))


def strip_wasm_preload_integrity(text):
    """Hapus integrity dari <link rel="preload" ... .wasm> saja. Mengembalikan (teks, jumlah diubah)."""
    changed = 0

    def repl(m):
        nonlocal changed
        tag = m.group(0)
        if not is_wasm_preload(tag) or not has_integrity(tag):
            return tag
        changed += 1
        return INTEGRITY_ATTR_RE.sub("", tag)

    return LINK_TAG_RE.sub(repl, text), changed


def modulepreload_integrity_count(text):
    return sum(1 for t in LINK_TAG_RE.findall(text) if REL_MODULEPRELOAD_RE.search(t) and has_integrity(t))


def robots_noindex(text):
    masked = HTML_COMMENT_RE.sub(lambda m: " " * len(m.group(0)), text)
    for m in ROBOTS_META_RE.finditer(masked):
        c = CONTENT_ATTR_RE.search(m.group(0))
        if c and NOINDEX_RE.search(c.group(2)):
            return True
    return False


def process_index(text):
    """Kembalikan (teks baru, jumlah preload wasm yang integrity-nya dihapus)."""
    before = modulepreload_integrity_count(text)
    text, stripped = strip_wasm_preload_integrity(text)
    if modulepreload_integrity_count(text) != before:
        fail("integrity modulepreload JS ikut berubah; pembersihan salah sasaran")
    if any(is_wasm_preload(t) and has_integrity(t) for t in LINK_TAG_RE.findall(text)):
        fail("masih ada <link rel=preload> wasm yang ber-integrity setelah pembersihan")
    return text, stripped


def check_branding(index_text, labels_path):
    t = TITLE_RE.search(index_text)
    if not t or UPSTREAM_NAME in t.group(1):
        fail(f"<title> masih memuat '{UPSTREAM_NAME}' atau tidak ada; patch 0008 tidak terpasang?")
    if not SPLASH_OK_RE.search(index_text):
        fail("teks splash bukan 'SipilCAD' (elemen splash-product); patch 0008/0009 tidak terpasang atau templat upstream berubah?")
    if not Path(labels_path).exists():
        fail("locale-labels.json tidak ada di hasil build")
    labels = json.loads(Path(labels_path).read_text(encoding="utf-8"))
    left = [k for k, v in labels.items() if isinstance(v, dict) and UPSTREAM_NAME in str(v.get("title", ""))]
    if left:
        fail(f"judul locale masih memuat '{UPSTREAM_NAME}': {', '.join(left[:5])}; apply.sh tidak menggantinya?")
    return len(labels)


def blank_data(dist):
    out = []
    for name in BLANK_DATA_FILES:
        p = Path(dist) / name
        before = None
        if p.exists():
            try:
                data = json.loads(p.read_text(encoding="utf-8"))
                before = len(data) if isinstance(data, list) else -1
            except ValueError:
                before = -1
        p.write_text("[]\n", encoding="utf-8")
        out.append((name, before))
    return out


def check_pwa(dist):
    """app.webmanifest dan sw.js (PWA upstream) tidak boleh masih memuat nama upstream ("Open CAD Studio", "OCS")."""
    manifest = Path(dist) / "app.webmanifest"
    sw = Path(dist) / "sw.js"
    for f in (manifest, sw):
        if not f.is_file():
            fail(f"{f.name} tidak ada di hasil build; upstream mengubah PWA-nya? (patch 0011, bagian cap di sipil-postbuild.py)")
    try:
        data = json.loads(manifest.read_text(encoding="utf-8"))
    except ValueError as e:
        fail(f"app.webmanifest bukan JSON yang valid: {e}")
    for key in ("name", "short_name"):
        value = str(data.get(key, ""))
        if not value or UPSTREAM_NAME in value or value.strip().upper() == "OCS":
            fail(f"app.webmanifest: {key}={value!r} masih nama upstream atau kosong; patch 0011 tidak terpasang?")
    if UPSTREAM_NAME in sw.read_text(encoding="utf-8"):
        fail(f"sw.js masih memuat '{UPSTREAM_NAME}'; patch 0011 tidak terpasang?")
    return data["name"]


def bundle_hash(dist, exclude=("sw.js",)):
    """Hash isi seluruh bundle (nama relatif + isi, berurutan) tanpa berkas yang dikecualikan; 16 heksa."""
    root = Path(dist)
    h = hashlib.sha256()
    for p in sorted(x for x in root.rglob("*") if x.is_file() and x.relative_to(root).as_posix() not in exclude):
        h.update(p.relative_to(root).as_posix().encode("utf-8") + b"\0")
        h.update(hashlib.sha256(p.read_bytes()).digest())
    return h.hexdigest()[:16]


def stamp_service_worker(dist):
    """Ganti nilai CACHE_VERSION di sw.js dengan hash bundle. Gagal bila penandanya bukan tepat satu."""
    sw = Path(dist) / "sw.js"
    text = sw.read_text(encoding="utf-8")
    if len(SW_VERSION_RE.findall(text)) != 1:
        fail("sw.js: penanda `const CACHE_VERSION = \"...\"` bukan tepat satu; upstream mengubah service worker-nya?")
    stamp = bundle_hash(dist)
    sw.write_text(SW_VERSION_RE.sub(lambda m: f"{m.group(1)}{m.group(2)}{stamp}{m.group(2)}", text), encoding="utf-8")
    if not SW_STAMP_RE.match(stamp) or f'"{stamp}"' not in sw.read_text(encoding="utf-8"):
        fail("cap CACHE_VERSION tidak terpasang di sw.js")
    return stamp


def render_source(template, values):
    text = template
    for key, value in values.items():
        text = text.replace(f"@{key}@", value)
    left = sorted(set(re.findall(r"@[A-Z_]+@", text)))
    if left:
        fail(f"SOURCE.txt.in memuat penanda yang belum terisi: {left}")
    return text


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("dist", help="folder hasil trunk build (berisi index.html)")
    ap.add_argument("--root", default=".", help="akar kode sumber (berisi LICENSE dan SOURCE.txt.in)")
    ap.add_argument("--tag", required=True, help="tag build di repo sumber, mis. build-<sha12>-<kit>")
    ap.add_argument("--commit", required=True, help="SHA commit di repo sumber untuk build ini")
    ap.add_argument("--upstream", default="HakanSeven12/OpenCADStudio")
    ap.add_argument("--upstream-sha", required=True)
    ap.add_argument("--source-repo", default="zlabx/SipilCAD")
    a = ap.parse_args(argv)

    dist, root = Path(a.dist), Path(a.root)
    index = dist / "index.html"
    if not index.is_file():
        fail(f"{index} tidak ada")
    wasm = find_main_wasm(dist)

    text, stripped = process_index(index.read_text(encoding="utf-8"))
    index.write_text(text, encoding="utf-8")
    if not robots_noindex(text):
        fail("meta robots noindex tidak ada di index.html (keputusan pemilik: tetap noindex); upstream mengubah web-app.html?")
    nlabels = check_branding(text, dist / "locale-labels.json")
    pwa_name = check_pwa(dist)

    blanked = blank_data(dist)

    lic = root / "LICENSE"
    if not lic.is_file():
        fail(f"{lic} tidak ada")
    (dist / "LICENSE.txt").write_bytes(lic.read_bytes())  # .txt: LICENSE tanpa ekstensi bisa terunduh, bukan tampil
    tpl = root / "SOURCE.txt.in"
    if not tpl.is_file():
        fail(f"{tpl} tidak ada")
    (dist / "SOURCE.txt").write_text(render_source(tpl.read_text(encoding="utf-8"), {
        "TAG": a.tag, "COMMIT": a.commit, "SOURCE_REPO": a.source_repo,
        "UPSTREAM": a.upstream, "UPSTREAM_SHA": a.upstream_sha, "WASM": wasm.name,
    }), encoding="utf-8")

    stamp = stamp_service_worker(dist)  # paling akhir: semua berkas lain sudah final

    note(f"wasm utama {wasm.name}: {wasm.stat().st_size / 1048576:.2f} MiB")
    note(f"index.html: integrity dihapus dari {stripped} preload wasm; "
         f"{modulepreload_integrity_count(text)} modulepreload JS tetap ber-integrity; noindex ada")
    note(f"branding: judul tab dan splash SipilCAD; {nlabels} locale tanpa nama upstream; PWA bernama {pwa_name!r}")
    note(f"sw.js: CACHE_VERSION = {stamp} (hash bundle)")
    note("data upstream dikosongkan: " + ", ".join(
        f"{n} ({'baru' if b is None else ('tidak valid' if b < 0 else f'{b} entri')} -> 0)" for n, b in blanked))
    note("LICENSE.txt dan SOURCE.txt ditulis")


if __name__ == "__main__":
    sys.exit(main())

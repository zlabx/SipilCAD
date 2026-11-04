#!/usr/bin/env python3
"""SIPIL: pasang halaman landing /sipilcad/ ke hasil build (dipanggil dari build_cmd di Apps/apps.json).

Struktur yang dilayani Worker (hasil `dist/sipilcad`, lihat BUILD.md):
    index.html, icon.svg, favicon.svg, screenshot.webp   halaman landing (sumber: sipil-landing/)
    sw.js                                                penghapus service worker lama (sumber: sipil-landing/)
    web/                                                 aplikasi web (hasil trunk build + sipil-postbuild.py)
    desktop/                                             installer dan latest.json (ditulis desktop-publish.yml, bukan build ini)

Yang dikerjakan: salin isi sipil-landing/ ke akar DIST, lalu periksa (gagal keras bila tidak):
  1. semua berkas sumber ada, dan web/index.html (aplikasi) sudah ada di DIST;
  2. src/href di index.html tidak root-absolute (apps-sync menolaknya, dan bakal rusak bila prefix berubah);
     setiap tautan relatif ke berkas lokal menunjuk berkas yang ada (mis. web/LICENSE.txt dari sipil-postbuild.py);
  3. tidak ada <meta name="robots" content="noindex">: halaman ini justru harus bisa ditemukan (aplikasinya noindex);
  4. sw.js yang terpasang adalah penghapus (tanpa handler fetch), supaya tidak ada yang mencegat /sipilcad/ lagi.
"""
import argparse
import re
import shutil
import sys
from pathlib import Path
from urllib.parse import urlsplit

FILES = ("index.html", "icon.svg", "favicon.svg", "screenshot.webp", "sw.js")
ATTR_RE = re.compile(r"""\b(?:src|href)\s*=\s*(["'])(.*?)\1""", re.IGNORECASE)
ROOT_ABS_RE = re.compile(r"""\b(?:src|href)\s*=\s*(["'])/(?!/)""", re.IGNORECASE)
ROBOTS_NOINDEX_RE = re.compile(r"""<meta\b[^>]*\bname\s*=\s*(["'])robots\1[^>]*\bnoindex""", re.IGNORECASE)


def fail(msg):
    print(f"::error title=sipil-landing::{msg}", flush=True)
    raise SystemExit(msg)


def install(src, dist):
    src, dist = Path(src), Path(dist)
    for name in FILES:
        if not (src / name).is_file():
            fail(f"{src / name} tidak ada")
    if not (dist / "web" / "index.html").is_file():
        fail(f"{dist / 'web' / 'index.html'} tidak ada: aplikasi web harus dibangun ke <dist>/web (trunk --dist)")
    for name in FILES:
        shutil.copyfile(src / name, dist / name)
    check(dist)


def local_target(dist, href):
    """Berkas lokal yang ditunjuk href relatif, atau None bila href bukan berkas lokal (https:, #, mailto:, ...)."""
    parts = urlsplit(href)
    if parts.scheme or parts.netloc or not parts.path:
        return None
    path = (dist / parts.path).resolve()
    if path.is_dir() or parts.path.endswith("/"):
        path = path / "index.html"
    return path


def check(dist):
    dist = Path(dist)
    html = (dist / "index.html").read_text(encoding="utf-8")
    if ROOT_ABS_RE.search(html):
        fail("index.html punya src/href root-absolute (/...): pakai URL relatif")
    if ROBOTS_NOINDEX_RE.search(html):
        fail("index.html bermeta robots noindex: halaman landing harus bisa diindeks")
    root = dist.resolve()
    for _, href in ATTR_RE.findall(html):
        target = local_target(dist, href)
        if target is None:
            continue
        if root != target and root not in target.parents:
            # ../launcher/... keluar dari dist: dilayani Worker dari prefix lain (/launcher/), tidak ada di build ini.
            continue
        if not target.is_file():
            fail(f"index.html menunjuk {href} tetapi {target.relative_to(root)} tidak ada di hasil build")
    sw = (dist / "sw.js").read_text(encoding="utf-8")
    if re.search(r"""addEventListener\(\s*["']fetch["']""", sw):
        fail("sw.js punya handler fetch: harus penghapus service worker saja")
    if "unregister()" not in sw:
        fail("sw.js tidak mencabut dirinya (unregister)")
    print(f"::notice title=sipil-landing::landing terpasang di {dist} ({len(FILES)} berkas)", flush=True)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n", 1)[0])
    ap.add_argument("dist", help="folder hasil build yang dilayani di /sipilcad/ (berisi web/)")
    ap.add_argument("--src", default=str(Path(__file__).resolve().parent.parent / "sipil-landing"),
                    help="folder sumber landing (default: ../sipil-landing)")
    a = ap.parse_args(argv)
    install(a.src, a.dist)
    return 0


if __name__ == "__main__":
    sys.exit(main())

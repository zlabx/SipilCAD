#!/usr/bin/env python3
"""Uji lokal untuk release_web.py (stdlib saja, tanpa R2/jaringan).

Jalankan dari akar repo:  python3 -m unittest discover -s .github/scripts -p 'test_*.py' -v

Fixture fixtures/index.trunk.html adalah index.html asli keluaran trunk (rilis web-v0.1.0)
yang dikembalikan ke bentuk sebelum patch. CI belum menjalankan berkas ini (workflow tidak
diubah); jalankan secara manual setiap kali release_web.py diubah.
"""
import argparse
import contextlib
import io
import json
import re
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import release_web as rw  # noqa: E402

FIXTURE = Path(__file__).resolve().parent / "fixtures" / "index.trunk.html"
WASM = "OpenCADStudio-a106f805b3db8319_bg.wasm"
R2 = "https://sipilcad-cdn.sipilstock.com"


def tags(html):
    return rw.LINK_TAG_RE.findall(html)


def run(fn, **kw):
    """Jalankan fungsi perintah, kembalikan stdout (anotasi ::notice dll.)."""
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        fn(argparse.Namespace(**kw))
    return buf.getvalue()


class PatchTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.dist = self.root / "dist"
        self.dist.mkdir()
        (self.dist / WASM).write_bytes(b"\0asm")
        self.index = self.dist / "index.html"
        self.index.write_text(FIXTURE.read_text(encoding="utf-8"), encoding="utf-8")

    def tearDown(self):
        self.tmp.cleanup()

    def patch(self):
        return run(rw.cmd_patch, dist=str(self.dist), metrics=str(self.root / "m.json"),
                   base_path="/sipilcad/", public_url=R2, prefix="sipilcad/web-v9.9.9")

    def test_fixture_punya_empat_tag_berintegrity(self):
        with_int = [t for t in tags(self.index.read_text(encoding="utf-8")) if rw.has_integrity(t)]
        self.assertEqual(len(with_int), 4)  # ikon, 2 modulepreload JS, 1 preload wasm

    def test_hanya_preload_wasm_kehilangan_integrity(self):
        before = tags(self.index.read_text(encoding="utf-8"))
        out = self.patch()
        after = tags(self.index.read_text(encoding="utf-8"))
        self.assertEqual(len(before), len(after))
        for b, a in zip(before, after):
            if rw.is_wasm_preload(b):
                self.assertTrue(rw.has_integrity(b))
                self.assertFalse(rw.has_integrity(a))
                self.assertIn(R2, a)  # URL R2 tetap terpasang
                self.assertIn('as="fetch"', a)
                self.assertIn('crossorigin="anonymous"', a)
            else:
                self.assertEqual(rw.has_integrity(b), rw.has_integrity(a), b)  # ikon & modulepreload utuh
        self.assertIn("integrity dihapus dari 1 preload wasm; 2 modulepreload JS tetap ber-integrity", out)

    def test_url_wasm_diganti_dan_tidak_ada_sisa(self):
        self.patch()
        html = self.index.read_text(encoding="utf-8")
        self.assertNotIn(f"/sipilcad/{WASM}", html.replace(f"{R2}/sipilcad/web-v9.9.9/{WASM}", ""))
        self.assertGreaterEqual(html.count(f"{R2}/sipilcad/web-v9.9.9/{WASM}"), 2)

    def test_metrics_mencatat_jumlah(self):
        self.patch()
        m = json.loads((self.root / "m.json").read_text())
        self.assertEqual(m["preload_integrity_removed"], 1)

    def test_tanpa_integrity_hanya_notice_bukan_gagal(self):
        # trunk suatu hari berhenti memberi integrity pada preload wasm: patch tidak boleh gagal
        html = self.index.read_text(encoding="utf-8")
        stripped, n = rw.strip_wasm_preload_integrity(html)
        self.assertEqual(n, 1)
        self.index.write_text(stripped, encoding="utf-8")
        out = self.patch()
        self.assertIn("tidak ada preload wasm ber-integrity", out)
        self.assertEqual(json.loads((self.root / "m.json").read_text())["preload_integrity_removed"], 0)

    def test_pembersihan_idempoten(self):
        html = self.index.read_text(encoding="utf-8")
        once, n1 = rw.strip_wasm_preload_integrity(html)
        twice, n2 = rw.strip_wasm_preload_integrity(once)
        self.assertEqual((n1, n2), (1, 0))
        self.assertEqual(once, twice)

    def test_variasi_atribut_dan_kutip(self):
        src = (
            "<link href='/a_bg.wasm?v=1' integrity='sha384-xyz' rel='preload' as='fetch'/>\n"
            '<link rel="preload" href="/x.woff2" integrity="sha384-keep" as="font">\n'
            '<link rel="modulepreload" href="/m.js" integrity="sha384-keep2">\n'
            '<link rel="preload" as="fetch" href="/b_bg.wasm" crossorigin integrity="sha384-qq">\n'
            '<link rel="stylesheet" href="/s.css" integrity="sha384-keep3">\n'
            '<link rel="modulepreload" href="/w_bg.wasm" integrity="sha384-keep4">\n'
        )
        new, n = rw.strip_wasm_preload_integrity(src)
        self.assertEqual(n, 2)
        lines = new.splitlines()
        self.assertNotIn("integrity", lines[0])
        self.assertIn("rel='preload'", lines[0])
        self.assertIn("integrity=\"sha384-keep\"", lines[1])   # preload non-wasm tetap
        self.assertIn("integrity=\"sha384-keep2\"", lines[2])  # modulepreload tetap
        self.assertNotIn("integrity", lines[3])
        self.assertIn("integrity=\"sha384-keep3\"", lines[4])  # stylesheet tetap
        self.assertIn("integrity=\"sha384-keep4\"", lines[5])  # modulepreload tetap walau href .wasm

    def test_nama_wasm_hilang_tetap_gagal(self):
        self.index.write_text("<html></html>", encoding="utf-8")
        with self.assertRaises(SystemExit):
            self.patch()


class PackageTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.dist = self.root / "dist"
        self.dist.mkdir()
        (self.dist / WASM).write_bytes(b"\0asm" * 4)
        (self.dist / "index.html").write_text("<html></html>", encoding="utf-8")
        (self.dist / "locale-labels.json").write_text('{"en-US":{"title":"x"}}', encoding="utf-8")
        (self.dist / "videos.json").write_text(
            json.dumps([{"id": "abcdefghijk", "title": "t"}] * 5), encoding="utf-8")
        (self.dist / "discussions.json").write_text(
            json.dumps([{"number": 1, "title": "t", "url": "https://github.com/x/y/discussions/1",
                         "author": "a", "updated_at": "2026-01-01"}] * 3), encoding="utf-8")
        self.license = self.root / "LICENSE"
        self.license.write_text("GPL text\n", encoding="utf-8")
        (self.root / "m.json").write_text(json.dumps({"wasm_url": f"{R2}/sipilcad/web-v9.9.9/{WASM}"}))
        self.out = self.root / "pkg"

    def tearDown(self):
        self.tmp.cleanup()

    def package(self):
        return run(rw.cmd_package, dist=str(self.dist), metrics=str(self.root / "m.json"),
                   out=str(self.out), tag="web-v9.9.9", commit="a" * 40, license=str(self.license))

    def read_pkg(self):
        pkg = self.out / "sipilcad-web-web-v9.9.9.tar.gz"
        with tarfile.open(pkg) as t:
            return {m.name: t.extractfile(m).read().decode("utf-8", "replace") for m in t.getmembers() if m.isfile()}

    def test_tiga_berkas_data_menjadi_daftar_kosong(self):
        out = self.package()
        files = self.read_pkg()
        for name in ("supporters.json", "videos.json", "discussions.json"):
            self.assertEqual(json.loads(files[name]), [], name)
        self.assertIn("supporters.json (baru -> 0)", out)
        self.assertIn("videos.json (5 entri -> 0)", out)
        self.assertIn("discussions.json (3 entri -> 0)", out)

    def test_berkas_lain_tidak_tersentuh(self):
        self.package()
        files = self.read_pkg()
        self.assertEqual(json.loads(files["locale-labels.json"]), {"en-US": {"title": "x"}})
        self.assertIn("LICENSE", files)
        self.assertIn("LICENSE.txt", files)
        self.assertIn("SOURCE.txt", files)
        self.assertNotIn(WASM, files)  # wasm utama tidak ikut paket

    def test_isi_dist_asli_tidak_diubah(self):
        self.package()
        # pengosongan hanya di folder stage; dist (sumber) tetap utuh selain wasm yang memang dihapus
        self.assertEqual(len(json.loads((self.dist / "videos.json").read_text())), 5)
        self.assertEqual(len(json.loads((self.dist / "discussions.json").read_text())), 3)
        self.assertFalse((self.dist / "supporters.json").exists())

    def test_source_txt_mencatat_perubahan_hasil_build(self):
        self.package()
        txt = self.read_pkg()["SOURCE.txt"]
        self.assertIn("Perubahan pada hasil build", txt)
        self.assertIn("supporters.json, videos.json, dan discussions.json berisi daftar kosong", txt)

    def test_berkas_data_rusak_ditimpa_dan_dilaporkan(self):
        (self.dist / "videos.json").write_text("{bukan json", encoding="utf-8")
        out = self.package()
        self.assertEqual(json.loads(self.read_pkg()["videos.json"]), [])
        self.assertIn("videos.json (tidak valid -> 0)", out)

    def test_metrics_mencatat_berkas_yang_dikosongkan(self):
        self.package()
        m = json.loads((self.root / "m.json").read_text())
        self.assertEqual(m["blanked_data"], ["supporters.json", "videos.json", "discussions.json"])


if __name__ == "__main__":
    unittest.main(verbosity=2)

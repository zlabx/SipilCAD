#!/usr/bin/env python3
"""Utilitas rilis web SipilCAD, dipanggil dari .github/workflows/release-web.yml.

Alur: hasil `trunk build` (dist/sipilcad) -> .wasm utama diunggah ke R2 dan URL-nya
di index.html diganti -> sisanya dipaketkan (tar.gz) untuk GitHub Release.
Skrip ini hanya memproses hasil build; kode aplikasi tidak disentuh.
"""
import argparse
import hashlib
import json
import os
import shutil
import sys
import tarfile
import time
import urllib.error
import urllib.request
from pathlib import Path

LIMIT = 25 * 1024 * 1024  # batas per berkas Cloudflare Pages
CACHE_CONTROL = "public, max-age=31536000, immutable"
MiB = 1024 * 1024
# Cloudflare memblokir (403) UA bawaan Python-urllib; UA ini terbukti lolos (probe di CI).
USER_AGENT = "release-check/1.0 (+https://github.com/zlabx)"


def mib(n):
    return f"{n / MiB:.2f} MiB"


def notice(msg):
    print(f"::notice title=release-web::{msg}", flush=True)


def fail(msg):
    print(f"::error title=release-web::{msg}", flush=True)
    raise SystemExit(msg)


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def find_main_wasm(dist):
    found = sorted(Path(dist).glob("*_bg.wasm"))
    if len(found) != 1:
        fail(f"Diharapkan tepat 1 berkas *_bg.wasm di {dist}, ditemukan {len(found)}: {[p.name for p in found]}")
    return found[0]


def load_metrics(path):
    p = Path(path)
    return json.loads(p.read_text()) if p.exists() else {}


def save_metrics(path, data):
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    Path(path).write_text(json.dumps(data, indent=2, sort_keys=True))


# ----------------------------------------------------------------------------- locate
def cmd_locate(a):
    print(find_main_wasm(a.dist).name)


# ----------------------------------------------------------------------------- patch
def cmd_patch(a):
    dist = Path(a.dist)
    wasm = find_main_wasm(dist).name
    index = dist / "index.html"
    text = index.read_text(encoding="utf-8")
    old = f"{a.base_path}{wasm}"
    new = f"{a.public_url.rstrip('/')}/{a.prefix.strip('/')}/{wasm}"
    count = text.count(old)
    if count < 1:
        fail(f"URL wasm '{old}' tidak ditemukan di index.html; pemuat berubah?")
    text = text.replace(old, new)
    if old in text:
        fail("Masih ada sisa URL wasm lama di index.html setelah penggantian")
    index.write_text(text, encoding="utf-8")
    others = []
    for p in sorted(dist.rglob("*")):
        if p.is_file() and p != index and p.suffix in {".js", ".html", ".json", ".css"}:
            if wasm in p.read_text(encoding="utf-8", errors="ignore"):
                others.append(str(p.relative_to(dist)))
    notice(f"index.html: {count} rujukan wasm diganti ke {new}")
    if others:
        notice(f"Berkas lain yang menyebut nama wasm (informasi saja): {', '.join(others)}")
    m = load_metrics(a.metrics)
    m.update({"wasm_name": wasm, "wasm_url": new, "patched_refs": count})
    save_metrics(a.metrics, m)


# ----------------------------------------------------------------------------- package
def source_txt(a, wasm, wasm_url):
    return f"""SipilCAD (build web)
====================

Rilis       : {a.tag}
Commit      : {a.commit}
Kode sumber : https://github.com/zlabx/SipilCAD/tree/{a.commit}
Build       : trunk build --locked --release --public-url /sipilcad/ --html-output index.html web-app.html
              (lihat .github/workflows/release-web.yml di commit yang sama)

Berkas WebAssembly utama ({wasm}) tidak ada di paket ini. Berkas itu dilayani dari
{wasm_url}
dan dibangun dari commit yang sama.

Berdasarkan Open CAD Studio oleh HakanSeven12 dan kontributor:
https://github.com/HakanSeven12/OpenCADStudio  (GPL-3.0)

Lisensi: GNU General Public License v3.0 (lihat berkas LICENSE).
Kode sumber lengkap, termasuk skrip build, tersedia di alamat di atas. Versi dependensi
dikunci oleh Cargo.lock pada commit tersebut.
"""


def cmd_package(a):
    dist = Path(a.dist)
    wasm = find_main_wasm(dist)
    m = load_metrics(a.metrics)
    wasm_url = m.get("wasm_url", "(belum diketahui)")
    out = Path(a.out)
    stage = out / "stage"
    if out.exists():
        shutil.rmtree(out)
    stage.mkdir(parents=True)

    def ignore(directory, names):
        return [n for n in names if Path(directory) == dist and n == wasm.name]

    shutil.copytree(dist, stage, dirs_exist_ok=True, ignore=ignore)
    big = [(str(p.relative_to(stage)), p.stat().st_size) for p in stage.rglob("*")
           if p.is_file() and p.stat().st_size > LIMIT]
    if big:
        fail("Berkas di paket melebihi 25 MiB: " + ", ".join(f"{n} ({mib(s)})" for n, s in big))
    shutil.copyfile(a.license, stage / "LICENSE")
    (stage / "SOURCE.txt").write_text(source_txt(a, wasm.name, wasm_url), encoding="utf-8")

    pkg = out / f"sipilcad-web-{a.tag}.tar.gz"
    with tarfile.open(pkg, "w:gz", compresslevel=9) as t:
        for child in sorted(stage.iterdir()):
            t.add(child, arcname=child.name)
    pkg_sha = sha256_file(pkg)
    (out / f"{pkg.name}.sha256").write_text(f"{pkg_sha}  {pkg.name}\n")
    files = sorted(str(p.relative_to(stage)) for p in stage.rglob("*") if p.is_file())
    biggest = max((p.stat().st_size for p in stage.rglob("*") if p.is_file()), default=0)
    m.update({
        "tag": a.tag, "commit": a.commit,
        "wasm_bytes": wasm.stat().st_size, "wasm_sha256": sha256_file(wasm),
        "package": pkg.name, "package_bytes": pkg.stat().st_size, "package_sha256": pkg_sha,
        "package_files": len(files), "package_biggest_bytes": biggest,
    })
    save_metrics(a.metrics, m)
    notice(f"Paket {pkg.name}: {mib(pkg.stat().st_size)}, {len(files)} berkas, berkas terbesar {mib(biggest)}")
    wasm.unlink()  # sudah di R2; jangan ikut tersisa di dist


# ----------------------------------------------------------------------------- R2
def r2_client():
    os.environ.setdefault("AWS_REQUEST_CHECKSUM_CALCULATION", "when_required")
    os.environ.setdefault("AWS_RESPONSE_CHECKSUM_VALIDATION", "when_required")
    import boto3
    from botocore.config import Config

    endpoint = os.environ.get("R2_ENDPOINT_URL") or f"https://{os.environ['R2_ACCOUNT_ID']}.r2.cloudflarestorage.com"
    return boto3.client(
        "s3",
        endpoint_url=endpoint,
        aws_access_key_id=os.environ["R2_ACCESS_KEY_ID"],
        aws_secret_access_key=os.environ["R2_SECRET_ACCESS_KEY"],
        region_name="auto",
        config=Config(signature_version="s3v4", retries={"max_attempts": 5, "mode": "standard"}),
    )


def cmd_upload(a):
    from boto3.s3.transfer import TransferConfig

    wasm = find_main_wasm(a.dist)
    key = f"{a.prefix.strip('/')}/{wasm.name}"
    size = wasm.stat().st_size
    s3 = r2_client()
    # satu PutObject (bukan multipart) agar sederhana; batas R2 jauh di atas 53 MiB
    s3.upload_file(
        str(wasm), a.bucket, key,
        ExtraArgs={"ContentType": "application/wasm", "CacheControl": CACHE_CONTROL},
        Config=TransferConfig(multipart_threshold=512 * MiB),
    )
    head = s3.head_object(Bucket=a.bucket, Key=key)
    if head["ContentLength"] != size:
        fail(f"Ukuran di R2 ({head['ContentLength']}) berbeda dari lokal ({size})")
    if head.get("ContentType") != "application/wasm":
        fail(f"Content-Type di R2 salah: {head.get('ContentType')}")
    notice(f"Terunggah ke R2: {key} ({mib(size)})")


def prune_plan(versions, keep, current):
    """versions: {nama: LastModified terbaru}. Kembalikan (dipertahankan, dihapus)."""
    order = sorted(versions, key=lambda n: versions[n], reverse=True)
    kept = list(order[:keep])
    if current in versions and current not in kept:
        kept.append(current)
    removed = [n for n in order if n not in kept]
    return kept, removed


def cmd_cleanup(a):
    root = a.root
    if not root.startswith("sipilcad/") or not root.endswith("/"):
        fail(f"Root pembersihan tidak aman: {root!r}")
    s3 = r2_client()
    versions, keys = {}, {}
    for page in s3.get_paginator("list_objects_v2").paginate(Bucket=a.bucket, Prefix=root):
        for o in page.get("Contents", []):
            rest = o["Key"][len(root):]
            if "/" not in rest:
                continue
            name = rest.split("/", 1)[0]
            versions[name] = max(versions.get(name, o["LastModified"]), o["LastModified"])
            keys.setdefault(name, []).append(o["Key"])
    kept, removed = prune_plan(versions, a.keep, a.current)
    notice(f"R2 versi dipertahankan: {kept}; dihapus: {removed}")
    if a.dry:
        return
    for name in removed:
        ks = keys[name]
        for i in range(0, len(ks), 1000):
            s3.delete_objects(Bucket=a.bucket, Delete={"Objects": [{"Key": k} for k in ks[i:i + 1000]]})


# ----------------------------------------------------------------------------- verify-remote
def _request(url, method, headers):
    req = urllib.request.Request(url, method=method, headers={"User-Agent": USER_AGENT, **headers})
    return urllib.request.urlopen(req, timeout=180)


def cmd_verify_remote(a):
    wasm = find_main_wasm(a.dist)
    url = load_metrics(a.metrics).get("wasm_url")
    if not url:
        fail("wasm_url belum ada di metrics; jalankan patch dulu")
    size = wasm.stat().st_size
    sha = sha256_file(wasm)
    origin = {"Origin": a.origin}

    last = None
    for attempt in range(1, 6):
        try:
            with _request(url, "HEAD", {**origin, "Accept-Encoding": "identity"}) as r:
                h = {k.lower(): v for k, v in r.headers.items()}
                if r.status != 200:
                    raise RuntimeError(f"HEAD status {r.status}")
            break
        except (urllib.error.URLError, RuntimeError) as e:
            last = e
            time.sleep(5)
    else:
        fail(f"HEAD ke {url} gagal: {last}")

    if not h.get("content-type", "").startswith("application/wasm"):
        fail(f"Content-Type salah: {h.get('content-type')}")
    acao = h.get("access-control-allow-origin")
    if acao not in ("*", a.origin):
        fail(f"Header CORS tidak ada atau salah: {acao}")
    if int(h.get("content-length", -1)) != size:
        fail(f"Content-Length {h.get('content-length')} != ukuran lokal {size}")

    # isi harus identik dengan berkas lokal
    digest, got = hashlib.sha256(), 0
    with _request(url, "GET", {**origin, "Accept-Encoding": "identity"}) as r:
        for chunk in iter(lambda: r.read(1024 * 1024), b""):
            digest.update(chunk)
            got += len(chunk)
    if digest.hexdigest() != sha:
        fail("SHA-256 unduhan dari R2 berbeda dari berkas lokal")

    # kompresi (informasi; tidak menggagalkan)
    enc, comp = None, None
    try:
        with _request(url, "GET", {**origin, "Accept-Encoding": "br"}) as r:
            enc = r.headers.get("Content-Encoding")
            comp = sum(len(c) for c in iter(lambda: r.read(1024 * 1024), b""))
    except urllib.error.URLError as e:
        print(f"::warning title=release-web::Uji kompresi gagal: {e}", flush=True)
    if not enc:
        print("::warning title=release-web::Respons wasm dari R2 tidak terkompresi (tanpa Content-Encoding)", flush=True)

    m = load_metrics(a.metrics)
    m.update({"remote_ok": True, "remote_encoding": enc, "remote_compressed_bytes": comp,
              "remote_cache_status": h.get("cf-cache-status")})
    save_metrics(a.metrics, m)
    notice(f"R2 OK: 200, application/wasm, CORS={acao}, sha256 cocok, "
           f"asli {mib(size)}, terkompresi {mib(comp) if comp else '-'} ({enc or 'tanpa kompresi'})")


# ----------------------------------------------------------------------------- notes
def cmd_notes(a):
    m = load_metrics(a.metrics)
    comp = m.get("remote_compressed_bytes")
    lines = [
        f"## SipilCAD web `{m.get('tag')}`",
        "",
        f"- Commit: `{m.get('commit')}`",
        f"- Paket: `{m.get('package')}` ({mib(m.get('package_bytes', 0))}, {m.get('package_files')} berkas, SHA-256 `{m.get('package_sha256')}`)",
        f"- WebAssembly utama: `{m.get('wasm_name')}` ({mib(m.get('wasm_bytes', 0))}), tidak ada di paket; dilayani dari R2:",
        f"  {m.get('wasm_url')}",
        f"- Terkompresi saat diunduh: {mib(comp) if comp else 'tidak terukur'} ({m.get('remote_encoding') or 'tanpa kompresi'})",
        "",
        "Paket ini dipasang ke `sipilcad/` di SipilStock oleh skrip build di repo utama.",
        "Sumber dan lisensi: lihat `SOURCE.txt` dan `LICENSE` di dalam paket (GPL-3.0, turunan Open CAD Studio).",
    ]
    print("\n".join(lines))


def main():
    p = argparse.ArgumentParser(description=__doc__)
    sub = p.add_subparsers(dest="cmd", required=True)

    def add(name, fn, **flags):
        sp = sub.add_parser(name)
        for flag, kw in flags.items():
            sp.add_argument("--" + flag.replace("_", "-"), **kw)
        sp.set_defaults(fn=fn)
        return sp

    d = {"default": "dist/sipilcad"}
    mt = {"default": "build/metrics.json"}
    add("locate", cmd_locate, dist=d)
    add("patch", cmd_patch, dist=d, metrics=mt, base_path={"default": "/sipilcad/"},
        public_url={"required": True}, prefix={"required": True})
    add("package", cmd_package, dist=d, metrics=mt, out={"default": "build/package"},
        tag={"required": True}, commit={"required": True}, license={"default": "LICENSE"})
    add("upload", cmd_upload, dist=d, bucket={"required": True}, prefix={"required": True})
    add("verify-remote", cmd_verify_remote, dist=d, metrics=mt, origin={"default": "https://sipilstock.com"})
    add("cleanup", cmd_cleanup, bucket={"required": True}, root={"default": "sipilcad/"},
        keep={"type": int, "default": 3}, current={"required": True}).add_argument("--dry", action="store_true")
    add("notes", cmd_notes, metrics=mt)
    a = p.parse_args()
    a.fn(a)


if __name__ == "__main__":
    main()

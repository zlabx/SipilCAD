#!/usr/bin/env python3
"""Jalur darurat dependensi git SipilCAD: alihkan sumber git ke mirror (zlabx/*-mirror).

Dipakai HANYA bila repo upstream (iced, winit, cryoglyph, iced_aw, opencad*) hilang atau di-force-push.
Build normal tetap memakai upstream. Pemetaan ada di .github/mirrors.json.

Perintah (dijalankan dari akar repo):
  check [--offline]   pemetaan menutupi semua sumber git di Cargo.lock, pin sama dengan commit di
                      Cargo.lock, dan (tanpa --offline) tiap commit bisa diambil dari mirror lewat git
  apply               ubah manifest (root + crates/*): URL git -> mirror, branch/rev -> rev penuh,
                      dan tambahkan [patch] untuk dependensi tidak langsung (winit, cryoglyph).
                      Cargo.lock TIDAK diedit tangan: biarkan cargo menulis ulang (cargo fetch)
  verify-lock --base F  setelah cargo menulis ulang lock: versi paket dan commit git harus identik
                      dengan lock asli F, dan tidak ada lagi sumber upstream

Urutan darurat (lihat README): check -> apply -> cargo fetch -> verify-lock --base <lock lama> -> build.
Hanya pustaka standar Python.
"""
import argparse
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

MIRRORS = Path(".github/mirrors.json")


def fail(msg):
    print(f"::error title=use_mirrors::{msg}", flush=True)
    raise SystemExit(msg)


def norm(url):
    return re.sub(r"(\.git)?/?$", "", url.strip()).lower()


def load_mirrors():
    return json.loads(MIRRORS.read_text(encoding="utf-8"))["mirrors"]


def lock_packages(text):
    """-> daftar (nama, versi, (url_git_ternormalisasi, sha) | None); duplikat tetap tercatat"""
    out = []
    for blk in text.split("[[package]]")[1:]:
        n = re.search(r'^name = "([^"]+)"', blk, re.M)
        v = re.search(r'^version = "([^"]+)"', blk, re.M)
        if not (n and v):
            continue
        s = re.search(r'^source = "git\+([^"?#]+)(?:\?[^"#]*)?#([0-9a-f]{40})"', blk, re.M)
        out.append((n.group(1), v.group(1), (norm(s.group(1)), s.group(2)) if s else None))
    return out


def git_refs_in_manifest(doc):
    """Dependensi bergit di sebuah manifest, tanpa bagian [patch]/[replace] (diabaikan Cargo pada non-root)."""
    def walk(d, key):
        if isinstance(d, dict):
            if isinstance(d.get("git"), str):
                yield norm(d["git"]), d.get("package") or key
            for k, v in d.items():
                if k in ("patch", "replace"):
                    continue
                yield from walk(v, k)
        elif isinstance(d, list):
            for v in d:
                yield from walk(v, key)
    return set(walk(doc, ""))


def workspace_manifests():
    return [Path("Cargo.toml")] + sorted(Path("crates").glob("*/Cargo.toml"))


# ----------------------------------------------------------------------------- check
def cmd_check(a):
    ms = load_mirrors()
    by = {norm(m["upstream"]): m for m in ms}
    pk = lock_packages(Path("Cargo.lock").read_text(encoding="utf-8"))
    problems, used = [], set()
    for n, v, src in sorted(pk):
        if not src:
            continue
        url, sha = src
        used.add(url)
        m = by.get(url)
        if not m:
            problems.append(f"sumber git {url} (paket {n}) tidak ada di mirrors.json")
        elif m["pin"] != sha:
            problems.append(f"pin {m['pin'][:8]} di mirrors.json beda dari commit {sha[:8]} di Cargo.lock untuk {url}: perbarui mirror lalu mirrors.json")
    for u in by:
        if u not in used:
            print(f"::warning title=use_mirrors::{u} ada di mirrors.json tetapi tidak dipakai Cargo.lock")
    if not a.offline:
        import tomllib
        needed = {}  # url upstream -> {nama paket: dirujuk dari}
        for m in ms:
            url = m["mirror"].rstrip("/") + ".git"
            with tempfile.TemporaryDirectory() as td:
                subprocess.run(["git", "init", "-q", td], check=True)
                f = subprocess.run(["git", "-C", td, "fetch", "-q", "--depth=1", url, m["pin"]], capture_output=True, text=True)
                ok = f.returncode == 0 and subprocess.run(["git", "-C", td, "cat-file", "-e", m["pin"] + "^{commit}"]).returncode == 0
                print(f"  mirror {url}: commit {m['pin'][:8]} {'bisa diambil' if ok else 'TIDAK bisa diambil'}")
                if not ok:
                    problems.append(f"commit {m['pin'][:8]} tidak bisa diambil dari {url}")
                    continue
                subprocess.run(["git", "-C", td, "checkout", "-q", "FETCH_HEAD"], capture_output=True)
                for cargo in Path(td).rglob("Cargo.toml"):
                    try:
                        doc = tomllib.loads(cargo.read_text(encoding="utf-8"))
                    except Exception:
                        continue
                    for tgt, name in git_refs_in_manifest(doc):
                        if tgt in by and tgt != norm(m["upstream"]):
                            needed.setdefault(tgt, {}).setdefault(name, norm(m["upstream"]).split("github.com/")[-1])
        for tgt, names in sorted(needed.items()):
            have = set(by[tgt].get("patch", []))
            for name, frm in sorted(names.items()):
                if name not in have:
                    problems.append(f"{frm} merujuk {name} dari {tgt} lewat git, tetapi '{name}' belum ada di daftar 'patch' mirrors.json (akan muncul dua salinan crate)")
    if problems:
        for p in problems:
            print(f"::error title=use_mirrors::{p}")
        raise SystemExit(1)
    print(f"check OK: {len(used)} sumber git tertutup pemetaan, pin sama dengan Cargo.lock" + ("" if a.offline else ", semua commit terambil dari mirror, semua rujukan tidak langsung tercakup daftar patch"))


# ----------------------------------------------------------------------------- apply
def cmd_apply(a):
    ms = load_mirrors()
    manifests = workspace_manifests()
    texts = {p: p.read_text(encoding="utf-8") for p in manifests}
    n_lines = 0
    for m in ms:
        url_pat = re.compile(r'(git\s*=\s*")' + re.escape(m["upstream"]) + r'(?:\.git)?(")')
        new_url = m["mirror"].rstrip("/") + ".git"
        for p, t in texts.items():
            out = []
            for ln in t.split("\n"):
                if url_pat.search(ln):
                    ln = url_pat.sub(lambda mo: mo.group(1) + new_url + mo.group(2), ln)
                    # tentukan satu commit pasti: ganti branch/rev apa pun dengan rev penuh
                    if re.search(r'\b(branch|rev|tag)\s*=\s*"[^"]*"', ln):
                        ln = re.sub(r'\b(branch|rev|tag)\s*=\s*"[^"]*"', f'rev = "{m["pin"]}"', ln, count=1)
                    else:
                        ln = url_pat.sub(lambda mo: mo.group(0) + f', rev = "{m["pin"]}"', ln, count=1)
                    n_lines += 1
                out.append(ln)
            texts[p] = "\n".join(out)
    # dependensi tidak langsung: [patch] di manifest root
    root = Path("Cargo.toml")
    for m in ms:
        pk = m.get("patch")
        if not pk:
            continue
        header = f'[patch."{m["upstream"]}.git"]'
        if header in texts[root]:
            continue
        block = "\n# Jalur darurat (use_mirrors.py): dependensi tidak langsung dialihkan ke mirror.\n" + header + "\n"
        for name in pk:
            block += f'{name} = {{ git = "{m["mirror"].rstrip("/")}.git", rev = "{m["pin"]}" }}\n'
        texts[root] = texts[root].rstrip("\n") + "\n" + block
    for p, t in texts.items():
        p.write_text(t, encoding="utf-8")
    left = [str(p) for p, t in texts.items() for m in ms if re.search(r'git\s*=\s*"' + re.escape(m["upstream"]), t)]
    if left:
        fail("masih ada rujukan upstream di manifest: " + ", ".join(left))
    print(f"apply OK: {n_lines} baris dependensi dialihkan ke mirror di {len(manifests)} manifest; [patch] ditambahkan untuk dependensi tidak langsung")


# ----------------------------------------------------------------------------- verify-lock
def cmd_verify_lock(a):
    from collections import Counter
    ms = load_mirrors()
    old = lock_packages(Path(a.base).read_text(encoding="utf-8"))
    new = lock_packages(Path("Cargo.lock").read_text(encoding="utf-8"))
    co, cn = Counter((n, v) for n, v, _ in old), Counter((n, v) for n, v, _ in new)
    problems = []
    for k in sorted(set(co) | set(cn)):
        if co[k] != cn[k]:
            what = "DUPLIKAT (dua salinan crate yang sama dari sumber berbeda)" if cn[k] > co[k] else "berubah jumlahnya"
            problems.append(f"{k[0]} {k[1]}: {co[k]} entri di lock asli, {cn[k]} di lock baru; {what}")
    oldsrc = {(n, v): s for n, v, s in old}
    for n, v, src in new:
        o = oldsrc.get((n, v))
        if o and src and o[1] != src[1]:
            problems.append(f"commit berubah untuk {n} {v}: {o[1][:12]} -> {src[1][:12]}")
    ups = {norm(m["upstream"]) for m in ms}
    mir = {norm(m["mirror"]) for m in ms}
    for n, v, src in new:
        if src and src[0] in ups:
            problems.append(f"{n} {v} masih bersumber dari upstream {src[0]}")
        elif src and src[0] not in mir:
            problems.append(f"{n} {v} bersumber dari git tak dikenal {src[0]}")
    if problems:
        for p in problems:
            print(f"::error title=use_mirrors::{p}")
        raise SystemExit(1)
    ng = sum(1 for _, _, s in new if s)
    print(f"verify-lock OK: {len(new)} entri, versi identik dengan lock asli tanpa duplikat; {ng} paket git kini dari mirror dengan commit yang sama")


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("check"); c.add_argument("--offline", action="store_true"); c.set_defaults(fn=cmd_check)
    sub.add_parser("apply").set_defaults(fn=cmd_apply)
    v = sub.add_parser("verify-lock"); v.add_argument("--base", required=True); v.set_defaults(fn=cmd_verify_lock)
    a = p.parse_args()
    a.fn(a)


if __name__ == "__main__":
    main()

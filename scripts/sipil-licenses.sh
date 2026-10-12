#!/usr/bin/env bash
# SIPIL: buat THIRD_PARTY_LICENSES.txt (lisensi pustaka Rust yang dikompilasi ke wasm) dengan cargo-about.
# Pakai: scripts/sipil-licenses.sh <folder hasil build>   (dijalankan dari akar kode sumber)
# Konfigurasi: about.toml (lisensi yang diterima) dan about.hbs (templat). Biner cargo-about diunduh
# dari rilis GitHub dan diverifikasi terhadap SHA-256 yang disematkan di bawah.
set -euo pipefail

CARGO_ABOUT_VERSION="0.9.2"
CARGO_ABOUT_SHA256="9099a59e820c38a68b9d65f300662a567d56562f9a10f6aa4c7e86c17c2566af"
ARCHIVE="cargo-about-${CARGO_ABOUT_VERSION}-x86_64-unknown-linux-musl.tar.gz"
URL="https://github.com/EmbarkStudios/cargo-about/releases/download/${CARGO_ABOUT_VERSION}/${ARCHIVE}"

OUT_DIR="$(cd "${1:?usage: sipil-licenses.sh <folder hasil build>}" && pwd)"
for f in about.toml about.hbs Cargo.lock; do
  [ -f "$f" ] || { echo "sipil-licenses: $f tidak ada di $(pwd)" >&2; exit 1; }
done

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
curl -fsSL --retry 3 --retry-delay 2 -o "$TMP/$ARCHIVE" "$URL" \
  || { echo "sipil-licenses: gagal mengunduh $URL" >&2; exit 1; }
echo "${CARGO_ABOUT_SHA256}  $TMP/$ARCHIVE" | sha256sum -c - >/dev/null \
  || { echo "sipil-licenses: SHA-256 cargo-about tidak cocok" >&2; exit 1; }
tar -xzf "$TMP/$ARCHIVE" -C "$TMP"
BIN="$(find "$TMP" -type f -name cargo-about | head -1)"
[ -n "$BIN" ] || { echo "sipil-licenses: biner cargo-about tidak ada di arsip" >&2; exit 1; }
chmod +x "$BIN"

"$BIN" generate --locked --workspace about.hbs -o "$OUT_DIR/THIRD_PARTY_LICENSES.txt"
test -s "$OUT_DIR/THIRD_PARTY_LICENSES.txt"
echo "sipil-licenses: $(wc -c < "$OUT_DIR/THIRD_PARTY_LICENSES.txt") byte (cargo-about ${CARGO_ABOUT_VERSION})"

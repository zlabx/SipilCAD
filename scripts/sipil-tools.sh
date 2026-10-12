#!/usr/bin/env bash
# SIPIL: pasang wasm-bindgen CLI pada versi yang SAMA dengan Cargo.lock.
# Hook post_build di Trunk.toml menjalankan scripts/build-web-worker.sh, yang memanggil `wasm-bindgen`
# dari PATH; versinya harus cocok dengan crate wasm-bindgen di Cargo.lock.
# Pakai (dari akar kode sumber): bash scripts/sipil-tools.sh
# Mencoba biner siap pakai dari rilis GitHub dulu; bila gagal, cargo install --locked dari crates.io.
set -euo pipefail

VERSION="$(sed -n '/^name = "wasm-bindgen"$/{n;s/^version = "\(.*\)"$/\1/p;q;}' Cargo.lock)"
[ -n "$VERSION" ] || { echo "sipil-tools: versi wasm-bindgen tidak ditemukan di Cargo.lock" >&2; exit 1; }
BIN_DIR="${CARGO_HOME:-$HOME/.cargo}/bin"
mkdir -p "$BIN_DIR"

if command -v wasm-bindgen >/dev/null 2>&1 && [ "$(wasm-bindgen --version | awk '{print $2}')" = "$VERSION" ]; then
  echo "sipil-tools: wasm-bindgen $VERSION sudah terpasang"
  exit 0
fi

NAME="wasm-bindgen-${VERSION}-x86_64-unknown-linux-musl"
URL="https://github.com/rustwasm/wasm-bindgen/releases/download/${VERSION}/${NAME}.tar.gz"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
if curl -fsSL --retry 3 --retry-delay 2 -o "$TMP/wb.tar.gz" "$URL" && tar -xzf "$TMP/wb.tar.gz" -C "$TMP" \
   && [ -x "$TMP/$NAME/wasm-bindgen" ]; then
  install -m 0755 "$TMP/$NAME/wasm-bindgen" "$BIN_DIR/wasm-bindgen"
  echo "sipil-tools: wasm-bindgen $VERSION dari rilis GitHub"
else
  echo "sipil-tools: unduhan biner gagal; cargo install wasm-bindgen-cli@${VERSION} (lebih lama)" >&2
  cargo install --locked --version "$VERSION" wasm-bindgen-cli
fi
GOT="$(wasm-bindgen --version | awk '{print $2}')"
[ "$GOT" = "$VERSION" ] || { echo "sipil-tools: terpasang $GOT, diharapkan $VERSION" >&2; exit 1; }

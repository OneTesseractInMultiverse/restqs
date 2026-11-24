#!/bin/sh
# Install a checksummed registry tool with its reviewed manifest and lockfile.
# Run from the checkout root with a crate name, exact version and archive SHA-256.
set -eu
tool=$1
package=$tool-$2
checksum=$3
settings=$PWD/tools/maintenance/$tool
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
trap 'exit 1' HUP INT TERM

curl --fail --silent --show-error --location --proto '=https' \
    "https://static.crates.io/crates/$tool/$package.crate" -o "$scratch/source.crate"
(cd "$scratch" && printf '%s  source.crate\n' "$checksum" | shasum -a 256 -c -)
tar -xzf "$scratch/source.crate" -C "$scratch"
if [ -f "$settings/manifest.patch" ]; then
    (cd "$scratch/$package" && patch -p0 < "$settings/manifest.patch")
fi
cp "$settings/Cargo.lock" "$scratch/$package/Cargo.lock"
cargo +1.98.1 install --path "$scratch/$package" --locked --force \
    --target-dir "${CARGO_TARGET_DIR:-$PWD/target}"

flatpak-sources:
    curl -fsSL https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py -o /tmp/flatpak-cargo-generator.py
    python3 /tmp/flatpak-cargo-generator.py Cargo.lock -o packaging/flatpak/cargo-sources.json

flatpak-manifest:
    appstreamcli validate --pedantic --explain resources/com.github.drwesleyadv.Ticker.metainfo.xml

# Baseline checks supported by the system Rust toolchain used on the development host.
check:
    cargo test --locked
    cargo build --release --locked

# Optional quality gate. Install rustfmt and clippy through the distribution packages
# (or another toolchain manager) before running this recipe.
quality:
    @command -v rustfmt >/dev/null || { echo "rustfmt is not installed"; exit 1; }
    @cargo clippy --version >/dev/null 2>&1 || { echo "clippy is not installed"; exit 1; }
    cargo fmt --check
    cargo clippy --all-targets --locked -- -D warnings

ci: quality
    cargo test --locked
    cargo build --release --locked

flatpak-sources:
    curl -fsSL https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py -o /tmp/flatpak-cargo-generator.py
    python3 /tmp/flatpak-cargo-generator.py Cargo.lock -o packaging/flatpak/cargo-sources.json

flatpak-manifest:
    appstreamcli validate --pedantic --explain resources/com.github.drwesleyadv.Ticker.metainfo.xml

check:
    cargo test --locked
    cargo build --release --locked

quality:
    @if command -v rustfmt >/dev/null 2>&1; then cargo fmt --check; else echo "rustfmt not installed; skipping format check"; fi
    @if cargo clippy --version >/dev/null 2>&1; then cargo clippy --locked --all-targets -- -D warnings; else echo "clippy not installed; skipping lint check"; fi

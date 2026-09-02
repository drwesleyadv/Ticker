flatpak-sources:
    curl -fsSL https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py -o /tmp/flatpak-cargo-generator.py
    python3 /tmp/flatpak-cargo-generator.py Cargo.lock -o packaging/flatpak/cargo-sources.json

flatpak-manifest:
    appstreamcli validate --pedantic --explain resources/com.github.drwesleyadv.Ticker.metainfo.xml

check: 
    cargo fmt --check
    cargo test --locked
    cargo build --release --locked

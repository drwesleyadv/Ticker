#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
APP_ID="com.github.drwesleyadv.Ticker"

if [[ ${EUID} -ne 0 ]]; then
    echo "Execute o instalador com privilégios administrativos:" >&2
    echo "  sudo bash install.sh" >&2
    exit 1
fi

BINARY="${ROOT_DIR}/bin/ticker"
if [[ ! -x "${BINARY}" ]]; then
    BINARY="${ROOT_DIR}/target/release/ticker"
fi

if [[ ! -x "${BINARY}" ]]; then
    echo "Binário do Ticker não encontrado." >&2
    echo "Se estiver usando o código-fonte, compile primeiro como usuário normal:" >&2
    echo "  cargo build --release --locked" >&2
    echo "Depois execute novamente:" >&2
    echo "  sudo bash install.sh" >&2
    exit 1
fi

install -Dm0755 "${BINARY}" /usr/local/bin/ticker
install -Dm0644 "${ROOT_DIR}/resources/${APP_ID}.desktop" "/usr/local/share/applications/${APP_ID}.desktop"
install -Dm0644 "${ROOT_DIR}/resources/${APP_ID}.metainfo.xml" "/usr/local/share/metainfo/${APP_ID}.metainfo.xml"
install -Dm0644 "${ROOT_DIR}/resources/${APP_ID}.svg" "/usr/local/share/icons/hicolor/scalable/apps/${APP_ID}.svg"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/local/share/applications >/dev/null 2>&1 || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t /usr/local/share/icons/hicolor >/dev/null 2>&1 || true
fi

echo "Ticker instalado com sucesso em /usr/local."
echo "Reinicie a sessão/painel COSMIC e adicione 'Solana Ticker' aos applets do painel."

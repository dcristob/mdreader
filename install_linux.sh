#!/usr/bin/env bash
# mdreader Linux installer
#
# Installs the latest mdreader release into ~/.local with desktop entry,
# icon, and MIME associations so .md / .mdx files open by double-click.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/dcristob/mdreader/main/install_linux.sh | bash
#
# Environment overrides:
#   MDREADER_VERSION   release tag to install (default: latest)
#   MDREADER_PREFIX    install prefix (default: $HOME/.local)

set -euo pipefail

REPO="dcristob/mdreader"
PREFIX="${MDREADER_PREFIX:-$HOME/.local}"
VERSION="${MDREADER_VERSION:-latest}"

BIN_DIR="$PREFIX/bin"
APP_DIR="$PREFIX/share/applications"
ICON_DIR="$PREFIX/share/icons"

arch="$(uname -m)"
case "$arch" in
    x86_64|amd64) asset="mdreader-linux-x86_64" ;;
    *)
        echo "error: unsupported architecture '$arch' (only x86_64 binaries are published)" >&2
        exit 1
        ;;
esac

if [ "$VERSION" = "latest" ]; then
    api_url="https://api.github.com/repos/$REPO/releases/latest"
else
    api_url="https://api.github.com/repos/$REPO/releases/tags/$VERSION"
fi

echo "==> Resolving release ($VERSION)"
tag="$(curl -fsSL "$api_url" | grep -m1 '"tag_name":' | sed -E 's/.*"tag_name":[[:space:]]*"([^"]+)".*/\1/')"
if [ -z "$tag" ]; then
    echo "error: could not resolve release tag from $api_url" >&2
    exit 1
fi
echo "    tag: $tag"

bin_url="https://github.com/$REPO/releases/download/$tag/$asset"
desktop_url="https://raw.githubusercontent.com/$REPO/$tag/mdreader.desktop"
icon_url="https://raw.githubusercontent.com/$REPO/$tag/markdown.png"

mkdir -p "$BIN_DIR" "$APP_DIR" "$ICON_DIR"

echo "==> Installing binary to $BIN_DIR/mdreader"
curl -fsSL "$bin_url" -o "$BIN_DIR/mdreader"
chmod +x "$BIN_DIR/mdreader"

echo "==> Installing desktop entry to $APP_DIR/mdreader.desktop"
curl -fsSL "$desktop_url" -o "$APP_DIR/mdreader.desktop"

echo "==> Installing icon to $ICON_DIR/mdreader.png"
curl -fsSL "$icon_url" -o "$ICON_DIR/mdreader.png"

if command -v xdg-mime >/dev/null 2>&1; then
    echo "==> Registering MIME associations"
    xdg-mime default mdreader.desktop text/markdown text/x-markdown text/x-mdx || true
else
    echo "    xdg-mime not found; skipping MIME association"
fi

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR" >/dev/null 2>&1 || true
fi

case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *) echo "note: $BIN_DIR is not on your PATH — add it to your shell rc to run 'mdreader' directly" ;;
esac

echo "==> Done. Installed mdreader $tag."

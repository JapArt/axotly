#!/usr/bin/env bash

set -e

REPO="JapArt/axotly"
BINARY="axotly"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

OS="$(uname -s)"
ARCH="$(uname -m)"

# Detect OS
case "$OS" in
Linux) OS="linux" ;;
Darwin) OS="macos" ;;
*)
  echo "Unsupported OS: $OS"
  exit 1
  ;;
esac

# Detect architecture
case "$ARCH" in
x86_64) ARCH="x86_64" ;;
arm64 | aarch64) ARCH="arm64" ;;
*)
  echo "Unsupported architecture: $ARCH"
  exit 1
  ;;
esac

FILENAME="${BINARY}-${OS}-${ARCH}"
URL="https://github.com/${REPO}/releases/latest/download/${FILENAME}"

echo "Installing Axotly"
echo "OS: $OS"
echo "Architecture: $ARCH"
echo "Downloading: $URL"

mkdir -p "$INSTALL_DIR"

curl -fsSL "$URL" -o "${INSTALL_DIR}/${BINARY}"
chmod +x "${INSTALL_DIR}/${BINARY}"

echo "Axotly installed to ${INSTALL_DIR}/${BINARY}"

if ! command -v axotly >/dev/null; then
  echo "Make sure ${INSTALL_DIR} is in your PATH"
  echo "export PATH=\"\$PATH:${INSTALL_DIR}\""
fi

echo "Run: axotly --help"

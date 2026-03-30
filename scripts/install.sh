#!/usr/bin/env bash
# Forge installer — https://forgejs.com/install
# Usage: curl -fsSL https://forgejs.com/install.sh | sh

set -euo pipefail

FORGE_VERSION="${FORGE_VERSION:-latest}"
INSTALL_DIR="${FORGE_INSTALL_DIR:-$HOME/.forge/bin}"

main() {
    detect_platform
    download_binary
    install_binary
    print_success
}

detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$OS" in
        linux)
            case "$ARCH" in
                x86_64) PLATFORM="linux-x64" ;;
                aarch64) PLATFORM="linux-arm64" ;;
                *) die "Unsupported architecture: $ARCH" ;;
            esac
            ;;
        darwin)
            case "$ARCH" in
                x86_64) PLATFORM="macos-x64" ;;
                arm64) PLATFORM="macos-arm64" ;;
                *) die "Unsupported architecture: $ARCH" ;;
            esac
            ;;
        *)
            die "Unsupported OS: $OS. Install Forge from source: https://github.com/100monkeys-ai/forgejs"
            ;;
    esac
}

download_binary() {
    if [ "$FORGE_VERSION" = "latest" ]; then
        DOWNLOAD_URL="https://github.com/100monkeys-ai/forgejs/releases/latest/download/forge-${PLATFORM}"
    else
        DOWNLOAD_URL="https://github.com/100monkeys-ai/forgejs/releases/download/${FORGE_VERSION}/forge-${PLATFORM}"
    fi

    echo "Downloading Forge from ${DOWNLOAD_URL}..."
    TMP_FILE=$(mktemp)
    curl -fsSL "$DOWNLOAD_URL" -o "$TMP_FILE"
    chmod +x "$TMP_FILE"
    BINARY="$TMP_FILE"
}

install_binary() {
    mkdir -p "$INSTALL_DIR"
    mv "$BINARY" "$INSTALL_DIR/forge"
    echo "Installed forge to $INSTALL_DIR/forge"
}

print_success() {
    echo ""
    echo "Forge installed successfully!"
    echo ""
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo "Add Forge to your PATH by adding this to your shell profile:"
        echo "  export PATH=\"\$HOME/.forge/bin:\$PATH\""
        echo ""
    fi
    echo "Get started:"
    echo "  forge new my-app"
    echo "  cd my-app && forge dev"
    echo ""
    echo "Documentation: https://forgejs.com/docs"
}

die() {
    echo "Error: $1" >&2
    exit 1
}

main

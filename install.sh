#!/bin/sh
set -e

REPO="ErickJ3/faze"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

get_latest_release() {
    curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux*)
            case "$ARCH" in
                x86_64) echo "linux-x86_64" ;;
                *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
            esac
            ;;
        Darwin*)
            case "$ARCH" in
                x86_64) echo "macos-x86_64" ;;
                arm64) echo "macos-aarch64" ;;
                *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
            esac
            ;;
        *)
            echo "Unsupported OS: $OS" >&2
            exit 1
            ;;
    esac
}

main() {
    echo "Detecting platform..."
    PLATFORM=$(detect_platform)
    echo "Platform: $PLATFORM"

    echo "Getting latest release..."
    VERSION=$(get_latest_release)
    echo "Latest version: $VERSION"

    ASSET_NAME="faze-${PLATFORM}"
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/${ASSET_NAME}.tar.gz"

    echo "Downloading from $DOWNLOAD_URL..."
    TEMP_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_DIR" EXIT

    curl -L -o "$TEMP_DIR/faze.tar.gz" "$DOWNLOAD_URL"

    echo "Extracting..."
    tar -xzf "$TEMP_DIR/faze.tar.gz" -C "$TEMP_DIR"

    echo "Installing to $INSTALL_DIR..."
    mkdir -p "$INSTALL_DIR"
    mv "$TEMP_DIR/faze" "$INSTALL_DIR/faze"
    chmod +x "$INSTALL_DIR/faze"

    echo ""
    echo "faze installed successfully to $INSTALL_DIR/faze"
    echo ""

    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        echo "NOTE: $INSTALL_DIR is not in your PATH."
        echo "Add it by running:"
        echo ""
        echo "  echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> ~/.bashrc"
        echo "  source ~/.bashrc"
        echo ""
        echo "Or for zsh users:"
        echo "  echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> ~/.zshrc"
        echo "  source ~/.zshrc"
    else
        echo "Run 'faze --help' to get started!"
    fi
}

main

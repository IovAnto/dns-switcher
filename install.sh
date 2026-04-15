#!/usr/bin/env bash
#
# DNS Switcher Installer
# https://github.com/IovAnto/dns-switcher
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/IovAnto/dns-switcher/main/install.sh | bash
#
# Options:
#   --uninstall    Remove dns-switcher
#   --help         Show this help

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

REPO="IovAnto/dns-switcher"
BINARY_NAME="dns-switcher"
INSTALL_DIR="/usr/local/bin"

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

check_deps() {
    local missing=()
    
    if ! command -v curl &> /dev/null && ! command -v wget &> /dev/null; then
        missing+=("curl or wget")
    fi
    
    if ! command -v tar &> /dev/null; then
        missing+=("tar")
    fi
    
    if ! command -v resolvectl &> /dev/null; then
        warn "resolvectl not found. dns-switcher requires systemd-resolved to function."
    fi

    if ! command -v iwctl &> /dev/null; then
        warn "iwctl (iwd) not found. dns-switcher requires iwd to function."
    fi
    
    if [ ${#missing[@]} -gt 0 ]; then
        error "Missing dependencies: ${missing[*]}"
    fi
}

get_latest_version() {
    local version
    if command -v curl &> /dev/null; then
        version=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
    else
        version=$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
    fi
    echo "$version"
}

download_and_install() {
    local version=$1
    local tmpdir
    tmpdir=$(mktemp -d)
    
    local url="https://github.com/${REPO}/releases/download/${version}/dns-switcher-${version}-x86_64-linux.tar.gz"
    
    info "Downloading dns-switcher ${version}..."
    
    if command -v curl &> /dev/null; then
        curl -fsSL "$url" -o "${tmpdir}/dns-switcher.tar.gz" || error "Download failed"
    else
        wget -q "$url" -O "${tmpdir}/dns-switcher.tar.gz" || error "Download failed"
    fi
    
    info "Extracting..."
    tar -xzf "${tmpdir}/dns-switcher.tar.gz" -C "$tmpdir"
    
    info "Installing to ${INSTALL_DIR}..."
    if [ -w "$INSTALL_DIR" ]; then
        install -m755 "${tmpdir}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    else
        sudo install -Dm755 "${tmpdir}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    fi
    
    rm -rf "$tmpdir"
    
    success "dns-switcher ${version} installed successfully!"
    echo ""
    echo "Run 'dns-switcher' to start the application."
}

build_from_source() {
    info "Building from source..."
    
    if ! command -v cargo &> /dev/null; then
        error "Rust/Cargo not found. Install from https://rustup.rs"
    fi
    
    local tmpdir
    tmpdir=$(mktemp -d)
    
    info "Cloning repository..."
    git clone --depth 1 "https://github.com/${REPO}.git" "$tmpdir"
    
    info "Building release binary..."
    cd "$tmpdir"
    cargo build --release
    
    info "Installing..."
    if [ -w "$INSTALL_DIR" ]; then
        install -m755 "target/release/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    else
        sudo install -Dm755 "target/release/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    fi
    
    rm -rf "$tmpdir"
    
    success "dns-switcher installed successfully from source!"
}

uninstall() {
    info "Uninstalling dns-switcher..."
    
    if [ -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
        if [ -w "$INSTALL_DIR" ]; then
            rm "${INSTALL_DIR}/${BINARY_NAME}"
        else
            sudo rm "${INSTALL_DIR}/${BINARY_NAME}"
        fi
        success "Binary removed"
    else
        warn "Binary not found at ${INSTALL_DIR}/${BINARY_NAME}"
    fi
    
    if [ -d "$HOME/.config/dns-switcher" ]; then
        read -p "Remove configuration (~/.config/dns-switcher)? [y/N] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$HOME/.config/dns-switcher"
            success "Configuration removed"
        fi
    fi
    
    success "dns-switcher uninstalled"
}

show_help() {
    cat << EOF
DNS Switcher Installer

Usage: $0 [OPTIONS]

Options:
    --uninstall     Remove dns-switcher from the system
    --source        Build and install from source (requires Rust)
    --help          Show this help message

Examples:
    # Install latest release
    $0
    
    # Install from source
    $0 --source
    
    # Uninstall
    $0 --uninstall
EOF
}

main() {
    echo ""
    echo "╔═══════════════════════════════════════╗"
    echo "║       DNS Switcher Installer          ║"
    echo "╚═══════════════════════════════════════╝"
    echo ""
    
    case "${1:-}" in
        --help|-h)
            show_help
            exit 0
            ;;
        --uninstall)
            uninstall
            exit 0
            ;;
        --source)
            check_deps
            build_from_source
            exit 0
            ;;
        "")
            check_deps
            local version
            version=$(get_latest_version)
            
            if [ -z "$version" ]; then
                warn "Could not determine latest version. Building from source..."
                build_from_source
            else
                download_and_install "$version"
            fi
            ;;
        *)
            error "Unknown option: $1. Use --help for usage."
            ;;
    esac
}

main "$@"

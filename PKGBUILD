# Maintainer: IovAnto <iovine.antonio44@gmail.com>
pkgname=dns-switcher
pkgver=0.2.0
pkgrel=1
pkgdesc="TUI application for real-time DNS switching on Linux"
arch=('x86_64')
url="https://github.com/IovAnto/dns-switcher"
license=('MIT')
depends=('iwd' 'systemd' 'polkit')
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/IovAnto/dns-switcher/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('5e4b14a9c3323f8048cc9cca2e6baa50283f86b64a4055b19c5ad4f85c640c04')

build() {
    cd "$pkgname-$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --release
}

check() {
    cd "$pkgname-$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    cargo test --release
}

package() {
    cd "$pkgname-$pkgver"
    install -Dm755 "target/release/dns-switcher" "$pkgdir/usr/bin/dns-switcher"
    install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
}

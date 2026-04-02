# Maintainer: IovAnto <iovine.antonio44@gmail.com>
pkgname=dns-switcher
pkgver=0.1.1
pkgrel=1
pkgdesc="TUI application for real-time DNS switching on Linux"
arch=('x86_64')
url="https://github.com/IovAnto/dns-switcher"
license=('MIT')
depends=('networkmanager' 'polkit')
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/IovAnto/dns-switcher/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('ed76952bc4115969a452f45e3687bd88b0c5d2b514052f5b805e7a01613fea30')

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

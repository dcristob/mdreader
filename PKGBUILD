# Maintainer: David Cristobal
pkgname=mdreader
pkgver=0.1.1
pkgrel=1
pkgdesc="A fast, lightweight desktop markdown viewer built with Rust and egui"
arch=('x86_64')
url="https://github.com/dcristob/mdreader"
license=('MIT')
depends=('gtk3')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/dcristob/mdreader/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

prepare() {
  cd "$pkgname-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
  cd "$pkgname-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  export CARGO_TARGET_DIR=target
  cargo build --frozen --release
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 "target/release/mdreader" "$pkgdir/usr/bin/mdreader"
  install -Dm644 "LICENSE.md" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  install -Dm644 "markdown.png" "$pkgdir/usr/share/icons/hicolor/512x512/apps/mdreader.png"
}

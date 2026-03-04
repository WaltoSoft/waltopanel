# Maintainer: WaltoSoft <https://github.com/WaltoSoft>
pkgname=waltopanel
pkgver=0.1.0
pkgrel=1
pkgdesc="A GTK4/Wayland panel bar"
arch=('x86_64')
url="https://github.com/WaltoSoft/waltopanel"
license=('MIT')
depends=('gtk4' 'libadwaita' 'gtk4-layer-shell')
makedepends=('rust' 'cargo' 'pkg-config')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

prepare() {
  cd "$pkgname-$pkgver"
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
  install -Dm755 "target/release/waltopanel" "$pkgdir/usr/bin/waltopanel"
  install -Dm644 /dev/stdin "$pkgdir/usr/lib/systemd/user/waltopanel.service" << 'EOF'
[Unit]
Description=WaltoPanel bar
PartOf=graphical-session.target
After=graphical-session.target

[Service]
ExecStart=/usr/bin/waltopanel
Restart=on-failure

[Install]
WantedBy=graphical-session.target
EOF
}

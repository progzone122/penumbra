Instructions to build antumbra as RPM.
Do not try to use `rust2rpm` as it won't work, penumbra uses dependencies from github.

## Fedora

```sh
$ sudo dnf install git cargo rust rpm-build tar gcc make cmake pkgconf-pkg-config systemd-devel libusb1-devel
$ git clone https://github.com/shomykohai/penumbra.git
$ cd penumbra
$ mkdir -p .cargo
$ cargo vendor --locked --versioned-dirs vendor > .cargo/config.toml
$ version=$(grep -m1 '^version' tui/Cargo.toml | cut -d'"' -f2)
$ mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
$ tar -czf ~/rpmbuild/SOURCES/antumbra-$version-vendored.tar.gz --anchored --exclude=./.git --exclude=./target .
$ cp tui/res/fedora/antumbra.spec ~/rpmbuild/SPECS/
$ rpmbuild -bb --define "_ver $version" ~/rpmbuild/SPECS/antumbra.spec
$ sudo dnf install ~/rpmbuild/RPMS/$(uname -m)/antumbra-$version-*.rpm
```

## Mageia

```sh
$ sudo urpmi --auto git cargo rust rpm-build tar gcc make cmake pkgconfig "pkgconfig(libudev)" "pkgconfig(libusb-1.0)"
$ git clone https://github.com/shomykohai/penumbra.git
$ cd penumbra
$ mkdir -p .cargo
$ cargo vendor --locked --versioned-dirs vendor > .cargo/config.toml
$ version=$(grep -m1 '^version' tui/Cargo.toml | cut -d'"' -f2)
$ mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
$ tar -czf ~/rpmbuild/SOURCES/antumbra-$version-vendored.tar.gz --anchored --exclude=./.git --exclude=./target .
$ cp tui/res/fedora/antumbra.spec ~/rpmbuild/SPECS/
$ rpmbuild -bb --define "_ver $version" ~/rpmbuild/SPECS/antumbra.spec
$ sudo urpmi ~/rpmbuild/RPMS/$(uname -m)/antumbra-$version-*.rpm
```

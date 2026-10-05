%{?!_ver:%global _ver %(cat %{_sourcedir}/antumbra.version 2>/dev/null || echo 0.0.0)}

%global debug_package %{nil}

Name:           antumbra
Version:        %{_ver}
Release:        1%{?dist}
Summary:        MTK flash tool written in Rust
License:        AGPL-3.0-or-later
URL:            https://github.com/shomykohai/penumbra
Source0:        %{name}-%{version}-vendored.tar.gz
BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  gcc
BuildRequires:  pkgconfig
BuildRequires:  pkgconfig(libudev)

%description
Terminal user interface and command line tool for interacting with, servicing
and flashing MediaTek based devices.

%prep
%autosetup -c

test -d vendor || { echo "vendor/ missing -- see tui/res/fedora/README.md" >&2; exit 1; }

%build
# Nightly features :)
export RUSTC_BOOTSTRAP=1
export CARGO_HOME="$PWD/.cargo-home"

cargo build --release --offline --locked --package antumbra

%install
install -D -p -m 0755 target/release/antumbra %{buildroot}%{_bindir}/antumbra

%files
%license LICENSE
%doc README.md
%{_bindir}/antumbra

%changelog

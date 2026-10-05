{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05-small";
    naersk.url = "github:nix-community/naersk";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    naersk,
    rust-overlay,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [rust-overlay.overlays.default];
    };

    rustToolchain = pkgs.rust-bin.nightly.latest.default;
    naerskLib = pkgs.callPackage naersk {
      rustc = rustToolchain;
      cargo = rustToolchain;
    };
  in {
    packages.${system}.default = naerskLib.buildPackage {
      src = ./.;
      cargoLock = ./Cargo.lock;
      buildInputs = [pkgs.glib pkgs.systemd.dev];
      nativeBuildInputs = [pkgs.pkg-config pkgs.copyDesktopItems];
      pname = "antumbra";

      postInstall = ''
        install -Dm644 app/res/common/icon.svg $out/share/icons/hicolor/scalable/apps/antumbra.svg
      '';

      desktopItems = [
        (pkgs.makeDesktopItem {
          name = "antumbra";
          desktopName = "Antumbra";
          exec = "antumbra --tui";
          terminal = true;
          categories = [ "Development" ];
        })
      ];
    };

    devShells.${system}.default = pkgs.mkShell {
      packages = [
        rustToolchain
        pkgs.rust-analyzer
        pkgs.nsis
      ];

      buildInputs = with pkgs; [
        glib

        systemd.dev
      ];

      nativeBuildInputs = [pkgs.pkg-config pkgs.libusb1];
    };
  };
}

{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = {
    self,
    nixpkgs,
    naersk,
  }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
    naerskLib = pkgs.callPackage naersk {};
  in {
    packages.${system}.default = naerskLib.buildPackage {
      src = ./.;
      cargoLock = ./Cargo.lock;
      buildInputs = [pkgs.glib pkgs.systemd.dev];
      nativeBuildInputs = [pkgs.pkg-config];
      pname = "antumbra";
    };

    devShells.${system}.default = pkgs.mkShell {
      packages = [
        pkgs.rustfmt
        pkgs.clippy
      ];

      buildInputs = with pkgs; [
        cargo
        rustc
        rust-analyzer
        glib

        systemd.dev
      ];

      nativeBuildInputs = [pkgs.pkg-config];

      env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    };
  };
}

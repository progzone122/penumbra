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
      buildInputs = [pkgs.glib];
      nativeBuildInputs = [pkgs.pkg-config];
    };

    shells.${system}.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        cargo
        rustc
        rustfmt
        clippy
        rust-analyzer
        glib
      ];

      nativeBuildInputs = [pkgs.pkg-config];

      env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    };
  };
}

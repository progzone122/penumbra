{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-25-05";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = {
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
    };
  };
}

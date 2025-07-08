{ pkgs ? import <nixpkgs> { } }:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "tsh";
  version = "0.1.0";
  cargoLock.lockFile = ./Cargo.lock;
  src = ./.;
}

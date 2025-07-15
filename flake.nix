{
  description = "Flake for tsh - a simple tool to help you think";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
  }: flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
        config.allowUnfree = false;
      };
      rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      rustDevTools = with pkgs; [ rustToolchain rust-analyzer bacon ];
      cargoAnalysisTools = with pkgs; [ cargo-license cargo-deny cargo-info ];
      generalDevTools = with pkgs; [ just litecli ];
    in {
      devShells.default = pkgs.mkShell {
        packages = rustDevTools ++ cargoAnalysisTools ++ generalDevTools;
        RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
      };
      packages.tsh = pkgs.callPackage ./default.nix { };
    }
  );
}

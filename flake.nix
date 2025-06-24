{
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
        overlays = [rust-overlay.overlays.default];
        config.allowUnfree = false;
      };
      rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      # rustDevPkgs = [ rustToolchain ] ++ (with pkgs; [ cargo-watch rust-analyzer ]);
      rustDevPkgs = [ rustToolchain ] ++ (with pkgs; [ rust-analyzer ]);
    in {
      devShells.default = pkgs.mkShell {
        packages = with pkgs; rustDevPkgs;
        RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
      };
      packages.rustEnv = pkgs.buildEnv {
        name = "arcagent-dev";
        paths = with pkgs; [ rustToolchain ] ++ [
          rust-analyzer
          gcc
        ];
      };
    }
  );
}

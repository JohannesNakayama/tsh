{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "tsh";
  version = "0.1.0";

  src = pkgs.fetchFromGitHub {
    owner = "JohannesNakayama";
    repo = "tsh";
    rev = "main";
    hash = "sha256-hTYnP3sMyskrvMrNooO9MZhJurrmb9NfERKZzUfQsHs=";
  };

  cargoHash = "sha256-U4/7VqA1pWov2DR76KYfoZKydQMXyh7bPTUlfvHyHQI=";

  buildInputs = [];

  nativeBuildInputs = [];

  meta = with pkgs.lib; {
    description = "A simple tool to help you think";
    homepage = "https://github.com/JohannesNakayama/tsh";
    license = licenses.asl20;
    platforms = platforms.linux;
    maintainers = "Johannes Nakayama";
  };
}

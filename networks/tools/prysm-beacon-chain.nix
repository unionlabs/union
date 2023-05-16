{ pkgs, system, ... }:
let
  version = "4.0.4";

  beacon-chain-targets = {
    x86_64-linux = pkgs.fetchurl {
      url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/beacon-chain-v${version}-linux-amd64";
      sha256 = "sha256-KqtIfZYQzWuuFwyD7Zz3ofgfwWmv00dnhNoqow27mE0=";
    };
    aarch64-linux = pkgs.fetchurl {
      url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/beacon-chain-v${version}-linux-arm64";
      sha256 = "sha256-Y7M48xhqLG4oxSNaLYoVV5dWjWVcvRcgzMbzzH+C9VA=";
    };
    x86_64-darwin = pkgs.fetchurl {
      url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/beacon-chain-v${version}-darwin-amd64";
      sha256 = "sha256-R+90aFz0p4327JuOQc/RFPY8q0sKGrXaVD99pYKYHWo=";
    };
    aarch64-darwin = pkgs.fetchurl {
      url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/beacon-chain-v${version}-darwin-arm64";
      sha256 = "sha256-5IWMSxHaqZJLq8F8684T7tAzDJXwebt9EIT4Q0Rhr60=";
    };
  };

  beacon-chain = beacon-chain-targets.${system};
in
pkgs.stdenv.mkDerivation {
  name = "beacon-chain";

  nativeBuildInputs = (if pkgs.stdenv.isDarwin then [ ] else [
    pkgs.autoPatchelfHook
  ]);

  src = ".";
  dontUnpack = true;

  installPhase = ''
    cp ${beacon-chain} .
    install -m775 -D *-beacon-chain-v${version}-*-* $out/bin/beacon-chain
  '';

  meta = {
    homepage = "https://github.com/prysmaticlabs/prysm";
    description = "This is a beacon chain implementation for Ethereum.";
    platforms = [ "x86_64-linux" "aarch64-linux" "x86_64-linux" "aarch64-linux" ];
  };
}

{ pkgs, system, ... }:
let
  version = "4.0.4";

  validator-targets = {
    x86_64-linux = pkgs.fetchurl {
      url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/validator-v${version}-linux-amd64";
      sha256 = "sha256-d9k/5UVyW0tzY1jU8+Bnp5+y2e5grEfgpU/oSiraOTM=";
    };
    aarch64-linux = pkgs.fetchurl {
      url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/validator-v${version}-linux-arm64";
      sha256 = "sha256-ILX+R1a6Mz0A0IoaBjaaSRQd4xrqk6ZrKoRjptOGHKA=";
    };
    x86_64-darwin = pkgs.fetchurl {
      url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/validator-v${version}-darwin-amd64";
      sha256 = "sha256-xk17V0BHr+6Flu47KdrCa/SAZ2N1IIf0xGRPBUZ9Ljs=";
    };
    aarch64-darwin = pkgs.fetchurl {
      url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/validator-v${version}-darwin-arm64";
      sha256 = "sha256-TsdZD7669Nz8IvuKYv8UHg9r1ojTzGy1bBc6iqc7cws=";
    };
  };

  validator = validator-targets.${system};
in
pkgs.stdenv.mkDerivation {
  name = "validator";

  nativeBuildInputs = (if pkgs.stdenv.isDarwin then [ ] else [
    pkgs.autoPatchelfHook
  ]);

  src = ".";

  dontUnpack = true;

  installPhase = ''
    cp ${validator} .
    install -m775 -D *-validator-v${version}-*-* $out/bin/validator
  '';

  meta = {
    homepage = "https://github.com/prysmaticlabs/prysm";
    description = "Launches an Ethereum validator client that interacts with a beacon chain, starts proposer and attester services, p2p connections, and more.";
    platforms = [ "x86_64-linux" "aarch64-linux" "x86_64-linux" "aarch64-linux" ];
  };
}

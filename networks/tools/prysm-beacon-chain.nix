{ pkgs, system, ... }:
pkgs.stdenv.mkDerivation rec {
  pname = "beacon-chain";
  version = "4.0.3";

  beacon-chain-amd64 = pkgs.fetchurl {
    url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/beacon-chain-v${version}-linux-amd64";
    sha256 = "sha256-OgGJrK0f/MKVTPv2XPyl/sv7HST7FggJD5yjbFUOH6k=";
  };

  beacon-chain-arm64 = pkgs.fetchurl {
    url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/beacon-chain-v${version}-linux-arm64";
    sha256 = "sha256-2Y5t3Szabau5jVtLX2lirEQjKta+UVqqFB+XDWHW+4s=";
  };

  beacon-chain = (if system == "x86_64-linux" then beacon-chain-amd64 else beacon-chain-arm64);

  nativeBuildInputs = [
    pkgs.autoPatchelfHook
  ];

  src = ".";
  dontUnpack = true;

  installPhase = ''
    cp ${beacon-chain} .
    install -m775 -D *-beacon-chain-v${version}-linux-* $out/bin/beacon-chain
  '';

  meta = {
    homepage = "https://github.com/prysmaticlabs/prysm";
    description = "This is a beacon chain implementation for Ethereum.";
    platforms = [ "x86_64-linux" "aarch64-linux" ];
  };
}

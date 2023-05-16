{ pkgs, ... }:
pkgs.stdenv.mkDerivation rec {
  pname = "beacon-chain";
  version = "4.0.3";

  beacon-chain = pkgs.fetchurl {
    url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/beacon-chain-v4.0.3-linux-amd64";
    sha256 = "sha256-OgGJrK0f/MKVTPv2XPyl/sv7HST7FggJD5yjbFUOH6k=";
  };

  nativeBuildInputs = [
    pkgs.autoPatchelfHook
  ];

  dontUnpack = true;

  src = ".";
  
  installPhase = ''
    cp ${beacon-chain} .
    install -m775 -D *-beacon-chain-v${version}-linux-amd64 $out/bin/beacon-chain
  '';

  meta = {
    homepage = "https://github.com/prysmaticlabs/prysm";
    description = "";
    platforms = [ "x86_64-linux" ];
  };
}

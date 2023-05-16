{ pkgs, ... }:
pkgs.stdenv.mkDerivation rec {
  pname = "validator";
  version = "4.0.3";

  prysmctl = pkgs.fetchurl {
    url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/validator-v4.0.3-linux-amd64";
    sha256 = "sha256-gmveVLd9fXXlZS65vZ2HznghLdlA7tY2oJFLMRWXT8Q=";
  };

  nativeBuildInputs = [
    pkgs.autoPatchelfHook
  ];

  dontUnpack = true;

  src = ".";
  
  installPhase = ''
    cp ${prysmctl} .
    install -m775 -D *-validator-v${version}-linux-amd64 $out/bin/validator
  '';

  meta = {
    homepage = "https://github.com/prysmaticlabs/prysm";
    description = "";
    platforms = [ "x86_64-linux" ];
  };
}

{ pkgs, system, ... }:
pkgs.stdenv.mkDerivation rec {
  pname = "validator";
  version = "4.0.3";

  validator-amd64 = pkgs.fetchurl {
    url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/validator-v4.0.3-linux-amd64";
    sha256 = "sha256-gmveVLd9fXXlZS65vZ2HznghLdlA7tY2oJFLMRWXT8Q=";
  };

  validator-arm64 = pkgs.fetchurl {
    url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/validator-v4.0.3-linux-amd64";
    sha256 = "sha256-gmveVLd9fXXlZS65vZ2HznghLdlA7tY2oJFLMRWXT8Q=";
  };

  validator = (if system == "x86_64-linux" then validator-amd64 else validator-arm64);

  nativeBuildInputs = [
    pkgs.autoPatchelfHook
  ];

  src = ".";

  dontUnpack = true;

  installPhase = ''
    cp ${validator} .
    install -m775 -D *-validator-v${version}-linux-* $out/bin/validator
  '';

  meta = {
    homepage = "https://github.com/prysmaticlabs/prysm";
    description = "";
    platforms = [ "x86_64-linux" "aarch64-linux" ];
  };
}

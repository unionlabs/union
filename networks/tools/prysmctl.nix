{ pkgs, ... }:
pkgs.stdenv.mkDerivation rec {
  pname = "prysmctl";
  version = "4.0.3";

  prysmctl = pkgs.fetchurl {
    url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/prysmctl-v4.0.3-linux-amd64";
    sha256 = "sha256-PDIVnswWXxjScLhzhGN6kmdaVBo8hfL7f+9FNKrsrM0=";
  };

  nativeBuildInputs = [
    pkgs.autoPatchelfHook
  ];

  dontUnpack = true;

  src = ".";
  
  installPhase = ''
    cp ${prysmctl} .
    install -m775 -D *-prysmctl-v${version}-linux-amd64 $out/bin/prysmctl
  '';

  meta = {
    homepage = "https://github.com/prysmaticlabs/prysm";
    description = "";
    platforms = [ "x86_64-linux" ];
  };
}

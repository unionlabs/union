{ pkgs, system, ... }:
pkgs.stdenv.mkDerivation rec {
  pname = "prysmctl";
  version = "4.0.3";

  prysmctl-amd64 = pkgs.fetchurl {
    url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/prysmctl-v${version}-linux-amd64";
    sha256 = "sha256-PDIVnswWXxjScLhzhGN6kmdaVBo8hfL7f+9FNKrsrM0=";
  };

  prysmctl-arm64 = pkgs.fetchurl {
    url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/prysmctl-v${version}-linux-arm64";
    sha256 = "sha256-tbsbrqxeA/D/hV3SCZkQJeDDyLJHwOsy6jhx0ukd57c=";
  };

  prysmctl = (if system == "x86_64-linux" then prysmctl-amd64 else prysmctl-arm64);

  nativeBuildInputs = [
    pkgs.autoPatchelfHook
  ];

  src = ".";

  dontUnpack = true;

  installPhase = ''
    cp ${prysmctl} .
    install -m775 -D *-prysmctl-v${version}-linux-* $out/bin/prysmctl
  '';

  meta = {
    homepage = "https://github.com/prysmaticlabs/prysm";
    description = "This is a command-line utility for common and one-off Ethereum proof-of-stake tasks, like helping users with validator exits or withdrawals. Most prysmctl commands require access to a fully synced beacon node.";
    platforms = [ "x86_64-linux" "aarch64-linux" ];
  };
}

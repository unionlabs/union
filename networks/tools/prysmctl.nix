{ pkgs, system, ... }:
let
  version = "4.0.4";

  prysmctl-targets = {
    x86_64-linux = pkgs.fetchurl {
      url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/prysmctl-v${version}-linux-amd64";
      sha256 = "sha256-k6VVqgeE8/py8DprwC/EpcEt0GmISOJ7qni1GYtZf2U=";
    };
    aarch64-linux = pkgs.fetchurl {
      url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/prysmctl-v${version}-linux-arm64";
      sha256 = "sha256-a5757173zNLWPIfDgxN7RwKRffJEz44teDu0meUi9E0=";
    };
    x86_64-darwin = pkgs.fetchurl {
      url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/prysmctl-v${version}-darwin-amd64";
      sha256 = "sha256-CdXNmtfaYAxtjfRVgsw3P+IjBfMpBVhoscb9Ty13258=";
    };
    aarch64-darwin = pkgs.fetchurl {
      url = "https://github.com/prysmaticlabs/prysm/releases/download/v${version}/prysmctl-v${version}-darwin-arm64";
      sha256 = "sha256-1iyawThu7kG2KUKshHVN0ClvnkXbYbmjO8nPOZQqpCQ=";
    };
  };

  prysmctl = prysmctl-targets.${system};
in
pkgs.stdenv.mkDerivation {
  name = "prysmctl";

  nativeBuildInputs = [
    pkgs.autoPatchelfHook
  ];

  src = ".";

  dontUnpack = true;

  installPhase = ''
    cp ${prysmctl} .
    install -m775 -D *-prysmctl-v${version}-*-* $out/bin/prysmctl
  '';

  meta = {
    homepage = "https://github.com/prysmaticlabs/prysm";
    description = "This is a command-line utility for common and one-off Ethereum proof-of-stake tasks, like helping users with validator exits or withdrawals. Most prysmctl commands require access to a fully synced beacon node.";
    platforms = [ "x86_64-linux" "aarch64-linux" ];
  };
}

{ lib, pkgs, ... }:
let
  sig-provider = pkgs.dockerTools.pullImage {
    imageName = "ghcr.io/blockscout/sig-provider";
    imageDigest = "sha256:0c8eb94f75c978cdf374a8fea87bce540276650e8e99cb8aba3b323bcc9ab1a2";
    sha256 = "sha256-etbrMM4xHNzwE/Envqflh0J00/BLU1B0ZfAuu/iNPoU=";
  };
in
{
  build.image = lib.mkForce sig-provider;
  service = {
    tty = true;
    stop_signal = "SIGINT";
    environment = {
      VISUALIZER__SERVER__GRPC__ENABLED = "false";
    };
  };
}

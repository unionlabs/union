{ lib, pkgs, ... }:
let
  visualizer = pkgs.dockerTools.pullImage {
    imageName = "ghcr.io/blockscout/visualizer";
    imageDigest = "sha256:012bde4c73ee54a530fc6d89868ae21458a94830483b59a03ff8a1ca302c924a";
    sha256 = "sha256-AIDFuBuuMHWb9JN07D7IDfSFqIQ3GPOFLpw57jSM3c0=";
  };
in
{
  build.image = lib.mkForce visualizer;
  service = {
    tty = true;
    stop_signal = "SIGINT";
  };
}

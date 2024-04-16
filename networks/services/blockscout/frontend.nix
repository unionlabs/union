{ lib, pkgs, env-utils, ... }:
let
  frontend = pkgs.dockerTools.pullImage {
    imageName = "ghcr.io/blockscout/frontend";
    imageDigest = "sha256:bf69a0a3b5eb92788ca59b2a9a9959385020fcaf9826b842de36928c666796e5";
    sha256 = "sha256-zb8+VhbWrWox3Q6nVNEvLi/3dkxD2j+M/8ffPr1Wu9o=";
  };
in
{
  build.image = lib.mkForce frontend;
  service = {
    tty = true;
    stop_signal = "SIGINT";
    environment = env-utils.lib.readEnvFile ./frontend.env;
    depends_on = {
      blockscout-backend = {
        condition = "service_started";
      };
    };
  };
}

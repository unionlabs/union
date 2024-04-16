{ lib, pkgs, ... }:
let
  blockscout = pkgs.dockerTools.pullImage {
    imageName = "nginx";
    imageDigest = "sha256:b455b84a67760b1cb8426240dc9c49f2099a211e9cff99d2efc8b05722638777";
    sha256 = "sha256-UUQAMGjlrUDTe8YYrfF2Prq0o2cr0Elo2SiiM3HpPzo=";
  };
in
{
  build.image = lib.mkForce blockscout;
  service = {
    tty = true;
    stop_signal = "SIGINT";
    restart = "always";
    environment = {
      BACK_PROXY_PASS = "http://blockscout-backend:4000";
      FRONT_PROXY_PASS = "http://blockscout-frontend:3000";
    };
    ports = [
      "80:80"
      "8080:8080"
      "8081:8081"
    ];
    volumes = [
      "${./proxy}:/etc/nginx/templates"
    ];
    depends_on = {
      blockscout-frontend = {
        condition = "service_started";
      };
      blockscout-backend = {
        condition = "service_started";
      };
      blockscout-stats = {
        condition = "service_started";
      };
    };
  };
}

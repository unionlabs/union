{ lib, pkgs, ... }:
let
  redis = pkgs.dockerTools.pullImage {
    imageName = "redis";
    imageDigest = "sha256:d6ecc832969a4827645a083da38345327b3447772fe907e7d4311c79b4e3a06e";
    sha256 = "sha256-1FMVwoB/R/klWFIGRGRMV6sXnOkJY1R2JVzv5OXa6oc=";
  };
in
{
  build.image = lib.mkForce redis;
  service = {
    tty = true;
    stop_signal = "SIGINT";
    command = "redis-server";
  };

}

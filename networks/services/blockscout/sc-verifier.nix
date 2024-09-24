{
  lib,
  pkgs,
  env-utils,
  evm-sources,
  ...
}:
let
  sc-verifier = pkgs.dockerTools.pullImage {
    imageName = "ghcr.io/blockscout/smart-contract-verifier";
    imageDigest = "sha256:cb1c2f3e27837b5bf6248b6a9647755069c3d91b00ad2dcc1b1fd37648cc5343";
    sha256 = "sha256-rSMULvdGY8yYeq/LNHfStfzEq7QjT4QIpBuXBk69nCw=";
  };
  sc-verifier-extended = pkgs.dockerTools.streamLayeredImage {
    name = "sc-verifier-extended";
    fromImage = sc-verifier;
    contents = [
      pkgs.coreutils
      pkgs.curl
      pkgs.jq
      evm-sources
    ];
    config = {
      # dockerTools only preserves Env from the base "fromImage"
      # this is directly coming from https://github.com/blockscout/blockscout-rs/blob/cf7076aabdedadfbb1a42d4bf2eec4fa5faa156c/docker/base.Dockerfile#L25C1-L26C13
      WorkingDir = "/app";
      Entrypoint = "./smart-contract-verifier-server";
    };
  };
in
{
  build.image = lib.mkForce sc-verifier-extended;
  service = {
    tty = true;
    stop_signal = "SIGINT";
    environment = env-utils.lib.readEnvFile ./sc-verifier.env;
    ports = [
      "8050:8050"
    ];
  };
}

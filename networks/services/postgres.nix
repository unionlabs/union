{ lib, pkgs, init-scripts, ... }:
let
  postgres = pkgs.dockerTools.pullImage {
    imageName = "postgres";
    imageDigest = "sha256:a5e89e5f2679863bedef929c4a7ec5d1a2cb3c045f13b47680d86f8701144ed7";
    sha256 = "0703g8p91bi3ybw35zj51zh2105r6wjjikyvliq9phwxsqrcfrxz";
    finalImageName = "postgres";
    finalImageTag = "latest";
  };
in
{
  build.image = lib.mkForce postgres;
  service = {
    tty = true;
    stop_signal = "SIGINT";
    ports = [
      "5432:5432"
    ];
    environment = {
      POSTGRES_PASSWORD = "postgrespassword";
      POSTGRES_DB = "default";
    };
    volumes = [
      "${init-scripts}:/docker-entrypoint-initdb.d"
    ];
  };
}

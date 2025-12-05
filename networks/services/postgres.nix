{
  lib,
  pkgs,
  system,
}:
let
  postgres =
    if system == "aarch64-linux" then
      pkgs.dockerTools.pullImage {
        imageName = "arm64v8/postgres";
        imageDigest = "sha256:1b6ca2021138a093566ef47bd851c3ad2c52a665bd8486609fc23cdc44563a4b";
        sha256 = "sha256-SNmjeAyMv1cxC3Qr3MZKHoWXsLMrrAEVWrhf/n13Y3U=";
        finalImageName = "arm64v8/postgres";
        finalImageTag = "18.1";
        arch = "arm64";
      }
    else if system == "x86_64-linux" then
      pkgs.dockerTools.pullImage {
        imageName = "";
        imageDigest = "";
        sha256 = "";
        finalImageName = "";
        finalImageTag = "";
        arch = "";
      }
    else
      throw "invalid system";
in
{
  build.image = lib.mkForce postgres;
  service = {
    tty = true;
    stop_signal = "SIGINT";
    ports = [
      "5432:5432"
    ];
    command = "postgres -c shared_buffers=1024MB -c effective_cache_size=2048MB";
    environment = {
      POSTGRES_PASSWORD = "postgrespassword";
      POSTGRES_DB = "default";
      POSTGRES_HOST_AUTH_METHOD = "trust";
    };
    healthcheck = {
      interval = "5s";
      retries = 10;
      start_period = "15s";
      test = [
        "CMD-SHELL"
        "pg_isready -h localhost -p 5432 -d default -U postgres"
      ];
    };
    # authentication = pkgs.lib.mkOverride 10 ''
    #   #type database  DBuser  auth-method
    #   local all       all     trust
    # '';
  };
}

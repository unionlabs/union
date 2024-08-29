{ lib, pkgs, ... }:
let
  postgres = pkgs.dockerTools.pullImage {
    imageName = "timescale/timescaledb";
    imageDigest = "sha256:eb8a3142384e8fd93ebd311783b297a04398ca61902b41233912a1a115279b69";
    sha256 = "sha256-zJ6HTYhxO7h+brEQOoJgDbHp74JfFe0Jcsfnz8MCFHM=";
    finalImageName = "timescaledb";
    finalImageTag = "2.14.1-pg16";
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
    command = "postgres -c shared_buffers=1024MB -c effective_cache_size=2048MB";
    environment = {
      POSTGRES_PASSWORD = "postgrespassword";
      POSTGRES_DB = "default";
      POSTGRES_HOST_AUTH_METHOD = "trust";
    };
    # authentication = pkgs.lib.mkOverride 10 ''
    #   #type database  DBuser  auth-method
    #   local all       all     trust
    # '';
  };
}

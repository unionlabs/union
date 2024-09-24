{ lib, pkgs, ... }:
let
  postgres = pkgs.dockerTools.pullImage {
    imageName = "postgres";
    imageDigest = "sha256:52495257b64779f90b46061a88d71237176613a9fb241d90ad15a643b0be6236";
    sha256 = "sha256-tL7AhKzyadiblX7CYSKnmhltzI91o7g34fx9Mp4DwCo=";
  };
in
{
  build.image = lib.mkForce postgres;
  service = {
    tty = true;
    stop_signal = "SIGINT";
    command = "postgres -c max_connections=200";
    environment = {
      POSTGRES_DB = "stats";
      POSTGRES_USER = "stats";
      POSTGRES_PASSWORD = "n0uejXPl61ci6ldCuE2gQU5Y";
    };
    healthcheck = {
      interval = "10s";
      timeout = "5s";
      retries = 5;
      start_period = "10s";
      test = [
        "CMD-SHELL"
        "pg_isready -U stats -d stats"
      ];
    };
  };
}

{ lib, pkgs, ... }:
let
  stats = pkgs.dockerTools.pullImage {
    imageName = "ghcr.io/blockscout/stats";
    imageDigest = "sha256:54d21fa78a0dc2cd0b59653203fc7e2b669b1f71ddbbdfc0fa7ec35516b86ff3";
    sha256 = "sha256-yTrDh0jMgIUp6voIP8Em3jVAvfxJ1dTkzv7M5W1SaXQ=";
  };
in
{
  build.image = lib.mkForce stats;
  service = {
    tty = true;
    stop_signal = "SIGINT";
    environment = {
      STATS__SERVER__HTTP__ENABLED = "true";
      STATS__SERVER__HTTP__ADDR = "0.0.0.0:8050";
      STATS__SERVER__HTTP__MAX_BODY_SIZE = "2097152";
      STATS__SERVER__GRPC__ENABLED = "false";
      STATS__SERVER__GRPC__ADDR = "0.0.0.0:8051";
      STATS__CREATE_DATABASE = "false";
      STATS__RUN_MIGRATIONS = "false";
      STATS__DEFAULT_SCHEDULE = "0 0 1 * * * *";
      STATS__FORCE_UPDATE_ON_START = "false";
      STATS__METRICS__ENABLED = "false";
      STATS__METRICS__ADDR = "0.0.0.0:6060";
      STATS__METRICS__ROUTE = "/metrics";
      STATS__JAEGER__ENABLED = "false";
      STATS__JAEGER__AGENT_ENDPOINT = "localhost:6831";
      STATS__TRACING__ENABLED = "true";
      STATS__TRACING__FORMAT = "default";
    } // {
      STATS__DB_URL = "postgres://stats:n0uejXPl61ci6ldCuE2gQU5Y@blockscout-stats-db:5432/stats";
      STATS__BLOCKSCOUT_DB_URL = "postgresql://blockscout:ceWb1MeLBEeOIfk65gU8EjF8@blockscout-db:5432/blockscout";
      STATS__CREATE_DATABASE = "true";
      STATS__RUN_MIGRATIONS = "true";
    };
    depends_on = {
      blockscout-stats-db = {
        condition = "service_started";
      };
      blockscout-backend = {
        condition = "service_started";
      };
    };
  };
}

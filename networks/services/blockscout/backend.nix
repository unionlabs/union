{
  lib,
  pkgs,
  env-utils,
  ...
}:
let
  blockscout = pkgs.dockerTools.pullImage {
    imageName = "blockscout/blockscout";
    imageDigest = "sha256:a3fd98f428ffd5f1ccef692d191b5ec5072692822ce05f1ad5deb69670779c1d";
    sha256 = "sha256-qU7WtjhUxyIkPYxMq/x4RT0VcV3rJ/uvc0dJnmwQSFE=";
  };
in
{
  build.image = lib.mkForce blockscout;
  service = {
    tty = true;
    stop_signal = "SIGINT";
    restart = "always";
    environment = env-utils.lib.readEnvFile ./backend.env;
    command = ''
      sh -c "bin/blockscout eval \"Elixir.Explorer.ReleaseTasks.create_and_migrate()\" && bin/blockscout start"
    '';
    depends_on = {
      geth = {
        condition = "service_healthy";
      };
      blockscout-db = {
        condition = "service_healthy";
      };
      blockscout-redis = {
        condition = "service_started";
      };
    };
  };
}

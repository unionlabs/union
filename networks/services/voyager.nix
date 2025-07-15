{
  self',
  pkgs,
}:
let
  voyager-start = pkgs.writeShellApplication {
    name = "voyager-start";
    text = ''
      ${self'.packages.voyager}/bin/voyager start 
    '';
  };
in
{
  build = {
    image = pkgs.dockerTools.buildImage {
      name = "voyager";

      copyToRoot = pkgs.buildEnv {
        name = "image-root";
        paths = [
          pkgs.coreutils
          pkgs.cacert
          self'.packages.voyager
          voyager-start
        ];
        pathsToLink = [ "/bin" ];
      };
      config = {
        Entrypoint = [ "ls" ];
        Cmd = [ "-la" ];
      };
    }
  };
  service = {
    tty = true;
    stop_signal = "SIGINT";
    ports = [
      # Beacon node rest API
      "9596:9596"
    ];
    healthcheck = {
      interval = "5s";
      retries = 6;
      test = [
        "CMD-SHELL"
        ''
          curl -f http://localhost:9596/eth/v2/beacon/blocks/2 || exit 1
        ''
      ];
    };
    depends_on = {
      geth = {
        condition = "service_healthy";
      };
    };
  };
}

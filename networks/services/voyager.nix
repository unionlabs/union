_:
{
  perSystem =
    {
      self',
      pkgs,
      lib,
      ...
    }:
    let
      voyagerService = voyagerConfig: {
        image = {
          enableRecommendedContents = true;
          contents = [
            pkgs.coreutils
            pkgs.curl
            pkgs.jq
            self'.packages.voyager
            voyagerConfig
            self'.packages.voyager-modules-plugins
          ];
        };

        service = {
          network_mode = "host";
          tty = true;
          stop_signal = "SIGINT";
          tmpfs = [ "/tmp" ];
          # these are not needed since the network_mode is "host"
          # ports = [
          #   # voyager rest laddr
          #   "7177:7177"
          #   # voyager rpc laddr
          #   "7178:7178"
          # ];
          command = [
            "sh"
            "-c"
            ''
              RUST_LOG=voyager=debug voyager -c ${voyagerConfig}/voyager-config.jsonc start
            ''
          ];
          healthcheck = {
            interval = "5s";
            retries = 10;
            start_period = "15s";
            test = [
              "CMD-SHELL"
              "voyager rpc info"
            ];
          };
          depends_on = {
            postgres = {
              condition = "service_healthy";
            };
          };
        };
      };

    in
    {
      _module.args.mkVoyagerImg = configFilePath: {
        project.name = "voyager";
        services = {
          voyager = voyagerService configFilePath;
          postgres = import ./postgres.nix { inherit lib pkgs; };
        };
      };
    };
}

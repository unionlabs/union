{ self, ... }: {
  perSystem = { self', pkgs, crane, ... }:
    let
      hubble = crane.buildWorkspaceMember {
        crateDirFromRoot = "hubble";
        cargoTestExtraAttrs = {
          partitions = 1;
          partitionType = "count";
        };
        extraEnv = {
          SQLX_OFFLINE = "1";
        };
      };
    in
    {
      inherit (hubble) checks;
      packages = {
        hubble = hubble.packages.hubble;

        hubble-image = pkgs.dockerTools.buildLayeredImage {
          name = "hubble";
          contents = [ pkgs.coreutils-full pkgs.cacert self'.packages.hubble ];
          config = {
            Entrypoint = [ (pkgs.lib.getExe self'.packages.hubble) ];
            Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
          };
        };
      };
    };

  flake.nixosModules.hubble = { lib, pkgs, config, ... }:
    with lib;
    let
      cfg = config.services.hubble;
    in
    {
      options.services.hubble = {
        enable = mkEnableOption "Hubble service";
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.hubble;
        };
        metrics-addr = mkOption {
          type = types.str;
          default = "0.0.0.0:9090";
        };
        api-key-file = mkOption {
          description = lib.mdDoc ''
            Path to a file containing the database secret to allow for inserts.
          '';
          example = "/run/keys/hubble.key";
          type = types.path;
          default = "";
        };
        indexers = mkOption {
          type = types.listOf (
            types.submodule {
              options.label = mkOption { type = types.str; example = "something-custom"; };
              options.urls = mkOption { type = types.listOf types.str; example = [ "https://rpc.example.com" ]; };
              options.chain_id = mkOption { type = types.nullOr types.str; example = "union-testnet-8"; default = null; };
              options.grpc_url = mkOption { type = types.nullOr types.str; example = "https://grpc.example.com"; default = null; };
              options.type = mkOption { type = types.enum [ "tendermint" "ethereum" "beacon" "bera" "ethereum-fork" ]; };
              options.start_height = mkOption { type = types.int; example = 1; default = 0; };
              options.chunk_size = mkOption { type = types.int; example = 1; default = 200; };
              options.until = mkOption { type = types.int; example = 1; default = 1000000000000; };
              options.harden = mkOption { type = types.bool; example = true; default = true; };
              options.interval = mkOption {
                example = { secs = 1; };
                default = { secs = 12; nanos = 0; };
                type = types.submodule {
                  options = {
                    secs = mkOption {
                      type = types.int;
                    };
                    nanos = mkOption {
                      type = types.int;
                      default = 0;
                    };
                  };
                };
              };
            }
          );
        };
        log-level = mkOption {
          type = types.str;
          default = "info";
          description = "RUST_LOG passed to hubble";
          example = "hubble=debug";
        };
        log-format = mkOption {
          type = types.enum [ "json" "plain" ];
          default = "json";
          example = "plain";
        };
      };

      config = mkIf cfg.enable {
        systemd.services.hubble =
          let
            hubble-systemd-script = pkgs.writeShellApplication {
              name = "hubble-systemd";
              runtimeInputs = [ pkgs.coreutils cfg.package ];
              text =
                let
                  indexersJson = builtins.toJSON cfg.indexers;
                in
                ''
                  ${pkgs.lib.getExe cfg.package}  \
                    --database-url "$(head -n 1 ${cfg.api-key-file})" \
                    --log-format ${cfg.log-format} \
                    --metrics-addr ${cfg.metrics-addr} \
                    --indexers '${indexersJson}' 
                '';
            };
          in
          {
            wantedBy = [ "multi-user.target" ];
            description = "Hubble";
            unitConfig = {
              StartLimitIntervalSec = mkForce 0;
            };
            serviceConfig = {
              Type = "simple";
              ExecStart = pkgs.lib.getExe hubble-systemd-script;
              Restart = mkForce "always";
              RestartSec = mkForce 3;
            };
            environment = {
              RUST_LOG = "${cfg.log-level}";
            };
          };
      };
    };
}

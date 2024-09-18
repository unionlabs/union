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
              options.indexer_id = mkOption {
                type = types.nullOr types.str;
                description = "Id of the indexer which is used by the internal administration of Hubble. Should never change.";
                example = "amazing-testnet";
                default = null;
              };
              options.internal_chain_id = mkOption {
                type = types.nullOr types.number;
                description = "Hubble internal chain id, used to fetch the current height when migrating to fetchers.";
                example = "4";
                default = null;
              };
              options.new_chain_override = mkOption {
                type = types.nullOr types.bool;
                description = "Indicator that this is a new chain, so the current height must not be used when migrating to fetchers.";
                example = "false";
                default = null;
              };
              options.label = mkOption { type = types.str; example = "something-custom"; };
              options.filter = mkOption {
                type = types.nullOr types.str;
                description = "A regex which if matches, removes the event from the insertion";
                example = "coin_received";
                default = null;
              };
              options.urls = mkOption {
                type = types.nullOr (types.listOf types.str);
                example = [ "https://rpc.example.com" ];
                default = null;
              };

              # arb consensus height indexer
              options.l1_url = mkOption {
                type = types.nullOr types.str;
                example = "https://rpc.example.com";
                default = null;
              };
              options.l2_url = mkOption {
                type = types.nullOr types.str;
                example = "https://rpc.example.com";
                default = null;
              };
              options.beacon_url = mkOption {
                type = types.nullOr types.str;
                example = "https://rpc.example.com";
                default = null;
              };
              options.rollup_finalization_config = mkOption {
                type = types.nullOr types.attrs;
                default = null;
              };

              # scroll
              options.scroll_api_url = mkOption {
                type = types.nullOr types.str;
                default = null;
              };

              options.chain_id = mkOption { type = types.nullOr types.str; example = "union-testnet-8"; default = null; };
              options.grpc_url = mkOption { type = types.nullOr types.str; example = "https://grpc.example.com"; default = null; };
              options.type = mkOption { type = types.enum [ "tendermint" "ethereum" "beacon" "bera" "ethereum-fork" "arb" "scroll" "eth-fetcher" ]; };
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

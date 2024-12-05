{ self, ... }:
{
  perSystem =
    {
      self',
      pkgs,
      crane,
      ...
    }:
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
        inherit (hubble.packages) hubble;

        hubble-image = pkgs.dockerTools.buildLayeredImage {
          name = "hubble";
          contents = [
            pkgs.coreutils-full
            pkgs.cacert
            self'.packages.hubble
          ];
          config = {
            Entrypoint = [ (pkgs.lib.getExe self'.packages.hubble) ];
            Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
          };
        };
      };
    };

  flake.nixosModules.hubble =
    {
      lib,
      pkgs,
      config,
      ...
    }:
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
              options.filter = mkOption {
                type = types.nullOr types.str;
                description = "A regex which if matches, removes the event from the insertion";
                example = "coin_received";
                default = null;
              };
              options.rpc_urls = mkOption {
                type = types.nullOr (types.listOf types.str);
                description = "List of rpc urls";
                example = [ "https://rpc.example.com" ];
                default = null;
              };
              options.grpc_urls = mkOption {
                type = types.nullOr (types.listOf types.str);
                description = "List of grpc urls";
                example = [ "https://grpc.example.com" ];
                default = null;
              };
              options.type = mkOption {
                type = types.enum [
                  "etherium"
                  "tendermint"
                  "aptos"
                ];
              };
              options.start_height = mkOption {
                type = types.int;
                example = 1;
                default = 0;
              };
              options.chunk_size = mkOption {
                type = types.int;
                example = 1;
                default = 200;
              };
              options.tx_search_max_page_size = mkOption {
                type = types.int;
                description = "Maximum number of transactions to fetch in one page";
                example = 1;
                default = 100;
              };
              options.finalizer = mkOption {
                description = "control finalizer behavior";
                example = {
                  reload = true;
                  delay_blocks = 5;
                };
                default = null;
                type = types.nullOr (
                  types.submodule {
                    options = {
                      delay_blocks = mkOption {
                        type = types.nullOr types.int;
                        default = null;
                        description = "how many blocks to wait until a block is considered finalized (ie. there should be no reorgs). compensates for height differences between rpcs.";
                      };
                      reload = mkOption {
                        type = types.nullOr types.bool;
                        default = null;
                        description = "reload all block data after a block is considered finalized. compensates for rpcs returning inconsistent results for non-finalized blocks.";
                      };
                      min_seconds_between_monitor_checks = mkOption {
                        type = types.nullOr types.int;
                        default = null;
                        description = "minimum time (in seconds) between checking hash changes of non finalized blocks.";
                      };
                      retry_later_sleep_seconds = mkOption {
                        type = types.nullOr types.int;
                        default = null;
                        description = "sleep time (in seconds) when there is nothing to finalize.";
                      };
                    };
                  }
                );
              };
              options.client_tracking = mkOption {
                type = types.nullOr types.bool;
                description = "control if client tracking is enabled. when enabled it automatically creates client entries which registers the counterparty chain of a client.";
                example = true;
                default = null;
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
        backtrace = mkOption {
          type = types.enum [
            "0"
            "1"
            "full"
          ];
          default = "1";
          description = "RUST_BACKTRACE passed to hubble";
          example = "full";
        };
        no-color = mkOption {
          type = types.enum [
            "0"
            "1"
          ];
          default = "1";
          description = "NO_COLOR passed to hubble";
          example = "1";
        };
        log-format = mkOption {
          type = types.enum [
            "json"
            "plain"
          ];
          default = "json";
          example = "plain";
        };
        tokens_urls = mkOption {
          type = types.nullOr (types.listOf types.str);
          description = "List of tokenlist urls";
          example = [ "https://static.optimism.io/optimism.tokenlist.json" ];
          default = null;
        };
      };

      config = mkIf cfg.enable {
        systemd.services.hubble =
          let
            hubble-systemd-script = pkgs.writeShellApplication {
              name = "hubble-systemd";
              runtimeInputs = [
                pkgs.coreutils
                cfg.package
              ];
              text =
                let
                  # convert to json, removing null values
                  filterNullValues = lib.attrsets.filterAttrsRecursive (_n: v: v != null);
                  indexersWithoutNulls = map filterNullValues cfg.indexers;
                  indexersJson = builtins.toJSON indexersWithoutNulls;
                  tokensUrlsJson = builtins.toJSON cfg.tokens_urls;
                in
                ''
                  ${pkgs.lib.getExe cfg.package}  \
                    --database-url "$(head -n 1 ${cfg.api-key-file})" \
                    --log-format ${cfg.log-format} \
                    --metrics-addr ${cfg.metrics-addr} \
                    --indexers '${indexersJson}' \
                    --tokens-urls '${tokensUrlsJson}'
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
              RUST_BACKTRACE = "${cfg.backtrace}";
              NO_COLOR = "${cfg.no-color}";
            };
          };
      };
    };
}

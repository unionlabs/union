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
      hubble = crane.buildWorkspaceMember "hubble" {
        # cargoTestExtraAttrs = {
        #   partitions = 1;
        #   partitionType = "count";
        # };
        extraEnv = {
          SQLX_OFFLINE = "1";
        };
      };
    in
    {
      packages = hubble // {
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
        nats-url-file = mkOption {
          description = lib.mdDoc ''
            Path to a file containing the nats connect string (e.g. nats://localhost).
          '';
          example = "/run/keys/nats.url";
          type = types.nullOr types.path;
          default = null;
        };
        nats-username-file = mkOption {
          description = lib.mdDoc ''
            Path to a file containing the nats username.
          '';
          example = "/run/keys/nats.username";
          type = types.nullOr types.path;
          default = null;
        };
        nats-password-file = mkOption {
          description = lib.mdDoc ''
            Path to a file containing the nats password.
          '';
          example = "/run/keys/nats.password";
          type = types.nullOr types.path;
          default = null;
        };
        nats-consumer = mkOption {
          description = lib.mdDoc ''
            Name of the nats consumer that reads message.
          '';
          example = "hubble-magenta";
          type = types.nullOr types.str;
          default = null;
        };
        indexers = mkOption {
          type = types.listOf (
            types.submodule {
              options.indexer_id = mkOption {
                type = types.str;
                description = "Id of the indexer which is used by the internal administration of Hubble. Should never change.";
                example = "amazing-testnet";
              };
              options.universal_chain_id = mkOption {
                type = types.str;
                description = "Universal chain id of the chain that is indexed.";
                example = "union-testnet-10.union";
              };
              options.rpc_urls = mkOption {
                type = types.nullOr (types.listOf types.str);
                description = "List of rpc urls";
                example = [ "https://rpc.example.com" ];
                default = null;
              };
              options.type = mkOption {
                type = types.enum [
                  "ethereum"
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
              options.testnet = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Testnet (default false)";
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
                  natsUrlArg = if cfg.nats-url-file != null then "--nats-url @${cfg.nats-url-file}" else "";
                  natsUsernameArg =
                    if cfg.nats-username-file != null then "--nats-username @${cfg.nats-username-file}" else "";
                  natsPasswordArg =
                    if cfg.nats-password-file != null then "--nats-password @${cfg.nats-password-file}" else "";
                  natsConsumerArg = if cfg.nats-consumer != null then "--nats-consumer ${cfg.nats-consumer}" else "";
                in
                ''
                  ${pkgs.lib.getExe cfg.package}  \
                    --database-url "$(head -n 1 ${cfg.api-key-file})" \
                    ${natsUrlArg} \
                    ${natsUsernameArg} \
                    ${natsPasswordArg} \
                    ${natsConsumerArg} \
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
              PrivateTmp = true; # workspace for generating abis
            };
            environment = {
              RUST_LOG = "${cfg.log-level}";
              RUST_BACKTRACE = "${cfg.backtrace}";
              NO_COLOR = "${cfg.no-color}";
              NIX_BIN = "${pkgs.nix}/bin/nix";
            };
          };
      };
    };
}

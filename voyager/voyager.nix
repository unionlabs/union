{ self, ... }:
{
  perSystem =
    {
      self',
      pkgs,
      system,
      config,
      crane,
      stdenv,
      dbg,
      mkCi,
      ...
    }:
    let
      attrs = {
        crateDirFromRoot = "voyager";
        extraEnv = {
          SQLX_OFFLINE = "1";
        };
      };

      voy-modules-list = builtins.filter (
        member:
        (pkgs.lib.hasPrefix "voyager/modules" member) || (pkgs.lib.hasPrefix "voyager/plugins" member)
      ) (builtins.fromTOML (builtins.readFile ../Cargo.toml)).workspace.members;

      voyager-modules = crane.buildWorkspaceMember {
        crateDirFromRoot = voy-modules-list;
        pname = "voyager-modules";
        version = "0.0.0";
        dev = true;
      };

      voyager = crane.buildWorkspaceMember attrs;
      voyager-dev =
        pkgs.lib.warn "voyager-dev is not intended to be used in production" crane.buildWorkspaceMember
          (attrs // { dev = true; });
    in
    {
      packages =
        voyager.packages
        // {
          voyager-modules-names = builtins.toFile "voyager-modules-list.json" (
            builtins.toJSON (map (p: (builtins.fromTOML (builtins.readFile "${../.}/${p}/Cargo.toml")).package.name) voy-modules-list)
          );
          ethereum-multi-send = pkgs.writeShellApplication {
            name = "ethereum-multi-send";
            runtimeInputs = [ self'.packages.forge ];
            text = ''
              set -e

              PRIVATE_KEY="''${PRIVATE_KEY:?private key is unset}"
              echo "$PRIVATE_KEY"

              RPC_URL="''${RPC_URL:?rpc url is unset}"
              echo "$RPC_URL"

              VALUE="''${VALUE:?value is unset}"
              echo "$VALUE"

              for var in "$@"
              do
                  cast send --rpc-url "$RPC_URL" --private-key "$PRIVATE_KEY" --value "$VALUE" "$var"
              done
            '';
          };
          voyager-dev = mkCi false voyager-dev.packages.voyager-dev;
        }
        // voyager-modules.packages;
      checks = voyager.checks // voyager-modules.checks;
    };

  flake.nixosModules.voyager =
    {
      lib,
      pkgs,
      config,
      ...
    }:
    with lib;
    let
      cfg = config.services.voyager;
    in
    {
      options.services.voyager = {
        enable = mkEnableOption "Voyager service";
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.voyager;
        };
        plugins = mkOption {
          # The configuration design is breaking quite often, would be a waste
          # of effort to fix the type for now.
          type = types.listOf (
            types.submodule {
              options = {
                enabled = mkOption {
                  type = types.bool;
                  default = true;
                };
                path = mkOption { type = types.path; };
                config = mkOption { type = types.attrs; };
              };
            }
          );
        };
        optimizer-delay-milliseconds = mkOption {
          type = types.int;
          default = 100;
        };
        workers = mkOption {
          type = types.int;
          default = 20;
        };
        runtime-max-secs = mkOption {
          type = types.int;
          default = 1800;
        };
        db-url = mkOption {
          type = types.str;
          default = "postgres://voyager:voyager@localhost/voyager";
        };
        db-min-conn = mkOption {
          type = types.int;
          default = 20;
        };
        db-max-conn = mkOption {
          type = types.int;
          default = 20;
        };
        log-level = mkOption {
          type = types.str;
          default = "info";
          # TODO: Support RUST_LOG per plugin (this will need to be done in voyager)
          description = "RUST_LOG passed to voyager and all of the plugins.";
          example = "voyager=debug";
        };
        log-format = mkOption {
          type = types.enum [
            "json"
            "text"
          ];
          default = "json";
          # TODO: This is kinda dirty, find a better way? Probably through each plugin's config
          description = "The log format for voyager. This will also be passed to all of the plugins as RUST_LOG_FORMAT.";
          example = "text";
        };
        stack-size = mkOption {
          type = types.nullOr types.number;
          description = "The stack size (in bytes) for worker threads. See <https://docs.rs/tokio/1.40.0/tokio/runtime/struct.Builder.html#method.thread_stack_size> for more information.";
          default = null;
          example = 20971520;
        };
        rest_laddr = mkOption {
          type = types.str;
          default = "0.0.0.0:7177";
          example = "0.0.0.0:7177";
        };
        rpc_laddr = mkOption {
          type = types.str;
          default = "0.0.0.0:7178";
          example = "0.0.0.0:7178";
        };
        voyager-extra = mkOption {
          type = types.attrs;
          default = { };
        };
      };

      config =
        let
          configJson = pkgs.writeText "config.json" (
            builtins.toJSON {
              inherit (cfg) plugins;
              voyager = cfg.voyager-extra // {
                num_workers = cfg.workers;
                queue = {
                  type = "pg-queue";
                  database_url = cfg.db-url;
                  min_connections = cfg.db-min-conn;
                  max_connections = cfg.db-max-conn;
                  idle_timeout = null;
                  max_lifetime = null;
                };
              };
            }
          );
        in
        mkIf cfg.enable {
          systemd.services = {
            # voyager-migration = {
            #   wantedBy = [ "multi-user.target" ];
            #   after = [ "network.target" ];
            #   description = "Voyager Migration";
            #   serviceConfig = {
            #     Type = "oneshot";
            #     ExecStart = ''
            #       ${pkgs.lib.meta.getExe cfg.package} \
            #         --config-file-path ${configJson} \
            #         -l ${cfg.log-format} \
            #         run-migrations
            #     '';
            #   };
            #   environment = {
            #     RUST_LOG = "debug";
            #     RUST_BACKTRACE = "full";
            #   };
            # };
            voyager = {
              wantedBy = [ "multi-user.target" ];
              # after = [ "voyager-migration.service" ];
              # partOf = [ "voyager-migration.service" ];
              # requires = [ "voyager-migration.service" ];
              description = "Voyager";
              serviceConfig = {
                Type = "simple";
                ExecStart = ''
                  ${pkgs.lib.getExe cfg.package} \
                    --config-file-path ${configJson} \
                    -l ${cfg.log-format} ${
                      pkgs.lib.optionalString (cfg.stack-size != null) "--stack-size ${toString cfg.stack-size}"
                    } \
                    relay
                '';
                Restart = mkForce "always";
                RestartSec = 10;
                RuntimeMaxSec = cfg.runtime-max-secs;
              };
              environment = {
                RUST_LOG = "${cfg.log-level}";
                RUST_LOG_FORMAT = "${cfg.log-format}";
              };
            };
          };
        };
    };
}

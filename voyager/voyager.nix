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

      voyager-modules-dev = crane.buildWorkspaceMember {
        crateDirFromRoot = voy-modules-list;
        pname = "voyager-modules";
        version = "0.0.0";
        dev = true;
      };

      voyager-modules = crane.buildWorkspaceMember {
        crateDirFromRoot = voy-modules-list;
        pname = "voyager-modules";
        version = "0.0.0";
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
          # voyager-modules-names = (
          #   builtins.toFile "voyager-modules-list.json" (
          #     builtins.toJSON (
          #       map (
          #         p: (builtins.fromTOML (builtins.readFile "${../.}/${p}/Cargo.toml")).package.name
          #       ) voy-modules-list
          #     )
          #   )
          # );
          voyager-dev = mkCi false voyager-dev.packages.voyager-dev;
        }
        // voyager-modules-dev.packages
        // voyager-modules.packages;
      # we don't actually have any tests currently
      # checks = voyager.checks // voyager-modules.checks;
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
        modules =
          let
            moduleConfigType =
              infoOptions:
              mkOption {
                type = types.listOf (
                  types.submodule {
                    options = {
                      enabled = mkOption {
                        type = types.bool;
                        default = true;
                      };
                      path = mkOption {
                        type = types.path;
                      };
                      config = mkOption {
                        type = types.attrs;
                        default = { };
                      };
                      info = mkOption {
                        type = types.submodule {
                          options = infoOptions;
                        };
                      };
                    };
                  }
                );
              };
          in
          mkOption {
            type = types.submodule {
              options = {
                client = moduleConfigType {
                  client_type = mkOption { type = types.str; };
                  consensus_type = mkOption { type = types.str; };
                  ibc_interface = mkOption { type = types.str; };
                  ibc_spec_id = mkOption { type = types.str; };
                };
                client_bootstrap = moduleConfigType {
                  chain_id = mkOption { type = types.str; };
                  client_type = mkOption { type = types.str; };
                };
                consensus = moduleConfigType {
                  chain_id = mkOption { type = types.str; };
                  consensus_type = mkOption { type = types.str; };
                };
                proof = moduleConfigType {
                  chain_id = mkOption { type = types.str; };
                  ibc_spec_id = mkOption { type = types.str; };
                };
                state = moduleConfigType {
                  chain_id = mkOption { type = types.str; };
                  ibc_spec_id = mkOption { type = types.str; };
                };
              };
            };
          };
        plugins = mkOption {
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
        voyager =
          let
            durationType = types.submodule {
              options = {
                secs = mkOption { type = types.int; };
                nanos = mkOption { type = types.int; };
              };
            };
            cacheType = types.submodule {
              options = {
                capacity = mkOption { type = types.int; };
                time_to_live = mkOption { type = types.int; };
                time_to_idle = mkOption { type = types.int; };
              };
            };
          in
          mkOption {
            type = types.submodule {
              options = {
                num_workers = mkOption {
                  type = types.int;
                };
                rest_laddr = mkOption {
                  type = types.nullOr types.str;
                  default = null;
                  example = "0.0.0.0:7177";
                };
                rpc_laddr = mkOption {
                  type = types.nullOr types.str;
                  default = null;
                  example = "0.0.0.0:7178";
                };
                queue = mkOption {
                  type = types.submodule {
                    options = {
                      database_url = mkOption {
                        type = types.str;
                        default = "postgres://voyager:voyager@localhost/voyager";
                      };
                      max_connections = mkOption {
                        type = types.int;
                      };
                      min_connections = mkOption {
                        type = types.int;
                      };
                      idle_timeout = mkOption {
                        type = types.nullOr durationType;
                        default = null;
                      };
                      optimize_batch_limit = mkOption {
                        type = types.nullOr types.int;
                        default = null;
                      };
                      max_lifetime = mkOption {
                        type = types.nullOr durationType;
                        default = null;
                      };
                      retryable_error_expo_backoff_max = mkOption {
                        type = types.nullOr types.float;
                        default = null;
                      };
                      retryable_error_expo_backoff_multiplier = mkOption {
                        type = types.nullOr types.float;
                        default = null;
                      };
                    };
                  };
                };
                optimizer_delay_milliseconds = mkOption {
                  type = types.nullOr types.int;
                  default = null;
                };
                ipc_client_request_timeout = mkOption {
                  type = durationType;
                };
                cache = mkOption {
                  type = types.submodule {
                    options = {
                      state = mkOption { type = cacheType; };
                    };
                  };
                };
              };
            };
          };
        runtime-max-secs = mkOption {
          type = types.int;
          default = 1800;
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
      };

      config =
        let
          configJson = pkgs.writeText "config.json" (
            builtins.toJSON (
              recursiveUpdate
                (filterAttrsRecursive (_n: v: v != null) (
                  filterAttrs (
                    n: _v:
                    builtins.elem n [
                      "modules"
                      "plugins"
                      "voyager"
                      "equivalent_chain_ids"
                      "schema"
                    ]
                  ) cfg
                ))
                {
                  voyager.queue.type = "pg-queue";
                }
            )
          );
        in
        mkIf cfg.enable {
          environment.systemPackages = [
            (pkgs.writeShellApplication {
              name = "voyager";
              runtimeInputs = [ cfg.package ];
              text = ''
                ${getExe cfg.package} --config-file-path ${configJson} "$@"
              '';
            })
          ];
          systemd.services = {
            voyager = {
              wantedBy = [ "multi-user.target" ];
              description = "Voyager";
              serviceConfig = {
                Type = "simple";
                ExecStart = ''
                  ${getExe cfg.package} \
                    --config-file-path ${configJson} \
                    -l ${cfg.log-format} ${
                      optionalString (cfg.stack-size != null) "--stack-size ${toString cfg.stack-size}"
                    } \
                    start
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

{ self, ... }:
{
  perSystem =
    {
      pkgs,
      crane,
      ...
    }:
    let
      voy-modules-list = builtins.filter (
        member:
        (pkgs.lib.hasPrefix "voyager/modules" member) || (pkgs.lib.hasPrefix "voyager/plugins" member)
      ) (builtins.fromTOML (builtins.readFile ../Cargo.toml)).workspace.members;

      voyager = crane.buildWorkspaceMember "voyager" {
        extraEnv = {
          SQLX_OFFLINE = "1";
        };
      };
    in
    {
      packages = voyager // {
        voyager-modules-plugins-names = builtins.toFile "voyager-modules-plugins-names.json" (
          builtins.toJSON (
            map (
              p: (builtins.fromTOML (builtins.readFile "${../.}/${p}/Cargo.toml")).package.name
            ) voy-modules-list
          )
        );
        voyager-modules-plugins =
          let
            builder =
              release:
              pkgs.symlinkJoin {
                name = "voyager-modules-plugins";
                paths = pkgs.lib.mapAttrsToList (_: path: if release then path.release else path) (
                  builtins.foldl' (acc: p: acc // (crane.buildWorkspaceMember p { })) { } voy-modules-list
                );
                postBuild = ''
                  rm $out/lib -r
                '';
              };
          in
          (builder false)
          // {
            release = builder true;
          };
      };
    };

  flake.nixosModules.voyager =
    {
      lib,
      pkgs,
      config,
      ...
    }:
    let
      inherit (lib) mkOption mkEnableOption types;
      mkMergeTopLevel =
        names: attrs:
        lib.getAttrs names (
          lib.mapAttrs (_k: v: lib.mkMerge v) (lib.foldAttrs (n: a: [ n ] ++ a) [ ] attrs)
        );
      cfg = config.services.voyager;
    in
    {
      options.services.voyager = {
        enable = mkEnableOption "Voyager services";
        instances = mkOption {
          default = [ ];
          type = types.listOf (
            types.submodule {
              options = {
                enable = mkOption {
                  type = types.bool;
                  default = true;
                };
                name = mkOption {
                  type = types.string;
                  default = "default";
                };
                package = mkOption {
                  type = types.package;
                  default = self.packages.${pkgs.system}.voyager;
                };
                settings = mkOption {
                  type = types.submodule {
                    options = {
                      runtime-max-secs = mkOption {
                        type = types.int;
                        default = 3600 * 24 * 30;
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
                        description = ''
                          The stack size (in bytes) for worker threads.
                          See <https://docs.rs/tokio/1.40.0/tokio/runtime/struct.Builder.html#method.thread_stack_size> for more information.
                        '';
                        default = null;
                        example = 20971520;
                      };
                    };
                  };
                };
                equivalent_chain_ids = mkOption {
                  type = types.listOf (types.listOf types.str);
                  default = [ ];
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
                modules = mkOption { type = types.attrs; };
                plugins = mkOption { type = types.listOf types.attrs; };
              };
            }
          );
        };
      };

      config = mkMergeTopLevel [ "systemd" "environment" "users" ] (
        map (
          instance:
          let
            configJson = pkgs.writeText "config-${instance.name}.json" (
              builtins.toJSON (
                lib.recursiveUpdate
                  (lib.filterAttrsRecursive (_n: v: v != null) {
                    inherit (instance)
                      equivalent_chain_ids
                      modules
                      plugins
                      voyager
                      ;
                  })
                  {
                    # required for the queue serde deserialization
                    voyager.queue.type = "pg-queue";
                  }
              )
            );
          in
          {
            environment.systemPackages = lib.mkIf (cfg.enable && instance.enable) [
              (pkgs.writeShellApplication {
                name = "voyager-${instance.name}";
                runtimeInputs = [ instance.package ];
                text = ''
                  ${lib.getExe instance.package} --config-file-path ${configJson} "$@"
                '';
              })
            ];
            systemd.services."voyager-${instance.name}" = lib.mkIf (cfg.enable && instance.enable) {
              wantedBy = [ "multi-user.target" ];
              description = "Voyager ${instance.name}";
              serviceConfig = {
                Type = "simple";
                User = instance.name;
                ExecStart = ''
                  ${lib.getExe instance.package} \
                    --config-file-path ${configJson} \
                    -l ${instance.settings.log-format} ${
                      lib.optionalString (
                        instance.settings.stack-size != null
                      ) "--stack-size ${toString instance.settings.stack-size}"
                    } \
                    start
                '';
                Restart = lib.mkForce "always";
                RestartSec = 10;
                RuntimeMaxSec = instance.settings.runtime-max-secs;
              };
              environment = {
                RUST_LOG = "${instance.settings.log-level}";
                RUST_LOG_FORMAT = "${instance.settings.log-format}";
              };
            };
            users = {
              users.${instance.name} = {
                isSystemUser = true;
                group = instance.name;
              };
              groups.${instance.name} = { };
            };
          }
        ) cfg.instances
      );
    };
}

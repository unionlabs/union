{ self, ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, dbg, mkCi, ... }:
    let
      attrs = {
        crateDirFromRoot = "voyager";
        extraEnv = {
          SQLX_OFFLINE = "1";
        };
      };

      voyager = crane.buildWorkspaceMember attrs;
      voyager-dev = pkgs.lib.warn
        "voyager-dev is not intended to be used in production"
        crane.buildWorkspaceMember
        (attrs // { dev = true; });
    in
    {
      packages = voyager.packages // {
        voy-send-msg = pkgs.writeShellApplication
          {
            name = "voy-send-msg";
            runtimeInputs = [ pkgs.curl ];
            text = ''
              set -e
              curl localhost:65534/msg -H "content-type: application/json" -d "$@"
            '';
          };
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
      };
      checks = voyager.checks;
    };

  flake.nixosModules.voyager = { lib, pkgs, config, ... }:
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
        chains = mkOption {
          # The configuration design is breaking quite often, would be a waste
          # of effort to fix the type for now.
          type = types.attrs;
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
          description = "RUST_LOG passed to voyager";
          example = "voyager=debug";
        };
        log-format = mkOption {
          type = types.enum [ "json" "text" ];
          default = "json";
          example = "text";
        };
        stack-size = mkOption {
          type = types.nullOr types.number;
          default = null;
          example = 20971520;
        };
        # laddr = mkOption {
        #   type = types.str;
        #   default = "0.0.0.0:65534";
        #   example = "0.0.0.0:65534";
        # };
        # max-batch-size = mkOption {
        #   type = types.number;
        #   example = 10;
        # };
        voyager-extra = mkOption {
          type = types.attrs;
          default = { };
        };
      };

      config =
        let
          configJson = pkgs.writeText "config.json" (builtins.toJSON {
            chain = cfg.chains;
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
          });
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
                    -l ${cfg.log-format} ${pkgs.lib.optionalString (cfg.stack-size != null) "--stack-size ${toString cfg.stack-size}"} \
                    relay
                '';
                Restart = mkForce "always";
                RestartSec = 10;
                RuntimeMaxSec = cfg.runtime-max-secs;
              };
              environment = {
                RUST_LOG = "${cfg.log-level}";
              };
            };
          };
        };
    };
}

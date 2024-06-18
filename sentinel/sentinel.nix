{ self, ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, dbg, ... }:
    let
      sentinel = crane.buildWorkspaceMember {
        crateDirFromRoot = "sentinel";
        extraEnv = {
          SQLX_OFFLINE = "1";
        };
      };
    in
    {
      packages = sentinel.packages;

    };
  flake.nixosModules.sentinel = { lib, pkgs, config, ... }:
    with lib;
    let
      cfg = config.services.sentinel;
    in
    {
      options.services.sentinel = {
        enable = mkEnableOption "Sentinel service";
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.sentinel;
        };
        chain_configs = mkOption {
          type = types.attrs;
        };
        interactions = mkOption {
          type = types.listOf types.attrs;
        };
        log-level = mkOption {
          type = types.str;
          default = "info";
          description = "RUST_LOG passed to sentinel";
          example = "sentinel=info";
        };
        log-format = mkOption {
          type = types.enum [ "json" "text" ];
          default = "json";
          example = "text";
        };
      };
      config =
        let
          configJson = pkgs.writeText "config.json" (builtins.toJSON {
            chain_configs = cfg.chain_configs;
            interactions = cfg.interactions;
          });

        in
        mkIf cfg.enable {
          systemd.services.sentinel = {
            description = "Sentinel";
            wantedBy = [ "multi-user.target" ];
            after = [ "network.target" ];
            serviceConfig = {
              Type = "simple";
              ExecStart = "${pkgs.lib.getExe cfg.package} --config ${configJson}";
              Restart = "always";
              RestartSec = 10;
              # User = "sentinel";
              # Group = "sentinel";
            };
            environment = {
              RUST_LOG = "INFO";
            };
          };
        };
    };
}

{ self, ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, dbg, mkCi, ... }:
    let
      faucet = crane.buildWorkspaceMember {
        crateDirFromRoot = "faucet-rs";
      };
    in
    {
      packages.faucet = faucet.packages.faucet-rs;
    };

  flake.nixosModules.faucet = { lib, pkgs, config, ... }:
    with lib;
    let cfg = config.services.faucet;
    in {
      options.services.faucet = {
        enable = mkEnableOption "Faucet service";
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.faucet;
        };
        config = mkOption {
          type = types.attrs;
          description = "config file";
        };
        log-level = mkOption {
          type = types.str;
          description = "RUST_LOG";
        };
      };

      config = mkIf cfg.enable {
        systemd.services.faucet =
          let
            faucet-systemd-script = pkgs.writeShellApplication {
              name = "faucet-systemd";
              runtimeInputs = [ pkgs.coreutils cfg.package ];
              text = ''
                ${pkgs.lib.getExe cfg.package} -c '${builtins.toFile "faucet-config.json" (builtins.toJSON cfg.config)}'
              '';
            };
          in
          {
            wantedBy = [ "multi-user.target" ];
            description = "Faucet";
            serviceConfig = {
              Type = "simple";
              ExecStart = pkgs.lib.getExe faucet-systemd-script;
              Restart = mkForce "always";
            };
            environment = {
              RUST_LOG = cfg.log-level;
            };
          };
      };
    };
}


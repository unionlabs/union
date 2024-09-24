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
      drip = crane.buildWorkspaceMember {
        crateDirFromRoot = "drip";
      };
    in
    {
      packages.drip = drip.packages.drip;
    };

  flake.nixosModules.drip =
    {
      lib,
      pkgs,
      config,
      ...
    }:
    with lib;
    let
      cfg = config.services.drip;
    in
    {
      options.services.drip = {
        enable = mkEnableOption "drip service";
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.drip;
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
        systemd.services.drip =
          let
            drip-systemd-script = pkgs.writeShellApplication {
              name = "drip-systemd";
              runtimeInputs = [
                pkgs.coreutils
                cfg.package
              ];
              text = ''
                ${pkgs.lib.getExe cfg.package} -c '${builtins.toFile "drip-config.json" (builtins.toJSON cfg.config)}'
              '';
            };
          in
          {
            wantedBy = [ "multi-user.target" ];
            description = "drip";
            serviceConfig = {
              Type = "simple";
              ExecStart = pkgs.lib.getExe drip-systemd-script;
              Restart = mkForce "always";
            };
            environment = {
              RUST_LOG = cfg.log-level;
            };
          };
      };
    };
}

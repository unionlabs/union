{ self, ... }:
{
  perSystem =
    {
      crane,
      ...
    }:
    let
      lst-ocw = crane.buildWorkspaceMember "cosmwasm/lst-ocw" { };
    in
    {
      packages = lst-ocw;
    };

  flake.nixosModules.lst-ocw =
    {
      lib,
      pkgs,
      config,
      ...
    }:
    let
      inherit (lib) mkOption mkEnableOption types;
      cfg = config.services.lst-ocw;
    in
    {
      options.services.lst-ocw = {
        enable = mkEnableOption "lst-ocw service";
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.lst-ocw;
        };
        log-level = mkOption {
          type = types.str;
          default = "info";
          description = "RUST_LOG passed to lst-ocw.";
          example = "lst-ocw=debug";
        };
        log-format = mkOption {
          type = types.enum [
            "json"
            "text"
          ];
          default = "json";
          description = "The log format to use.";
          example = "text";
        };
        rpc-url = mkOption {
          type = types.str;
          description = "The RPC url to use for this chain.";
        };
        lst-hub = mkOption {
          type = types.str;
          description = "The address of the lst-hub on this chain.";
        };
        private-key = mkOption {
          type = types.str;
          description = "The 0x-prefixed private key for the signer that will be used to submit transactions.";
        };
      };

      config = lib.mkIf cfg.enable {
        systemd.services.lst-ocw =
          let
            lst-ocw-systemd-script = pkgs.writeShellApplication {
              name = "lst-ocw-systemd";
              runtimeInputs = [
                pkgs.coreutils
                cfg.package
              ];
              text = ''
                ${lib.getExe cfg.package} \
                  run \
                  --rpc-url ${cfg.rpc-url} \
                  --private-key ${cfg.private-key} \
                  --lst-hub ${cfg.lst-hub} \
                  --log-format ${cfg.log-format}
              '';
            };
          in
          {
            wantedBy = [ "multi-user.target" ];
            description = "lst-ocw";
            serviceConfig = {
              Type = "simple";
              ExecStart = lib.getExe lst-ocw-systemd-script;
              Restart = lib.mkForce "always";
            };
            environment = {
              RUST_LOG = cfg.log-level;
            };
          };
      };
    };
}

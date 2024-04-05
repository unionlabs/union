{ self, ... }: {
  perSystem = { self', pkgs, crane, ... }:
    let
      nostromo = crane.buildWorkspaceMember {
        crateDirFromRoot = "nostromo";
        cargoTestExtraAttrs = {
          partitions = 1;
          partitionType = "count";
        };
      };
    in
    {
      inherit (nostromo) checks;
      packages = {
        nostromo = nostromo.packages.nostromo;

        nostromo-image = pkgs.dockerTools.buildLayeredImage {
          name = "nostromo";
          contents = [ pkgs.coreutils-full pkgs.cacert self'.packages.nostromo ];
          config = {
            Entrypoint = [ (pkgs.lib.getExe self'.packages.nostromo) ];
            Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
          };
        };
      };
    };

  flake.nixosModules.nostromo = { lib, pkgs, config, ... }:
    with lib;
    let
      cfg = config.services.nostromo;
    in
    {
      options.services.nostromo = {
        enable = mkEnableOption "Nostromo service";
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.nostromo;
        };
        url = mkOption {
          type = types.str;
          default = "https://nostromo.union.build";
        };
        metrics-addr = mkOption {
          type = types.str;
          default = "0.0.0.0:9090";
        };
        log-level = mkOption {
          type = types.str;
          default = "info";
          description = "RUST_LOG passed to nostromo";
          example = "nostromo=debug";
        };
      };

      config = mkIf cfg.enable {
        systemd.services.nostromo =
          let
            nostromo-systemd-script = pkgs.writeShellApplication {
              name = "nostromo-systemd";
              runtimeInputs = [ pkgs.coreutils cfg.package ];
              text =
                ''
                  ${pkgs.lib.getExe cfg.package}
                '';
            };
          in
          {
            wantedBy = [ "multi-user.target" ];
            description = "Nostromo";
            serviceConfig = {
              Type = "simple";
              ExecStart = pkgs.lib.getExe nostromo-systemd-script;
              Restart = mkForce "always";
            };
            environment = {
              RUST_LOG = "${cfg.log-level}";
            };
          };
      };
    };
}

{ self, ... }: {
  perSystem = { self', pkgs, crane, ... }:
    let
      hubble = crane.buildWorkspaceMember {
        crateDirFromRoot = "hubble";
        additionalSrcFilter = path: _type: pkgs.lib.hasPrefix "hubble/src/graphql/" path;
        cargoTestExtraAttrs = {
          partitions = 1;
          partitionType = "count";
        };
      };
    in
    {
      inherit (hubble) checks;
      packages = {
        hubble = hubble.packages.hubble;

        hubble-image = pkgs.dockerTools.buildLayeredImage {
          name = "hubble";
          contents = [ pkgs.coreutils-full pkgs.cacert self'.packages.hubble ];
          config = {
            Entrypoint = [ (pkgs.lib.getExe self'.packages.hubble) ];
            Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
          };
        };
      };
    };

  flake.nixosModules.hubble = { lib, pkgs, config, ... }:
    with lib;
    let cfg = config.services.hubble;
    in {
      options.services.hubble = {
        enable = mkEnableOption "Hubble service";
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.hubble;
        };
        url = mkOption {
          type = types.str;
          default = "https://graphql.union.build";
        };
        metrics-addr = mkOption {
          type = types.str;
          default = "0.0.0.0:9090";
        };
        hasura-admin-secret = mkOption {
          type = types.str;
          default = "";
        };
        indexers = mkOption {
          type = types.listOf (
            types.submodule {
              options.url = mkOption { type = types.str; example = "https://rpc.example.com"; };
              options.type = mkOption { type = types.enum [ "tendermint" ]; };
            }
          );
        };
      };

      config = mkIf cfg.enable {
        systemd.services.hubble =
          let
            hubble-systemd-script = pkgs.writeShellApplication {
              name = "hubble-systemd";
              runtimeInputs = [ pkgs.coreutils cfg.package ];
              text =
                let
                  secretArg = if cfg.hasura-admin-secret != "" then "--secret ${cfg.hasura-admin-secret}" else "";
                  indexersJson = builtins.toJSON cfg.indexers;
                in
                ''
                  ${pkgs.lib.getExe cfg.package}  \
                    --metrics-addr ${cfg.metrics-addr} \
                    --url ${cfg.url} ${secretArg} \
                    --indexers '${indexersJson}'
                '';
            };
          in
          {
            wantedBy = [ "multi-user.target" ];
            description = "Hubble";
            serviceConfig = {
              Type = "simple";
              ExecStart = pkgs.lib.getExe hubble-systemd-script;
              Restart = mkForce "always";
            };
          };
      };
    };

}

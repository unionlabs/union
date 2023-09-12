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
          type = types.string;
          default = "https://graphql.union.build";
        };
        hasura-admin-secret = mkOption {
          type = types.string;
          default = "";
        };
        indexers = mkOption {
          type = types.listOf types.attrset;
          description = ''
            Note that example.chain_id is optional.
          '';
          example = [
            {
              url = "https://rpc.example.com";
              chain_id = "union-example-devnet";
            }
          ];
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
                  ${pkgs.lib.getExe cfg.package} --url ${cfg.url} ${secretArg} --indexers ${indexersJson}
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

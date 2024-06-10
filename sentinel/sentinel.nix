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
        db_url = mkOption {
          type = types.str;
        };
        ethereum = mkOption {
          # The configuration design is breaking quite often, would be a waste
          # of effort to fix the type for now.
          type = types.attrs;
        };
        osmosis = mkOption {
          # The configuration design is breaking quite often, would be a waste
          # of effort to fix the type for now.
          type = types.attrs;
        };
        union = mkOption {
          # The configuration design is breaking quite often, would be a waste
          # of effort to fix the type for now.
          type = types.attrs;
        };
        osmosis_contract = mkOption {
          # The configuration design is breaking quite often, would be a waste
          # of effort to fix the type for now.
          type = types.str;
        };
        union_contract = mkOption {
          # The configuration design is breaking quite often, would be a waste
          # of effort to fix the type for now.
          type = types.str;
        };
        ethereum_contract = mkOption {
          # The configuration design is breaking quite often, would be a waste
          # of effort to fix the type for now.
          type = types.str;
        };
        ethereum_priv_key = mkOption {
          # The configuration design is breaking quite often, would be a waste
          # of effort to fix the type for now.
          type = types.str;
        };
        channel = mkOption {
          # The configuration design is breaking quite often, would be a waste
          # of effort to fix the type for now.
          type = types.str;
          default = "channel-0";
        };
        counterparty_channel = mkOption {
          # The configuration design is breaking quite often, would be a waste
          # of effort to fix the type for now.
          type = types.str;
          default = "channel-0";
        };
        amount = mkOption {
          # The configuration design is breaking quite often, would be a waste
          # of effort to fix the type for now.
          type = types.str;
          default = "1";
        };
        # datadog_data = mkOption {
        #   # The configuration design is breaking quite often, would be a waste
        #   # of effort to fix the type for now.
        #   type = types.attrs;
        # };
        connections = mkOption {
          # The configuration design is breaking quite often, would be a waste
          # of effort to fix the type for now.
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
            ethereum = cfg.ethereum;
            osmosis = cfg.osmosis;
            union = cfg.union;
            osmosis_contract = cfg.osmosis_contract;
            ethereum_contract = cfg.ethereum_contract;
            union_contract = cfg.union_contract;
            ethereum_priv_key = cfg.ethereum_priv_key;
            channel = cfg.channel;
            counterparty_channel = cfg.counterparty_channel;
            amount = cfg.amount;
            # datadog_data = cfg.datadog_data;
            connections = cfg.connections;
            db_url = cfg.db_url;
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
              # Restart = "always";
              # RestartSec = 10;
              # User = "sentinel";
              # Group = "sentinel";
            };
            environment = {
              RUST_LOG = "${cfg.log-level}";
            };
          };
        };
    };
}


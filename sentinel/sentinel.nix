{ self, ... }:
{
  # This “perSystem” function is where we build the sentinel package.
  perSystem = { pkgs, unstablePkgs, ensureAtRepositoryRoot, ... }:
    let
      sentinelApp = unstablePkgs.writeShellApplication {
        name = "sentinel";
        text = ''
          # Make sure we are at repository root
          # where are we?
          echo "Current directory: $(pwd)"
          
          ${ensureAtRepositoryRoot}

          # Navigate to sentinel/ subdirectory
          cd sentinel/

          # Install dependencies and build
          npm install
          npm run build

          # Run it
          node dist/sentinel.js "$@"
        '';
      };
    in
    {
      packages = {
        sentinel = sentinelApp;
      };
    };

  # This is your module that makes a systemd service and so forth
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
          # Now we can reference it correctly
          default = self.packages.${pkgs.system}.sentinel;
        };
        cycleIntervalMs = mkOption {
          type = types.number;
          description = "Interval between cycles in milliseconds";
        };
        interactions = mkOption {
          type = types.listOf types.attrs;
          description = "Interactions for cross-chain communication.";
        };
        logLevel = mkOption {
          type = types.str;
          default = "info";
          description = "Log level for Sentinel";
        };
      };

      config = mkIf cfg.enable {
        # Write config.json from user-provided cycleIntervalMs & interactions
        # so the sentinel script can read them
        systemd.services.sentinel = {
          description = "Sentinel Service";
          wantedBy = [ "multi-user.target" ];
          after = [ "network.target" ];
          serviceConfig = {
            Type = "simple";
            ExecStart = ''
              ${pkgs.lib.getExe cfg.package} --config ${
                pkgs.writeText "config.json" (builtins.toJSON {
                  cycleIntervalMs = cfg.cycleIntervalMs;
                  interactions = cfg.interactions;
                })
              }
            '';
            Restart = "always";
            RestartSec = 10;
          };
          environment = {
            NODE_ENV = "production";
            LOG_LEVEL = cfg.logLevel;
          };
        };
      };
    };
}

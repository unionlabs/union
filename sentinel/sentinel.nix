{ self, ... }: {
  perSystem = { pkgs, unstablePkgs, ensureAtRepositoryRoot, ... }:
    let
      # Define the TypeScript-based Sentinel application
      sentinelApp = unstablePkgs.writeShellApplication {
        name = "sentinel";
        text = ''
          ${ensureAtRepositoryRoot}

          # Navigate to the correct project directory
          cd sentinel/

          # Ensure dependencies are installed
          if [ ! -f package.json ]; then
            echo "Error: package.json not found in sentinel directory"
            exit 1
          fi
          npm install

          # Build the TypeScript application
          npm run build

          # Run the built application with arguments
          node dist/sentinel.js "$@"
        '';
      };
    in
    {
      # Expose the app for use
      apps = {
        sentinel = {
          type = "app";
          program = sentinelApp;
        };
      };
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
          default = self.apps.sentinel.program;
        };
        cycleIntervalMs = mkOption {
          type = types.attrs;
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
          example = "info";
        };
        logFormat = mkOption {
          type = types.enum [ "json" "text" ];
          default = "json";
          description = "Log format for Sentinel output";
          example = "text";
        };
      };

      config =
        let
          # Generate the config.json file from NixOS options
          configJson = pkgs.writeText "config.json" (builtins.toJSON {
            cycleIntervalMs = cfg.cycleIntervalMs;
            interactions = cfg.interactions;
          });
        in
        mkIf cfg.enable {
          # Define the systemd service
          systemd.services.sentinel = {
            description = "Sentinel Service";
            wantedBy = [ "multi-user.target" ];
            after = [ "network.target" ];
            serviceConfig = {
              Type = "simple";
              ExecStart = "${cfg.package} --config ${configJson} -l ${cfg.logFormat}";
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

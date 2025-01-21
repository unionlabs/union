{ self, ... }:
{
  perSystem =
    { pkgs
    , jsPkgs
    , ensureAtRepositoryRoot
    , lib
    , ...
    }:
    let
      deps = with jsPkgs; [
        python3
        pkg-config
        nodePackages_latest.nodejs
        nodePackages_latest."patch-package"
      ];
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        sentinel = jsPkgs.buildNpmPackage {
          npmDepsHash = "sha256-ftC6pM+l9fiyJ52voMYILusrVd0BuJ1FFJy+0gY8jyo=";
          src = ./.;
          sourceRoot = "sentinel";
          npmFlags = [
            "--loglevel=verbose"
            "--enable-pre-post-scripts"
          ];
          pname = packageJSON.name;
          inherit (packageJSON) version;
          nativeBuildInputs = deps;
          buildInputs = deps;
          installPhase = ''
            echo "Current directory: $(pwd)"
            echo "out is $out"
            mkdir -p $out
            echo "under build: $(ls ./build)"
            cp -r ./build/* $out
          '';
          doDist = false;
          NODE_OPTIONS = "--no-warnings";
        };
      };

      apps = {
        sentinel-dev = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "sentinel-dev-server";
            runtimeInputs = deps;
            text = ''
                      ${ensureAtRepositoryRoot}
                      cd sentinel/

                  npm run build
              node dist/sentinel.js "$@"
            '';
          };
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

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
          npmDepsHash = "sha256-TMCjwCmLKmjGGn1t0v3zq+f0XaluP4rvJtm5RWE14H4=";
          src = ./.;
          sourceRoot = "sentinel";
          npmFlags = [
            "--loglevel=verbose"
            "--enable-pre-post-scripts"
          ];
          pname = packageJSON.name;
          inherit (packageJSON) version;
          nativeBuildInputs = deps;
          buildInputs = [ pkgs.bashInteractive ];
          installPhase = ''
                        echo "Current directory: $(pwd)"
                        echo "out is $out"

                        # 1) Copy the compiled ESM .js
                        mkdir -p $out/lib
                        cp -r dist/* $out/lib

                        # 2) Copy node_modules
                        cp -r node_modules $out/lib/node_modules

                        # 3) Copy package.json
                        cp package.json $out/lib

                        # 4) Create a Bash wrapper in $out/bin
                        mkdir -p $out/bin

                        # IMPORTANT: Expand $out now, at build time, so the final script has a literal store path
                          cat <<EOF > $out/bin/sentinel
            #!${pkgs.bashInteractive}/bin/bash
            cd "$out/lib"
            export NODE_PATH="$out/lib/node_modules"
            EOF


                        echo 'exec '"${pkgs.nodePackages_latest.nodejs}/bin/node"' sentinel.js "$@"' >> $out/bin/sentinel

                        chmod +x $out/bin/sentinel
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
        transfers = mkOption {
          type = types.listOf types.attrs;
          description = "Array for cross-chain transfers.";
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
                  transfers = cfg.transfers;
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

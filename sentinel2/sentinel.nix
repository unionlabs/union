{ self, ... }:
let
  sentinel2Module =
    { lib, pkgs, config, ... }:
    with lib;
    let
      cfg = config.services.sentinel2;
    in
    {
      options.services.sentinel2 = {
        enable = mkEnableOption "sentinel2 Service";
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.sentinel2;
        };
        cycleIntervalMs = mkOption {
          type = types.number;
          description = "Interval between cycles in milliseconds";
        };
        hasuraEndpoint = mkOption {
          type = types.str;
          description = "Hasura endpoint for graphql requests.";
        };
        transfers = mkOption {
          type = types.listOf types.attrs;
          description = "Array for cross-chain transfers.";
        };
        signer_account_mnemonic = mkOption {
          type = types.str;
          description = "mnemonic to send tokens to babylon users";
        };
        chainConfig = mkOption {
          type = types.attrs;
          description = "chainConfig for escrow-totalsupply control.";
        };
        interactions = mkOption {
          type = types.listOf types.attrs;
          description = "Interactions for cross-chain communication.";
        };
        logLevel = mkOption {
          type = types.str;
          default = "info";
          description = "Log level for sentinel2";
        };
      };

      config = mkIf cfg.enable {
        systemd.services.sentinel2 = {
          description = "sentinel2 Service";
          wantedBy = [ "multi-user.target" ];
          after = [ "network.target" ];

          ## ← NEW: create and run in a writable directory
          StateDirectory   = "sentinel2";
          WorkingDirectory = "/var/lib/sentinel2";

          serviceConfig = {
            Type       = "simple";
            Restart    = "always";
            RestartSec = 10;

            ExecStart = ''
              ${pkgs.lib.getExe cfg.package} --config /var/lib/sentinel2/config.json
            '';
          };

          environment = {
            NODE_ENV  = "production";
            LOG_LEVEL = cfg.logLevel;
          };

          ## ← NEW: dump your JSON config into /var/lib/sentinel2 before start
          preStart = ''
            cat > /var/lib/sentinel2/config.json <<EOF
${builtins.toJSON {
  cycleIntervalMs         = cfg.cycleIntervalMs;
  interactions            = cfg.interactions;
  transfers               = cfg.transfers;
  signer_account_mnemonic = cfg.signer_account_mnemonic;
  chainConfig             = cfg.chainConfig;
  hasuraEndpoint          = cfg.hasuraEndpoint;
}}
EOF
          '';
        };
      };
    };
in
{
  perSystem =
    { pkgs,
      pkgsUnstable,
      ensureAtRepositoryRoot,
      lib,
      ...
    }:
    let
      deps = with pkgsUnstable; [
        python3
        pkg-config
        sqlite
        nodePackages_latest."patch-package"
      ];
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        sentinel2 = pkgsUnstable.buildNpmPackage {
          nodejs = pkgs.nodejs;
          npmDepsHash = "sha256-4Od3bakA4AqPCnw+8mYqQOmf65qlYJ9kLEMgSZ5JVpQ=";
          src = ./.;
          sourceRoot = "sentinel2";
          npmFlags = [
            "--loglevel=verbose"
            "--enable-pre-post-scripts"
          ];
          pname = packageJSON.name;
          inherit (packageJSON) version;
          nativeBuildInputs = with pkgs; [
            python3
            pkg-config
            sqlite
            nodejs
            nodePackages_latest."patch-package"
          ];

          buildInputs = [ pkgs.bashInteractive pkgs.sqlite ];
          postBuild = ''
            npm rebuild better-sqlite3 --build-from-source
          '';
          installPhase = ''
            echo "Current directory: $(pwd)"
            echo "out is $out"

            # 1) Copy compiled ESM
            mkdir -p $out/lib
            cp -r dist/* $out/lib

            # 2) Copy node_modules (with rebuilt better-sqlite3)
            rm -rf $out/lib/node_modules
            cp -r node_modules $out/lib/node_modules

            # 3) Copy package.json
            cp package.json $out/lib

            # 4) Wrapper that runs in /var/lib/sentinel2
            mkdir -p $out/bin
            cat <<EOF > $out/bin/sentinel2
#!${pkgs.bashInteractive}/bin/bash
export PATH=${pkgs.nodejs}/bin:\$PATH
export NODE_PATH="$out/lib/node_modules:$out/lib"

# ← run from a writable directory so funded‑txs.db can be opened/created
# cd /var/lib/sentinel2

exec node $out/lib/src/sentinel2.js "\$@"
EOF
            chmod +x $out/bin/sentinel2
          '';

          doDist = false;
          NODE_OPTIONS = "--no-warnings";
        };
      };

      apps = {
        sentinel2-dev = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "sentinel2-dev-server";
            runtimeInputs = deps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd sentinel2/

              npm install
              npm run build
              node dist/src/sentinel2.js "$@"
            '';
          };
        };
      };
    };

  # expose the module
  flake.nixosModules.sentinel2 = sentinel2Module;
}

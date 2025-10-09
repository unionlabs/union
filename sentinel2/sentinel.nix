{ self, ... }:
let
  sentinel2Module =
    {
      lib,
      pkgs,
      config,
      ...
    }:
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
        trigger_betterstack = mkOption {
          type = types.bool;
          description = "Boolean to decide if trigger betterstack or not";
        };
        rpcHostEndpoints = mkOption {
          type = types.listOf types.str;
          description = "RPC endpoints for SSL certificate control.";
        };
        signer_account_mnemonic = mkOption {
          type = types.str;
          description = "mnemonic to send tokens to babylon users";
        };
        dbPath = mkOption {
          type = types.str;
          description = "Path for sqlite db";
        };
        betterstack_api_key = mkOption {
          type = types.str;
          description = "Betterstack api key";
        };
        chainConfig = mkOption {
          type = types.attrs;
          description = "chainConfig for escrow-totalsupply control.";
        };
        signerBalances = mkOption {
          type = types.attrs;
          description = "Signer balances mapping for balance control";
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
          serviceConfig = {
            WorkingDirectory = "/var/lib/sentinel2";
            ExecStartPre = ''
              ${pkgs.coreutils}/bin/install -d -m0755 -o sentinel2 -g sentinel2 /var/lib/sentinel2
            '';
            User = "sentinel2";
            Group = "sentinel2";
            Type = "simple";
            ExecStart = ''
              ${pkgs.lib.getExe cfg.package} --config ${
                pkgs.writeText "config.json" (
                  builtins.toJSON {
                    inherit (cfg) cycleIntervalMs;
                    inherit (cfg) signer_account_mnemonic;
                    inherit (cfg) betterstack_api_key;
                    inherit (cfg) trigger_betterstack;
                    inherit (cfg) dbPath;
                    inherit (cfg) chainConfig;
                    inherit (cfg) signerBalances;
                    inherit (cfg) hasuraEndpoint;
                    inherit (cfg) rpcHostEndpoints;
                  }
                )
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
in
{
  perSystem =
    {
      pkgs,
      pkgsUnstable,
      ensureAtRepositoryRoot,
      buildPnpmPackage,
      ...
    }:
    let
      deps = with pkgsUnstable; [
        sqlite
      ];
    in
    {
      packages = {
        sentinel2 = buildPnpmPackage {
          packageJsonPath = ./package.json;
          hash = "sha256-2894Wpx9xSNckonH7EKxRtZqtkWtMsoo5oskfBstTSA=";
          extraSrcs = [
            ../sentinel2
            ../ts-sdk
            ../ts-sdk-evm
            ../ts-sdk-cosmos
          ];
          npmFlags = [
            "--loglevel=verbose"
            "--enable-pre-post-scripts"
          ];
          pnpmWorkspaces = [
            "sentinel2"
            "@unionlabs/sdk"
            "@unionlabs/sdk-cosmos"
            "@unionlabs/sdk-evm"
          ];
          nativeBuildInputs = with pkgs; [
            python3
            pkg-config
            sqlite
          ];
          buildInputs = [
            pkgs.bashInteractive
            pkgs.sqlite
          ];
          buildPhase = ''
            runHook preBuild
            pnpm --filter=sentinel2 build
            runHook postBuild
          '';
          postBuild = ''
            # TODO(ehegnes): restore sqlite
            # pnpm rebuild better-sqlite3 --build-from-source
          '';
          installPhase = ''
            mkdir -p $out
            cp -r ./sentinel2/build/* $out

            # 2) Copy node_modules (with rebuilt better-sqlite3)
            # rm -rf $out/lib/node_modules
            # cp -r node_modules $out/lib/node_modules

            # 3) Copy package.json
            # cp package.json $out/lib

            # 4) Create a Bash wrapper in $out/bin
            # mkdir -p $out/bin

            # IMPORTANT: Expand $out now, at build time, so the final script has a literal store path
            # cat <<EOF > $out/bin/sentinel2
            # #!${pkgs.bashInteractive}/bin/bash
            # export PATH=${pkgsUnstable.nodejs_24}/bin:\$PATH
            # cd "$out/lib"
            # export NODE_PATH="$out/lib/node_modules"
            # EOF

            # echo 'exec '"${pkgsUnstable.nodejs_24}/bin/node"' esm/sentinel2.js "$@"' >> $out/bin/sentinel2
            # chmod +x $out/bin/sentinel2
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

              pnpm install
              pnpm --filter=sentinel2 build
              node dist/src/sentinel2.js "$@"
            '';
          };
        };
      };
    };

  flake.nixosModules.sentinel2 = sentinel2Module;
}

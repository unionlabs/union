{ ... }: {
  perSystem = { self', pkgs, system, config, inputs', crane, stdenv, ... }:
    let
      attrs = crane.commonAttrs // {
        inherit (crane) cargoArtifacts;
        cargoExtraArgs = "-p relayer";
      } // (crane.lib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; });

      relayer = crane.lib.buildPackage attrs;
    in
    {
      packages.relayer = relayer;
      packages.round-trip = pkgs.writeShellApplication {
        name = "relayer-round-trip";
        runtimeInputs = [ relayer ];
        text = ''
          export RUST_LOG=relayer=debug,tendermint=debug,info

          relayer client create union ethereum08-wasm --counterparty ethereum-devnet --on union-devnet --evm-preset minimal

          relayer client create evm cometbls --on ethereum-devnet --counterparty union-devnet

          relayer connection open --to-chain union-devnet --to-client 08-wasm-0 --from-chain ethereum-devnet --from-client cometbls-new-0
        '';
      };

      checks = crane.mkChecks "relayer" {
        clippy = crane.lib.cargoClippy (attrs // {
          cargoClippyExtraArgs = "-- --deny warnings --no-deps";
        });

        tests = crane.lib.cargoNextest attrs;
      };
    };

  flake.nixosModules.relayer = { lib, pkgs, config, ... }:
    with lib;
    let
      cfg = config.services.relayer;
    in
    {
      options.services.relayer = {
        enable = mkEnableOption "Union relayer service";
        config_file = mkOption {
          type = types.attrsOf types.inferred;
        };
      };

      config = mkIf cfg.enable {
        systemd.services.relayer =
          let
            relayer-systemd-script = pkgs.writeShellApplication {
              name = "relayer-systemd";
              runtimeInputs = [ pkgs.coreutils self.packages.${pkgs.system}.relayer ];
              text = ''
                mkdir -p /var/lib/relayer 
                # for dump output
                mkdir -p /var/lib/relayer/dump 
                cd /var/lib/relayer 

                cp ${pkgs.writeText "config.json" (builtins.toJSON cfg.config_file)} /var/lib/relayer/config.json

                relayer --config-file-path /var/lib/relayer/config.json relay --between union-testnet:sepolia
              '';
            };
          in
          {
            wantedBy = [ "multi-user.target" ];
            description = "relayer";
            serviceConfig = {
              Type = "simple";
              ExecStart = pkgs.lib.getExe relayer-systemd-script;
              Restart = mkForce "always";
            };
          };
      };
    };
}


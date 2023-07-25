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
          RUST_LOG=relayer=debug,tendermint=debug,info

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
}


{ ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, ... }:
    let
      relayerAll = (crane.buildWorkspaceMember {
        crateDirFromRoot = "relayer";
      });
    in
    {
      packages = {
        relayer = relayerAll.packages.relayer;
        round-trip = pkgs.writeShellApplication {
          name = "relayer-round-trip";
          runtimeInputs = [ relayerAll.packages.relayer ];
          text = ''
            export RUST_LOG=relayer=debug,tendermint=debug,info

            relayer client create union ethereum08-wasm --counterparty ethereum-devnet --on union-devnet --evm-preset minimal

            relayer client create evm cometbls --on ethereum-devnet --counterparty union-devnet

            relayer connection open --to-chain union-devnet --to-client 08-wasm-0 --from-chain ethereum-devnet --from-client cometbls-new-0
          '';
        };
      };

      checks = relayerAll.checks;
    };
}

{ ... }: {
  perSystem = { self', pkgs, system, config, inputs', crane, ... }:
    let
      attrs = crane.commonAttrs
        // (crane.lib.crateNameFromCargoToml { cargoToml = ./ethereum-light-client/Cargo.toml; })
        // {
        cargoExtraArgs = "-p union-ethereum-lc --features eth-minimal";
      };

      # cargoArtifacts = crane.lib.buildDepsOnly attrs;

      CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
    in
    {
      packages = {
        wasm-ethereum-lc = (crane.withBuildTarget CARGO_BUILD_TARGET).buildPackage (attrs // {
          inherit CARGO_BUILD_TARGET;

          # RUSTFLAGS are used to optimize the binary size
          installPhase = ''
            mkdir -p $out/lib
            # Optimize the binary size a little bit more
            mv target/wasm32-unknown-unknown/release/union_ethereum_lc.wasm $out/lib/union_ethereum_lc.wasm
            # ${pkgs.binaryen}/bin/wasm-opt -Os target/wasm32-unknown-unknown/release/union_ethereum_lc.wasm -o $out/lib/union_ethereum_lc.wasm
            # We also zip the binary since it is smaller
            gzip -fk $out/lib/union_ethereum_lc.wasm
          '';
        });
      };

      checks = crane.mkChecks "wasm-ethereum-lc" {
        clippy = crane.lib.cargoClippy (attrs // { inherit (crane) cargoArtifacts; });
        test = crane.lib.cargoTest (attrs // { inherit (crane) cargoArtifacts; });
      };
    };
}

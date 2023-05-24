{ ... }: {
  perSystem = { self', pkgs, system, config, inputs', crane, ... }:
    let
      rustToolchain = crane.withBuildTarget CARGO_BUILD_TARGET;

      attrs = (crane.lib.crateNameFromCargoToml { cargoToml = ./ethereum-light-client/Cargo.toml; })
        // {
        cargoExtraArgs = "-p union-ethereum-lc --features eth-minimal";
        src = crane.rustSrc;
        cargoVendorDir = crane.lib.vendorMultipleCargoDeps {
          inherit (crane.lib.findCargoFiles crane.rustSrc) cargoConfigs;
          cargoLockList = [
            ../Cargo.lock
            "${rustToolchain.toolchain.passthru.availableComponents.rust-src}/lib/rustlib/src/rust/Cargo.lock"
          ];
        };
      };

      CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
    in
    {
      packages = {
        wasm-ethereum-lc = rustToolchain.buildPackage (attrs // {
          inherit CARGO_BUILD_TARGET;

          cargoBuildCommand = "RUSTFLAGS='-C target-feature=-sign-ext -C link-arg=-s -C target-cpu=mvp' cargo -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort build --release --lib --target ${CARGO_BUILD_TARGET}";

          checkPhase = ''
            cargo test ${attrs.cargoExtraArgs} --target ${crane.hostTarget}

            # grep exits 0 if a match is found
            if ${pkgs.binaryen}/bin/wasm-dis target/wasm32-unknown-unknown/release/union_ethereum_lc.wasm | grep -P '\.extend\d{1,2}_s'
            then
              echo "wasm binary contains invalid opcodes!"
              exit 1
            else
              echo "wasm binary doesn't contain any sign-extension opcodes!"
            fi
          '';

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

      checks = crane.mkChecks "wasm-ethereum-lc"
        {
          clippy = crane.lib.cargoClippy (attrs // { inherit (crane) cargoArtifacts; });
          test = crane.lib.cargoTest (attrs // { inherit (crane) cargoArtifacts; });
        };
    };
}

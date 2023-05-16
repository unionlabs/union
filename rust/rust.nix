{ ... }: {
  perSystem = { pkgs, crane, ... }: {
    packages = {
      wasm-ethereum-lc = crane.nightly.buildPackage {
        pname = "union-ethereum-lc";
        version = "0.1.0";
        src = ./.;
        # RUSTFLAGS are used to optimize the binary size
        cargoBuildCommand = "RUSTFLAGS='-C link-arg=-s' cargo build --release --lib --target wasm32-unknown-unknown -p union-ethereum-lc --features eth-minimal";
        installPhase = ''
          mkdir -p $out/lib
          # Optimize the binary size a little bit more
          mv target/wasm32-unknown-unknown/release/union_ethereum_lc.wasm $out/lib/union_ethereum_lc.wasm
          # ${pkgs.binaryen}/bin/wasm-opt -Os target/wasm32-unknown-unknown/release/union_ethereum_lc.wasm -o $out/lib/union_ethereum_lc.wasm
          # We also zip the binary since it is smaller
          gzip -fk $out/lib/union_ethereum_lc.wasm
        '';
        cargoTestCommand = "cargo test --features eth-minimal --profile release";
      };
    };
  };
}

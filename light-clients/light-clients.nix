{ ... }: {
  perSystem = { pkgs, crane, ... }: {
    packages = {
      wasm-ethereum-lc = crane.nightly.buildPackage {
        pname = "union-ethereum-lc";
        version = "0.1.0";
        src = ./.;
        # RUSTFLAGS are used to optimize the binary size
        # cargoBuildCommand = "RUSTFLAGS='-C link-arg=-s' cargo build --release --lib --target wasm32-unknown-unknown -p union-ethereum-lc";
        cargoBuildCommand = "RUSTFLAGS='-C target-feature=-sign-ext -C link-arg=-s -C target-cpu=mvp' cargo -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort build --release --lib --target wasm32-unknown-unknown -p union-ethereum-lc --features eth-minimal";
        installPhase = ''
          mkdir -p $out/lib
          # Optimize the binary size a little bit more
          ${pkgs.binaryen}/bin/wasm-opt -Os target/wasm32-unknown-unknown/release/union_ethereum_lc.wasm -o $out/lib/union_ethereum_lc.wasm
          # We also put zipped binary since it is smaller
          gzip -fk $out/lib/union_ethereum_lc.wasm
        '';
        checkPhase = ''
          cargo test --features eth-minimal   
          # TODO(aeryz): check if the binary produces floating point operation
        '';
      };
    };
  };
}

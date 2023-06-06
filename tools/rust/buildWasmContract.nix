{ crane
, pkgs
, lib
,
}:
{ cargoToml
, features ? [ ]
, cargoLock
, doCheck ? true
, ...
}:
let
  rustToolchain = crane.withBuildTarget CARGO_BUILD_TARGET;
  info = (crane.lib.crateNameFromCargoToml { cargoToml = cargoToml; });
  artifact = builtins.replaceStrings [ "-" ] [ "_" ] info.pname;
  fts = if features != [ ] then lib.concatStringsSep " " ([ "--features" ] ++ features) else "";
  cargoExtraArgs = "-p ${info.pname} ${fts}";
  attrs = info
    // {
    cargoExtraArgs = cargoExtraArgs;
    src = crane.rustSrc;
    cargoVendorDir = crane.lib.vendorMultipleCargoDeps {
      inherit (crane.lib.findCargoFiles crane.rustSrc) cargoConfigs;
      cargoLockList = [
        cargoLock
        "${rustToolchain.toolchain.passthru.availableComponents.rust-src}/lib/rustlib/src/rust/Cargo.lock"
      ];
    };
  };
  CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
  clippy = crane.lib.cargoClippy (attrs // { inherit (crane) cargoArtifacts; cargoClippyExtraArgs = "--tests"; });
  test = crane.lib.cargoTest (attrs // { inherit (crane) cargoArtifacts; });
in

rustToolchain.buildPackage (attrs // {
  inherit CARGO_BUILD_TARGET;

  cargoBuildCommand = "RUSTFLAGS='-C target-feature=-sign-ext -C link-arg=-s -C target-cpu=mvp' cargo -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort build --release --lib --target ${CARGO_BUILD_TARGET}";

  doCheck = doCheck;
  checkPhase = ''
    ls ${clippy} > /dev/null 2>&1
    ls ${test} > /dev/null 2>&1

    # grep exits 0 if a match is found
    if ${pkgs.binaryen}/bin/wasm-dis target/wasm32-unknown-unknown/release/${info.pname}.wasm | grep -P '\.extend\d{1,2}_s'
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
    mv target/wasm32-unknown-unknown/release/${artifact}.wasm $out/lib/${artifact}.wasm
    # ${pkgs.binaryen}/bin/wasm-opt -Os target/wasm32-unknown-unknown/release/${artifact}.wasm -o $out/lib/${artifact}.wasm
    # We also zip the binary since it is smaller
    gzip -fk $out/lib/${artifact}.wasm
  '';
})

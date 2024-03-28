{ inputs, ... }: {
  perSystem = { pkgs, self', crane, rust, system, ensureAtRepositoryRoot, srcWithVendoredSources, dbg, ... }:
    let
      CARGO_BUILD_TARGET = rust.staticBuildTarget system;

      rustToolchain = rust.mkNightly {
        channel = "nightly-2024-01-27";
        target = CARGO_BUILD_TARGET;
      };

      mkLibwasmvm =
        wasmvm:
        let
          attrs = {
            inherit CARGO_BUILD_TARGET;
            pname = "libwasmvm";
            version = wasmvm.rev;
            buildInputs = [ rustToolchain ];
            src = "${wasmvm}/libwasmvm";
            installCargoArtifactsMode = "use-zstd";
          } // (if pkgs.stdenv.isLinux then {
            cargoExtraArgs = "--locked --offline --example=wasmvmstatic";
            installPhaseCommand = ''
              mkdir -p $out/lib
              mv target/${CARGO_BUILD_TARGET}/release/examples/libwasmvmstatic.a $out/lib/libwasmvm.${builtins.head (pkgs.lib.strings.splitString "-" system)}.a
            '';
          } else if pkgs.stdenv.isDarwin then {
            # non-static dylib build on macOS
            cargoExtraArgs = "--locked --offline";
            installPhaseCommand = ''
              mkdir -p $out/lib
              mv target/${CARGO_BUILD_TARGET}/release/deps/libwasmvm.dylib $out/lib/libwasmvm.dylib
            '';
          } else throw "unknown system");
          craneLib = crane.lib.overrideToolchain rustToolchain;
        in
        craneLib.buildPackage (attrs // {
          cargoArtifacts = craneLib.buildDepsOnly attrs;
        });
    in
    {
      packages.libwasmvm = mkLibwasmvm inputs.wasmvm;
      packages.libwasmvm-1_5_0 = mkLibwasmvm inputs.wasmvm-1_5_0;
    };
}

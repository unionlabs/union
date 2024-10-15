{ inputs, ... }:
{
  perSystem =
    {
      pkgs,
      self',
      crane,
      rust,
      system,
      ensureAtRepositoryRoot,
      srcWithVendoredSources,
      dbg,
      ...
    }:
    let
      throwBadSystem = throw "libwasmvm cannot be built on system `${system}`";

      CARGO_BUILD_TARGET =
        if system == "aarch64-linux" then
          "aarch64-unknown-linux-musl"
        else if system == "x86_64-linux" then
          "x86_64-unknown-linux-musl"
        else if system == "aarch64-darwin" then
          "aarch64-apple-darwin"
        else if system == "x86_64-darwin" then
          "x86_64-apple-darwin"
        else
          throwBadSystem;

      rustToolchain-2024-01-27 = rust.mkNightly {
        channel = "nightly-2024-01-27";
        targets = [ CARGO_BUILD_TARGET ];
      };
      rustToolchain-2024-09-17 = rust.mkNightly {
        channel = "nightly-2024-09-17";
        targets = [ CARGO_BUILD_TARGET ];
      };

      mkLibwasmvm_v1 =
        wasmvm:
        let
          attrs =
            {
              inherit CARGO_BUILD_TARGET;
              pname = "libwasmvm";
              version = wasmvm.rev;
              buildInputs = [ rustToolchain-2024-01-27 ];
              src = "${wasmvm}/libwasmvm";
              installCargoArtifactsMode = "use-zstd";
            }
            // (
              if pkgs.stdenv.isLinux then
                {
                  cargoExtraArgs = "--locked --offline --example=wasmvmstatic";
                  installPhaseCommand = ''
                    mkdir -p $out/lib
                    mv target/${CARGO_BUILD_TARGET}/release/examples/libwasmvmstatic.a $out/lib/libwasmvm.${builtins.head (pkgs.lib.strings.splitString "-" system)}.a
                  '';
                }
              else if pkgs.stdenv.isDarwin then
                {
                  # non-static dylib build on macOS
                  cargoExtraArgs = "--locked --offline";
                  installPhaseCommand = ''
                    mkdir -p $out/lib
                    mv target/${CARGO_BUILD_TARGET}/release/deps/libwasmvm.dylib $out/lib/libwasmvm.dylib
                  '';
                }
              else
                throwBadSystem
            );
          craneLib = crane.lib.overrideToolchain rustToolchain-2024-01-27;
        in
        craneLib.buildPackage (
          attrs
          // {
            cargoArtifacts = craneLib.buildDepsOnly attrs;
          }
        );
      mkLibwasmvm_v2 =
        wasmvm:
        let
          attrs =
            {
              inherit CARGO_BUILD_TARGET;
              pname = "libwasmvm";
              version = wasmvm.rev;
              buildInputs = [ rustToolchain-2024-09-17 ];
              src = "${wasmvm}/libwasmvm";
              installCargoArtifactsMode = "use-zstd";
            }
            // (
              if pkgs.stdenv.isLinux then
                {
                  cargoExtraArgs = "--locked --offline --example=wasmvmstatic";
                  installPhaseCommand = ''
                    mkdir -p $out/lib
                    mv target/${CARGO_BUILD_TARGET}/release/examples/libwasmvmstatic.a $out/lib/libwasmvm.${builtins.head (pkgs.lib.strings.splitString "-" system)}.a
                  '';
                }
              else if pkgs.stdenv.isDarwin then
                {
                  # non-static dylib build on macOS
                  cargoExtraArgs = "--locked --offline";
                  installPhaseCommand = ''
                    mkdir -p $out/lib
                    mv target/${CARGO_BUILD_TARGET}/release/deps/libwasmvm.dylib $out/lib/libwasmvm.dylib
                  '';
                }
              else
                throwBadSystem
            );
          craneLib = crane.lib.overrideToolchain rustToolchain-2024-09-17;
        in
        craneLib.buildPackage (
          attrs
          // {
            cargoArtifacts = craneLib.buildDepsOnly attrs;
          }
        );
    in
    {
      packages.libwasmvm = mkLibwasmvm_v1 inputs.wasmvm;
      packages.libwasmvm-1_5_0 = mkLibwasmvm_v1 inputs.wasmvm-1_5_0;
      packages.libwasmvm-2_0_1 = mkLibwasmvm_v2 inputs.wasmvm-2_0_1;
      packages.libwasmvm-2_3_1 = mkLibwasmvm_v2 inputs.wasmvm-2_1_3;
    };
}

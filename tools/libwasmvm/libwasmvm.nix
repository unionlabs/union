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

      rustToolchain = rust.mkNightly {
        channel = "nightly-2024-01-27";
        targets = [ CARGO_BUILD_TARGET ];
      };

      mkLibwasmvm =
        wasmvm:
        let
          attrs =
            {
              inherit CARGO_BUILD_TARGET;
              pname = "libwasmvm";
              version = wasmvm.rev;
              buildInputs = [ rustToolchain ];
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
          craneLib = crane.lib.overrideToolchain rustToolchain;
        in
        craneLib.buildPackage (
          attrs
          // {
            cargoArtifacts = craneLib.buildDepsOnly attrs;
          }
        );
    in
    {
      packages.libwasmvm = mkLibwasmvm inputs.wasmvm;
      packages.libwasmvm-1_5_0 = mkLibwasmvm inputs.wasmvm-1_5_0;
      packages.libwasmvm-2_0_1 = mkLibwasmvm inputs.wasmvm-2_0_1;
    };
}

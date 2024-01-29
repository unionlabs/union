{ inputs, ... }: {
  perSystem = { pkgs, self', crane, rust, system, ensureAtRepositoryRoot, srcWithVendoredSources, dbg, ... }:
    let
      throwBadSystem = throw "libwasmvm cannot be built on system `${system}`";

      CARGO_BUILD_TARGET =
        if system == "aarch64-linux" then "aarch64-unknown-linux-musl"
        else if system == "x86_64-linux" then "x86_64-unknown-linux-musl"
        else if system == "aarch64-darwin" then "aarch64-apple-darwin"
        else if system == "x86_64-darwin" then "x86_64-apple-darwin"
        else throwBadSystem;

      rustToolchain = rust.mkNightly { target = CARGO_BUILD_TARGET; };

      mkLibwasmvm =
        wasmvm:
        let
          attrs =
            {
              inherit CARGO_BUILD_TARGET;

              pname = "libwasmvm";
              version = (dbg wasmvm).rev;

              # cargoArtifacts = null;

              buildInputs = [ rustToolchain ];

              src = "${wasmvm}/libwasmvm";

              # dummySrc = crane.lib.mkDummySrc "${wasmvm}/libwasmvm";

              # cargoVendorDir = crane.lib.vendorMultipleCargoDeps {
              #   inherit (crane.lib.findCargoFiles wasmvm) cargoConfigs;
              #   cargoLockList = [
              #     workspaceCargoLockPath
              #   ] ++ (lib.optionals (buildStdTarget != null) ([
              #     ./rust-std-Cargo.lock
              #   ]));
              # };


              # cargoLock = "${wasmvm}/libwasmvm/Cargo.lock";
              # # cargoVendorDir = vendorDir;
              # doCheck = false;
              # doInstallCargoArtifacts = false;
              # buildPhaseCargoCommand = "";
            } // (if pkgs.stdenv.isLinux then {
              cargoExtraArgs = "--locked --offline --example=wasmvmstatic";
              installPhase = ''
                mkdir -p $out/lib
                mv target/${CARGO_BUILD_TARGET}/release/examples/libwasmvmstatic.a $out/lib/libwasmvm.${builtins.head (pkgs.lib.strings.splitString "-" system)}.a
              '';
            } else if pkgs.stdenv.isDarwin then {
              # non-static dylib build on macOS
              cargoExtraArgs = "--locked --offline";
              installPhase = ''
                mkdir -p $out/lib
                mv target/${CARGO_BUILD_TARGET}/release/deps/libwasmvm.dylib $out/lib/libwasmvm.dylib
              '';
            } else throwBadSystem);

          craneLib = crane.lib.overrideToolchain (rust.mkNightly { target = CARGO_BUILD_TARGET; });
        in
        craneLib.buildPackage (attrs // {
          cargoArtifacts = craneLib.buildDepsOnly attrs;
        });
    in
    {
      packages.libwasmvm = mkLibwasmvm inputs.wasmvm;
    };
}

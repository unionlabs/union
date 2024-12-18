{ ... }:
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
      rustToolchain-1-82 = rust.mkNightly {
        channel = "1.82.0";
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
              buildInputs = [ rustToolchain-1-82 ];
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
          craneLib = crane.lib.overrideToolchain rustToolchain-1-82;
        in
        craneLib.buildPackage (
          attrs
          // {
            cargoArtifacts = craneLib.buildDepsOnly attrs;
          }
        );
      wasmvm-1_5_2 = pkgs.fetchFromGitHub {
        owner = "CosmWasm";
        repo = "wasmvm";
        rev = "v1.5.2";
        hash = "sha256-3KJq5jFllFSqlm85/iRWYMhu99iuokvR3Ib9Gq3gIjc=";
      };
      wasmvm-2_1_2 = pkgs.fetchFromGitHub {
        owner = "CosmWasm";
        repo = "wasmvm";
        rev = "v2.1.2";
        hash = "sha256-Y3BVRR2T5MLOtXdPK38W8MX8etIuqGcTjvxkaEOyvVM=";
      };
      wasmvm-2_1_3 = pkgs.fetchFromGitHub {
        owner = "CosmWasm";
        repo = "wasmvm";
        rev = "v2.1.3";
        hash = "sha256-gYrK2EHhXnearJgLX38O6NLI6TfoGtpzA9be/7S/0ZU=";
      };
    in
    {
      packages.libwasmvm-1_5_2 = mkLibwasmvm_v1 wasmvm-1_5_2;
      packages.libwasmvm-2_1_2 = mkLibwasmvm_v2 wasmvm-2_1_2;
      packages.libwasmvm-2_1_3 = mkLibwasmvm_v2 wasmvm-2_1_3;
    };
}

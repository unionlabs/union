{ ... }: {
  perSystem = { pkgs, self', crane, rust, system, ensureAtRepositoryRoot, srcWithVendoredSources, ... }:
    let
      throwBadSystem = throw "libwasmvm cannot be built on system `${system}`";

      CARGO_BUILD_TARGET =
        if system == "aarch64-linux" then "aarch64-unknown-linux-musl"
        else if system == "x86_64-linux" then "x86_64-unknown-linux-musl"
        else if system == "aarch64-darwin" then "aarch64-apple-darwin"
        else if system == "x86_64-darwin" then "x86_64-apple-darwin"
        else throwBadSystem;

      rustToolchain = rust.mkNightly { target = CARGO_BUILD_TARGET; };

      wasmvm_1_3_0 = pkgs.fetchFromGitHub {
        owner = "CosmWasm";
        repo = "wasmvm";
        rev = "v1.3.0"; # wasmd 0.41.0
        hash = "sha256-rsTYvbkYpDkUE4IvILdSL3hXMgAWxz5ltGotJB2t1e4=";
      };
      wasmvm_1_5_0 = pkgs.fetchFromGitHub {
        owner = "CosmWasm";
        repo = "wasmvm";
        rev = "v1.5.0";
        hash = "sha256-l0cNF0YjviEl/JLJ4VdvDtIGuAYyFfncVo83ROfQFD8=";
      };

      mkLibwasmvm =
        wasmvm:
        let
          attrs =
            {
              inherit CARGO_BUILD_TARGET;

              pname = "libwasmvm";
              version = wasmvm.rev;

              # cargoArtifacts = null;

              buildInputs = [ rustToolchain ];

              src = "${wasmvm}/libwasmvm";
              # cargoLock = "${wasmvm}/libwasmvm/Cargo.lock";
              # # cargoVendorDir = vendorDir;
              # doCheck = false;
              # doInstallCargoArtifacts = false;
              # buildPhaseCargoCommand = "";
            } // (if pkgs.stdenv.isLinux then {
              cargoExtraArgs = "--release --locked --offline --example=wasmvmstatic";
              # installPhase = ''
              #   mkdir -p $out/lib
              #   mv target/${CARGO_BUILD_TARGET}/release/examples/libwasmvmstatic.a $out/lib/libwasmvm.${builtins.head (pkgs.lib.strings.splitString "-" system)}.a
              # '';
            } else if pkgs.stdenv.isDarwin then {
              # non-static dylib build on macOS
              cargoExtraArgs = "cargo build --release --locked --offline";
              # installPhase = ''
              #   mkdir -p $out/lib
              #   mv target/${CARGO_BUILD_TARGET}/release/deps/libwasmvm.dylib $out/lib/libwasmvm.dylib
              # '';
            } else throwBadSystem);
        in
        crane.lib.buildPackage (attrs // {
          cargoArtifacts = crane.lib.buildDepsOnly attrs;
        });
    in
    {
      _module.args.libwasmvmCargoToml_1_3_0 = "${wasmvm_1_3_0}/libwasmvm/Cargo.toml";
      _module.args.libwasmvmCargoToml_1_5_0 = "${wasmvm_1_5_0}/libwasmvm/Cargo.toml";

      packages.libwasmvm = mkLibwasmvm wasmvm_1_3_0;
      packages.libwasmvm_1_5_0 = mkLibwasmvm wasmvm_1_5_0;
    };
}

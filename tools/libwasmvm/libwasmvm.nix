{ ... }: {
  perSystem = { pkgs, self', crane, rust, system, ensureAtRepositoryRoot, ... }:
    let
      vendorDir = "tools/libwasmvm/vendor/";
      vendorDirPath = ./vendor;

      throwBadSystem = throw "libwasmvm cannot be built on system `${system}`";

      CARGO_BUILD_TARGET =
        if system == "aarch64-linux" then "aarch64-unknown-linux-musl"
        else if system == "x86_64-linux" then "x86_64-unknown-linux-musl"
        else if system == "aarch64-darwin" then "aarch64-apple-darwin"
        else if system == "x86_64-darwin" then "x86_64-apple-darwin"
        else throwBadSystem;

      rustToolchain = rust.mkNightly { target = CARGO_BUILD_TARGET; };

      wasmvm = pkgs.fetchFromGitHub {
        owner = "CosmWasm";
        repo = "wasmvm";
        rev = "v1.3.0"; # wasmd 0.41.0
        hash = "sha256-rsTYvbkYpDkUE4IvILdSL3hXMgAWxz5ltGotJB2t1e4=";
      };

    srcWithVendoredSources =
      let
        configToml = ''
          [source.crates-io]
          replace-with = "vendored-sources"

          [source."git+https://github.com/CosmWasm/cosmwasm.git?rev=v1.3.0"]
          git = "https://github.com/CosmWasm/cosmwasm.git"
          rev = "v1.3.0"
          replace-with = "vendored-sources"

          [source.vendored-sources]
          directory = "tools/libwasmvm/vendor/"
        '';
      in
      pkgs.stdenv.mkDerivation {
        name = "libwasmvm-with-vendored-sources-cargo-config-toml";
        src = "${wasmvm}/libwasmvm";
        buildPhase = ''
          cp -r . $out

          mkdir -p $out/${vendorDir}
          
          cp -r --no-preserve=mode ${vendorDirPath}/. $out/${vendorDir}/

          diff -r $out/${vendorDir} ${vendorDirPath}

          mkdir -p $out/.cargo
          echo '${configToml}' >> $out/.cargo/config.toml
        '';
      };
    in
    {
      packages.libwasmvm =
        pkgs.stdenv.mkDerivation (
          {
            inherit CARGO_BUILD_TARGET;

            name = "libwasmvm";
            version = wasmvm.rev;

            # cargoArtifacts = null;

            buildInputs = [ rustToolchain ];

            src = srcWithVendoredSources;
            # cargoLock = "${wasmvm}/libwasmvm/Cargo.lock";
            # # cargoVendorDir = vendorDir;
            # doCheck = false;
            # doInstallCargoArtifacts = false;
            # buildPhaseCargoCommand = "";
          } // (if pkgs.stdenv.isLinux then {
            buildPhase = "ls -al ${vendorDir}/typenum; cargo build --release --locked --offline --example=wasmvmstatic";
            installPhase = ''
              mkdir -p $out/lib
              mv target/${CARGO_BUILD_TARGET}/release/examples/libwasmvmstatic.a $out/lib/libwasmvm.${builtins.head (pkgs.lib.strings.splitString "-" system)}.a
            '';
          } else if pkgs.stdenv.isDarwin then {
            # non-static dylib build on macOS
            buildPhase = "cargo build --release --locked --offline";
            installPhase = ''
              mkdir -p $out/lib
              mv target/${CARGO_BUILD_TARGET}/release/deps/libwasmvm.dylib $out/lib/libwasmvm.dylib
            '';
          } else throwBadSystem)
        );

      packages.vendorLibwasmvm = pkgs.writeShellApplication {
        name = "vendor-libwasmvm";
        text =
          ''
            ${ensureAtRepositoryRoot}

            cargo --version

            cargo vendor --manifest-path ${wasmvm}/libwasmvm/Cargo.toml ${vendorDir}
          '';
        };

      packages.vendoredSource = srcWithVendoredSources;
    };
}

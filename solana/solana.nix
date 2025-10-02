_: {
  perSystem =
    {
      pkgs,
      system,
      pkgsUnstableSolana,
      rust,
      dbg,
      crane,
      ...
    }:
    let
      platformToolsVersion = "1.51";

      platform-tools = dbg (
        pkgs.stdenv.mkDerivation {
          pname = "platform-tools";
          version = platformToolsVersion;
          src = platform-tools-stripped;
          nativeBuildInputs = [ pkgs.autoPatchelfHook ];
          buildInputs = with pkgs; [
            zlib
            pkgs.stdenv.cc.cc
            openssl
            python310
            ncurses
            libxml2
            libedit
            xz
          ];

          postFixup = ''
            patchelf --replace-needed libedit.so.2 libedit.so $out/llvm/lib/liblldb.so.19.1.7-rust-dev
          '';

          installPhase = ''
            mkdir -p $out;
            cp -r $src/llvm $out;
            cp -r $src/rust $out;
            chmod 0755 -R $out;
          '';
        }
      );

      platform-tools-versions = {
        "x86_64-linux" = pkgs.fetchurl {
          url = "https://github.com/anza-xyz/platform-tools/releases/download/v1.51/platform-tools-linux-x86_64.tar.bz2";
          sha256 = "sha256-CTPgXdlkgm6OLbXFjDSuJV47rwzhcRVoVS3KgbVAems=";
        };
        "aarch64-linux" = pkgs.fetchurl {
          url = "https://github.com/anza-xyz/platform-tools/releases/download/v1.51/platform-tools-linux-aarch64.tar.bz2";
          sha256 = "sha256-4oHGs4Mg5Y726Cf2ymucSRSCX47eKc/C89qjhYW3YLs=";
        };
      };

      platform-tools-stripped = pkgs.runCommand "platform-tools-stripped" { } ''
        mkdir -p $out
        tar --strip-components=0 -xjf ${platform-tools-versions.${system}} -C $out;
      '';

      cargo-solana = pkgs.stdenv.mkDerivation {
        pname = "cargo-solana";
        version = "0.0.0";

        src = pkgsUnstableSolana.solana-cli;

        nativeBuildInputs = [
          pkgs.pkg-config
          rust.toolchains.dev
          platform-tools
        ];

        installPhase = ''
          mkdir -p $out
          cp -r $src/* $out
          chmod -R +w $out
          mkdir -p $out/bin/platform-tools-sdk/sbf/dependencies/platform-tools
          cp -r ${platform-tools}/llvm $out/bin/platform-tools-sdk/sbf/dependencies/platform-tools/llvm
          cp -r ${platform-tools}/rust $out/bin/platform-tools-sdk/sbf/dependencies/platform-tools/rust
        '';
      };

      solana-ibc =
        (crane.buildWorkspaceMember "solana/ibc" {
          cargoBuildRustToolchain = dbg "${platform-tools}/rust";
          extraBuildInputs = [ cargo-solana ];
          # NOTE: Only used for build-std
          extraVendorPaths = [
            "${platform-tools}/rust/lib/rustlib/src/rust/library/Cargo.lock"
          ];
          # NOTE: Only used for build-std
          overrideVendorGitCheckout =
            ps: drv:
            # libm isn't vendored correctly when vendoring the solana fork of compiler-builtins, override it's installPhase to make sure that libm is included in the vendored sources
            if
              pkgs.lib.any (
                p:
                (pkgs.lib.hasInfix "https://github.com/anza-xyz/compiler-builtins?tag=solana-tools-v1.51" p.source)
              ) ps
            then
              drv.overrideAttrs (
                old:
                old
                // {
                  installPhase = ''
                    cargoToml=$(realpath Cargo.toml)
                    crate=$(
                      cargo metadata --format-version 1 --no-deps --manifest-path "$cargoToml" | jq -r '.packages[] | select(.manifest_path == "'"$cargoToml"'") | "\(.name)-\(.version)"'
                    )
                    mkdir -p "$out/$crate"
                    cp -r . "$out/$crate"
                    echo '{"files":{}, "package":null}' > "$out/$crate/.cargo-checksum.json"
                  '';
                }
              )
            else
              drv;
          rustflags = "-Zlocation-detail=none";
          extraArgs = {
            doNotPostBuildInstallCargoBinaries = true;
            buildPhaseCargoCommand = ''
              cargo build-sbf \
                --arch v2 \
                --skip-tools-install \
                --tools-version ${platformToolsVersion} \
                --no-rustup-override \
                --offline \
                -- \
                -p ibc-union-solana

                # TODO: Have a .release builder that doe -j1 for reproducible builds
                # -j1 \
                # NOTE: Only used for build-std
                # -Z build-std=panic_abort,std,core,alloc
            '';
          };
          cargoBuildInstallPhase = ''
            mkdir -p $out
            ls -alh target/
            ls -alh target/deploy
            cp --no-preserve=mode target/deploy/* $out
          '';
        }).ibc-union-solana;

      solana-ibc-for-tests = (crane.buildWorkspaceMember "solana/ibc" { }).ibc-union-solana;

    in
    {
      packages = {
        inherit cargo-solana solana-ibc;
        # fetchSolanaRustStdCargoLock = pkgs.writeShellScript "fetch-solana-rust-std-cargo-lock" ''

        # '';
      };

      checks = {
        solana-ibc = crane.lib.cargoTest (
          solana-ibc-for-tests.passthru.craneAttrs
          // {
            doCheck = true;
          }
        );
      };

    };
}

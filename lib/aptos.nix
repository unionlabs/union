{ ... }: {
  perSystem = { self', lib, unstablePkgs, pkgs, system, config, rust, crane, stdenv, dbg, ... }:
    let

      # aptos = pkgs.stdenv.mkDerivation {
      #   name = "aptos";
      #   buildInputs = [ pkgs.makeWrapper ];
      #   src =
      #     (crane.buildWorkspaceMember {
      #       crateDirFromRoot = "near/near-ibc-tests";
      #       extraEnv = {
      #         PROTOC = "${pkgs.protobuf}/bin/protoc";
      #         LIBCLANG_PATH = "${pkgs.llvmPackages_14.libclang.lib}/lib";
      #       };
      #       extraBuildInputs = [ pkgs.pkg-config pkgs.openssl pkgs.perl pkgs.gnumake ];
      #       extraNativeBuildInputs = [ pkgs.clang ];
      #       extraEnv = { };
      #     }).packages.near-ibc-tests;
      #   installPhase = ''
      #     mkdir -p $out/bin
      #     cp -r $src/bin/near-ibc-tests $out/bin/near-ibc-tests
      #     wrapProgram $out/bin/near-ibc-tests \
      #       --set NEAR_SANDBOX_BIN_PATH "${near-sandbox}/bin/neard" \
      #       --set IBC_WASM_FILEPATH "${self'.packages.near-ibc}/lib/near_ibc.wasm" \
      #       --set NEAR_LC_WASM_FILEPATH "${self'.packages.near-light-client}/lib/near_light_client.wasm" \
      #       --set IBC_APP_WASM_FILEPATH "${self'.packages.dummy-ibc-app}/lib/dummy_ibc_app.wasm";
      #   '';
      #   meta.mainProgram = "near-ibc-tests";
      # };

      throwBadSystem = throw "aptos cannot be built on system `${system}`";

      CARGO_BUILD_TARGET =
        if system == "aarch64-linux" then "aarch64-unknown-linux-musl"
        else if system == "x86_64-linux" then "x86_64-unknown-linux-musl"
        else if system == "aarch64-darwin" then "aarch64-apple-darwin"
        else if system == "x86_64-darwin" then "x86_64-apple-darwin"
        else throwBadSystem;

      rustToolchain = rust.mkNightly {
        channel = "1.78.0";
        targets = [ CARGO_BUILD_TARGET ];
      };

      craneLib = crane.lib.overrideToolchain rustToolchain;

      aptos = pkgs.stdenv.mkDerivation {
        name = "movement";
        buildInputs = [ pkgs.makeWrapper ];
        src = aptosSrc;
        installPhase = ''
          mkdir -p $out/bin
          cp -r $src/bin/movement $out/bin/movement
          wrapProgram $out/bin/movement \
            --set LD_LIBRARY_PATH "${lib.makeLibraryPath [ pkgs.gcc13Stdenv.cc.cc]}" \
        '';
      };

      aptosSrc = craneLib.buildPackage rec {
        pname = "movement";
        version = "17c10f224fd6d76101881799de3cf49750acfe03";

        buildInputs = [ pkgs.pkg-config pkgs.openssl pkgs.systemd config.treefmt.build.programs.rustfmt pkgs.elfutils pkgs.lld pkgs.mold ] ++ (
          lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ]
        );

        nativeBuildInputs = [
          pkgs.clang
        ];

        cargoExtraArgs = "-p movement";
        
        LIBCLANG_PATH = "${pkgs.llvmPackages_14.libclang.lib}/lib";

        CARGO_PROFILE = "cli";

        CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";

        src = builtins.fetchGit {
          url = "https://github.com/aeryz/aptos-core";
          ref = "main";
          rev = version;
        };

        doCheck = false;
      };

    in
    {
      packages = {
        inherit aptos aptosSrc;
      };

    };
}

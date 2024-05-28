{ ... }: {
  perSystem = { self', lib, unstablePkgs, pkgs, system, config, rust, crane, stdenv, dbg, ... }:
    let

      near-ibc-tests = pkgs.stdenv.mkDerivation {
        name = "near-ibc-tests";
        buildInputs = [ pkgs.makeWrapper ];
        src =
          (crane.buildWorkspaceMember {
            crateDirFromRoot = "near/near-ibc-tests";
            extraEnv = {
              PROTOC = "${pkgs.protobuf}/bin/protoc";
              LIBCLANG_PATH = "${pkgs.llvmPackages_14.libclang.lib}/lib";
            };
            extraBuildInputs = [ pkgs.pkg-config pkgs.openssl pkgs.perl pkgs.gnumake ];
            extraNativeBuildInputs = [ pkgs.clang ];
          }).packages.near-ibc-tests;
        installPhase = ''
          mkdir -p $out/bin
          cp -r $src/bin/near-ibc-tests $out/bin/near-ibc-tests
          wrapProgram $out/bin/near-ibc-tests \
            --set NEAR_SANDBOX_BIN_PATH "${near-sandbox}/bin/neard";
        '';
        meta.mainProgram = "near-ibc-tests";
      };

      cargo-near = craneLib.buildPackage rec {
        pname = "cargo-near";
        version = "v0.6.2";

        buildInputs = [ pkgs.pkg-config pkgs.openssl pkgs.perl pkgs.gnumake pkgs.systemd ] ++ (
          lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ]
        );

        nativeBuildInputs = [
          # pkgs.llvmPackages_14.libclang
          # pkgs.llvmPackages_14.libcxxClang
          pkgs.clang
          # pkgs.stdenv.cc.libc
        ];

        LIBCLANG_PATH = "${pkgs.llvmPackages_14.libclang.lib}/lib";
        PROTOC = "${pkgs.protobuf}/bin/protoc";
        NEAR_SANDBOX_BIN_PATH = "${near-sandbox}/bin/neard";

        # The integration tests are incredibly cursed
        # https://github.com/near/cargo-near/blob/main/cargo-near/src/types/metadata.rs#L48
        doCheck = false;

        # cargoExtraArgs = " --verbose --verbose -p neard --features sandbox";

        src = pkgs.fetchFromGitHub {
          owner = "near";
          repo = pname;
          rev = "cargo-near-${version}";
          hash = "sha256-kFMrsryyP/XpSzR88o/edaGEchbyDB1JpzwC6QoEMfA=";
        };
      };


      rustToolchain = rust.mkNightly {
        channel = "1.78.0";
        targets = [ "wasm32-unknown-unknown" ];
      };

      craneLib = crane.lib.overrideToolchain rustToolchain;

      near-sandbox = craneLib.buildPackage rec {
        pname = "neard";
        version = "326c6098c652c0fe3419067ad0ff839804658b7d";

        buildInputs = [ pkgs.pkg-config pkgs.openssl pkgs.perl pkgs.gnumake ] ++ (
          lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ]
        );

        nativeBuildInputs = [
          # pkgs.llvmPackages_14.libclang
          # pkgs.llvmPackages_14.libcxxClang
          pkgs.clang
          # pkgs.stdenv.cc.libc
        ];

        LIBCLANG_PATH = "${pkgs.llvmPackages_14.libclang.lib}/lib";

        cargoExtraArgs = " --verbose --verbose -p neard --features sandbox";

        src = pkgs.fetchFromGitHub {
          owner = "near";
          repo = "nearcore";
          rev = version;
          hash = "sha256-zGKyBwQrCfDYGlqd7sEf/JbTrKkMG5MEwbGvsJvOZVQ=";
        };
      };
    in
    {
      packages = {
        inherit near-ibc-tests near-sandbox cargo-near;
      };
    };
}

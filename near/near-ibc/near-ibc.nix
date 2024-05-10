{ ... }: {
  perSystem = { self', lib, unstablePkgs, pkgs, system, config, rust, crane, stdenv, dbg, ... }:
    let
      near-ibc = crane.buildWasmContract {
        crateDirFromRoot = "near/near-ibc";
      };

      near-integration-tests = crane.buildWorkspaceMember {
        crateDirFromRoot = "near/near-ibc";
        cargoBuildExtraArgs = "--example integration-tests";
        extraEnv = {
          NEAR_SANDBOX_BIN_PATH = "${near-sandbox}/bin/neard";
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
        near-sandbox = near-sandbox;
      } // near-integration-tests.packages;

    };
}

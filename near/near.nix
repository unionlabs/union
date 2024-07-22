{ ... }: {
  perSystem = { self', lib, unstablePkgs, pkgs, system, config, rust, crane, stdenv, dbg, python, ... }:
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
            extraEnv = { };
          }).packages.near-ibc-tests;
        installPhase = ''
          mkdir -p $out/bin
          cp -r $src/bin/near-ibc-tests $out/bin/near-ibc-tests
          wrapProgram $out/bin/near-ibc-tests \
            --set NEAR_SANDBOX_BIN_PATH "${near-sandbox}/bin/neard" \
            --set IBC_WASM_FILEPATH "${self'.packages.near-ibc}/lib/near_ibc.wasm" \
            --set NEAR_LC_WASM_FILEPATH "${self'.packages.near-light-client}/lib/near_light_client.wasm" \
            --set IBC_APP_WASM_FILEPATH "${self'.packages.dummy-ibc-app}/lib/dummy_ibc_app.wasm";
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
          pkgs.clang
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

      nearcore = craneLib.buildPackage rec {
        pname = "neard";
        version = "177c8657acd79a9a33f4e9f2ecadfabad792eae1";

        buildInputs = [ pkgs.pkg-config pkgs.openssl pkgs.perl pkgs.gnumake ] ++ (
          lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ]
        );

        nativeBuildInputs = [
          pkgs.clang
        ];

        LIBCLANG_PATH = "${pkgs.llvmPackages_14.libclang.lib}/lib";

        cargoExtraArgs = " --verbose --verbose -p neard";

        src = pkgs.fetchFromGitHub {
          owner = "aeryz";
          repo = "nearcore";
          rev = version;
          hash = "sha256-2Iii+prFl5W4OS9VLwbce+QssKe8dLH/P+bVG8AWJ2c=";
        };
      };

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

      near-light-client = (crane.buildWasmContract {
        crateDirFromRoot = "light-clients/near/near";
        extraBuildInputs = [ pkgs.pkg-config pkgs.openssl pkgs.perl pkgs.gnumake ];
        extraNativeBuildInputs = [ pkgs.clang ];
      });

      dummy-ibc-app = (crane.buildWasmContract {
        crateDirFromRoot = "near/dummy-ibc-app";
        extraBuildInputs = [ pkgs.pkg-config pkgs.openssl pkgs.perl pkgs.gnumake ];
        extraNativeBuildInputs = [ pkgs.clang ];
      });

      near-ibc = (crane.buildWasmContract {
        crateDirFromRoot = "near/near-ibc";
        extraBuildInputs = [ pkgs.pkg-config pkgs.openssl pkgs.perl pkgs.gnumake ];
        extraNativeBuildInputs = [ pkgs.clang ];
      });

      near-localnet = pkgs.writeShellApplication {
        name = "near-localnet";
        # runtimeInputs = [ nearup ];
        runtimeInputs = [(python.withPackages (py-pkgs: [
          py-pkgs.nearup
        ]))] ++ [ pkgs.strace pkgs.iproute pkgs.busybox unstablePkgs.nodePackages_latest.near-cli ];
        text = ''
          mkdir /tmp
          export TMPDIR=/tmp
          export TEMP=/tmp

          nearup run --override --binary-path ${nearcore}/bin localnet
          sleep 3
          echo Deploying ibc..
          ls -la ~/.near
          mkdir neardev
          echo N | near dev-deploy \
            --networkId asd \
            --wasmFile ${self'.packages.near-ibc}/lib/near_ibc.wasm \
            --masterAccount node0 \
            --keyPath ~/.near/localnet/node0/validator_key.json \
            --nodeUrl http://localhost:3030 \
            --accountId ibc-union
          tail -f /.nearup/logs/localnet/node0.log
        '';
      };
    in
    {
      packages = near-light-client.packages // dummy-ibc-app.packages // near-ibc.packages // {
        inherit near-ibc-tests near-sandbox cargo-near nearcore near-localnet;
      };

      checks = near-light-client.checks // near-ibc.checks;
    };
}

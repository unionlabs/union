_: {
  perSystem =
    {
      self',
      pkgs,
      pkgsUnstable,
      crane,
      rust,
      dbg,
      ...
    }:
    let
      craneLib = (crane.lib.overrideToolchain (_: rust.mkNightly { channel = "nightly-2025-05-09"; }));

      cairoVersion = "v2.12.1";

      pyPkgs = pkgsUnstable.python312Packages;

      scarb = craneLib.buildPackage rec {
        pname = "scarb";
        version = cairoVersion;
        src = pkgs.fetchFromGitHub {
          name = pname;
          owner = "software-mansion";
          repo = pname;
          rev = version;
          sha256 = "sha256-PlUZsr99TVH/9k2Ecq2+rAcUVTTneEh2v85zCMikkXU=";
        };
        cargoExtraArgs = "-p scarb";
        doCheck = false;
        meta.mainProgram = "scarb";
        SCARB_CORELIB_LOCAL_PATH = "${
          pkgs.fetchFromGitHub rec {
            pname = repo;
            owner = "starkware-libs";
            repo = "cairo";
            rev = cairoVersion;
            sha256 = "sha256-NQYtlyttIvTxPa6dLbFOkWO5RysaJz2T2S3Z9fg1bg4=";
          }
        }/corelib";
      };

      universal-sierra-compiler = craneLib.buildPackage rec {
        pname = "universal-sierra-compiler";
        version = "v2.6.0";
        src = pkgs.fetchFromGitHub {
          name = pname;
          owner = "software-mansion";
          repo = pname;
          rev = version;
          sha256 = "sha256-qbS1ru37aPPavyo8kQsTqAxMOIQjqAKtTEb7SIkW0y4=";
        };
        doCheck = false;
        meta.mainProgram = "universal-sierra-compiler";
      };

      cairo-format = craneLib.buildPackage rec {
        pname = "cairo";
        version = cairoVersion;
        src = pkgs.fetchFromGitHub {
          name = pname;
          owner = "starkware-libs";
          repo = pname;
          rev = cairoVersion;
          sha256 = "sha256-NQYtlyttIvTxPa6dLbFOkWO5RysaJz2T2S3Z9fg1bg4=";
        };
        cargoExtraArgs = "-p cairo-format";
        doCheck = false;
        meta.mainProgram = "universal-sierra-compiler";
      };

      cairols = craneLib.buildPackage rec {
        pname = "cairols";
        version = cairoVersion;
        src = pkgs.fetchFromGitHub {
          name = pname;
          owner = "software-mansion";
          repo = pname;
          rev = version;
          sha256 = "sha256-F2JPandJB9yQspTi69Zl4DgN5vcK8vlJ02slDolv6KQ=";
        };
        doCheck = false;
        meta.mainProgram = "cairo-language-server";
      };

      starknet-foundry =
        let
          baseArgs = rec {
            pname = "starknet-foundry";
            version = "v0.49.0";
            buildInputs = [ pkgs.perl ];
            doCheck = false;
            src = pkgs.fetchFromGitHub {
              name = pname;
              owner = "foundry-rs";
              repo = "starknet-foundry";
              rev = version;
              sha256 = "sha256-X91KPHL9ELDfaL5HaQEh3B3zFlARfonFMEQGmBaqLuY=";
            };
            cargoExtraArgs = "-p forge";
          };
        in
        craneLib.buildPackage (
          baseArgs
          // {
            cargoVendorDir = craneLib.vendorCargoDeps (
              baseArgs
              // {
                overrideVendorGitCheckout =
                  ps: drv:
                  if
                    (pkgs.lib.any (
                      p:
                      p.source
                      == "git+https://github.com/software-mansion/scarb?rev=210da8dfd0b370f0f1970b33a373f1a7afe6ae33#210da8dfd0b370f0f1970b33a373f1a7afe6ae33"
                    ))
                      ps
                  then
                    craneLib.downloadCargoPackageFromGit {
                      git = "https://github.com/software-mansion/scarb";
                      # this is 210da8dfd0b370f0f1970b33a373f1a7afe6ae33 in the original source but that commit doesn't exist since the original branch was deleted
                      # https://github.com/software-mansion/scarb/pull/2510
                      rev = "77d1911c2fdfacfe194cd95216cf3c8d59284870";
                      hash = "sha256-dVfSxeY84MICu7qnfwAdsddZKMtvmrDbAs6sJVutF9U=";
                    }
                  else
                    drv;
              }
            );
          }
        );

      garaga =
        let
          pname = "garaga";
          version = "v0.18.2";
          src = pkgs.fetchFromGitHub {
            name = pname;
            owner = "keep-starknet-strange";
            repo = pname;
            rev = version;
            sha256 = "sha256-PrVBwSnUxXa+iTkmiT5Dh6u8caVuncMbkZ6leRUw51Y=";
          };
        in
        pyPkgs.buildPythonApplication {
          inherit pname version;
          format = "pyproject";
          nativeBuildInputs = [
            pyPkgs.pythonImportsCheckHook
            pkgs.rustPlatform.cargoSetupHook
            pkgs.rustPlatform.maturinBuildHook
          ];
          # preferWheel = true;
          # pythonImportsCheck = [ "garaga" ];
          cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
            inherit src;
            name = "${pname}-${version}";
            hash = "sha256-vnOVRMoZUVHCe4MKZJDfQpciUovaR2xLPqIUgi+50yU=";
          };
          maturinBuildFlags = [
            "--features"
            "python"
          ];
          runtimeInputs = with pyPkgs; [ pydantic ];
          inherit src;
          propagatedBuildInputs =
            (with pyPkgs; [
              fastecdsa
              typer
              python-dotenv
              pydantic
              requests
              filelock
            ])
            ++ [
              starknet-py
              sympy_1_12_1
            ];
        };

      poseidon-py = pyPkgs.buildPythonPackage rec {
        pname = "poseidon_py";
        version = "0.1.5";

        format = "pyproject";
        build-system = [ pyPkgs.setuptools ];
        dependencies = (with pyPkgs; [ ]) ++ [ crypto-cpp-py ];
        doCheck = false;
        postPatch = ''
          patchShebangs ./build.sh
        '';
        src = pkgs.fetchPypi {
          inherit pname version;
          sha256 = "sha256-rPoPeRdlBSJtx5wn4aalXhGEdTkgRjgmEBovHC3S+/Y=";
        };
      };

      sympy_1_12_1 = pyPkgs.buildPythonPackage rec {
        pname = "sympy";
        version = "1.12.1";
        format = "setuptools";

        src = pyPkgs.fetchPypi {
          inherit pname version;
          hash = "sha256-KHewP5mM2MCPB80N5bdnEZzT70DQn0HDDXIvZoaw+4g=";
        };

        nativeCheckInputs = [ pyPkgs.glibcLocales ];

        propagatedBuildInputs = [ pyPkgs.mpmath ];

        # tests take ~1h
        doCheck = false;
        pythonImportsCheck = [ "sympy" ];

        passthru.tests = {
          inherit (pyPkgs) sage;
        };
      };

      crypto-cpp-py = pyPkgs.buildPythonPackage {
        pname = "crypto_cpp_py";
        version = "1.4.5";
        format = "pyproject";
        build-system = [ pyPkgs.setuptools ];
        cmakeArgs = "-DFETCHCONTENT_SOURCE_DIR_GOOGLETEST=${dbg pkgsUnstable.gtest}";
        nativeBuildInputs = [
          pkgs.bash
          pkgs.cmake
          pkgsUnstable.gtest
          pkgsUnstable.gtest.dev
        ];
        nativeCheckInputs = [
          "${pkgs.gtest.src}/googlemock/src/gmock"
          pkgs.gtest
        ];
        doCheck = false;
        preConfigure = ''
          pushd crypto-cpp
        '';
        postConfigure = ''
          popd
        '';
        postPatch = ''
          patchShebangs ./build_extension.sh

          substituteInPlace ./crypto-cpp/CMakeLists.txt \
            --replace-fail "FetchContent_MakeAvailable(googletest)" ""

          # jfc man
          substituteInPlace ./crypto-cpp/src/starkware/utils/prng_test.cc \
            --replace-fail "900" "900UL" \
            --replace-fail "910" "910UL"

          substituteInPlace ./crypto-cpp/src/starkware/starkex/order_test.cc \
            --replace-fail "4142879348967097428" "4142879348967097428UL" \
            --replace-fail "7162605823528514760" "7162605823528514760UL" \
            --replace-fail "1127571908062083388" "1127571908062083388UL"

          # https://github.com/NixOS/nixpkgs/blob/3e601ca1056848e2b2b12a74bcd1d1235fb5cc3f/pkgs/by-name/mu/multipass/multipassd.nix#L74
          # Configure CMake to use gtest from the nix store since we disabled fetching from the internet.
          cat >> ./crypto-cpp/CMakeLists.txt <<'EOF'
            add_library(gtest INTERFACE)
            target_include_directories(gtest INTERFACE ${pkgsUnstable.gtest.dev}/include)
            target_link_libraries(gtest INTERFACE ${pkgsUnstable.gtest}/lib/libgtest.so ''${CMAKE_THREAD_LIBS_INIT})
            add_dependencies(gtest GMock)

            add_library(gtest_main INTERFACE)
            target_include_directories(gtest_main INTERFACE ${pkgsUnstable.gtest.dev}/include)
            target_link_libraries(gtest_main INTERFACE ${pkgsUnstable.gtest}/lib/libgtest_main.so gtest)

            add_library(gmock INTERFACE)
            target_include_directories(gmock INTERFACE ${pkgsUnstable.gtest.dev}/include)
            target_link_libraries(gmock INTERFACE ${pkgsUnstable.gtest}/lib/libgmock.so gtest)

            add_library(gmock_main INTERFACE)
            target_include_directories(gmock_main INTERFACE ${pkgsUnstable.gtest.dev}/include)
            target_link_libraries(gmock_main INTERFACE ${pkgsUnstable.gtest}/lib/libgmock_main.so gmock gtest_main)
          EOF
        '';

        # these specific versions are pinned
        dependencies = [
          (pyPkgs.buildPythonPackage rec {
            pname = "ecdsa";
            version = "0.18.0";
            format = "setuptools";

            src = pyPkgs.fetchPypi {
              inherit pname version;
              hash = "sha256-GQNIBBVZ4hsiodZc7khSgsoRpvgdUD/duE1QF+ntHkk=";
            };

            propagatedBuildInputs = [ pyPkgs.six ];
            # Only needed for tests
            nativeCheckInputs = [ pkgs.openssl ];
          })
          sympy_1_12_1
        ];
        src = dbg (
          (pkgs.fetchFromGitHub {
            owner = "software-mansion-labs";
            repo = "crypto-cpp-py";
            rev = "65ed90aafa24bad3d3945e6c97170df623ee0d64";
            hash = "sha256-F7+bBEP5Xgt/ECG/kSNcnnxtY1BcDDkJ375/9sMojmM=";
            fetchSubmodules = true;
            deepClone = true;
          }).overrideAttrs
            (oldAttrs: {
              env = oldAttrs.env or { } // {
                GIT_CONFIG_COUNT = 1;
                GIT_CONFIG_KEY_0 = "url.https://github.com/.insteadOf";
                GIT_CONFIG_VALUE_0 = "git@github.com:";
              };
            })
        );
      };

      starknet-py = pyPkgs.buildPythonPackage rec {
        # starknet-py==0.28.0-rc.3
        # don't ask me why the format is different
        pname = "starknet_py";
        version = "0.26.2";

        format = "pyproject";
        build-system = [ pyPkgs.setuptools ];
        dependencies =
          (with pyPkgs; [
            cython
            typing-extensions
            marshmallow-dataclass
            marshmallow-oneofschema
            lark
            aiohttp
            pycryptodome
            asgiref
            eth-keyfile
            eth-keys
            websockets
            tkinter
          ])
          ++ [
            crypto-cpp-py
            poseidon-py
          ];
        doCheck = false;
        cephSupport = false;
        enableDocs = false;
        src = pkgs.fetchPypi {
          inherit pname version;
          sha256 = "sha256-r60Oqx0Bmle7z/ez0Kb19a3qSWUPo61J8vq/XP3YGrE=";
        };
      };
    in
    {
      packages = {
        inherit
          universal-sierra-compiler
          starknet-foundry
          cairo-format
          crypto-cpp-py
          ;
        garaga = pkgs.writeShellApplication {
          name = "garaga";
          runtimeInputs = [
            garaga
            scarb
          ];
          text = ''
            garaga "$@"
          '';
        };
        cairols = pkgs.writeShellApplication {
          name = "cairo-language-server";
          runtimeInputs = [
            cairols
            scarb
            starknet-foundry
            universal-sierra-compiler
          ];
          text = ''
            export SCARB="${scarb}/bin/scarb"
            cairo-language-server "$@"
          '';
        };
        scarb = pkgs.writeShellApplication {
          name = "scarb";
          runtimeInputs = [
            scarb
            starknet-foundry
            universal-sierra-compiler
          ];
          text = ''
            scarb "$@"
          '';
        };
      };
    };
}

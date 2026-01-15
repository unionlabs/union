_: {
  perSystem =
    {
      pkgs,
      pkgsUnstable,
      crane,
      rust,
      dbg,
      ...
    }:
    let
      craneLib = crane.lib.overrideToolchain (_: rust.mkToolchain { channel = "1.91.1"; });

      cairoVersion = "v2.13.1";

      python = pkgsUnstable.python312.override {
        packageOverrides = _final: prev: rec {
          eth-keyfile = prev.buildPythonPackage rec {
            pname = "eth_keyfile";
            version = "0.8.1";
            # pyproject = true;
            doCheck = false;
            format = "setuptools";
            src = prev.fetchPypi {
              inherit pname version;
              hash = "sha256-lwi8MfOGtSzKCWkjj/NbGscr16cYbyqEuGEQ08lzvsE=";
            };
          };
          bip-utils = prev.buildPythonPackage rec {
            pname = "bip_utils";
            version = "2.8.0";
            doCheck = false;
            format = "setuptools";
            src = prev.fetchPypi {
              inherit pname version;
              hash = "sha256-flzG7QtfCD9UtM3b6XerOumzWukDLQN+mKSTBZpm5/g=";
            };
          };
          ecdsa = prev.buildPythonPackage rec {
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
          };
          ledgerwallet = prev.buildPythonPackage rec {
            pname = "ledgerwallet";
            version = "0.5.3";
            format = "pyproject";
            buildInputs = [
              prev.setuptools
              prev.setuptools-scm
            ];
            propagatedBuildInputs =
              (with prev; [
                cryptography
                click
                construct
                hidapi
                intelhex
                pillow
                protobuf
                requests
                tabulate
                toml
              ])
              ++ [ ecdsa ];

            # Regenerate protobuf bindings to lift the version upper-bound and enable
            # compatibility the current default protobuf library.
            preBuild = ''
              protoc --python_out=. --pyi_out=. ledgerwallet/proto/*.proto
            '';

            pythonImportsCheck = [ "ledgerwallet" ];

            postPatch = ''
              substituteInPlace pyproject.toml \
                --replace-fail '"protobuf >=3.20,<4"' '"protobuf >=3.20"'
            '';

            src = prev.fetchPypi {
              inherit pname version;
              hash = "sha256-Hy06MwzFV170EA9nCNe+HNhqJ5WG4B+E9SfDoH/KqNE=";
            };
          };
        };
      };

      pyPkgs = python.pkgs;

      scarb-src = pkgs.fetchFromGitHub {
        name = "scarb-src";
        owner = "software-mansion";
        repo = "scarb";
        rev = cairoVersion;
        sha256 = "sha256-cX4sDoPpn7Wr1lcR3BsGWOMIUGK+G7BHwqiGJumDbsQ=";
      };

      cairo-src = pkgs.fetchFromGitHub {
        name = "cairo-src";
        owner = "starkware-libs";
        repo = "cairo";
        rev = cairoVersion;
        sha256 = "sha256-T4p4usng7xhiUZo0JB26bY9IQAAtX1bXj8hdKsrVbTk=";
      };

      scarb = craneLib.buildPackage rec {
        pname = "scarb";
        version = cairoVersion;
        src = pkgs.fetchFromGitHub {
          name = pname;
          owner = "software-mansion";
          repo = pname;
          rev = version;
          sha256 = "sha256-cX4sDoPpn7Wr1lcR3BsGWOMIUGK+G7BHwqiGJumDbsQ=";
        };
        cargoExtraArgs = "-p scarb";
        doCheck = false;
        meta.mainProgram = "scarb";
        SCARB_CORELIB_LOCAL_PATH = "${cairo-src}/corelib";
      };

      scarb-doc = craneLib.buildPackage {
        pname = "scarb-doc";
        version = cairoVersion;
        src = scarb-src;
        cargoExtraArgs = "-p scarb-doc";
        doCheck = false;
        meta.mainProgram = "scarb-doc";
      };

      scarb-mdbook = craneLib.buildPackage {
        pname = "scarb-mdbook";
        version = cairoVersion;
        src = scarb-src;
        cargoExtraArgs = "-p scarb-mdbook";
        doCheck = false;
        meta.mainProgram = "scarb-mdbook";
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

      cairo-format = craneLib.buildPackage {
        pname = "cairo-format";
        version = cairoVersion;
        src = cairo-src;
        cargoExtraArgs = "-p cairo-format";
        doCheck = false;
        meta.mainProgram = "cairo-format";
      };

      cairo-language-server =
        let
          baseArgs = rec {
            pname = "cairols";
            version = cairoVersion;
            src = dbg (
              pkgs.stdenv.mkDerivation {
                name = "cairols-patched-source";
                src = pkgs.fetchFromGitHub {
                  name = pname;
                  owner = "software-mansion";
                  repo = pname;
                  rev = version;
                  sha256 = "sha256-T//raZMQEdJ+INzuDsGmUc7jCVe05nlpwMb/yZFD2ho=";
                };
                patches = [
                  ./cairols-remove-tests.patch
                ];
                installPhase = ''
                  cp -r --no-preserve=mode . $out
                '';
              }
            );
            doCheck = false;
          };
        in
        craneLib.buildPackage baseArgs;

      starknet-foundry =
        let
          baseArgs = rec {
            pname = "starknet-foundry";
            version = "v0.53.0-rc.0";
            buildInputs = [ pkgs.perl ];
            doCheck = false;
            src = pkgs.fetchFromGitHub {
              name = pname;
              owner = "foundry-rs";
              repo = "starknet-foundry";
              rev = version;
              sha256 = "sha256-d+nQvbMWLqoY6G/53r4JqBUkk6n4zGyGIEh+eqx+cAU=";
            };
            cargoExtraArgs = "-p forge -p sncast";
            ALCHEMY_API_KEY = "bullshit";
          };
        in
        craneLib.buildPackage baseArgs;

      garaga =
        let
          pname = "garaga";
          version = "6fb59ac369ab2bc4699e8ff7c09bdeee81cfcaca";
          # The following garaga fork include the changes to be able to generate proof calldata for cometbls
          src = pkgs.fetchFromGitHub {
            name = pname;
            owner = "aeryz";
            repo = pname;
            rev = version;
            sha256 = "sha256-24CC53c6A9cozYWbTMpbnB/7wrcRq4Z9jDWzuG8Yf1k=";
          };
        in
        pyPkgs.buildPythonApplication {
          inherit pname version;
          format = "pyproject";
          nativeBuildInputs = [
            pyPkgs.pythonImportsCheckHook
            pkgsUnstable.rustPlatform.cargoSetupHook
            pkgsUnstable.rustPlatform.maturinBuildHook
          ];
          # preferWheel = true;
          # pythonImportsCheck = [ "garaga" ];
          cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
            inherit src;
            name = "${pname}-${version}";
            hash = "sha256-R0OitN0yUiY3+cc7jwov3lkDkDgxcfehBymiQm0mbPw=";
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
              # starknet-py
              starknet-py-unbroken
              sympy_1_12_1
            ];
        };

      poseidon-py = pyPkgs.buildPythonPackage rec {
        pname = "poseidon_py";
        version = "0.1.5";

        format = "pyproject";
        build-system = [ pyPkgs.setuptools ];
        dependencies = [ crypto-cpp-py ];
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
          "${pkgsUnstable.gtest.src}/googlemock/src/gmock"
          pkgsUnstable.gtest
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

      starknet-py-unbroken = pyPkgs.buildPythonPackage rec {
        # starknet-py==0.28.0-rc.3
        # don't ask me why the format is different
        pname = "starknet_py_unbroken";
        version = "0.29.0rc3";

        format = "pyproject";
        build-system = [ pyPkgs.setuptools ];
        dependencies =
          (with pyPkgs; [
            bip-utils
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
            ledgerwallet
            semver
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
          sha256 = "sha256-j8YHm8HUdASd591cfbO/+w997xTxt16IdJg9G9xU1yQ=";
        };
      };
    in
    {
      packages = {
        inherit
          universal-sierra-compiler
          crypto-cpp-py
          ;
        cairo-format = pkgs.writeShellApplication {
          name = "cairo-format";
          runtimeInputs = [
            cairo-format
          ];
          text = ''
            cairo-format --merge-use-items true "$@"
          '';
        };
        snforge = pkgs.writeShellApplication {
          name = "snforge";
          runtimeInputs = [
            starknet-foundry
            universal-sierra-compiler
          ];
          text = ''
            snforge "$@"
          '';
        };
        sncast = pkgs.writeShellApplication {
          name = "sncast";
          runtimeInputs = [
            starknet-foundry
            # universal-sierra-compiler
          ];
          text = ''
            sncast "$@"
          '';
        };
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
        cairo-language-server = pkgs.writeShellApplication {
          name = "cairo-language-server";
          runtimeInputs = [
            cairo-language-server
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
            scarb-doc
            scarb-mdbook
            scarb
            starknet-foundry
            universal-sierra-compiler
          ];
          text = ''
            # mdbook
            scarb "$@"
          '';
        };
      };
    };
}

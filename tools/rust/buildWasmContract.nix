{ buildWorkspaceMember
, crateCargoToml
, pkgs
, lib
, craneLib
, rust
, dbg
}:
let
  CARGO_BUILD_TARGET = "wasm32-unknown-unknown";

  dashesToUnderscores = builtins.replaceStrings [ "-" ] [ "_" ];

  featuresString = features: if features == null then "" else (lib.concatMapStrings (feature: "-${feature}") features);
  allChecks = checks: builtins.concatLists [
    checks
    [
      (file_name: ''
        blob=$(${pkgs.binaryen}/bin/wasm-dis ${file_name})

        if [ $? -ne 0 ]
        then
          echo $blob
          exit $?
        fi

        # grep exits 0 if a match is found
        if echo $blob | grep -P '\.extend\d{1,2}_s'
        then
          echo "wasm binary contains invalid opcodes!"
          exit 1
        else
          echo "wasm binary doesn't contain any sign-extension opcodes!"
        fi
      '')
    ]
  ];

  cargoBuildInstallPhase = { features, contractFileNameWithoutExt, checks }:
    let
      outputFilePath = "$out/lib/${contractFileNameWithoutExt}${dashesToUnderscores (featuresString features)}.wasm";
    in
    ''
      ${
        builtins.concatStringsSep
          "\n\n"
          (map
            (check: check "target/wasm32-unknown-unknown/release/${contractFileNameWithoutExt}.wasm")
            (allChecks checks)
          )
      }

      mkdir -p $out/lib
      mv target/wasm32-unknown-unknown/release/${contractFileNameWithoutExt}.wasm ${outputFilePath}

      ${pkgs.binaryen}/bin/wasm-opt -O3 ${outputFilePath} -o ${outputFilePath}

      # gzip the binary to ensure it's not too large to upload
      gzip -fk ${outputFilePath}

      # TODO: check that the size isn't over the max size allowed to be uploaded?
    '';
  cargoBuildExtraArgs = features: "--no-default-features --lib ${if features != null then lib.concatStringsSep " " ([ "--features" ] ++ features) else ""}";
  rustflags = "-C target-feature=-sign-ext -C link-arg=-s -C target-cpu=mvp -C opt-level=z -C passes=adce,loop-deletion";
in
{
  buildWasmContract =
    { crateDirFromRoot
    , features ? null
    , additionalSrcFilter ? _: _: false
    , additionalTestSrcFilter ? _: _: false
      # list of fns taking the file path as an argument and producing arbitrary shell script
    , checks ? [ ]
    }:
    let
      contractFileNameWithoutExt =
        dashesToUnderscores (crateCargoToml crateDirFromRoot).package.name;

      all =
        buildWorkspaceMember {
          # extraEnv = {
          #   nativeBuildInputs = [ pkgs.breakpointHook ];
          # };
          inherit crateDirFromRoot additionalSrcFilter additionalTestSrcFilter;
          buildStdTarget = CARGO_BUILD_TARGET;
          pnameSuffix = featuresString features;

          cargoBuildExtraArgs = cargoBuildExtraArgs features;
          inherit rustflags;

          cargoBuildCheckPhase = ''
            ls target/wasm32-unknown-unknown/release

            sha256sum target/wasm32-unknown-unknown/release/${contractFileNameWithoutExt}.wasm  
          '';
          cargoBuildInstallPhase = cargoBuildInstallPhase { inherit features contractFileNameWithoutExt checks; };
        };
    in
    {
      inherit (all) checks packages;
    };

  buildRemoteWasmContract =
    { src
    , version
    , package ? null
    , features ? null
    , contractFileNameWithoutExt ? package
      # list of fns taking the file path as an argument and producing arbitrary shell script
    , checks ? [ ]
    }:
    let
      craneLib' = craneLib.overrideToolchain (rust.mkBuildStdToolchain { target = CARGO_BUILD_TARGET; });

      attrs = {
        pname = contractFileNameWithoutExt;
        inherit src version CARGO_BUILD_TARGET;

        cargoVendorDir = craneLib.vendorMultipleCargoDeps {
          inherit (craneLib.findCargoFiles src) cargoConfigs;
          cargoLockList = [
            "${src}/Cargo.lock"
            ./rust-std-Cargo.lock
          ];
        };

        doCheck = false;
        pnameSuffix = featuresString features;
        cargoCheckExtraArgs = "";
        cargoExtraArgs = (cargoBuildExtraArgs features) + " -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target ${CARGO_BUILD_TARGET} -j1 " + (pkgs.lib.optionalString (package != null) " -p ${package}");
        RUSTFLAGS = rustflags;

        installPhaseCommand = cargoBuildInstallPhase { inherit features contractFileNameWithoutExt checks; };
      };
    in
    craneLib'.buildPackage (attrs // {
      cargoArtifacts = craneLib'.buildDepsOnly attrs;
    });
}

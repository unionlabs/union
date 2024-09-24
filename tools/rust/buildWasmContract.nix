{
  buildWorkspaceMember,
  crateCargoToml,
  pkgs,
  lib,
  craneLib,
  rust,
  dbg,
}:
let
  CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
  DEFAULT_MAX_SIZE = 800 * 1024;

  dashesToUnderscores = builtins.replaceStrings [ "-" ] [ "_" ];

  featuresString =
    features: if features == null then "" else (lib.concatMapStrings (feature: "-${feature}") features);
  allChecks =
    checks: _maxSize:
    builtins.concatLists [
      checks
      # [
      #   (file_name: ''
      #     file_size=$(stat -c %s "${file_name}")
      #     max_size_str="${toString maxSize}"

      #     if [ "$file_size" -gt "$max_size_str" ]; then
      #       echo "Error: File size: $file_size exceeds $max_size_str bytes"
      #       exit 1
      #     else
      #       echo "File size: $file_size bytes"
      #     fi
      #   '')
      # ]
    ];

  cargoBuildInstallPhase =
    {
      features,
      contractFileNameWithoutExt,
      checks,
      maxSize,
    }:
    let
      outputFilePath = "$out/lib/${contractFileNameWithoutExt}${dashesToUnderscores (featuresString features)}.wasm";
    in
    ''
      mkdir -p $out/lib
      mv target/wasm32-unknown-unknown/release/${contractFileNameWithoutExt}.wasm ${outputFilePath}

      ${pkgs.binaryen}/bin/wasm-opt -O3 ${outputFilePath} -o ${outputFilePath}

      ${builtins.concatStringsSep "\n\n" (
        map (check: check "${outputFilePath}") (allChecks checks maxSize)
      )}

      # gzip the binary to ensure it's not too large to upload
      gzip -fk ${outputFilePath}
    '';
  cargoBuildExtraArgs =
    features:
    "--no-default-features --lib ${
      if features != null then lib.concatStringsSep " " ([ "--features" ] ++ features) else ""
    }";
  rustflags = "-C link-arg=-s -C target-cpu=mvp -C opt-level=z -C passes=adce,loop-deletion";
in
{
  buildWasmContract =
    {
      crateDirFromRoot,
      features ? null,
      # list of fns taking the file path as an argument and producing arbitrary shell script
      checks ? [ ],
      # maximum size of the wasm output
      maxSize ? DEFAULT_MAX_SIZE,
      extraBuildInputs ? [ ],
      extraNativeBuildInputs ? [ ],
    }:
    let
      contractFileNameWithoutExt = dashesToUnderscores (crateCargoToml crateDirFromRoot).package.name;

      all = buildWorkspaceMember {
        # extraEnv = {
        #   nativeBuildInputs = [ pkgs.breakpointHook ];
        # };
        inherit crateDirFromRoot;
        inherit extraBuildInputs;
        inherit extraNativeBuildInputs;
        buildStdTarget = CARGO_BUILD_TARGET;
        pnameSuffix = featuresString features;

        cargoBuildExtraArgs = cargoBuildExtraArgs features;
        inherit rustflags;

        cargoBuildCheckPhase = ''
          ls target/wasm32-unknown-unknown/release

          sha256sum target/wasm32-unknown-unknown/release/${contractFileNameWithoutExt}.wasm  
        '';
        cargoBuildInstallPhase = cargoBuildInstallPhase {
          inherit
            features
            contractFileNameWithoutExt
            checks
            maxSize
            ;
        };
      };
    in
    {
      inherit (all) checks packages;
    };

  buildRemoteWasmContract =
    {
      src,
      version,
      package ? null,
      features ? null,
      contractFileNameWithoutExt ? package,
      # list of fns taking the file path as an argument and producing arbitrary shell script
      checks ? [ ],
      maxSize ? DEFAULT_MAX_SIZE,
    }:
    let
      craneLib' = craneLib.overrideToolchain (
        rust.mkBuildStdToolchain { targets = [ CARGO_BUILD_TARGET ]; }
      );

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
        cargoExtraArgs =
          (cargoBuildExtraArgs features)
          + " -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target ${CARGO_BUILD_TARGET} -j1 "
          + (pkgs.lib.optionalString (package != null) " -p ${package}");
        RUSTFLAGS = rustflags;

        installPhaseCommand = cargoBuildInstallPhase {
          inherit
            features
            contractFileNameWithoutExt
            checks
            maxSize
            ;
        };
      };
    in
    craneLib'.buildPackage (
      attrs
      // {
        cargoArtifacts = craneLib'.buildDepsOnly attrs;
      }
    );
}

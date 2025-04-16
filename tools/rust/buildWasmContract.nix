{
  buildWorkspaceMember,
  crateCargoToml,
  pkgs,
  lib,
  craneLib,
  rust,
  dbg,
  gitRev,
}:
let
  CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
  DEFAULT_MAX_SIZE = 800 * 1024;

  dashesToUnderscores = builtins.replaceStrings [ "-" ] [ "_" ];

  mkFeaturesString =
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
    ''
      # mkdir -p $out/lib
      mv target/wasm32-unknown-unknown/release/${contractFileNameWithoutExt}.wasm $out

      ${pkgs.binaryen}/bin/wasm-opt -O3 $out -o $out

      ${builtins.concatStringsSep "\n\n" (map (check: check "$out") (allChecks checks maxSize))}

      # gzip the binary to ensure it's not too large to upload
      gzip -fk $out
    '';
  cargoBuildExtraArgs =
    features:
    "--no-default-features --lib ${
      if features != null then lib.concatStringsSep " " ([ "--features" ] ++ features) else ""
    }";
  rustflags = "-C link-arg=-s -C target-cpu=mvp -C opt-level=z -C passes=adce,loop-deletion -Zlocation-detail=none";
in
{
  buildWasmContract =
    crateDirFromRoot:
    {
      features ? null,
      # list of fns taking the file path as an argument and producing arbitrary shell script
      checks ? [ ],
      # maximum size of the wasm output
      maxSize ? DEFAULT_MAX_SIZE,
      extraBuildInputs ? [ ],
      extraNativeBuildInputs ? [ ],
    }:
    let
      packageName = "${(crateCargoToml crateDirFromRoot).package.name}${mkFeaturesString features}";

      contract-basename = dashesToUnderscores (crateCargoToml crateDirFromRoot).package.name;

      all = buildWorkspaceMember crateDirFromRoot {
        inherit extraBuildInputs extraNativeBuildInputs;
        buildStdTarget = CARGO_BUILD_TARGET;
        pnameSuffix = mkFeaturesString features;

        cargoBuildExtraArgs = cargoBuildExtraArgs features;
        inherit rustflags;

        cargoBuildCheckPhase = ''
          ls target/wasm32-unknown-unknown/release

          sha256sum target/wasm32-unknown-unknown/release/${contract-basename}.wasm  
        '';
        cargoBuildInstallPhase = cargoBuildInstallPhase {
          inherit
            features
            checks
            maxSize
            ;
          contractFileNameWithoutExt = contract-basename;
        };
      };

      addPackageName =
        old:
        old
        // {
          pname = "${packageName}.wasm";
          passthru = (old.passthru or { }) // {
            inherit packageName;
          };
        };
    in
    (all.${packageName}.overrideAttrs addPackageName)
    // {
      release = all.${packageName}.release.overrideAttrs addPackageName;
    };
}

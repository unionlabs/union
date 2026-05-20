{
  buildWorkspaceMember,
  crateCargoToml,
  pkgs,
  pkgsUnstable,
  lib,
}:
let
  CARGO_BUILD_TARGET = "wasm32-unknown-unknown";

  dashesToUnderscores = builtins.replaceStrings [ "-" ] [ "_" ];

  mkFeaturesString =
    features: if features == null then "" else (lib.concatMapStrings (feature: "-${feature}") features);

  cargoBuildInstallPhase =
    {
      crateDirFromRoot,
      contractFileNameWithoutExt,
    }:
    ''
      ${pkgs.lib.meta.getExe pkgsUnstable.wasm-bindgen-cli_0_2_100} \
        target/wasm32-unknown-unknown/wasm-release/${contractFileNameWithoutExt}.wasm \
        --out-dir $out \
        --typescript \
        --target web

      ls -alh $out

      ${pkgs.binaryen}/bin/wasm-opt \
        $out/${dashesToUnderscores (crateCargoToml crateDirFromRoot).package.name}_bg.wasm \
        -Oz -o $out/${dashesToUnderscores (crateCargoToml crateDirFromRoot).package.name}_bg.wasm
    '';
  cargoBuildExtraArgs =
    features:
    "--no-default-features --lib ${
      if features != null then lib.concatStringsSep " " ([ "--features" ] ++ features) else ""
    }";
  mkRustflags = "-C opt-level=z -C link-arg=-s -C target-cpu=mvp -C passes=adce,loop-deletion -Zlocation-detail=none";
in
crateDirFromRoot:
{
  features ? null,
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
    rustflags = mkRustflags;

    cargoBuildInstallPhase = cargoBuildInstallPhase {
      inherit crateDirFromRoot;
      contractFileNameWithoutExt = contract-basename;
    };
    extraEnv = {
      CARGO_PROFILE = "wasm-release";
    };
  };
in
all.${packageName}

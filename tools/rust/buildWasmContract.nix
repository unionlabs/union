{ buildWorkspaceMember
, crateCargoToml
, pkgs
, lib
}:
{ crateDirFromRoot
, features ? null
, additionalSrcFilter ? _: _: false
, additionalTestSrcFilter ? _: _: false
}:
let
  CARGO_BUILD_TARGET = "wasm32-unknown-unknown";

  dashesToUnderscores = builtins.replaceStrings [ "-" ] [ "_" ];

  contractFileNameWithoutExt =
    dashesToUnderscores (crateCargoToml crateDirFromRoot).package.name;

  featuresString = if features == null then "" else (lib.concatMapStrings (feature: "-${feature}") features);

  all =
    buildWorkspaceMember {
      inherit crateDirFromRoot additionalSrcFilter additionalTestSrcFilter;
      buildStdTarget = CARGO_BUILD_TARGET;
      pnameSuffix = featuresString;

      cargoBuildExtraArgs = "--no-default-features --lib ${if features != null then lib.concatStringsSep " " ([ "--features" ] ++ features) else ""}";
      rustflags = "-C target-feature=-sign-ext -C link-arg=-s -C target-cpu=mvp";

      cargoBuildCheckPhase = ''
        ls target/wasm32-unknown-unknown/release
        blob=$(${pkgs.binaryen}/bin/wasm-dis target/wasm32-unknown-unknown/release/${contractFileNameWithoutExt}.wasm)
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
      '';

      cargoBuildInstallPhase = ''
        mkdir -p $out/lib
        mv target/wasm32-unknown-unknown/release/${contractFileNameWithoutExt}.wasm $out/lib/${contractFileNameWithoutExt}${dashesToUnderscores featuresString}.wasm
        # TODO: Re-enable this?
        # Optimize the binary size a little bit more
        # ${pkgs.binaryen}/bin/wasm-opt -Os target/wasm32-unknown-unknown/release/${contractFileNameWithoutExt}.wasm -o $out/lib/${contractFileNameWithoutExt}.wasm

        # gzip the binary to ensure it's not too large to upload
        gzip -fk $out/lib/${contractFileNameWithoutExt}${dashesToUnderscores featuresString}.wasm
        # TODO: check that the size isn't over the max size allowed to be uploaded?
      '';
    };
in
{
  inherit (all) checks packages;
}

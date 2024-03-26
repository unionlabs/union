{ ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, dbg, ... }:
    let
      voyager = crane.buildWorkspaceMember {
        crateDirFromRoot = "voyager";
        # temporarily, to keep warnings in-editor until i fix them
        cargoClippyExtraArgs = "--allow deprecated";
        extraEnv = {
          SQLX_OFFLINE = "1";
        };
      };
    in
    {
      packages = voyager.packages;
      checks = voyager.checks;
    };
}

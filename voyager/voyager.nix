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
      packages = voyager.packages // {
        voy-send-msg = pkgs.writeShellApplication {
          name = "voy-send-msg";
          runtimeInputs = [ pkgs.curl ];
          text = ''
            set -e
            curl localhost:65534/msg -H "content-type: application/json" -d "$@"
          '';
        };
      };
      checks = voyager.checks;
    };
}

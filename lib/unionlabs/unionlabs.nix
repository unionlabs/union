{ inputs, ... }: {
  perSystem = { self', inputs', pkgs, system, config, crane, stdenv, dbg, ... }:
    {
      inherit (crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/unionlabs";
      }) checks;
    };
}

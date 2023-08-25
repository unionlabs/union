{ ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, ... }:
    let
      voyagerAll = (crane.buildWorkspaceMember {
        crateDirFromRoot = "voyager";
      });
    in
    {
      inherit (voyagerAll) packages checks;
    };
}

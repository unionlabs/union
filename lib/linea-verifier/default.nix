{ ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, dbg, lib, ... }:
    let
      linea-verifier-all = (crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/linea-verifier";
      });
    in
    {
      inherit (linea-verifier-all) checks;
    };
}

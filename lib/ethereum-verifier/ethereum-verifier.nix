{ ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, dbg, lib, ... }:
    let
      ethereum-verifier-all = (crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/ethereum-verifier";
        additionalTestSrcFilter = path: _:
          (lib.hasPrefix "lib/ethereum-verifier/src/test" path)
          && (lib.strings.hasSuffix ".json" path);
      });
    in
    {
      inherit (ethereum-verifier-all) packages checks;
    };
}

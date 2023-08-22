{ ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, dbg, ... }:
    let
      ethereum-verifier-all = (crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/ethereum-verifier";
        additionalTestSrcFilter = path: _: crane.ensureDirectoryIncluded {
          path' = path;
          pathToInclude = "light-clients/ethereum-light-client/src/test";
        };
      });
    in
    {
      inherit (ethereum-verifier-all) packages checks;
    };
}

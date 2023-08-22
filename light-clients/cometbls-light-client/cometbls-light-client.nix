{ ... }: {
  perSystem = { crane, lib, ... }:
    let
      workspace = (crane.buildWasmContract {
        crateDirFromRoot = "light-clients/cometbls-light-client";
      });
    in
    {
      inherit (workspace) packages checks;
    };
}

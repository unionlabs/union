{ ... }: {
  perSystem = { crane, lib, ... }:
    let
      workspace = (crane.buildWasmContract {
        crateDirFromRoot = "light-clients/cometbls-light-client";
      });
    in
    {
      packages = workspace.packages;
      checks = workspace.checks;
    };
}

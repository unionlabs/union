{ ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, ... }:
    let
      workspace = (crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/cometbls-groth16-verifier";
      });
    in
    {
      packages = workspace.packages;
      checks = workspace.checks;
    };
}

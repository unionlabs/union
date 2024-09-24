_: {
  perSystem =
    {
      self',
      pkgs,
      system,
      config,
      crane,
      stdenv,
      ...
    }:
    let
      workspace = crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/cometbls-groth16-verifier";
      };
    in
    {
      inherit (workspace) checks;
    };
}

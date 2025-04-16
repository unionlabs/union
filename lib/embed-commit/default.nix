_: {
  perSystem =
    { crane, ... }:
    {
      packages = crane.buildWorkspaceMember "lib/embed-commit/verifier" { };
    };
}

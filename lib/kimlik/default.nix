_: {
  perSystem =
    { crane, ... }:
    {
      packages = crane.buildWorkspaceMember "lib/kimlik/verifier" { };
    };
}

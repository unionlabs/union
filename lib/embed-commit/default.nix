_: {
  perSystem =
    { crane, pkgs, ... }:
    {
      packages = crane.buildWorkspaceMember "lib/embed-commit/verifier" {
        # extraNativeBuildInputs = [ pkgs.cmake ];
      };
    };
}

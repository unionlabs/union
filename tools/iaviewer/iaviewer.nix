_: {
  perSystem =
    { pkgs, lib, ... }:
    let
      name = "iaviewer";
    in
    {
      packages = {
        iaviewer = pkgs.buildGoModule {
          inherit name;
          version = "0.20.1";
          src = pkgs.fetchFromGitHub {
            name = "iaviewer";
            owner = "cosmos";
            repo = "iavl";
            rev = "ff621dbfe554c8b0911ff3f3a9cf4956268e3433";
            sha256 = "sha256-2M4d0R+Bi5dl5IcO8JMRlFZx9tQUvzL1KI7GSIvXGEY=";
          };
          vendorHash = "sha256-f5GRp7PIBgJjJbrJoLQuTIJyWnH0+ezUYrOSsp1qblg=";
          meta.mainProgram = "iaviewer";
          subPackages = [ "./cmd/iaviewer" ];
        };
      };
    };
}

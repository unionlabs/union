_: {
  perSystem =
    {
      pkgs,
      goPkgs,
      crane,
      ...
    }:
    {
      packages = {
        devnet-utils =
          (crane.buildWorkspaceMember {
            crateDirFromRoot = "tools/devnet-utils";
            dev = true;
          }).packages.devnet-utils-dev;

        ignite-cli = goPkgs.buildGoModule {
          name = "ignite-cli";
          src = pkgs.fetchFromGitHub {
            owner = "ignite";
            repo = "cli";
            rev = "v28.7.0";
            sha256 = "sha256-/gBykwBlZsHUWCJ01rdluU10xuEEmPmCfzSWlO6znW8=";
          };
          doCheck = false;
          vendorHash = "sha256-ks9wZUIwN0dOcXxxRk1Amxd0TPJBbLfKC9lzV4IMdjk=";
        };
      };
    };
}

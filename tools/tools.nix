{ ... }: {
  perSystem = { pkgs, crane, ... }: {
    packages.devnet-utils = (crane.buildWorkspaceMember {
      crateDirFromRoot = "tools/devnet-utils";
    }).packages.devnet-utils;
  };
}

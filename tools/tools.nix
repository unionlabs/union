{ ... }: {
  perSystem = { pkgs, crane, ... }: {
    packages.keygen = (crane.buildWorkspaceMember {
      crateDirFromRoot = "tools/keygen";
    }).packages.keygen;
  };
}

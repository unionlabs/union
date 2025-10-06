{
  ...
}:
{
  perSystem =
    {
      crane,
      ...
    }:
    let
      mkTest =
        name:
        (crane.buildWorkspaceMember "tools/union-test" {
          dontRemoveDevDeps = true;
          cargoBuildExtraArgs = "--tests";
          cargoBuildInstallPhase = ''
            mkdir -p $out
            FNAME=$(printf `ls -f -I '.*d' target/release/deps/${name}-*`)
            mv "$FNAME" $out/${name}
          '';
        }).union-test;
    in
    {
      packages.e2e-lst-tests = mkTest "lst";
    };
}

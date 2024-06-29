{ self, ... }: {
  perSystem = { pkgs, rust, crane, mkCi, ... }:
    let
      tidy = (crane.buildWorkspaceMember {
        crateDirFromRoot = "tools/tidy";
        dev = true;
      }).packages.tidy-dev;
    in
    {
      checks.cargo-tidy = pkgs.stdenv.mkDerivation {
        name = "cargo-tidy";
        dontUnpack = true;
        src = pkgs.lib.cleanSourceWith {
          name = "cargo-tidy-source";
          src = self.outPath;
          filter = path: type: crane.lib.filterCargoSources path type;
        };
        buildInputs = [ rust.toolchains.dev tidy ];
        doCheck = true;
        checkPhase = ''
          cd $src/.

          ls -al

          tidy Cargo.toml

          touch $out
        '';
      };
    };
}

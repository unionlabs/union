{ self, ... }:
{
  perSystem =
    {
      pkgs,
      rust,
      crane,
      ...
    }:
    let
      inherit (crane.buildWorkspaceMember "tools/tidy" { }) tidy;
    in
    {
      checks.cargo-tidy = pkgs.stdenv.mkDerivation {
        name = "cargo-tidy";
        dontUnpack = true;
        # TODO: Improve this source to only include Cargo.toml files
        src = pkgs.lib.cleanSourceWith {
          name = "cargo-tidy-source";
          src = self.outPath;
          filter = crane.lib.filterCargoSources;
        };
        buildInputs = [
          rust.toolchains.dev
          tidy
        ];
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

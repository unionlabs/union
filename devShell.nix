_: {
  perSystem =
    {
      biome,
      pkgs,
      unstablePkgs,
      lib,
      ensureAtRepositoryRoot,
      ...
    }:
    let
      pkgsDeps = with pkgs; [
        pkg-config
        biome
      ];
      nodeDeps = with unstablePkgs; [
        vips
        nodePackages_latest.nodejs
      ];
      combinedDeps = pkgsDeps ++ nodeDeps;
    in
    {
      apps = {
        pre-commit = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "pre-commit";
            runtimeInputs = combinedDeps;
            text = ''
              ${ensureAtRepositoryRoot}

              echo "Applying nix fmt (through fmt-site)"
              nix run .#fmt-site

              # seems deprecated
              # echo "Applying biome fmt"
              # ${lib.getExe biome} check . --write --unsafe \
              #   --log-level="info" \
              #   --log-kind="pretty" \
              #   --diagnostic-level="info"

              echo "Checking spelling"
              nix build .\#checks.${pkgs.system}.spellcheck -L

              echo "Running Site Check"
              nix run .\#site-check

              echo "Running Docs Check"
              nix run .\#docs-check
            '';
          };
        };
      };
    };
}

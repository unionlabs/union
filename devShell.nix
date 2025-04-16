_: {
  perSystem =
    {
      pkgs,
      pkgsUnstable,
      ensureAtRepositoryRoot,
      ...
    }:
    let
      deps = with pkgsUnstable; [
        vips
        pkg-config
        nodePackages_latest.nodejs
      ];
    in
    {
      apps = {
        pre-commit = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "pre-commit";
            runtimeInputs = deps;
            text = ''
              ${ensureAtRepositoryRoot}

              echo "Applying nix fmt (through fmt-site)"
              nix run .#fmt-site

              echo "Applying biome fmt"
              ${pkgsUnstable.biome}/bin/biome check . --write --unsafe \
                --log-level="info" \
                --log-kind="pretty" \
                --diagnostic-level="info"

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

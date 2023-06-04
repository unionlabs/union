{ ... }: {
  perSystem = { pkgs, self', system, ... }:
    {
      packages = rec {
        mdx-gen = pkgs.writeShellApplication {
          name = "mdx-gen";
          runtimeInputs = [ mdx ];
          text = ''
            node index.js "$1"
          '';
        };
        mdx = pkgs.buildNpmPackage {
          name = "mdx";
          src = ./.;
          npmDepsHash = null;
          dontNpmBuild = true;
        };
      };
    };
}

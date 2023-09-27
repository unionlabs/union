{ ... }: {
  perSystem = { pkgs, lib, nix-filter, ... }:
    let
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = rec {
        site-static =
          let
            yarnPackage = pkgs.mkYarnPackage rec {
              name = "${packageJSON.name}-${version}";
              version = packageJSON.version;
              src = nix-filter
                {
                  name = "union-site-source";
                  root = ./.;
                  exclude = [
                    (nix-filter.matchExt "nix")
                    ./README.md
                  ];
                };
              packageJson = "${src}/package.json";
              yarnLock = "${src}/yarn.lock";
              buildPhase = ''
                yarn --offline build 
              '';
              distPhase = "true";
            };
          in
          pkgs.stdenv.mkDerivation {
            name = "union-site-src";
            phases = [ "installPhase" ];
            installPhase = ''
              mkdir -p $out
              cp -R ${yarnPackage}/libexec/${packageJSON.name}/deps/${packageJSON.name}/build/* $out
            '';
          };


        site-server = pkgs.writeShellApplication {
          name = packageJSON.name;

          runtimeInputs = [ site-static pkgs.miniserve ];

          text = ''
            miniserve --index index.html --spa ${site-static}
          '';
        };
      };

      apps = {
        docs-dev-server = { };
      };
    };
}

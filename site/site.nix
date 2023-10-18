{ ... }: {
  perSystem = { pkgs, lib, nix-filter, ensureAtRepositoryRoot, ... }:
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
                yarn run postinstall
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

        impure-site-deploy = pkgs.writeShellApplication {
          name = "impure-site-deploy";

          runtimeInputs = [ pkgs.nodePackages.vercel ];

          text = ''
            ${ensureAtRepositoryRoot}

            cd site
            yarn
            yarn build

            # cspell:disable
            export VERCEL_PROJECT_ID=prj_HWQLgBiGFHNPSy5qJ3WpCeX1l492
            export VERCEL_ORG_ID=team_lY7Vs9wFi3Ifb2A24bOxiA68
            # cspell:enable

            vercel deploy --prebuilt --scope unionbuild
          '';
        };
      };
    };
}

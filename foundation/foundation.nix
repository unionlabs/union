_: {
  perSystem =
    {
      lib,
      pkgs,
      pkgsUnstable,
      ensureAtRepositoryRoot,
      ...
    }:
    let
      deps = with pkgsUnstable; [
        pkg-config
        python3
        nodePackages_latest.nodejs
      ];
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        foundation = pkgsUnstable.buildNpmPackage {
          npmDepsHash = "sha256-TlNud4Sg1luWDcgxmudXuq8D3ir8ea5otDrnTIgjDUU=";
          src = ./.;
          sourceRoot = "foundation";
          npmFlags = [ "--legacy-peer-deps" ];
          pname = packageJSON.name;
          inherit (packageJSON) version;
          nativeBuildInputs = deps;
          buildInputs = deps;
          installPhase = ''
            mkdir -p $out
            cp -r ./build/* $out
          '';
          doDist = false;
          NODE_OPTIONS = "--no-warnings";
        };
      };

      apps = {
        foundation-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "foundation-dev-server";
            runtimeInputs = deps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd foundation/

              export NODE_OPTIONS="--no-warnings"

              npm install
              npm run dev -- --host
            '';
          };
        };
      };
    };
}

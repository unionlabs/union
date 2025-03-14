_: {
  perSystem =
    {
      jsPkgs,
      ensureAtRepositoryRoot,
      lib,
      ...
    }:
    let
      packageJson = lib.importJSON ./package.json;
    in
    {
      packages = {
        ts-sdk = jsPkgs.buildNpmPackage {
          pname = packageJson.name;
          version = packageJson.version;

          src = ./.;

          npmDepsHash = "sha256-dFrd5jFaLjE3w9gXUdCx94/Nb3fi5RdxVKwGP6RdKAE=";
        };

      };
      apps = {};
    };
}

_: {
  perSystem =
    {
      jsPkgs,
      ensureAtRepositoryRoot,
      ...
    }:
    {
      packages = {
        ts-sdk = jsPkgs.buildNpmPackage {
          pname = "@unionlabs/sdk";
          version = "1.0.0";

          src = ./.;

          npmDepsHash = "sha256-dFrd5jFaLjE3w9gXUdCx94/Nb3fi5RdxVKwGP6RdKAE=";
        };

      };
      apps = {};
    };
}

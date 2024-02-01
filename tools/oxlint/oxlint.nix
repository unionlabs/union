{ ... }: {
  perSystem = { pkgs, crane, ... }: {
    _module.args.oxlint = crane.lib.buildPackage rec {
      name = "oxlint";
      version = "0.1.2";
      src = pkgs.fetchFromGitHub {
        owner = "web-infra-dev";
        repo = "oxc";
        rev = "oxlint_v${version}";
        hash = "sha256-XQDkNfgqjfUSDwC3JgdzCqYT4O14UWGImpk5gVyQKfE=";
      };
      buildInputs = [
        pkgs.jemalloc
      ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
        pkgs.darwin.apple_sdk.frameworks.Security
      ];

      cargoBuildFlags = [ "--bin=oxlint" ];
      cargoTestFlags = cargoBuildFlags;

      meta.mainProgram = "oxlint";
    };
  };
}

{ ... }: {
  perSystem = { pkgs, crane, ... }: {
    _module.args.oxlint = crane.lib.buildPackage rec {
      name = "oxlint";
      version = "0.2.6";
      src = pkgs.fetchFromGitHub {
        owner = "web-infra-dev";
        repo = "oxc";
        rev = "oxlint_v${version}";
        hash = "sha256-yjF8/oJ4GsCLk2qt1ssJ9KVMO18XsM70EnS6LMZQY3I=";
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

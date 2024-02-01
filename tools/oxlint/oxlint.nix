{ inputs, ... }: {
  perSystem = { pkgs, crane, ... }: {
    _module.args.oxlint = crane.lib.buildPackage rec {
      pname = "oxlint";
      version = "0.2.6";
      src = inputs.oxlint;
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

{ ... }: {
  perSystem = { pkgs, self', inputs', ... }:
    let
      arion = inputs'.arion.packages.default;

      evm-services = {
        geth = import ./services/geth-sepolia.nix {
          inherit pkgs;
          config = self'.packages.testnet-evm-config;
        };
        lodestar = import ./services/lodestar-sepolia.nix {
          inherit pkgs;
          config = self'.packages.testnet-evm-config;
        };
      };

      testnet-evm = {
        project.name = "union-testnet-evm";
        services = evm-services;
      };

      spec-evm = {
        modules = [ testnet-evm ];
      };

      build-evm = arion.build spec-evm;
    in
    {
      packages.testnet-evm =
        pkgs.writeShellApplication {
          name = "union-testnet-evm";
          runtimeInputs = [ arion ];
          text = ''
            arion --prebuilt-file ${build-evm} up --build --force-recreate -V --always-recreate-deps --remove-orphans
          '';
        };
      overlayAttrs = {
        networks = {
          inherit testnet-evm;
        };
      };
      _module.args.networks = {
        inherit testnet-evm;
      };
    };
}

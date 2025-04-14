_: {
  perSystem =
    {
      self',
      pkgs,
      ensureAtRepositoryRoot,
      ...
    }:
    {
      packages = {
        update-deployments-json = pkgs.writeShellApplication {
          name = "update-deployments-json";
          text = ''
            ${ensureAtRepositoryRoot}

            echo "updating evm deployments..."
            ${pkgs.lib.getExe self'.packages.evm-scripts.update-deployments-json}
            echo "updated evm deployments"

            echo "updating cosmwasm deployments..."
            ${pkgs.lib.getExe self'.packages.cosmwasm-scripts.update-deployments-json}
            echo "updated cosmwasm deployments"
          '';
        };
      };
    };
}

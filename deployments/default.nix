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
        update-clients-json = pkgs.writeShellApplication {
          name = "update-clients-json";
          runtimeInputs = [
            self'.packages.voyager
            pkgs.moreutils
            pkgs.jq
          ];
          text = ''
            ${ensureAtRepositoryRoot}

            chain="''${1:?missing chain id}";
            readarray -d . -t raw_chain_id_arr <<< "$chain"
            raw_chain_id=$(echo "''${raw_chain_id_arr[1]}" | tr -d '[:space:]')
            echo "$raw_chain_id"

            jq '.[$chain] | keys[]' ./deployments/clients.json --arg chain "$chain" -r | \
            while read -r client_id; do 
              jq '.[$chain][$client_id]' --arg chain "$chain" --arg client_id "$client_id" ./deployments/clients.json

              voyager rpc -r voy.run client-info "$raw_chain_id" "$client_id" 
            done
          '';
        };
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

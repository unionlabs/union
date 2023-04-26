{ ... }: {
  perSystem = { devnetConfig, pkgs, self', inputs', ... }: {
    packages.devnet =
      let
        arion = inputs'.arion.packages.default;
        uniond = pkgs.lib.getExe self'.packages.uniond;

        getNodeID = nodeFile:
          pkgs.runCommand "get-node-id" { } ''
            ${uniond} init testnet bn254 --home .
            cp ${self'.packages.devnet-validator-node-ids}/${nodeFile} ./config/node_key.json
            NODE_ID=$(${uniond} tendermint show-node-id --home .)
            echo -n $NODE_ID > $out
          '';

        mkUniondService = { id }:
          let
            seedNode = builtins.readFile (getNodeID "valnode-0.json");
            # All nodes connects to node 0
            params = if id == 0 then "" else "--p2p.persistent_peers ${seedNode}@uniond-0:26656";
          in
          {
            image.enableRecommendedContents = true;
            image.contents = [ pkgs.coreutils self'.packages.devnet-genesis self'.packages.uniond self'.packages.devnet-validator-keys self'.packages.devnet-validator-node-ids ];
            service.networks = [ "union-devnet" ];
            service.command = [
              "sh"
              "-c"
              ''
                cp -R ${self'.packages.devnet-genesis} .
                cp ${self'.packages.devnet-validator-keys}/valkey-${toString id}.json ./config/priv_validator_key.json
                cp ${self'.packages.devnet-validator-node-ids}/valnode-${toString id}.json ./config/node_key.json
                echo ${params}
                ${uniond} start --home . ${params}
              ''
            ];
            service.ports = [
              # TODO: map ports used to interact with the network on the first node?
            ];
            service.stop_signal = "SIGINT";
          };

        spec = {
          modules = [{
            project.name = "union-devnet";
            networks.union-devnet = { };
            services = builtins.listToAttrs (builtins.genList
              (i: {
                name = "uniond-${toString i}";
                value = mkUniondService {
                  id = i;
                };
              }
              )
              devnetConfig.validatorCount);
          }];
        };
        build = arion.build spec;
      in
      pkgs.writeShellApplication {
        name = "union-devnet";
        runtimeInputs = [ arion ];
        text = ''
          arion --prebuilt-file ${build} up --build --force-recreate -V --always-recreate-deps --remove-orphans
        '';
      };
  };
}

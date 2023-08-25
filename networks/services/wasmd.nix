{ pkgs, wasmd, devnet-genesis, devnet-validator-keys, devnet-validator-node-ids, id }:
let
  getNodeID = nodeFile:
    pkgs.runCommand "get-node-id" { buildInputs = [ pkgs.musl ]; } ''
      ${wasmd}/bin/wasmd init testnet --home .
      cp ${devnet-validator-node-ids}/${nodeFile} ./config/node_key.json
      NODE_ID=$(${wasmd}/bin/wasmd tendermint show-node-id --home .)
      echo -n $NODE_ID > $out
    '';

  seedNode = builtins.readFile (getNodeID "valnode-0.json");
  # All nodes connect to node 0
  params = if id == 0 then "" else "--p2p.persistent_peers ${seedNode}@wasmd-0:26656";
in
{
  image = {
    enableRecommendedContents = true;
    contents = [
      pkgs.coreutils
      devnet-genesis
      wasmd
      devnet-validator-keys
      devnet-validator-node-ids
    ];
  };
  service = {
    tty = true;
    stop_signal = "SIGINT";
    ports = [
      # CometBLS JSONRPC 26657
      "${toString (25657 + id)}:26657"
      # Cosmos SDK GRPC 9090
      "${toString (9190 + id)}:9090"
      # Cosmos SDK REST 1317
      "${toString (1417 + id)}:1317"
    ];
    command = [
      "sh"
      "-c"
      ''
        cp -R ${devnet-genesis} .
        cp ${devnet-validator-keys}/valkey-${toString id}.json ./config/priv_validator_key.json
        cp ${devnet-validator-node-ids}/valnode-${toString id}.json ./config/node_key.json
        echo ${params}
        ${wasmd}/bin/wasmd start --home . ${params} --rpc.laddr tcp://0.0.0.0:26657 --api.address tcp://0.0.0.0:1317 --grpc.address 0.0.0.0:9090
      ''
    ];
    healthcheck = {
      interval = "5s";
      retries = 8;
      test = [
        "CMD-SHELL"
        ''
          curl http://127.0.0.1:26657/block?height=2 --fail || exit 1
        ''
      ];
    };
  };
}


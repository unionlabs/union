{ pkgs
, uniond
, devnet-genesis
, devnet-priv-validator-keys
, devnet-validator-node-keys
, devnet-validator-node-ids
, id
, mkNodeId
, dbg
}:
{
  image = {
    enableRecommendedContents = true;
    contents = [
      pkgs.coreutils
      devnet-genesis
      uniond
      devnet-priv-validator-keys
      devnet-validator-node-ids
      devnet-validator-node-keys
    ];
  };
  service = {
    tty = true;
    stop_signal = "SIGINT";
    ports = [
      # CometBLS JSONRPC 26657
      "${toString (26657 + id)}:26657"
      # Cosmos SDK GRPC 9090
      "${toString (9090 + id)}:9090"
      # Cosmos SDK REST 1317
      "${toString (1317 + id)}:1317"
    ];
    command = [
      "sh"
      "-c"
      ''
        mkdir home

        cp --no-preserve=mode -RL ${devnet-genesis}/* home
        cp --no-preserve=mode -L ${devnet-priv-validator-keys}/priv_validator_key_${toString id}.json home/config/priv_validator_key.json
        cp --no-preserve=mode -L ${devnet-validator-node-keys}/node-key-${toString id}.json home/config/node_key.json

        # chmod -R 777 home

        # cat home/config/genesis.json

        ls -al home

        # cat ${mkNodeId 0}
        # All nodes connect to node 0
        params="${if id == 0 then "" else "--p2p.persistent_peers $(cat ${mkNodeId 0})@uniond-0:26656"}"

        echo "$${params}"

        mkdir ./tmp
        export TMPDIR=./tmp

        ${uniond}/bin/uniond comet show-node-id --home home

        ${uniond}/bin/uniond \
          start \
          --home home \
          $$params \
          --rpc.laddr tcp://0.0.0.0:26657 \
          --api.enable true \
          --rpc.unsafe \
          --api.address tcp://0.0.0.0:1317 \
          --grpc.address 0.0.0.0:9090
      ''
    ];
    healthcheck = {
      interval = "5s";
      start_period = "20s";
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


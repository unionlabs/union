  {pkgs, config, self', ... }: 
  let 
    prysm = self'.packages.prysm;

    beacon-init = 
      pkgs.writeShellApplication {
        name = "beacon-init";
        runtimeInputs = [];
        text = ''
          DATADIR=/beaconchain
          mkdir "$DATADIR"
        
          ${prysm}/bin/prysmctl testnet generate-genesis \
            --fork capella \
            --num-validators 64 \
            --chain-config-file /${config}/beacon-config.yml \
            --geth-genesis-json-in /${config}/genesis.json \
            --geth-genesis-json-out "$DATADIR/genesis.json" \
            --output-ssz "$DATADIR/genesis.ssz" \
            --genesis-time 1710464327 \
            --config-name minimal

          ${prysm}/bin/beacon-chain \
            --datadir "$DATADIR" \
            --min-sync-peers 0 \
            --genesis-state "$DATADIR/genesis.ssz" \
            --bootstrap-node= \
            --interop-eth1data-votes \
            --chain-config-file ${config}/beacon-config.yml \
            --contract-deployment-block 0 \
            --chain-id 32382 \
            --accept-terms-of-use \
            --jwt-secret ${config}/dev-jwt.prv \
            --suggested-fee-recipient 0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD \
            --minimum-peers-per-subnet 0 \
            --enable-debug-rpc-endpoints \
            --execution-endpoint http://geth:8551 \
            --grpc-gateway-host 0.0.0.0 \
            --rpc-host 0.0.0.0
            # --enable-lightclient
        '';
      };
  in
  {
  image = {
    enableRecommendedContents = true;
    contents = [
      pkgs.coreutils
      pkgs.curl
      pkgs.jq
      beacon-init
    ];
  };
  service = {
    tty = true;
    stop_signal = "SIGINT";
    restart = "always";
    ports = [
      # Beacon node rest API
      "3500:3500"
      "4000:4000"
      "12000:12000"
      "13000:13000"
    ];
    command = [ "${beacon-init}/bin/beacon-init" ];
    depends_on = {
      geth = {
        condition = "service_healthy";
      };
    };
  };
  
}

{ pkgs, config, ... }:
let
  geth-init =
    pkgs.writeShellApplication {
      name = "geth-init";
      runtimeInputs = [ pkgs.go-ethereum config ];
      text = ''
        ETH_DATADIR=/geth

        geth init --datadir "$ETH_DATADIR" /${config}/genesis.json
        geth account import --datadir "$ETH_DATADIR" \
          --password /dev/null ${config}/dev-key0.prv
        geth account import --datadir "$ETH_DATADIR" \
          --password /dev/null ${config}/dev-key1.prv

        geth \
          --vmdebug \
          --verbosity=4 \
          --http \
          --http.api=eth,debug,net,web3,admin,engine,txpool \
          --http.addr=0.0.0.0 \
          --http.vhosts="*" \
          --ws \
          --ws.api=eth,debug,net,web3,admin,engine,txpool \
          --ws.addr=0.0.0.0 \
          --authrpc.vhosts="*" \
          --authrpc.addr=0.0.0.0 \
          --authrpc.jwtsecret=${config}/dev-jwt.prv \
          --datadir=$ETH_DATADIR \
          --allow-insecure-unlock \
          --unlock=0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD \
          --password=/dev/null \
          --nodiscover \
          --syncmode=full \
          --gcmode=archive \
          --nat=none
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
      geth-init
    ];
  };
  service = {
    tty = true;
    stop_signal = "SIGINT";
    ports = [
      # Rest HTTP
      "8545:8545"
      # WS RPC
      "8546:8546"
      # Auth RPC
      "8551:8551"
    ];
    command = [ "${geth-init}/bin/geth-init" ];
    healthcheck = {
      interval = "5s";
      retries = 3;
      test = [
        "CMD-SHELL"
        ''
          curl http://geth:8545 \
            -X POST \
            -H 'Content-Type: application/json' \
            -d '{"jsonrpc": "2.0", "id": "1", "method": "eth_getBlockByNumber","params": ["0x0", false]}' | jq -r '.result.hash' || exit 1
        ''
      ];
    };
  };
}

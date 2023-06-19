{ pkgs, config }:
let
  geth-init =
    pkgs.writeShellApplication {
      name = "geth-init";
      runtimeInputs = [ pkgs.go-ethereum config ];
      text = ''
        geth \
          --sepolia \
          --networkid 11155111 \
          --datadir /data/geth \
          --authrpc.jwtsecret ${config}/dev-jwt.prv \
          --http \
          --http.corsdomain "*" \
          --http.addr 0.0.0.0 \
          --authrpc.vhosts "*" \
          --authrpc.addr 0.0.0.0 \
          --authrpc.port 8551
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
    stop_signal = "SIGINT";
    ports = [
      # Rest HTTP
      "8545:8545"
      # Auth RPC
      "8551:8551"
    ];
    command = [ "${geth-init}/bin/geth-init" ];
    healthcheck = {
      interval = "5s";
      retries = 2;
      test = [
        "CMD-SHELL"
        ''
          curl http://127.0.0.1:8545 \
            -X POST \
            -H 'Content-Type: application/json' \
            -d '{"jsonrpc": "2.0", "id": "1", "method": "eth_getBlockByNumber","params": ["0x0", false]}' | jq -r '.result.hash' || exit 1
        ''
      ];
    };
  };
}

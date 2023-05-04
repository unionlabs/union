{ pkgs, geth-init }: {
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
    networks = [ "union-devnet" ];
    ports = [
      # Rest HTTP
      "8545:8545"
      # Auth RPC
      "8551:8551"
    ];
    command = [ "sh" "-c" "${geth-init}/bin/geth-init" ];
    healthcheck = {
      start_period = "5s";
      interval = "10s";
      retries = 4;
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

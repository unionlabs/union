{ pkgs, config, validatorCount }:
let
  lodestar-init = pkgs.writeTextFile {
    name = "lodestar-init";
    text = ''
      #!/usr/bin/env sh

      ETH_ENDPOINT=http://geth:8545
      EXECUTION_ENDPOINT=http://geth:8551
      curl "$ETH_ENDPOINT" \
                        -X POST \
                        -H 'Content-Type: application/json' \
                        -d '{"jsonrpc": "2.0", "id": "1", "method": "eth_getBlockByNumber","params": ["0x0", false]}'
      ETH_GENESIS_HASH=$(curl "$ETH_ENDPOINT" \
        -X POST \
        -H 'Content-Type: application/json' \
        -d '{"jsonrpc": "2.0", "id": "1", "method": "eth_getBlockByNumber","params": ["0x0", false]}' | jq -r '.result.hash')
      GENESIS_TIMESTAMP=$(date +%s)
      node ./packages/cli/bin/lodestar dev \
        --genesisTime="$GENESIS_TIMESTAMP" \
        --genesisEth1Hash="$ETH_GENESIS_HASH" \
        --genesisValidators=${toString validatorCount} \
        --startValidators="0..${toString (validatorCount - 1)}" \
        --rest.address="0.0.0.0" \
        --rest.cors="*" \
        --eth1.providerUrls="$ETH_ENDPOINT" \
        --execution.urls="$EXECUTION_ENDPOINT" \
        --reset \
        --terminal-total-difficulty-override=0 \
        --params.ALTAIR_FORK_EPOCH=0 \
        --params.BELLATRIX_FORK_EPOCH=0 \
        --params.CAPELLA_FORK_EPOCH=0 \
        --eth1=true \
        --jwt-secret=/dev-jwt.prv \
        --rest.namespace="*"
    '';
    executable = true;
    destination = "/lodestar-init";
  };

  context =
    let
      files = pkgs.symlinkJoin {
        name = "docker-context";
        paths = [
          lodestar-init
          config
          (pkgs.writeTextFile {
            name = "Dockerfile";
            text = ''
              FROM chainsafe/lodestar:v1.8.0
              RUN apk update && apk add jq curl
              COPY lodestar-init /bin/lodestar-init
              COPY dev-jwt.prv /dev-jwt.prv
            '';
            destination = "/Dockerfile";
          })
        ];
      };
    in
    pkgs.stdenv.mkDerivation {
      name = "context";
      phases = [ "installPhase" ];
      installPhase = ''
        mkdir $out
        cp -rL ${files}/* $out
      '';
    };
in
{
  service = {
    build = { context = "${context}"; };
    stop_signal = "SIGINT";
    networks = [ "union-devnet" ];
    ports = [
      # Beacon node rest API
      "9596:9596"
    ];
    entrypoint = "lodestar-init";
    depends_on = {
      geth = {
        condition = "service_healthy";
      };
    };
  };
}

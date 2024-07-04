{ pkgs, config, validatorCount }:
let
  lodestar-init = pkgs.writeShellApplication {
    name = "lodestar-init";
    runtimeInputs = [ pkgs.coreutils pkgs.curl pkgs.jq config ];
    text = ''
      ETH_ENDPOINT=http://geth:8545
      EXECUTION_ENDPOINT=http://geth:8551

      ETH_GENESIS_HASH=$(curl "$ETH_ENDPOINT" \
        -X POST \
        -H 'Content-Type: application/json' \
        -d '{"jsonrpc": "2.0", "id": "1", "method": "eth_getBlockByNumber","params": ["0x0", false]}' | jq -r '.result.hash')

      echo "ETH_GENESIS_HASH = $ETH_GENESIS_HASH"

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
        --params.DENEB_FORK_EPOCH=0 \
        --eth1=true \
        --jwt-secret=${config}/dev-jwt.prv \
        --rest.namespace="*"
    '';
  };
in
{
  build = {
    image = pkgs.lib.mkForce (pkgs.dockerTools.streamLayeredImage {
      name = "lodestar-extended";
      fromImage = pkgs.dockerTools.pullImage ({
        imageName = "chainsafe/lodestar";
        finalImageName = "chainsafe/lodestar";
        finalImageTag = "v1.15.1";
      } // (if pkgs.stdenv.isx86_64 then {
        imageDigest = "sha256:02adf60640dddd8f1bbab9eda09563d85aa675414af57a47a2234a1a40bde2e3";
        sha256 = "sha256-iq9Jukk2lzIdXj3PgyxxgXLvikgAV35NaDU1siXqSNQ=";
        arch = "amd64";
      } else {
        imageDigest = "sha256:1c6c3f043bfda4cb7e8d8423a46a30594561659e796fe975b3fbff2337f1ce24";
        sha256 = "sha256-iRh/HOEgtqV6JOqDY9CEzvkwBEcsjCrneqQlob8M48o=";
        arch = "arm64";
      }));
      contents = [
        pkgs.coreutils
        pkgs.curl
        pkgs.jq
        lodestar-init
      ];
      config = {
        # dockerTools only preserves Env from the base "fromImage"
        # this is directly coming from https://github.com/ChainSafe/lodestar/blob/402c46f0d9f1f964066efb3e0e53863d6a913a80/Dockerfile#L39
        WorkingDir = "/usr/app";
        Entrypoint = pkgs.lib.meta.getExe lodestar-init;
      };
    });
  };
  service = {
    tty = true;
    stop_signal = "SIGINT";
    ports = [
      # Beacon node rest API
      "9596:9596"
    ];
    healthcheck = {
      interval = "5s";
      retries = 6;
      test = [
        "CMD-SHELL"
        ''
          curl -f http://localhost:9596/eth/v2/beacon/blocks/2 || exit 1
        ''
      ];
    };
    depends_on = {
      geth = {
        condition = "service_healthy";
      };
    };
  };
}

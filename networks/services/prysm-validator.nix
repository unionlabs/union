{ pkgs, prysm-validator, config, ... }:
let
  prysm-validator-init = pkgs.writeShellApplication {
    name = "prysm-validator-init";
    runtimeInputs = [
      prysm-validator
      config
    ];
    text = ''
      ${prysm-validator}/bin/validator \
      --beacon-rpc-provider=prysm-beacon:4000 \
      --datadir=./validatordata \
      --accept-terms-of-use \
      --interop-num-validators=64 \
      --interop-start-index=0 \
      --chain-config-file=${config}/beacon-config.yml
    '';
  };
in
{
  image = {
    enableRecommendedContents = true;
    contents = [ prysm-validator-init ];
  };
  service = {
    stop_signal = "SIGINT";
    networks = [ "union-devnet" ];
    command = [ "${prysm-validator-init}/bin/prysm-validator-init" ];
    depends_on = {
      geth = {
        condition = "service_healthy";
      };
    };
  };
}

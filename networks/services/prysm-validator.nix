{ pkgs, prysm-validator, config, validatorCount, ... }:
let
  prysm-validator-init = pkgs.writeShellApplication {
    name = "prysm-validator-init";
    runtimeInputs = [
      prysm-validator
      config
    ];
    text = ''
      ${pkgs.lib.getExe prysm-validator} \
      --beacon-rpc-provider=prysm-beacon:4000 \
      --datadir=./validatordata \
      --accept-terms-of-use \
      --interop-num-validators=${toString validatorCount} \
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
    command = [ (pkgs.lib.getExe prysm-validator-init) ];
    depends_on = {
      prysm-beacon = {
        condition = "service_started";
      };
    };
  };
}

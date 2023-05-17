{ pkgs, prysmctl, validatorCount, config, sharedVolume, ... }:
let
  prysm-bootstrap = pkgs.writeShellApplication {
    name = "prysm-bootstrap";
    runtimeInputs = [
      prysmctl
      config
    ];
    text = ''
      ${pkgs.lib.getExe prysmctl} testnet generate-genesis \
        --fork=bellatrix \
        --num-validators=${toString validatorCount} \
        --chain-config-file=${config}/beacon-config.yml \
        --geth-genesis-json-in=${config}/genesis.json \
        --geth-genesis-json-out=/${sharedVolume}/genesis.json \
        --output-ssz=/${sharedVolume}/genesis.ssz
    '';
  };
in
{
  image = {
    enableRecommendedContents = true;
    contents = [ prysm-bootstrap ];
  };
  service = {
    stop_signal = "SIGINT";
    networks = [ "union-devnet" ];
    command = [ (pkgs.lib.getExe prysm-bootstrap) ];
    volumes = [
      "${sharedVolume}:/${sharedVolume}"
    ];
  };
}

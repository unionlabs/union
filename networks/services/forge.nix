{ pkgs, forge, evm-sources, ... }:
let
  forge-deploy =
    pkgs.writeShellApplication {
      name = "forge-deploy";
      runtimeInputs = [ forge ];
      text = ''
        mkdir -p /evm
        cd /evm
        cp --no-preserve=mode -r ${evm-sources}/* .
        PRIVATE_KEY=0x${builtins.readFile ./../genesis/devnet-eth/dev-key0.prv} FOUNDRY_PROFILE="script" forge script scripts/Deploy.s.sol:DeployDeployerAndIBC -vvv --rpc-url http://geth:8545 --broadcast --verify --verifier blockscout --verifier-url http://blockscout-proxy/api
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
      forge-deploy
      evm-sources
    ];
  };
  service = {
    tty = true;
    stop_signal = "SIGINT";
    ports = [ ];
    command = [ (pkgs.lib.getExe forge-deploy) ];
    depends_on = {
      lodestar = {
        condition = "service_healthy";
      };
    };
  };
}

{ pkgs, forge, evm-sources, ... }:
let
  forge-deploy =
    pkgs.writeShellApplication {
      name = "forge-deploy";
      runtimeInputs = [ forge ];
      text = ''
        PRIVATE_KEY=0x${builtins.readFile ./../genesis/devnet-eth/dev-key0.prv} FOUNDRY_PROFILE="script" forge script -vvv ${evm-sources}/scripts/Deploy.s.sol:DeployDeployerAndIBC -vvv --rpc-url http://geth:8545 --broadcast
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
    ];
  };
  service = {
    tty = true;
    stop_signal = "SIGINT";
    ports = [ ];
    command = [ "${forge-deploy}/bin/forge-deploy" ];
    depends_on = {
      lodestar = {
        condition = "service_healthy";
      };
    };
  };
}

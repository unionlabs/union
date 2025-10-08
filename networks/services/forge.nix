{
  pkgs,
  forge,
  evm-sources,
  evm-contracts,
  ...
}:
let
  forge-deploy = pkgs.writeShellApplication {
    name = "forge-deploy";
    runtimeInputs = [ forge ];
    text = ''
      mkdir -p /evm
      cd /evm
      cp --no-preserve=mode -r ${evm-contracts}/* .
      cp --no-preserve=mode -r ${evm-sources}/* .
      WETH_ADDRESS="0x0000000000000000000000000000000000000000" \
      DEPLOYER="0x0000000000000000000000000000000000000000" \
      SENDER="0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD" \
        RATE_LIMIT_ENABLED="false" \
        PRIVATE_KEY=0x${builtins.readFile ./../genesis/devnet-eth/dev-key0.prv} \
        FOUNDRY_PROFILE="script" \
        BYPASS_GITREV=1 \
        NATIVE_TOKEN_NAME=Ether \
        NATIVE_TOKEN_DECIMALS=18 \
        NATIVE_TOKEN_SYMBOL=ETH \
        forge script scripts/Deploy.s.sol:DeployDeployerAndIBC -vvv --rpc-url http://geth:8545 --broadcast ${
          pkgs.lib.optionalString (
            pkgs.stdenv.isx86_64 && (builtins.getEnv "NO_BLOCKSCOUT" == "")
          ) "--verify --verifier blockscout --verifier-url http://blockscout-proxy/api"
        }
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

{
  e2e,
  pkgs,
  unionvisor,
  bundle,
  ...
}:
let
  unionvisorBin = pkgs.lib.meta.getExe unionvisor;

  mkUpgradeProposal =
    version: height: denom:
    pkgs.runCommand "upgrade-proposal" { } ''
      mkdir -p $out
      echo '{
       "messages": [
        {
         "@type": "/cosmos.upgrade.v1beta1.MsgSoftwareUpgrade",
         "authority": "union10d07y265gmmuvt4z0w9aw880jnsr700js4jdcz",
         "plan": {
          "name": "${version}",
          "height": "${toString height}",
          "info": "${version}"
         }
        }
       ],
       "deposit": "10000000000000000000${denom}",
       "title": "${version}",
       "summary": "Upgrade to ${version}"
      }' > proposal-${version}.json
      mv proposal-${version}.json $out
    '';

  forEachNode = f: ''
    ${f "0"}
    ${f "1"}
    ${f "2"}
    ${f "3"}
  '';

  upgradeTo = version: height: denom: gasPrice: ''
    union.succeed('docker cp ${
      mkUpgradeProposal version height denom
    }/proposal-${version}.json union-v1-union-v1-0-1:/proposal-${version}.json')

    print(union.succeed('docker exec union-v1-union-v1-0-1 ${unionvisorBin} --root ./.unionvisor call --bundle ${bundle} -- tx gov submit-proposal proposal-${version}.json --from alice --keyring-backend test -y --gas auto --gas-adjustment 1.4 --gas-prices ${gasPrice}${denom}'))
    time.sleep(1)
    print(union.succeed("docker exec union-v1-union-v1-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query tx CE774F5DE3E73743C353A4823FE62A93F3DCAF05A37A4F30C9340941EB178933"))

    print(union.succeed("docker exec union-v1-union-v1-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query gov proposal ${toString (height / 10)} --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.proposal.status == \"PROPOSAL_STATUS_VOTING_PERIOD\"'"))
    print(union.succeed("docker exec union-v1-union-v1-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query gov proposals --output json"))
    union.wait_until_succeeds("[[ $(docker exec union-v1-union-v1-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query gov proposal ${toString (height / 10)} --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.proposal.status == \"PROPOSAL_STATUS_VOTING_PERIOD\"') == true ]]", timeout=30)

    ${forEachNode (
      id:
      "print(union.succeed('docker exec union-v1-union-v1-${id}-1 ${unionvisorBin} --root ./.unionvisor call --bundle ${bundle} -- tx gov vote ${toString (height / 10)} yes --keyring-backend test --from valoper-${id} -y --gas auto --gas-adjustment 1.8 --gas-prices ${gasPrice}${denom}'))"
    )}

    union.wait_until_succeeds("[[ $(docker exec union-v1-union-v1-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query gov proposal ${toString (height / 10)} --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.proposal.status == \"PROPOSAL_STATUS_PASSED\"') == true ]]", timeout=60)

    union.wait_until_succeeds('[[ $(curl "http://localhost:26660/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > ${toString height}") == "true" ]]', timeout=120)
  '';
in
{
  upgrade-from-genesis = e2e.mkTest {
    name = "upgrade-from-genesis";

    testScript = ''
      import time
      union.wait_for_open_port(${toString e2e.unionNode.wait_for_open_port})

      # Ensure the union network commits more than one block
      union.wait_until_succeeds('[[ $(curl "http://localhost:26657/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > 1") == "true" ]]')

      ${upgradeTo "v1.1.0" 10 "muno" "1"}
      ${upgradeTo "v1.2.0" 20 "muno" "1"}

      print(union.succeed('docker exec union-v1-union-v1-0-1 ${unionvisorBin} --root ./.unionvisor call --bundle ${bundle} -- tx bank multi-send alice union1qp4uzhet2sd9mrs46kemse5dt9ncz4k3hjst5m union1d348dktd9nz0y6afzh3az5j39qahc93cmwkdjf union1asxs295fuy7jph8p8eqtc2r8zxggdc204s7unx union1fktal7292h36h7glff5edq59vpdfn7504duw5m 15000000000000au --keyring-backend test -y --gas auto --gas-adjustment 1.4 --gas-prices 1au'))
      time.sleep(6)

      ${upgradeTo "v1.3.0" 30 "au" "1"}
      print(union.succeed("docker exec union-v1-union-v1-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query staking validators"))
    '';

    nodes = {
      union = e2e.unionTestnetGenesisNode.node;
    };
  };
}

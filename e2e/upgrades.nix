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
    version: height:
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
       "deposit": "15000000au",
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

  upgradeTo = version: height: ''
    union.succeed('docker cp ${mkUpgradeProposal version height}/proposal-${version}.json devnet-union-minimal-union-minimal-0-1:/proposal-${version}.json')

    print(union.succeed('docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} --root ./.unionvisor call --bundle ${bundle} -- tx gov submit-proposal proposal-${version}.json --from valoper-0 --keyring-backend test -y --gas auto --gas-adjustment 1.4 --gas-prices 1au'))

    print(union.succeed("docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query gov proposal ${toString (height / 10)} --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.proposal.status == \"PROPOSAL_STATUS_VOTING_PERIOD\"'"))
    union.wait_until_succeeds("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query gov proposal ${toString (height / 10)} --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.proposal.status == \"PROPOSAL_STATUS_VOTING_PERIOD\"') == true ]]", timeout=30)

    ${forEachNode (
      id:
      "print(union.succeed('docker exec devnet-union-minimal-union-minimal-${id}-1 ${unionvisorBin} --root ./.unionvisor call --bundle ${bundle} -- tx gov vote ${toString (height / 10)} yes --keyring-backend test --from valoper-${id} -y --gas auto --gas-adjustment 1.8 --gas-prices 1au'))"
    )}

    union.wait_until_succeeds("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query gov proposal ${toString (height / 10)} --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.proposal.status == \"PROPOSAL_STATUS_PASSED\"') == true ]]", timeout=60)

    union.wait_until_succeeds('[[ $(curl "http://localhost:26660/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > ${toString height}") == "true" ]]', timeout=120)
  '';
in
{
  upgrade-from-genesis = e2e.mkTest {
    name = "upgrade-from-genesis";

    testScript = ''
      union.wait_for_open_port(${toString e2e.unionNode.wait_for_open_port})

      # Ensure the union network commits more than one block
      union.wait_until_succeeds('[[ $(curl "http://localhost:26657/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > 1") == "true" ]]')

      ${upgradeTo "v1.1.0" 10}
      ${upgradeTo "v1.2.0" 20}
      union.wait_until_succeeds('[[ $(curl "http://localhost:26660/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > 40") == "true" ]]', timeout=120)

      # Ensure foundation owns 4 delegations
      union.succeed("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query staking delegations union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2 --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.pagination.total | tonumber == 4') == true ]]")

      # Ensure vals have 0 delegations
      union.succeed("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query staking delegations union1qp4uzhet2sd9mrs46kemse5dt9ncz4k3hjst5m --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.delegation_responses') == null ]]")
      union.succeed("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query staking delegations union1d348dktd9nz0y6afzh3az5j39qahc93cmwkdjf --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.delegation_responses') == null ]]")
      union.succeed("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query staking delegations union1asxs295fuy7jph8p8eqtc2r8zxggdc204s7unx --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.delegation_responses') == null ]]")
      union.succeed("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query staking delegations union1fktal7292h36h7glff5edq59vpdfn7504duw5m --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.delegation_responses') == null ]]")

      # Ensure legacy tokens are burnt
      union.succeed("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query bank denom-owners au --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.denom_owners') == null ]]")
      union.succeed("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query bank denom-owners ugas --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.denom_owners') == null ]]")
      union.succeed("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query bank denom-owners upoa --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.denom_owners') == null ]]")
    '';

    nodes = {
      union = e2e.unionTestnetGenesisNode.node;
    };
  };
}

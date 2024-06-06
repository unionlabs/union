{ e2e, pkgs, unionvisor, bundle, ... }:
let
  unionvisorBin = pkgs.lib.meta.getExe unionvisor;

  mkUpgradeProposal = version: height: pkgs.runCommand "upgrade-proposal" { } ''
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
     "deposit": "15000000muno",
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
   
    print(union.succeed('docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} --root ./.unionvisor call --bundle ${bundle} -- tx gov submit-proposal proposal-${version}.json --from valoper-0 --keyring-backend test -y --gas 3000000000'))

    union.wait_until_succeeds("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query gov proposal ${toString (height / 10)} --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.proposal.status == 2') == true ]]", timeout=30)

    ${forEachNode (id: "print(union.succeed('docker exec devnet-union-minimal-union-minimal-${id}-1 ${unionvisorBin} --root ./.unionvisor call --bundle ${bundle} -- tx gov vote ${toString (height / 10)} yes --keyring-backend test --from valoper-${id} -y'))")}

    union.wait_until_succeeds("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query gov proposal ${toString (height / 10)} --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.proposal.status == 3') == true ]]", timeout=60)

    union.wait_until_succeeds('[[ $(curl "http://localhost:26660/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > ${toString height}") == "true" ]]', timeout=60)
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

      ${upgradeTo "v0.22.0" 10}
      ${upgradeTo "v0.23.0" 20}
    '';

    nodes = {
      union = e2e.unionTestnetGenesisNode.node;
    };
  };

  upgrade-with-tokenfactory-state = e2e.mkTest {
    name = "upgrade-with-tokenfactory-state";

    testScript = ''
      import time
      union.wait_for_open_port(${toString e2e.unionNode.wait_for_open_port})

      # Ensure the union network commits more than one block
      union.wait_until_succeeds('[[ $(curl "http://localhost:26657/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > 1") == "true" ]]')

      print(union.succeed("docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} --root ./.unionvisor call --bundle ${bundle} -- tx tokenfactory create-denom bazinga --from valoper-0 --keyring-backend test -y --gas 3000000000"))

      print(union.succeed("docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query tx 4B2B3BF0F4C7316E60BC04FD278BD1298677D490DC80AC8ED41DACB577B072FF --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.'"))
      print(union.succeed("docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query tokenfactory denom-authority-metadata factory/union1qp4uzhet2sd9mrs46kemse5dt9ncz4k3hjst5m/bazinga --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.'"))
      union.succeed("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query tokenfactory denom-authority-metadata factory/union1qp4uzhet2sd9mrs46kemse5dt9ncz4k3hjst5m/bazinga --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.authority_metadata.admin == \"union1qp4uzhet2sd9mrs46kemse5dt9ncz4k3hjst5m\"') == true ]]")

      ${upgradeTo "v0.22.0" 10}
      union.succeed("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query tokenfactory denom-authority-metadata factory/union1qp4uzhet2sd9mrs46kemse5dt9ncz4k3hjst5m/bazinga --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.authority_metadata.admin == \"union1qp4uzhet2sd9mrs46kemse5dt9ncz4k3hjst5m\"') == true ]]")
      ${upgradeTo "v0.23.0" 20}
      union.succeed("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query tokenfactory denom-authority-metadata factory/union1qp4uzhet2sd9mrs46kemse5dt9ncz4k3hjst5m/bazinga --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.authority_metadata.admin == \"union1qp4uzhet2sd9mrs46kemse5dt9ncz4k3hjst5m\"') == true ]]")
    '';

    nodes = {
      union = e2e.unionTestnetGenesisNode.node;
    };
  };
}

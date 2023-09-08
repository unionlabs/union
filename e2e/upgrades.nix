{ e2e, pkgs, unionvisor, bundle, ... }:
let
  unionvisorbn = pkgs.lib.meta.getExe unionvisor;

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
     "deposit": "15000000stake",
     "title": "${version}",
     "summary": "Upgrade to ${version}"
    }' > proposal-${version}.json
    mv proposal-${version}.json $out
  '';

  upgradeTo = version: height: ''
    union.succeed('docker cp ${mkUpgradeProposal version height}/proposal-${version}.json devnet-minimal-uniond-0-1:/proposal-${version}.json')
    union.succeed('docker exec devnet-minimal-uniond-0-1 ${unionvisorbn} --root . call --bundle ${bundle} -- tx gov submit-proposal proposal-${version}.json --from val-0 --keyring-backend test --home ./home -y')

    union.succeed('docker exec devnet-minimal-uniond-0-1 ${unionvisorbn} --root . call --bundle ${bundle} -- tx gov vote 1 yes --keyring-backend test --from val-0 --home ./home -y')
    union.succeed('docker exec devnet-minimal-uniond-1-1 ${unionvisorbn} --root . call --bundle ${bundle} -- tx gov vote 1 yes --keyring-backend test --from val-1 --home ./home -y')
    union.succeed('docker exec devnet-minimal-uniond-2-1 ${unionvisorbn} --root . call --bundle ${bundle} -- tx gov vote 1 yes --keyring-backend test --from val-2 --home ./home -y')
    union.succeed('docker exec devnet-minimal-uniond-3-1 ${unionvisorbn} --root . call --bundle ${bundle} -- tx gov vote 1 yes --keyring-backend test --from val-3 --home ./home -y')
    union.wait_until_succeeds('[[ $(curl "http://localhost:26660/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > ${toString height}") == "true" ]]')
  '';
in
{
  upgrade-from-genesis = e2e.mkTest {
    name = "upgrade-from-genesis";

    testScript = ''
      union.wait_for_open_port(${toString e2e.unionNode.wait_for_open_port})

      # Ensure the union network commits more than one block
      union.wait_until_succeeds('[[ $(curl "http://localhost:26660/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > 1") == "true" ]]')

      ${upgradeTo "v0.9.0" 10}
      ${upgradeTo "v0.10.0" 20}
    '';

    nodes = {
      union = e2e.unionTestnetGenesisNode.node;
    };
  };
}

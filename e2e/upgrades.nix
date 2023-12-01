{ e2e, pkgs, bundle, unionvisor, ... }:
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
     "deposit": "15000000stake",
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
    union.succeed('docker cp ${mkUpgradeProposal version height}/proposal-${version}.json devnet-minimal-uniond-0-1:/proposal-${version}.json')
    union.succeed('docker exec devnet-minimal-uniond-0-1 ${unionvisorBin} --root . call --bundle /bundle -- tx gov submit-proposal proposal-${version}.json --from val-0 --keyring-backend test --home ./home -y')

    union.shell_interact()

    ${forEachNode (id: "union.succeed('docker exec devnet-minimal-uniond-${id}-1 ${unionvisorBin} --root . call --bundle /bundle -- tx gov vote 1 yes --keyring-backend test --from val-${id} --home ./home -y')")}

    union.wait_until_succeeds('[[ $(curl "http://localhost:26660/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > ${toString height}") == "true" ]]')
  '';
in
{
  upgrade-from-genesis = e2e.mkTest {
    name = "upgrade-from-genesis";

    testScript = ''
      union.wait_for_open_port(${toString e2e.unionNode.wait_for_open_port})

      ${forEachNode (id: "union.succeed('docker cp ${bundle} devnet-minimal-uniond-${id}-1:/bundle')")}

      # Ensure the union network commits more than one block
      union.wait_until_succeeds('[[ $(curl "http://localhost:26660/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > 1") == "true" ]]')

      ${upgradeTo "v0.15.0" 10}
    '';

    nodes = {
      union = e2e.unionTestnetGenesisNode.node;
    };
  };
}

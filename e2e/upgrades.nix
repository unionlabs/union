{ e2e, pkgs, inputs, ... }:
let
  uniond-v0_8_0 = pkgs.lib.meta.getExe inputs.v0_8_0.packages.uniond;
  uniond-v0_9_0 = pkgs.lib.meta.getExe inputs.v0_9_0.packages.uniond;
  uniond-v0_10_0 = pkgs.lib.meta.getExe inputs.v0_10_0.packages.uniond;

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

  upgradeTo = uniondTo: uniondFrom: version: height: ''
    union.succeed('docker cp ${mkUpgradeProposal version height}/proposal-${version}.json devnet-minimal-uniond-0-1:/proposal-${version}.json')
    union.succeed('docker exec devnet-minimal-uniond-0-1 ${uniondFrom} tx gov submit-proposal proposal-${version}.json --from val-0 --keyring-backend test --home . -y')

    union.succeed('docker exec devnet-minimal-uniond-0-1 ${uniondFrom} tx gov vote 1 yes --keyring-backend test --from val-0 --home . -y')
    union.succeed('docker exec devnet-minimal-uniond-1-1 ${uniondFrom} tx gov vote 1 yes --keyring-backend test --from val-1 --home . -y')
    union.succeed('docker exec devnet-minimal-uniond-2-1 ${uniondFrom} tx gov vote 1 yes --keyring-backend test --from val-2 --home . -y')
    union.succeed('docker exec devnet-minimal-uniond-3-1 ${uniondFrom} tx gov vote 1 yes --keyring-backend test --from val-3 --home . -y')
    union.wait_until_succeeds('docker exec devnet-minimal-uniond-3-1 ${uniondFrom} query block ${toString height} --home .')

    print('Union PID:')
    print(union.succeed('docker exec devnet-minimal-uniond-0-1 ps -aux'))
    print(union.succeed('docker exec devnet-minimal-uniond-1-1 ps -aux'))
    print(union.succeed('docker exec devnet-minimal-uniond-2-1 ps -aux'))
    print(union.succeed('docker exec devnet-minimal-uniond-3-1 ps -aux'))

    union.succeed('docker exec devnet-minimal-uniond-0-1 kill 1')
    union.succeed('docker exec devnet-minimal-uniond-1-1 kill 1')
    union.succeed('docker exec devnet-minimal-uniond-2-1 kill 1')
    union.succeed('docker exec devnet-minimal-uniond-3-1 kill 1')

    union.succeed('docker exec devnet-minimal-uniond-0-1 ${uniondTo} start --home . --rpc.laddr tcp://0.0.0.0:26657 --api.address tcp://0.0.0.0:1317 --grpc.address 0.0.0.0:9090')
    union.succeed('docker exec devnet-minimal-uniond-1-1 ${uniondTo} start --home . --rpc.laddr tcp://0.0.0.0:26657 --api.address tcp://0.0.0.0:1317 --grpc.address 0.0.0.0:9090')
    union.succeed('docker exec devnet-minimal-uniond-2-1 ${uniondTo} start --home . --rpc.laddr tcp://0.0.0.0:26657 --api.address tcp://0.0.0.0:1317 --grpc.address 0.0.0.0:9090')
    union.succeed('docker exec devnet-minimal-uniond-3-1 ${uniondTo} start --home . --rpc.laddr tcp://0.0.0.0:26657 --api.address tcp://0.0.0.0:1317 --grpc.address 0.0.0.0:9090')
  '';
in
{
  upgrade-from-genesis = e2e.mkTest {
    name = "upgrade-from-genesis";

    testScript = ''
      union.wait_for_open_port(${toString e2e.unionNode.wait_for_open_port})

      # Ensure the union network commits more than one block
      union.wait_until_succeeds('[[ $(curl "http://localhost:26660/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > 1") == "true" ]]')

      union.wait_until_succeeds('docker exec devnet-minimal-uniond-3-1 ${uniond-v0_8_0} query block')

      ${upgradeTo uniond-v0_9_0 uniond-v0_8_0 "v0.9.0" 5}
    '';

    nodes = {
      union = e2e.unionTestnetGenesisNode.node;
    };
  };
}

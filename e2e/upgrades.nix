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

    time.sleep(6)

    print(union.succeed("docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query gov proposals --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.'"))

    union.wait_until_succeeds("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query gov proposal ${toString (height / 10)} --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.proposal.status == 2') == true ]]", timeout=30)

    ${forEachNode (id: "print(union.succeed('docker exec devnet-union-minimal-union-minimal-${id}-1 ${unionvisorBin} --root ./.unionvisor call --bundle ${bundle} -- tx gov vote ${toString (height / 10)} yes --keyring-backend test --from valoper-${id} -y'))")}

    time.sleep(6)

    print(union.succeed("docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query tx 28CC443642753F1DF792D9D99BF33316EA355A9AE6DA209F0E5D0A6AE7D74762 --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.'"))
    print(union.succeed("docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query tx 6235733D6327CBBAFAAD847B0F51F8B5C192781C05D6A95EB6DEFA148885881A --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.'"))
    print(union.succeed("docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query tx AF68F6BA4163E34EC102ED5599441E4570A1D5618F02566F53C4511336A17470 --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.'"))
    print(union.succeed("docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query tx E8A5C52ED2943BBC32AD8C161C868A7E0D7ED07C29CDF1C4B2F14B39CB60F634 --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.'"))
    print(union.succeed("docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query gov proposal 1 --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.'"))

    union.wait_until_succeeds("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query gov proposal ${toString (height / 10)} --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.proposal.status == 3') == true ]]", timeout=30)

    union.wait_until_succeeds('[[ $(curl "http://localhost:26660/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > ${toString height}") == "true" ]]')
  '';
in
{
  upgrade-from-genesis = e2e.mkTest {
    name = "upgrade-from-genesis";

    testScript = ''
      import time
      union.wait_for_open_port(${toString e2e.unionNode.wait_for_open_port})

      union.wait_for_open_port(${toString e2e.unionNode.wait_for_open_port})

      # Ensure the union network commits more than one block
      union.wait_until_succeeds('[[ $(curl "http://localhost:26657/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > 1") == "true" ]]')

      ${upgradeTo "v0.20.0" 10}
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

      union.succeed("docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} --root ./.unionvisor call --bundle ${bundle} -- tx tokenfactory create-denom bazinga --from valoper-0 --keyring-backend test -y --gas 3000000000")
      time.sleep(6)
      union.succeed("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query tokenfactory denom-authority-metadata factory/union14fldwd959h7glh2e3k45veuqfszvgm69q07jhw/bazinga --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.authority_metadata.admin == \"union14fldwd959h7glh2e3k45veuqfszvgm69q07jhw\"') == true ]]")

      ${upgradeTo "v0.20.0" 10}
      union.succeed("[[ $(docker exec devnet-union-minimal-union-minimal-0-1 ${unionvisorBin} -l off --root ./.unionvisor call --bundle ${bundle} -- query tokenfactory denom-authority-metadata factory/union14fldwd959h7glh2e3k45veuqfszvgm69q07jhw/bazinga --output json | ${pkgs.lib.meta.getExe pkgs.jq} '.authority_metadata.admin == \"union14fldwd959h7glh2e3k45veuqfszvgm69q07jhw\"') == true ]]")
    '';

    nodes = {
      union = e2e.unionTestnetGenesisNode.node;
    };
  };
}

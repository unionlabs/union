{ inputs, ... }:
{
  perSystem =
    {
      pkgs,
      nixpkgs,
      networks,
      mkVoyagerImg,
      self',
      ...
    }:
    let
      mkTest =
        let
          nixos-lib = import "${nixpkgs}/nixos/lib" { };
        in
        {
          name,
          testScript,
          nodes,
        }:
        nixos-lib.runTest {
          inherit name testScript nodes;

          hostPkgs = pkgs;
          passthru = {
            ci = false;
          };
          skipTypeCheck = true;
        };

      devnetEthNode = {
        wait_for_console_text = "Synced - slot: [1-9][0-9]*";
        wait_for_open_port = 8546;
        node =
          { pkgs, ... }:
          {
            imports = [
              inputs.arion.nixosModules.arion
            ];
            virtualisation = {
              diskSize = 16 * 1024;
              memorySize = 8 * 1024;
              arion = {
                backend = "docker";
                projects.devnet-eth.settings = networks.modules.devnet-eth-minimal;
              };
              vlans = [ 1 ];
            };
            networking.hostName = "devnetEth";

            environment.systemPackages = with pkgs; [ jq ];
          };
      };

      mkVoyagerNode = configFilePath: rec {
        wait_for_open_port = 7177;
        readiness_probe = "";
        voyagerConfig = pkgs.runCommand "voyager-config" { } ''
          mkdir $out
          cp ${configFilePath} $out/voyager-config.jsonc
        '';
        node =
          { pkgs, ... }:
          {
            imports = [
              inputs.arion.nixosModules.arion
            ];
            virtualisation = {
              diskSize = 16 * 1024;
              memorySize = 8 * 1024;
              arion = {
                backend = "docker";
                projects.voyager.settings = mkVoyagerImg voyagerConfig;
              };
              vlans = [ 1 ];
            };
            environment.systemPackages = with pkgs; [ jq ];
            services.resolved.enable = true;
            nix.settings.sandbox = false;
          };
      };

      unionTestnetGenesisNode = {
        node =
          { ... }:
          {
            imports = [
              inputs.arion.nixosModules.arion
            ];
            virtualisation = {
              diskSize = 4 * 1024;
              memorySize = 4 * 1024;
              arion = {
                backend = "docker";
                projects.union-devnet.settings = networks.modules.union-v1;
              };
            };
          };
      };

      unionNode = {
        wait_for_console_text = "height=[1-9][0-9]*";
        wait_for_open_port = 26657;
        node = _: {
          imports = [
            inputs.arion.nixosModules.arion
          ];
          virtualisation = {
            diskSize = 4 * 1024;
            memorySize = 4 * 1024;
            arion = {
              backend = "docker";
              projects.union.settings = networks.modules.devnet-union;
            };
            vlans = [ 1 ];
          };
          networking.hostName = "devnetUnion";
        };
      };
    in
    {
      _module.args.e2e = {
        inherit
          mkTest
          unionNode
          unionTestnetGenesisNode
          devnetEthNode
          mkVoyagerNode
          ;

        # TODO: This is poorly named, it only starts devnet-union and devnet-eth
        mkTestWithDevnetSetup =
          {
            name,
            testScript,
            nodes,
          }:
          mkTest {
            inherit name;

            testScript = ''
              # NOTE: Start union first!
              union.wait_for_open_port(${toString unionNode.wait_for_open_port})
              devnetEth.wait_for_open_port(${toString devnetEthNode.wait_for_open_port})

              # match non-zero blocks
              union.wait_until_succeeds('[[ $(curl "http://localhost:26660/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > 1") == "true" ]]')
              devnetEth.wait_for_console_text('${devnetEthNode.wait_for_console_text}')

              devnetEth.wait_until_succeeds('[[ $(curl http://localhost:9596/eth/v2/beacon/blocks/head --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} \'.data.message.slot | tonumber > 0\') == "true" ]]')

              ${testScript}
            '';

            nodes =
              (pkgs.lib.throwIf (builtins.hasAttr "union" nodes) "union node already exists; use a different name")
                (pkgs.lib.throwIf (builtins.hasAttr "devnetEth" nodes) "devnetEth node already exists; use a different name")
                (
                  {
                    union = unionNode.node;
                    devnetEth = devnetEthNode.node;
                  }
                  // nodes
                );
          };

        mkE2eTestEthUnion =
          voyagerConfigFile:
          {
            name,
            testScript,
            nodes ? { },
            openConnection ? false,
          }:
          let
            voyagerNode = mkVoyagerNode voyagerConfigFile;
            voyagerBin = "${self'.packages.voyager}/bin/voyager";
            _30Mins = "18000";
          in
          mkTest {
            inherit name;

            testScript = ''
              openConnection = ${if openConnection then "True" else "False"}

              devnetUnion.wait_for_open_port(${toString unionNode.wait_for_open_port})
              devnetEth.wait_for_open_port(${toString devnetEthNode.wait_for_open_port})

              # we are waiting until slot 20 so that we are sure that the contracts are deployed
              devnetEth.wait_until_succeeds('[[ $(curl http://localhost:9596/eth/v2/beacon/blocks/head --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} \'.data.message.slot | tonumber > 32\') == "true" ]]', timeout = ${_30Mins})

              # match non-zero blocks
              devnetUnion.wait_until_succeeds('[[ $(curl "http://localhost:26657/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > 1") == "true" ]]', timeout = ${_30Mins})

              # deploy contract on union
              devnetUnion.succeed('${self'.packages.cosmwasm-scripts.union-devnet.deploy}/bin/union-devnet-deploy-full')

              devnetVoyager.wait_for_open_port(${toString voyagerNode.wait_for_open_port})
              devnetVoyager.wait_until_succeeds('${voyagerBin} rpc info')

              # index the chains on voyager
              devnetVoyager.wait_until_succeeds('${voyagerBin} -c ${voyagerNode.voyagerConfig} index union-devnet-1 -e')
              devnetVoyager.wait_until_succeeds('${voyagerBin} -c ${voyagerNode.voyagerConfig} index 32382 -e')

              # create clients
              devnetVoyager.wait_until_succeeds('${voyagerBin} -c ${voyagerNode.voyagerConfig} msg create-client --on union-devnet-1 --tracking 32382 --ibc-interface ibc-cosmwasm --ibc-spec-id ibc-union --client-type trusted/evm/mpt -e')
              devnetVoyager.wait_until_succeeds('${voyagerBin} -c ${voyagerNode.voyagerConfig} msg create-client --on 32382 --tracking union-devnet-1 --ibc-interface ibc-solidity --ibc-spec-id ibc-union --client-type cometbls -e')

              # give some time for the clients to be created
              devnetVoyager.wait_until_succeeds('sleep 10')

              if openConnection:
                devnetVoyager.succeed(
                  "echo '{\"@type\":\"call\",\"@value\":{\"@type\":\"submit_tx\",\"@value\":{\"chain_id\":\"union-devnet-1\",\"datagrams\":[{\"ibc_spec_id\":\"ibc-union\",\"datagram\":{\"@type\":\"connection_open_init\",\"@value\":{\"client_id\":1,\"counterparty_client_id\":1}}}]}}}' > /tmp/payload.json"
                )

                devnetVoyager.wait_until_succeeds("${voyagerBin} -c ${voyagerNode.voyagerConfig} q e $(cat /tmp/payload.json)")

                # wait until the connection is opened
                devnetVoyager.wait_until_succeeds("[[ $(${voyagerBin} rpc ibc-state 32382 '{ \"connection\": { \"connection_id\": 1 } }' | jq '.state.state == \"open\"') == true ]]", timeout = ${_30Mins})
              else:
                print("Skipping connection...")

              ${testScript}
            '';

            nodes =
              (pkgs.lib.throwIf (builtins.hasAttr "devnetUnion" nodes) "union node already exists; use a different name")
                (pkgs.lib.throwIf (builtins.hasAttr "devnetEth" nodes) "devnetEth node already exists; use a different name")
                (pkgs.lib.throwIf (builtins.hasAttr "voyager" nodes) "voyager node already exists; use a different name")
                (
                  {
                    devnetUnion = unionNode.node;
                    devnetEth = devnetEthNode.node;
                    devnetVoyager = voyagerNode.node;
                  }
                  // nodes
                );
          };
      };
    };
}

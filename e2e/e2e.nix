{ inputs, ... }:
{
  perSystem =
    {
      pkgs,
      nixpkgs,
      system,
      networks,
      inputs',
      mkVoyagerImg,
      galois-arion-project,
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
                projects.devnet-eth.settings = networks.modules.devnet-eth;
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
            networking.hostName = "devnetVoyager";

            environment.systemPackages = with pkgs; [ jq ];
          };
      };

      unionTestnetGenesisNode = {
        node =
          { pkgs, ... }:
          {
            imports = [
              inputs.arion.nixosModules.arion
            ];
            virtualisation = {
              diskSize = 4 * 1024;
              memorySize = 4 * 1024;
              arion = {
                backend = "docker";
                projects.union-devnet.settings = networks.modules.devnet-union-minimal;
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

      galoisNode = {
        wait_for_console_text = "Serving...";
        wait_for_open_port = 9999;
        node = _: {
          imports = [
            inputs.arion.nixosModules.arion
          ];
          virtualisation = {
            diskSize = 16 * 1024;
            memorySize = 32 * 1024;
            # TODO(aeryz): remove this
            cores = 32;
            arion = {
              backend = "docker";
              projects.galois.settings = galois-arion-project;
            };
            vlans = [ 1 ];
          };
          networking.hostName = "galois";
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

        # TODO: This is poorly named, it only starts devnet-union and devnet-eth
        mkE2eTestEthUnion =
          {
            name,
            testScript,
            nodes ? { },
          }:
          let
            voyagerNode = mkVoyagerNode ../tools/union-test/config.jsonc;
            voyagerBin = "${self'.packages.voyager}/bin/voyager";
          in
          mkTest {
            inherit name;

            testScript = ''
              galois.start()
              devnetUnion.wait_for_open_port(${toString unionNode.wait_for_open_port})
              devnetEth.wait_for_open_port(${toString devnetEthNode.wait_for_open_port})

              devnetUnion.succeed('${self'.packages.cosmwasm-scripts.union-devnet.deploy}/bin/union-devnet-deploy-full')

              # match non-zero blocks
              devnetUnion.wait_until_succeeds('[[ $(curl "http://localhost:26657/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > 1") == "true" ]]')
              devnetEth.wait_until_succeeds('[[ $(curl http://localhost:9596/eth/v2/beacon/blocks/head --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} \'.data.message.slot | tonumber > 1\') == "true" ]]')

              devnetVoyager.wait_for_open_port(${toString voyagerNode.wait_for_open_port})
              devnetVoyager.wait_until_succeeds('${voyagerBin} rpc info')

              galois.wait_for_console_text('${galoisNode.wait_for_console_text}')

              devnetUnion.wait_until_succeeds('[[ $(curl "http://localhost:26660/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > 200000") == "true" ]]')
            '';

            nodes =
              (pkgs.lib.throwIf (builtins.hasAttr "devnetUnion" nodes) "union node already exists; use a different name")
                (pkgs.lib.throwIf (builtins.hasAttr "devnetEth" nodes) "devnetEth node already exists; use a different name")
                (pkgs.lib.throwIf (builtins.hasAttr "voyager" nodes) "voyager node already exists; use a different name")
                (pkgs.lib.throwIf (builtins.hasAttr "galois" nodes) "galois node already exists; use a different name")
                (
                  {
                    devnetUnion = unionNode.node;
                    devnetEth = devnetEthNode.node;
                    devnetVoyager = voyagerNode.node;
                    galois = galoisNode.node;
                  }
                  // nodes
                );
          };
      };
    };
}

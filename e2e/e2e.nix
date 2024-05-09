{ inputs, ... }: {
  perSystem = { pkgs, nixpkgs, system, networks, inputs', ... }:
    let
      mkTest =
        let
          nixos-lib = import "${nixpkgs}/nixos/lib" { };
        in
        { name, testScript, nodes }:
        nixos-lib.runTest {
          inherit name testScript nodes;
          hostPkgs = pkgs; # the Nixpkgs package set used outside the VMs
          passthru = { ci = false; };
        };


      devnetEthNode = {
        wait_for_console_text = "Synced - slot: [1-9][0-9]*";
        wait_for_open_port = 8546;
        node = { pkgs, ... }: {
          imports = [
            inputs.arion.nixosModules.arion
          ];
          virtualisation = {
            diskSize = 8 * 1024;
            memorySize = 4 * 1024;
            arion = {
              backend = "docker";
              projects.devnet-eth.settings = networks.modules.devnet-eth;
            };
          };

          environment.systemPackages = with pkgs; [ jq ];
        };
      };

      unionTestnetGenesisNode = {
        node = { pkgs, ... }: {
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
          };
        };
      };
    in
    {
      _module.args.e2e = {
        inherit mkTest unionNode unionTestnetGenesisNode devnetEthNode;

        # TODO: This is poorly named, it only starts devnet-union and devnet-eth
        mkTestWithDevnetSetup = { name, testScript, nodes }:
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
                ({
                  union = unionNode.node;
                  devnetEth = devnetEthNode.node;
                } // nodes);
          };
      };
    };
}

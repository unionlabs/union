{ inputs, ... }: {
  perSystem = { pkgs, nixpkgs, system, networks, ... }:
    let
      mkTest =
        let
          nixos-lib = import "${nixpkgs}/nixos/lib" { };
        in
        { name, testScript, nodes }:
        nixos-lib.runTest {
          inherit name testScript nodes;
          hostPkgs = pkgs; # the Nixpkgs package set used outside the VMs
        };


      sepoliaNode = {
        wait_for_console_text = "Synced - slot: [1-9][0-9]*";
        wait_for_open_port = 8546;
        node = { pkgs, ... }: {
          imports = [
            inputs.arion.nixosModules.arion
          ];
          virtualisation = {
            diskSize = 4 * 1024;
            arion = {
              backend = "docker";
              projects.sepolia.settings = networks.sepolia;
            };
          };

          environment.systemPackages = with pkgs; [ jq ];
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
            diskSize = 2 * 1024;
            arion = {
              backend = "docker";
              projects.union.settings = networks.union;
            };
          };
        };
      };
    in
    {
      _module.args.e2e = {
        inherit mkTest unionNode sepoliaNode;

        mkTestWithDevnetSetup = { name, testScript, nodes }:
          mkTest {
            inherit name;

            testScript = ''
              # NOTE: Start union first!
              union.wait_for_open_port(${toString unionNode.wait_for_open_port})
              sepolia.wait_for_open_port(${toString sepoliaNode.wait_for_open_port})

              # match non-zero blocks
              union.wait_until_succeeds('[[ $(curl "http://localhost:26660/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > 1") == "true" ]]')
              sepolia.wait_for_console_text('${sepoliaNode.wait_for_console_text}')

              sepolia.wait_until_succeeds('[[ $(curl http://localhost:9596/eth/v2/beacon/blocks/head --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} \'.data.message.slot | tonumber > 0\') == "true" ]]')

              ${testScript}
            '';

            nodes =
              (pkgs.lib.throwIf (builtins.hasAttr "union" nodes) "union node already exists; use a different name")
                (pkgs.lib.throwIf (builtins.hasAttr "sepolia" nodes) "sepolia node already exists; use a different name")
                ({
                  union = unionNode.node;
                  sepolia = sepoliaNode.node;
                } // nodes);
          };
      };
    };
}

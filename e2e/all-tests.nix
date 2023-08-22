{ lib, withSystem, inputs, ... }: {
  flake.checks = lib.genAttrs [ "x86_64-linux" "aarch64-linux" ]
    (lib.flip withSystem ({ e2e, networks, pkgs, nixpkgs, crane, ... }:
      {
        ensure-blocks = import ./ensure-blocks/ensure-blocks.nix { inherit e2e networks pkgs nixpkgs crane; };

        virtualisation-works = e2e.mkTest {
          name = "devnet";
          nodes = {
            devnet = _: {
              imports = [
                inputs.arion.nixosModules.arion
              ];
              virtualisation = {
                diskSize = 4 * 1024;
                arion = {
                  backend = "docker";
                  projects.devnet.settings = networks.devnet;
                };
              };
            };
          };
          testScript = ''
            devnet.wait_for_unit("arion-${networks.devnet.project.name}")
          '';
        };

        sepolia-runs = e2e.mkTest {
          name = "sepolia-runs";

          testScript = ''
            start_all()

            sepolia.wait_for_open_port(${toString e2e.sepoliaNode.wait_for_open_port})

            sepolia.wait_for_console_text('${e2e.sepoliaNode.wait_for_console_text}')
          '';

          nodes = {
            sepolia = e2e.sepoliaNode.node;
          };
        };

        union-runs = e2e.mkTest {
          name = "union-runs";

          testScript = ''
            start_all()

            union.wait_for_open_port(${toString e2e.unionNode.wait_for_open_port})

            # Ensure the union network commits more than one block
            union.wait_until_succeeds('[[ $(curl "http://localhost:26660/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > 1") == "true" ]]')
          '';

          nodes = {
            union = e2e.unionNode.node;
          };
        };

        epoch-completes = e2e.mkTest {
          name = "epoch-completes";

          testScript = ''
            union.wait_for_open_port(${toString e2e.unionNode.wait_for_open_port})

            union.wait_until_succeeds('[[ $(curl "http://localhost:26660/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > 1") == "true" ]]')

            # Ensure we get through one epoch
            union.wait_for_console_text('Rotating validator set due to end of epoch.')
          '';

          nodes = {
            union = e2e.unionNode.node;
          };
        };

        forced-set-rotation = e2e.mkTest {
          name = "forced-set-rotation";

          testScript = ''
            union.wait_for_open_port(${toString e2e.unionNode.wait_for_open_port})

            union.wait_until_succeeds('[[ $(curl "http://localhost:26660/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > 1") == "true" ]]')

            union.wait_for_console_text('Rotating validator set due to end of epoch.')

            # Ensure validators exist in docker
            union.wait_until_succeeds('docker container ls | grep union')
            # Stop docker nodes
            union.wait_until_succeeds('docker stop union-uniond-2-1')

            union.wait_for_console_text('Rotating validator set due to exceeding the threshold of jailed validators.')
          '';

          nodes = {
            union = e2e.unionNode.node;
          };
        };
      }));
}

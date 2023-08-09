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

            union.wait_for_console_text('${e2e.unionNode.wait_for_console_text}')
          '';

          nodes = {
            union = e2e.unionNode.node;
          };
        };

        epoch-completes = e2e.mkTest {
          name = "epoch-completes";

          testScript = ''
            start_all()

            union.wait_for_open_port(${toString e2e.unionNode.wait_for_open_port})

            union.wait_for_console_text('${e2e.unionNode.wait_for_console_text}')
          '';

          nodes = {
            union = e2e.unionNode.node;
          };
        };
      }));
}

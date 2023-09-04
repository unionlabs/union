{ lib, withSystem, inputs, ... }: {
  flake.checks = lib.genAttrs [ "x86_64-linux" "aarch64-linux" ]
    (lib.flip withSystem ({ e2e, networks, pkgs, nixpkgs, crane, ... }:
      let
        epoch-staking = import ./epoch-staking.nix { inherit e2e pkgs; };
      in
      {
        ensure-blocks = import ./ensure-blocks/ensure-blocks.nix { inherit e2e networks pkgs nixpkgs crane; };
        hubble-e2e = import ./hubble/e2e.nix { inherit e2e pkgs networks; };


        # Tests from ./epoch-staking.nix
        epoch-completes = epoch-staking.epoch-completes;
        forced-set-rotation = epoch-staking.forced-set-rotation;

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
      }));
}

{ lib, withSystem, inputs, ... }: {
  flake.checks = lib.genAttrs [ "x86_64-linux" "aarch64-linux" ]
    (lib.flip withSystem ({ e2e, networks, pkgs, nixpkgs, crane, ... }:
      let
        ensure-blocks = pkgs.lib.meta.getExe (crane.lib.buildPackage {
          inherit (crane.lib.crateNameFromCargoToml { cargoToml = ../tools/e2e/ensure-blocks/Cargo.toml; }) pname version;
          buildInputs = [ pkgs.pkg-config pkgs.openssl ];
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          src = crane.lib.cleanCargoSource ../tools/e2e/ensure-blocks;
          doCheck = false;
          cargoVendorDir = crane.lib.vendorCargoDeps { cargoLock = ../tools/e2e/ensure-blocks/Cargo.lock; };
        });

        sepoliaNode = _: {
          imports = [
            inputs.arion.nixosModules.arion
          ];
          virtualisation = {
            diskSize = 4 * 1024;
            arion = {
              backend = "podman-socket";
              projects.sepolia.settings = networks.sepolia;
            };
          };
        };

        unionNode = _: {
          imports = [
            inputs.arion.nixosModules.arion
          ];
          virtualisation = {
            diskSize = 2048;
            arion = {
              backend = "podman-socket";
              projects.union.settings = networks.union;
            };
          };
        };
      in
      {
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
        ensure-blocks = e2e.mkTest {
          name = "ensure-blocks";

          testScript = ''
            start_all()

            # match non-zero blocks
            union.wait_for_console_text("height=[1-9][0-9]*")
            sepolia.wait_for_console_text("Synced - slot: [1-9][0-9]*")

            # union.wait_for_open_port(26657)
            # sepolia.wait_for_open_port(8546)

            with open("output.log", "w") as file:
              output = client.succeed("RUST_LOG=debug ${ensure-blocks} ws://union:26657/websocket ws://sepolia:8546 2>&1")
              file.write(output)
          '';

          nodes = {
            union = unionNode;
            sepolia = sepoliaNode;
            # empty node used to communicate with the other nodes
            client = _: { };
          };
        };
        sepolia-runs = e2e.mkTest {
          name = "sepolia-runs";

          testScript = ''
            start_all()

            # match non-zero blocks
            sepolia.wait_for_console_text("Synced - slot: [1-9][0-9]*")

            sepolia.wait_for_open_port(8546)
          '';

          nodes = {
            sepolia = sepoliaNode;
          };
        };
        union-runs = e2e.mkTest {
          name = "sepolia-runs";

          testScript = ''
            start_all()

            # match non-zero blocks
            union.wait_for_console_text("height=[1-9][0-9]*")
            union.wait_for_open_port(26657)
          '';

          nodes = {
            union = unionNode;
          };
        };
      }));
}

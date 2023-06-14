{ lib, withSystem, ... }: {
  flake.checks = lib.genAttrs [ "x86_64-linux" "aarch64-linux" ]
    (lib.flip withSystem ({ e2e, networks, ... }:
      {
        virtualisation-works = let name = "devnet"; in e2e.mkTest {
          inherit name;
          network = networks.devnet;
          testScript = ''
            ${name}.wait_for_unit("arion-${networks.devnet.project.name}")
          '';
        };
      }));
}


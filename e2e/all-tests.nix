{ ... }: {
  perSystem = { e2e, networks, ... }:
    {
      checks.virtualisation-works = let name = "devnet"; in e2e.mkTest {
        inherit name;
        network = networks.devnet;
        testScript = ''
          ${name}.wait_for_unit("arion-${networks.devnet.project.name}")
        '';
      };
    };
}


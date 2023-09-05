# Hubble should automatically be indexing and exposing a graphql endpoint of 
# processed blocks. This tests checks for console output to ensure the 
# indexing process is live.
# 
# We verify:
# - migrations work from a clean DB.
# - graphql definitions are in sync with migrations.
# - Hubble is capable of querying nodes.
# - Hubble progresses against produced nodes.
{ inputs, networks, e2e, ... }:
let
  service = networks.hubble // {
    services.hubble.service = {
      environment.HUBBLE_INDEXERS = ''{"type": "Tm", "url": "http://union:26657"}'';
      depends_on = {
        hasura = {
          condition = "service_healthy";
        };
      };
    };
  };

  hubble = {
    imports = [
      inputs.arion.nixosModules.arion
    ];
    virtualisation = {
      diskSize = 4 * 1024;
      arion = {
        backend = "docker";
        projects.hubble.settings = service;
      };
    };
  };
in
e2e.mkTestWithDevnetSetup {
  name = "hubble-indexes";

  testScript = ''
    start_all()
    hubble.wait_for_console_text("indexing block 2")
  '';

  nodes = {
    inherit hubble;
  };
}

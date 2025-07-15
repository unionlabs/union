{
  e2e,
  pkgs,
  ...
}:
{
  voyager-queue-works = e2e.mkE2eTestEthUnion {
    name = "voyager-queue-works";

    testScript = ''
    '';

    nodes = {
      # empty node used to communicate with the other nodes
      client = _: { };
    };
  };
}

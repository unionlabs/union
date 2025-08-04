{
  e2e,
  pkgs,
  ...
}:
{
  all-works = e2e.mkE2eTestEthUnion {
    name = "all-works";

    testScript = '''';

    nodes = {
    };
  };
}

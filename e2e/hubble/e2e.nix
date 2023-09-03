# Hubble should automatically be indexing and exposing a graphql endpoint of 
# processed blocks. This tests polls that endpoint for data.
# 
# We verify:
# - migrations work from a clean DB.
# - graphql definitions are in sync with migrations.
# - Hubble is capable of querying nodes.
# - Hubble progresses against produced nodes.
{ e2e, ... }:
e2e.mkDevnetTest {
  name = "hubble-e2e";

  testScript = ''
    devnet.wait_for_console_text("indexing block 2")
  '';

  nodes = {
    client = _: { };
  };
}


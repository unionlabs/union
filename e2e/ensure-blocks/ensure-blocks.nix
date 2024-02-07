{ e2e, pkgs, crane, ... }:
let
  ensure-blocks = pkgs.lib.meta.getExe (crane.buildWorkspaceMember {
    crateDirFromRoot = "e2e/ensure-blocks";
  }).packages.ensure-blocks;
in

e2e.mkTestWithDevnetSetup {
  name = "ensure-blocks";

  testScript = ''
    client.wait_until_succeeds('[[ $(curl http://devnetEth:9596/eth/v2/beacon/blocks/head --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} \'.data.message.slot | tonumber > 0\') == "true" ]]')

    client.succeed("RUST_LOG=info ${ensure-blocks} ws://union:26657/websocket ws://devnetEth:8546 |& tee output.txt")

    client.copy_from_vm("output.txt", "")
  '';

  nodes = {
    # empty node used to communicate with the other nodes
    client = _: { };
  };
}


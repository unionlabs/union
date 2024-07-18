{ e2e, pkgs, ... }:
{
  epoch-completes = e2e.mkTest {
    name = "epoch-completes";

    testScript = ''
      union.wait_for_open_port(${toString e2e.unionNode.wait_for_open_port})

      union.wait_until_succeeds('[[ $(curl "http://localhost:26660/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > 1") == "true" ]]')

      # Ensure we get through one epoch
      union.wait_for_console_text('Rotating validator set due to end of epoch.')
    '';

    nodes = {
      union = e2e.unionNode.node;
    };
  };

  forced-set-rotation = e2e.mkTest {
    name = "forced-set-rotation";

    testScript = ''
      union.wait_for_open_port(${toString e2e.unionNode.wait_for_open_port})

      print("I don't fail!")
      union.wait_until_succeeds('[[ $(curl "http://localhost:26660/block" --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} ".result.block.header.height | tonumber > 1") == "true" ]]')

      union.wait_for_console_text('Rotating validator set due to end of epoch.')

      # Ensure validators exist in docker
      union.wait_until_succeeds('docker container ls | grep union', timeout=60)
      # Stop docker nodes
      union.wait_until_succeeds('docker stop devnet-union-union-2-1', timeout=60)

      union.wait_for_console_text('Rotating validator set due to exceeding the threshold of jailed validators.')
    '';

    nodes = {
      union = e2e.unionNode.node;
    };
  };
}

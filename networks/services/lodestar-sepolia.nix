{ pkgs, config }:
let
  lodestar-init = pkgs.writeShellApplication {
    name = "lodestar-init";
    runtimeInputs = [ pkgs.coreutils pkgs.curl pkgs.jq config ];
    text = ''
      node ./packages/cli/bin/lodestar \
        beacon \
        --network sepolia \
        --dataDir /data/lodestar \
        --execution.urls http://geth:8551 \
        --rest.address 0.0.0.0 \
        --rest.namespace "*" \
        --jwt-secret ${config}/dev-jwt.prv \
        --logFileDailyRotate 5 \
        --checkpointSyncUrl https://beaconstate-sepolia.chainsafe.io
    '';
  };
in
{
  build = {
    image = pkgs.lib.mkForce (pkgs.dockerTools.streamLayeredImage {
      name = "lodestar-extended";
      fromImage = pkgs.dockerTools.pullImage ({
        imageName = "chainsafe/lodestar";
        imageDigest = "sha256:5e262f6e631ed3d60ba867200d8b53da6e06ba965eac1a0fdc9b0621c5f65a61";
        finalImageName = "chainsafe/lodestar";
        finalImageTag = "v1.8.0";
      } // (if pkgs.stdenv.isx86_64 then {
        sha256 = "1p5kc4gs9g6igcs4g0ppgji50xkq79jkyyg3z9cdn2d9m5vam4fm";
        arch = "amd64";
      } else {
        sha256 = "0gnkk3y90wcz78ngqx341kfh25zbjm15z3jdidwl7vh5hbmpsjrz";
        arch = "arm64";
      }));
      contents = [
        lodestar-init
      ];
      config = {
        # dockerTools only preserves Env from the base "fromImage"
        # this is directly coming from https://github.com/ChainSafe/lodestar/blob/402c46f0d9f1f964066efb3e0e53863d6a913a80/Dockerfile#L39
        WorkingDir = "/usr/app";
        Entrypoint = pkgs.lib.meta.getExe lodestar-init;
      };
    });
  };
  service = {
    stop_signal = "SIGINT";
    ports = [
      # Beacon node rest API
      "9596:9596"
    ];
    depends_on = {
      geth = {
        condition = "service_healthy";
      };
    };
  };
}

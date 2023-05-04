{ pkgs, lodestar-init }: {
  image = {
    enableRecommendedContents = true;
    contents = [
      pkgs.coreutils
      lodestar-init
    ];
  };
  service = {
    stop_signal = "SIGINT";
    networks = [ "union-devnet" ];
    ports = [
      # Beacon node rest API
      "9596:9596"
    ];
    command = [ "sh" "-c" "${lodestar-init}/bin/lodestar-init" ];
    depends_on = {
      geth = {
        condition = "service_healthy";
      };
    };
  };
}

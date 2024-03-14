  {pkgs, config, self', ... }: 
  let 
    prysm = self'.packages.prysm;
    validator-init =
      pkgs.writeShellApplication {
        name = "validator-init";
        runtimeInputs = [ ];
        text = ''
          ${prysm}/bin/validator \
            --datadir /validatordata \
            --accept-terms-of-use \
            --interop-num-validators 64 \
            --chain-config-file ${config}/beacon-config.yml
        '';        
      };
  in
  {
  image = {
    enableRecommendedContents = true;
    contents = [
      pkgs.coreutils
      pkgs.curl
      pkgs.jq
      validator-init
    ];
  };
  service = {
    tty = true;
    stop_signal = "SIGINT";
    restart = "always";
    ports = [
      # RPC
      "7000:7000"
    ];
    command = [ "${validator-init}/bin/validator-init" ];
    depends_on = {
      geth = {
        condition = "service_healthy";
      };
    };
  };
  
}

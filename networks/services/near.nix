{ nearcore, nearup, pkgs, ... }:
let 
  near-localnet = pkgs.writeShellApplication {
    name = "near-localnet";
    runtimeInputs = [ nearup ];
    text = ''
      mkdir /home
      nearup run --home /home/localnet --binary-path ${nearcore}/bin localnet
    '';
  };
in
{  
  image = {
    enableRecommendedContents = true;
    contents = [
      pkgs.coreutils
      pkgs.curl
      near-localnet
      nearcore
    ];
  };

  service = {
    tty = true;
    stop_signal = "SIGINT";
    ports = [
    ];
    command = [
      "${near-localnet}/bin/near-localnet"
    ];
  };
}

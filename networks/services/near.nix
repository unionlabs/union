{ pkgs, near-localnet, nearup }:
{
  image = {
    enableRecommendedContents = true;
    contents = [
      pkgs.coreutils
      near-localnet
    ];
  };
  service = {
    tty = true;
    stop_signal = "SIGINT";
    command = [ "${near-localnet}/bin/near-localnet" ];
    ports = [
      "3030:3030"
    ];
  };
}

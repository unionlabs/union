{
  self',
  pkgs,
}:
let
  voyager-start = pkgs.writeShellApplication {
    name = "voyager-start";
    text = ''
      ${self'.packages.voyager}/bin/voyager start 
    '';
  };
in
{
  build = {
    image = {
      enableRecommendedContents = true;
      contents = [
        pkgs.coreutils
        pkgs.curl
        pkgs.jq
        voyager-start
        self'.packages.voyager-modules-plugins
      ];
    };
    service = {
      tty = true;
      stop_signal = "SIGINT";
      ports = [
        # voyager rest laddr
        "7177:7177"
        # voyager rpc laddr
        "7178:7178"
      ];
      command = [ "ls -la /bin" ];
      healthcheck = {
        interval = "5s";
        retries = 3;
        test = [
          "CMD-SHELL"
        ];
      };
    };
  };
}

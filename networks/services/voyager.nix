{
    self',
    pkgs,
    inputs',
    lib,
  }:
let
  arion = inputs'.arion.packages.default;
  # voyager-start = pkgs.writeShellApplication {
  #   name = "voyager-start";
  #   text = ''
  #     ${self'.packages.voyager}/bin/voyager start 
  #   '';
  # };
  voyager-service = {
    image = {
      enableRecommendedContents = true;
      contents = [
        pkgs.coreutils
        pkgs.curl
        pkgs.jq
        self'.packages.voyager
        # self'.packages.voyager-modules-plugins
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
      command = [
        "sh"
        "-c"
        ''
          ls -la /bin
        ''
      ];
      # healthcheck = {
      #   interval = "5s";
      #   retries = 3;
      #   test = [
      #     "CMD-SHELL"
      #   ];
      # };
      depends_on = {
        postgres = {
          condition = "service_healthy";
        };
      };
    };
  };

in
{
  mkVoyagerImg = {
    project.name = "voyager";
    services = {
      voyager = voyager-service;
      postgres = import ./postgres.nix { inherit lib pkgs; };
    };
  };
  # packages.voyager-img = pkgs.writeShellApplication {
  #   name = "voyager-img";
  #   runtimeInputs = [ arion ];    
  #   text = ''
  #     arion --prebuilt-file ${arion.build
  #     {
  #       modules = [
  #         {
  #           project.name = "voyager";
  #           services = {
  #             voyager = voyager-service;
  #             postgres = import ./postgres.nix { inherit lib pkgs; };
  #           };
  #         }
  #       ];
  #     }} up --build --force-recreate -V --always-recreate-deps --remove-orphans
  #   '';
  # };
}

{ ... }: {
  perSystem = { pkgs, self', inputs', ... }: {
    packages.devnet =
      let
        arion = inputs'.arion.packages.default;

        mkUniondService = id: {
          image.enableRecommendedContents = true;
          image.contents = [ pkgs.coreutils self'.packages.devnet-genesis self'.packages.uniond ];
          service.command = [
            "sh"
            "-c"
            ''
              cp -R ${self'.packages.devnet-genesis} .
              ${self'.packages.uniond}/bin/uniond start --home .
            ''
          ];
          service.ports = [
            "8000:8000" # host:container
          ];
          service.stop_signal = "SIGINT";
        };

        spec = {
          modules = [{
            project.name = "union-devnet";
            services = {
              uniond-1 = mkUniondService {
                id = 1;
              };
            };

          }];
        };
        build = arion.build spec;
      in
      pkgs.writeShellApplication {
        name = "union-devnet";
        runtimeInputs = [ arion ];
        text = ''
          arion --prebuilt-file ${build} up --build --force-recreate -V --always-recreate-deps --remove-orphans
        '';
      };
  };
}

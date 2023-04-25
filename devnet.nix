{ ... }: {
  perSystem = { pkgs, self', inputs', ... }: {
    packages = {
      devnet = pkgs.writeShellApplication {
        name = "union-devnet";
        runtimeInputs = [ inputs'.arion.packages.default ];
        text = ''
          arion
        '';
      };
    };

    checks = { };
  };
}

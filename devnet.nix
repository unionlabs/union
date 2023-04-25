
{ inputs, ... }: {
  perSystem = { pkgs, self', ... }: {
    packages = {
      devnet = pkgs.writeShellApplication {
        name = "union-devnet";
        runtimeInputs = [ self'.inputs.arion.packages.default ];
        text = ''
          arion
        '';
     
      };
    };

    checks = {
    };
  };
}
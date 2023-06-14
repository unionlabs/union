{ inputs, ... }: {
  perSystem = { pkgs, nixpkgs, system, ... }:
    let
      arion = inputs.arion;
      mkTest = import ./mkTest.nix { inherit nixpkgs arion pkgs system; };
    in
    {
      _module.args.e2e = {
        inherit mkTest;
      };
    };
}

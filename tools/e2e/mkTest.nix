{ nixpkgs
, arion
, pkgs
, system
}:
{ name
, network
, testScript
, ...
}:
let
  nixos-lib = import "${nixpkgs}/nixos/lib" { };
in
nixos-lib.runTest {
  inherit name testScript;

  nodes.${name} =
    { pkgs, lib, ... }:
    {
      imports = [
        arion.nixosModules.arion
      ];
      virtualisation.arion = {
        backend = "podman-socket";
        projects.${network.project.name}.settings = network;
      };
    };
  hostPkgs = pkgs; # the Nixpkgs package set used outside the VMs
}

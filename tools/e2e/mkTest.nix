{ nixpkgs
, arion
, pkgs
, system
}:
let
  nixos-lib = import "${nixpkgs}/nixos/lib" { };
in
{ name
, testScript
, nodes
}:
nixos-lib.runTest {
  inherit name testScript nodes;
  hostPkgs = pkgs; # the Nixpkgs package set used outside the VMs
}

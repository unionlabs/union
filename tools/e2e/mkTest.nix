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

  test = nixos-lib.runTest {
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
  };
in
pkgs.stdenv.mkDerivation
{
  pname = name;
  version = "1.2.3";
  buildInputs = [ test ];
  src = ./.;
  doCheck = true;
  checkPhase = ''
    ls ${test}
  '';
  buildPhase = ''
    touch $out
  '';
  requiredSystemFeatures = [ "kvm" "nixos-test" ];
}

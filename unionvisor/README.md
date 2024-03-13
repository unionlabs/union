
<!-- GENERATED: DO NOT EDIT -->


# Unionvisor

Unionvisor is a utility for managing [`uniond`](../uniond) deployments. It manages upgrade lifecycles and integrates well with NixOS.

## NixOS Configuration

An example flake.nix configuration can be found in [`usage.nix`](./usage.nix):

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    union.url = "git+ssh://git@github.com/unionlabs/union";
  };
  outputs = { self, nixpkgs, union, ... }:
    {
      nixosConfigurations.testnet-validator =
        let
          system = "x86_64-linux";
          pkgs = import nixpkgs { inherit system; };
        in
        nixpkgs.lib.nixosSystem {
          inherit system;
          modules = [
            union.nixosModules.unionvisor
            {
              system.stateVersion = "23.11";
              # Base configuration for openstack-based VPSs
              imports = [ "${nixpkgs}/nixos/modules/virtualisation/openstack-config.nix" ];

              # Allow other validators to reach you
              networking.firewall.allowedTCPPorts = [ 80 443 26656 26657 ];

              # Unionvisor module configuration
              services.unionvisor = {
                enable = true;
                moniker = "your-testnet-moniker";
              };

              # OPTIONAL: Some useful inspection tools for when you SSH into your validator
              environment.systemPackages = with pkgs; [
                bat
                bottom
                helix
                jq
                neofetch
                tree
              ];
            }
          ];
        };
    };
}

```

The configuration creates a production-ready machine running a validator under unionvisor, using the unionbundle. Bundles are packages that contain historic `uniond` binaries. They are capable of syncing a chain from zero and performing upgrades, effectively [bootstrapping](<https://en.wikipedia.org/wiki/Bootstrapping_(compilers)>) and verifying the full history.

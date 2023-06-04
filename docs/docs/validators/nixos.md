---
title: "NixOS"
---

NixOS deployments can use our [module](https://github.com/unionfi/union/blob/82ad5ef76b42e76a18617a614c1cdcd41d1fbe93/unionvisor/unionvisor.nix#L68) to easily manage their validator. It creates a systemd service with a production configuration

:::caution
The current example does not support remote signers yet. We will expand the guide once [horcrux](https://github.com/strangelove-ventures/horcrux).
:::

## Configuration

Below is an example configuration.nix which can be used in production.

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    union.url = "github:unionfi/union";
  };
  outputs = inputs@{self, nixpkgs,... }:
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

You can then deploy the configuration by running

```
nixos-rebuild --target-host $NODE_IP switch --flake .\#my-moniker
```

## Upgrading

To upgrade to newer versions, simply run

```
nix flake update
nixos-rebuild --target-host $NODE_IP switch --flake .\#my-moniker
```

This will pull in the latest changes to union configurations and prepare your node for future upgrades.

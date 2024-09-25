{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    union.url = "git+ssh://git@github.com/unionlabs/union";
  };
  outputs =
    {
      self,
      nixpkgs,
      union,
      ...
    }:
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
              networking.firewall.allowedTCPPorts = [
                80
                443
                26656
                26657
              ];

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

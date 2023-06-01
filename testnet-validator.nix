{ self, ... }: {
  flake = {
    nixosConfigurations.testnet-validator =
      let
        nixpkgs = self.inputs.nixpkgs;
        pkgs = import nixpkgs { system = "x86_64-linux"; };
      in
      nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";

        modules = [
          self.nixosModules.unionvisor
          {
            imports = [
              "${nixpkgs}/nixos/modules/virtualisation/openstack-config.nix"
            ];
            system.stateVersion = "23.11";
            environment.systemPackages = with pkgs; [
              neofetch
              bat
              bottom
              tree
              helix
              jq
            ];
            networking.firewall = { allowedTCPPorts = [ 80 443 26656 26657 ]; };

            services.unionvisor = {
              enable = true;
              moniker = "cor-systems";
            };
          }
        ];
      };
  };
}

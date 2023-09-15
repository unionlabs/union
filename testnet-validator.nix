{ self, ... }: {
  flake.nixosConfigurations.testnet-validator =
    let
      system = "x86_64-linux";
      nixpkgs = self.inputs.nixpkgs;
      pkgs = import nixpkgs { inherit system; };
    in
    nixpkgs.lib.nixosSystem {
      inherit system;

      modules = [
        self.nixosModules.unionvisor
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
            zellij
            bat
            bottom
            helix
            jq
            neofetch
            tree
          ];

          # OPTIONAL: Settings to improve the Nix experience
          nix = {
            settings = {
              sandbox = "relaxed";
              substituters = [
                "https://union.cachix.org/"
              ];
              trusted-public-keys = [
                "union.cachix.org-1:TV9o8jexzNVbM1VNBOq9fu8NK+hL6ZhOyOh0quATy+M="
              ];
            };
            extraOptions = ''
              experimental-features = nix-command flakes
              keep-outputs = true
              keep-derivations = true
            '';
          };
        }
      ];
    };
}

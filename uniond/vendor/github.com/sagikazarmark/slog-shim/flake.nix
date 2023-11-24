{
  inputs = {
    # nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixpkgs.url = "github:NixOS/nixpkgs/master";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devenv.url = "github:cachix/devenv";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.devenv.flakeModule
      ];

      systems = [ "x86_64-linux" "x86_64-darwin" "aarch64-darwin" ];

      perSystem = { config, self', inputs', pkgs, system, ... }: rec {
        devenv.shells = {
          default = {
            languages = {
              go.enable = true;
              go.package = pkgs.lib.mkDefault pkgs.go_1_21;
            };

            # https://github.com/cachix/devenv/issues/528#issuecomment-1556108767
            containers = pkgs.lib.mkForce { };
          };

          ci = devenv.shells.default;

          ci_1_19 = {
            imports = [ devenv.shells.ci ];

            languages = {
              go.package = pkgs.go_1_19;
            };
          };

          ci_1_20 = {
            imports = [ devenv.shells.ci ];

            languages = {
              go.package = pkgs.go_1_20;
            };
          };

          ci_1_21 = {
            imports = [ devenv.shells.ci ];

            languages = {
              go.package = pkgs.go_1_21;
            };
          };
        };
      };
    };
}

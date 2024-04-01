{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/master";
    flake-utils.url = "github:numtide/flake-utils";
    gomod2nix = {
      url = "github:nix-community/gomod2nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.utils.follows = "flake-utils";
    };
  };

  outputs = { self, nixpkgs, gomod2nix, flake-utils }:
    {
      overlays.default = self: super: {
        rocksdb = super.rocksdb.overrideAttrs (_: rec {
          version = "8.9.1";
          src = self.fetchFromGitHub {
            owner = "facebook";
            repo = "rocksdb";
            rev = "v${version}";
            sha256 = "sha256-Pl7t4FVOvnORWFS+gjy2EEUQlPxjLukWW5I5gzCQwkI=";
          };
        });
      };
    } //
    (flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            config = { };
            overlays = [
              gomod2nix.overlays.default
              self.overlays.default
            ];
          };
        in
        rec {
          devShells = rec {
            default = with pkgs; mkShell {
              buildInputs = [
                go_1_20 # Use Go 1.20 version
                rocksdb
              ];
            };
          };
          legacyPackages = pkgs;
        }
      ));
}

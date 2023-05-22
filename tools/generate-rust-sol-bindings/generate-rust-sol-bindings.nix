{ ... }: {
  perSystem = { self', pkgs, system, config, inputs', crane, stdenv, ... }:
    let
      attrs = {
        src = crane.lib.cleanCargoSource ./.;
        doCheck = false;
        cargoVendorDir = crane.lib.vendorCargoDeps { cargoLock = ./Cargo.lock; };
      } // (crane.lib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; });

      generate-rust-sol-bindings-crate = crane.lib.buildPackage attrs;

      # cargo-toml = crane.lib.writeTOML "Cargo.toml" {
      #   package = {
      #     name = "contracts";
      #     version = "0.0.0";
      #     edition = "2021";
      #   };
      #   dependencies = {
      #     # TODO(benluelo): use workspace dependencies
      #     ethers = "2.0.4";
      #   };
      # };

      rust-sol-bindings = pkgs.stdenv.mkDerivation {
        name = "generate-rust-sol-bindings";
        pname = "generate-rust-sol-bindings";
        src = ./.;
        buildInputs = [ config.treefmt.build.programs.rustfmt pkgs.taplo ];
        buildPhase = ''
          mkdir $out

					${generate-rust-sol-bindings-crate}/bin/generate-rust-sol-bindings \
						--cratedir ./out/ \
						${self'.packages.evm-contracts}/out/IBCHandler.sol/IBCHandler.json \
						${self'.packages.evm-contracts}/out/Glue.sol/Glue.json
						# ${self'.packages.evm-contracts}/out/CometblsHelp.sol/CometblsHelp.json

          ls -al ./out/

          # format and normalize comments in generated code
          # rustfmt --config normalize_comments=true --edition "2021" lib.rs

					# mkdir $out/src
          # cp -r ./lib.rs $out/src/lib.rs
          # cp -r $ {cargo-toml} $out/Cargo.toml

          cp -r ./out/* $out

          taplo format $out/Cargo.toml
        '';
      };
    in
    {
      packages = {
        rust-sol-bindings = rust-sol-bindings;

        generate-rust-sol-bindings = pkgs.writeShellApplication {
          name = "generate-rust-sol-bindings";
          runtimeInputs = [ rust-sol-bindings ];
          text = ''
	          # If the current directory contains flake.nix, then we are at the repository root
	          if [[ -f flake.nix ]]
	          then
	            echo "We are at the repository root. Starting generation..."
	          else
	            echo "We are NOT at the repository root. Please cd to the repository root and try again."
	            exit 1
	          fi

	          outdir="generated/contracts"

	          cp -r --no-preserve=mode ${rust-sol-bindings}/* $outdir

	          echo "Generation successful!"
	        '';
        };
      };
    };
}

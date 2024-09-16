_: {
  perSystem =
    {
      self',
      pkgs,
      system,
      config,
      crane,
      stdenv,
      ensureAtRepositoryRoot,
      mkCi,
      ...
    }:
    let
      generate-rust-sol-bindings-crate =
        (crane.buildWorkspaceMember {
          crateDirFromRoot = "tools/generate-rust-sol-bindings";
        }).packages.generate-rust-sol-bindings;

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
        buildInputs = [ pkgs.taplo ];
        buildPhase = ''
          mkdir $out

          ${generate-rust-sol-bindings-crate}/bin/generate-rust-sol-bindings \
            --cratedir ./out/ \
            ${self'.packages.evm-contracts}/out/IBCHandler.sol/IBCHandler.json \
            ${self'.packages.evm-contracts}/out/IBCClient.sol/IBCClient.json \
            ${self'.packages.evm-contracts}/out/IBCConnection.sol/IBCConnection.json \
            ${self'.packages.evm-contracts}/out/IBCChannel.sol/IBCChannel.json \
            ${self'.packages.evm-contracts}/out/IBCPacket.sol/IBCPacket.json \
            ${self'.packages.evm-contracts}/out/Glue.sol/Glue.json \
            ${self'.packages.evm-contracts}/out/ERC20.sol/ERC20.json \
            ${self'.packages.evm-contracts}/out/Relay.sol/UCS01Relay.json \
            ${self'.packages.evm-contracts}/out/CometblsClientV2.sol/CometblsClient.json \
            ${self'.packages.evm-contracts}/out/ILightClient.sol/ILightClient.json \
            ${self'.packages.evm-contracts}/out/Multicall.sol/Multicall.json

          ls -al ./out/

          # format and normalize comments in generated code
          # rustfmt --config normalize_comments=true --edition "2021" lib.rs

          cp -r ./out/* $out

          taplo format $out/Cargo.toml

          sed -i 's/version = "2"/workspace = true/g' $out/Cargo.toml

          # heredocs confuse me
          echo "[features]" >> $out/Cargo.toml
          echo "providers = []" >> $out/Cargo.toml

          taplo format $out/Cargo.toml
        '';
      };
    in
    {
      packages = {
        rust-sol-bindings = mkCi false rust-sol-bindings;

        generate-rust-sol-bindings = mkCi false (
          pkgs.writeShellApplication {
            name = "generate-rust-sol-bindings";
            runtimeInputs = [ rust-sol-bindings ];
            text = ''
              ${ensureAtRepositoryRoot}

              outdir="generated/rust/contracts"

              cp -r --no-preserve=mode ${rust-sol-bindings}/* $outdir

              echo "Generation successful!"
            '';
          }
        );
      };
    };
}

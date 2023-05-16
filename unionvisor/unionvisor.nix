{ self, ... }: {
  perSystem = { self', pkgs, system, config, inputs', ... }:
    let
      crane = rec {
        lib = self.inputs.crane.lib.${system};
        stable = lib.overrideToolchain self'.packages.rust-stable;
      };
      src = ./.;

      commonArgs = {
        inherit src;
        buildInputs = [ pkgs.pkg-config pkgs.openssl ]
          ++ (
          pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [
            Security
          ])
        );
        doCheck = false;
        cargoBuildCommand = "cargo build --release";
        PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
      };
      cargoArtifacts = crane.stable.buildDepsOnly commonArgs;

      unionvisor = crane.stable.buildPackage (commonArgs // { });

      mkBundle = name: versions: pkgs.linkFarm "union-bundle-${name}" ([
        {
          name = "unionvisor";
          path = "${unionvisor}/bin/unionvisor";
        }
      ] ++ map
        (version: {
          name = "bins/${version}/uniond";
          path = "${inputs'."${version}".packages.uniond}/bin/uniond";
        })
        versions);
    in
    {
      packages = {
        inherit unionvisor;
        rust-stable = inputs'.rust-overlay.packages.rust.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" ];
        };

        bundle-testnet = mkBundle "testnet" [ "v0.0.2" "v0.3.0" "v0.4.2" ];
        bundle-mainnet = mkBundle "mainnet" [ "v0.0.2" "v0.3.0" ];
      };

      checks = {
        inherit unionvisor;

        unionvisor-clippy = crane.stable.cargoClippy (commonArgs // {
          inherit cargoArtifacts;
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        });

        unionvisor-rustfmt = crane.stable.cargoFmt {
          inherit src;
        };

        # unionvisor-tests = crane.stable.cargoNextest (commonArgs // {
        #   inherit cargoArtifacts;
        #   partitions = 1;
        #   partitionType = "count";
        #   doCheck = true;
        #   preConfigureHooks = [
        #     "cp ${self'.packages.uniond}/bin/uniond $PWD/src/testdata/test_init_cmd/bins/genesis"
        #   ];
        # });
      };
    };
}

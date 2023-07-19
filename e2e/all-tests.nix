{ lib, withSystem, inputs, ... }: {
  flake.checks = lib.genAttrs [ "x86_64-linux" "aarch64-linux" ]
    (lib.flip withSystem ({ e2e, networks, pkgs, nixpkgs, crane, ... }:
      let
        e2e = crane.lib.buildPackage ({
          buildInputs = [ pkgs.pkg-config pkgs.openssl ];
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          src = crane.lib.cleanCargoSource ../tools/e2e/e2e;
          doCheck = false;
          cargoVendorDir = crane.lib.vendorCargoDeps { cargoLock = ../tools/e2e/e2e/Cargo.lock; };
        } // (crane.lib.crateNameFromCargoToml { cargoToml = ../tools/e2e/e2e/Cargo.toml; }));
      in
      {
        virtualisation-works = let name = "devnet"; in e2e.mkTest {
          inherit name;
          network = networks.devnet;
          testScript = ''
            ${name}.wait_for_unit("arion-${networks.devnet.project.name}")
          '';
        };
        relayer-e2e =
          let
            name = "relayer-e2e";
            nixos-lib = import "${nixpkgs}/nixos/lib" { };

            test = nixos-lib.runTest {
              inherit name;

              testScript = ''
                start_all()

                # match non-zero blocks
                union.wait_for_console_text("height=[1-9][0-9]*")
                sepolia.wait_for_console_text("Synced - slot: [1-9][0-9]*")

                union.wait_for_open_port(26657)
                sepolia.wait_for_open_port(8546)

                with open("output.log", "w") as file:
                  output = client.succeed("RUST_LOG=debug ${pkgs.lib.meta.getExe e2e} ws://union:26657/websocket ws://sepolia:8546 2>&1")
                  file.write(output)
              '';

              nodes = {
                union =
                  { pkgs, lib, ... }:
                  {
                    imports = [
                      inputs.arion.nixosModules.arion
                    ];
                    virtualisation = {
                    diskSize = 2048;
                    arion = {
                      backend = "docker";
                      projects.union.settings = networks.union;
                    };};
                  };

                sepolia =
                  { pkgs, lib, ... }:
                  {
                    imports = [
                      inputs.arion.nixosModules.arion
                    ];
                    virtualisation = {
                      diskSize = 4 * 1024;
                      arion = {
                        backend = "docker";
                        projects.sepolia.settings = networks.sepolia;
                      };
                    };
                  };

                client =
                  { pkgs, lib, ... }:
                  { };
              };
              hostPkgs = pkgs; # the Nixpkgs package set used outside the VMs
            };
          in
          # (pkgs.stdenv.mkDerivation
          #   {
          #     pname = name;
          #     version = "1.2.3";
          #     buildInputs = [ test ];
          #     src = ./.;
          #     doCheck = true;
          #     checkPhase = ''
          #       ls ${test}
          #     '';
          #     buildPhase = ''

          #       touch $out
          #     '';
          #     requiredSystemFeatures = [ "kvm" "nixos-test" ];
          #   }).driver;
          test;
      }));
}


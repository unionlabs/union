{ e2e, pkgs, crane, ... }:
let
  ensure-blocks = pkgs.lib.meta.getExe (crane.lib.buildPackage {
    inherit (crane.lib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; }) pname version;
    buildInputs = [ pkgs.pkg-config pkgs.openssl ];
    PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
    src = crane.lib.cleanCargoSource ./.;
    doCheck = false;
    cargoVendorDir = crane.lib.vendorCargoDeps { cargoLock = ./Cargo.lock; };
  });
in

e2e.mkTestWithDevnetSetup {
  name = "ensure-blocks";

  testScript = ''
    client.wait_until_succeeds('[[ $(curl http://sepolia:9596/eth/v2/beacon/blocks/head --fail --silent | ${pkgs.lib.meta.getExe pkgs.jq} \'.data.message.slot | tonumber > 0\') == "true" ]]')

    client.succeed("RUST_LOG=info ${ensure-blocks} ws://union:26657/websocket ws://sepolia:8546 |& tee output.txt")

    client.copy_from_vm("output.txt", "")
  '';

  nodes = {
    # empty node used to communicate with the other nodes
    client = _: { };
  };
}


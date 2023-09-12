{ lib, image, ... }:
{
  build.image = lib.mkForce image;
  service = {
    tty = true;
    stop_signal = "SIGINT";
    network_mode = "host";
    environment = {
      HUBBLE_URL = "http://localhost:8080/v1/graphql";
      HUBBLE_INDEXERS = ''{"type": "Tm", "url": "http://localhost:26657"}'';
      HUBBLE_SECRET = "secret";
      RUST_LOG = "WARN";
    };
    depends_on = {
      hasura = {
        condition = "service_healthy";
      };
      uniond-0 = { };
    };
  };
}

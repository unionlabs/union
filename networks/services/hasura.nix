{ pkgs, lib, ... }:
let
  hasura = pkgs.dockerTools.pullImage {
    imageName = "hasura/graphql-engine";
    imageDigest = "sha256:02577c5e456726cbe1888e377d21c464979ed888636047adf3aae678d9ab70e6";
    sha256 = "0dbc7rwr0nd599y6rdhrmhqbvbbi2s29l0695w877gi9m1hx7wx9";
    finalImageName = "hasura/graphql-engine";
    finalImageTag = "v2.33.1.cli-migrations-v3";
  };
  data-connector-agent = pkgs.dockerTools.pullImage {
    imageName = "hasura/graphql-data-connector";
    imageDigest = "sha256:8984a79636abce86e31512af50948b5ed26b97d8f5302f1dd2bcc94622348915";
    sha256 = "0jcq8agfzm281qpkp2d1f2n9jvmag4yldl997ik1yzqdv40ffs4r";
    finalImageName = "hasura/graphql-data-connector";
    finalImageTag = "v2.33.0";
  };
in
{
  hasura = {
    build.image = lib.mkForce hasura;
    service = {
      tty = true;
      stop_signal = "SIGINT";
      ports = [
        "8080:8080"
      ];
      environment = {
        HASURA_GRAPHQL_METADATA_DATABASE_URL = "postgres://postgres:postgrespassword@postgres:5432/default";
        PG_DATABASE_URL = "postgres://postgres:postgrespassword@postgres:5432/default";
        HASURA_GRAPHQL_ENABLE_CONSOLE = "true";
        HASURA_GRAPHQL_DEV_MODE = "true";
        HASURA_GRAPHQL_ADMIN_SECRET = "secret";
        HASURA_GRAPHQL_METADATA_DEFAULTS = ''{"backend_configs":{"dataconnector":{"athena":{"uri":"http://data-connector-agent:8081/api/v1/athena"},"mariadb":{"uri":"http://data-connector-agent:8081/api/v1/mariadb"},"mysql8":{"uri":"http://data-connector-agent:8081/api/v1/mysql"},"oracle":{"uri":"http://data-connector-agent:8081/api/v1/oracle"},"snowflake":{"uri":"http://data-connector-agent:8081/api/v1/snowflake"}}}}'';
      };
      volumes = [
        "${../../hubble/hasura/metadata}:/hasura-metadata"
        "${../../hubble/hasura/migrations}:/hasura-migrations"
      ];
      depends_on = {
        data-connector-agent = {
          condition = "service_healthy";
        };
      };
    };
  };
  data-connector-agent = {
    build.image = lib.mkForce data-connector-agent;
    service = {
      restart = "always";
      ports = [
        "8081:8081"
      ];
      healthcheck = {
        interval = "5s";
        retries = 8;
        timeout = "10s";
        test = [
          "CMD-SHELL"
          ''
            curl -f http://localhost:8081/api/v1/athena/health
          ''
        ];
      };
    };
  };
}

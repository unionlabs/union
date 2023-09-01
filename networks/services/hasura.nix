{ ... }:
{
  hasura.service = {
    image = "hasura/graphql-engine:v2.33.0";
    tty = true;
    stop_signal = "SIGINT";
    ports = [
      "8080:8080"
    ];
    environment = {
      HASURA_GRAPHQL_METADATA_DATABASE_URL = "postgres://postgres:postgrespassword@postgres:5432/postgres";
      PG_DATABASE_URL = "postgres://postgres:postgrespassword@postgres:5432/postgres";
      HASURA_GRAPHQL_ENABLE_CONSOLE = "true";
      HASURA_GRAPHQL_DEV_MODE = "true";
      HASURA_GRAPHQL_ADMIN_SECRET = "secret";
      HASURA_GRAPHQL_METADATA_DEFAULTS = ''{"backend_configs":{"dataconnector":{"athena":{"uri":"http://data-connector-agent:8081/api/v1/athena"},"mariadb":{"uri":"http://data-connector-agent:8081/api/v1/mariadb"},"mysql8":{"uri":"http://data-connector-agent:8081/api/v1/mysql"},"oracle":{"uri":"http://data-connector-agent:8081/api/v1/oracle"},"snowflake":{"uri":"http://data-connector-agent:8081/api/v1/snowflake"}}}}'';
    };
    depends_on = {
      data-connector-agent = {
        condition = "service_healthy";
      };
    };
  };
  data-connector-agent.service = {
    image = "hasura/graphql-data-connector:v2.33.0";
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
}


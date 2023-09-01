{ ... }:
{
  service = {
    image = "postgres:latest";
    tty = true;
    stop_signal = "SIGINT";
    ports = [
      "5432:5432"
    ];
    environment = {
      POSTGRES_PASSWORD = "postgrespassword";
      POSTGRES_DB = "default";
    };
  };
}

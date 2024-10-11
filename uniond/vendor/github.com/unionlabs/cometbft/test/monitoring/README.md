# localnet monitoring
Prometheus and Grafana server for localnet.

# How to run
```bash
docker compose up -d
```
This will start both Prometheus and Grafana.

# Details
This docker compose (`compose.yml`) creates a local Granafa and Prometheus server.
It is useful for local debugging and monitoring, especially for `localnet` setups.

Prometheus will connect to the host machine's ports for data. By default it will connect to 26670,26671,26672,26673
which are the prometheus ports defined for `node0`, `node1`, `node2`, `node3` in `make localnet-start`.

Ports and other settings can be changed in the `config-prometheus/prometheus.yml` file before startup.

You can access the Grafana web interface at `http://localhost:3000` and the Prometheus web interface at `http://localhost:9090`.

The default Grafana username and password is `admin`/`admin`. You will only need it if you want to change something. The
pre-loaded dashboards can be viewed without a password.

Data from Grafana and Prometheus end up in the `data-grafana` and `data-prometheus` folders on your host machine.
This allows you to stop and restart the servers without data loss. The folders are excluded from Git.

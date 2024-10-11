# Setup
Enable prometheus metrics in wasmd:

* Edit `$HOME/config/app.toml`
```toml
[telemetry]

# Enabled enables the application telemetry functionality. When enabled,
# an in-memory sink is also enabled by default. Operators may also enabled
# other sinks such as Prometheus.
enabled =true
# ...

# PrometheusRetentionTime, when positive, enables a Prometheus metrics sink.
prometheus-retention-time = 15
```

`retention-time` must be >0 (see prometheus scrape config)


* Edit `$HOME/config/config.toml`
```toml
[instrumentation]

# When true, Prometheus metrics are served under /metrics on
# PrometheusListenAddr.
# Check out the documentation for the list of available metrics.
prometheus = true
```

Test manually at:
`http://localhost:1317/metrics?format=prometheus`

Note the `format` parameter in the request for the endpoint:


# Local testing
## Run Prometheus
```sh
# port 9090 is used by wasmd already
docker run -it -v $(pwd)/scripts/contrib/prometheus:/prometheus  -p9091:9090  prom/prometheus --config.file=/prometheus/prometheus.yaml
```
* Open [console](http://localhost:9091) and find `wasm_`service metrics

## Run Grafana

```shell
docker run -it -p 3000:3000 grafana/grafana
```
* Add Prometheus data source
`http://host.docker.internal:9091`
### Labels
* `wasm_contract_create` = nanosec
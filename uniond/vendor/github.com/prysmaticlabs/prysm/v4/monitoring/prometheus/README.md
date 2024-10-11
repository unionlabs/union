# How to monitor with prometheus

## Prerequisites:
 - [Prometheus](https://prometheus.io/docs/prometheus/latest/getting_started/) (Install to scrap metrics and start to monitor)
 - (optional) [Grafana](https://grafana.com/grafana/download) (For better graphs)
 - (optional) [Setup prometheus+grafana](https://prometheus.io/docs/visualization/grafana/)

## Start scrapping services
To start scrapping with prometheus you must create or edit the prometheus config file and add all the services you want to scrap, like these:

```diff
global:
  scrape_interval:     15s # By default, scrape targets every 15 seconds.

  # Attach these labels to any time series or alerts when communicating with
  # external systems (federation, remote storage, Alertmanager).
  external_labels:
    monitor: 'codelab-monitor'

# A scrape configuration containing exactly one endpoint to scrape:
# Here it's Prometheus itself.
scrape_configs:
  # The job name is added as a label `job=<job_name>` to any timeseries scraped from this config.
  - job_name: 'prometheus'

    # Override the global default and scrape targets from this job every 5 seconds.
    scrape_interval: 5s

    static_configs:
      - targets: ['localhost:9090']
+  - job_name: 'beacon-chain'
+    static_configs:
+      - targets: ['localhost:8080']
```

After creating/updating the prometheus file run it:
```sh
$ prometheus --config.file=your-prometheus-file.yml
```

Now, you can add the prometheus server as a data source on grafana and start building your dashboards.

## How to add additional metrics

The prometheus service export the metrics from the `DefaultRegisterer` so just need to register your metrics with the `prometheus` or `promauto` libraries.
To know more [Go application guide](https://prometheus.io/docs/guides/go-application/)

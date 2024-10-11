---
sidebar_position: 1
---

# Telemetry

:::note Synopsis
Gather relevant insights about your application and modules with custom metrics and telemetry.
:::

The Cosmos SDK enables operators and developers to gain insight into the performance and behavior of
their application through the use of the `telemetry` package. To enable telemetrics, set `telemetry.enabled = true` in the app.toml config file.

The Cosmos SDK currently supports enabling in-memory and prometheus as telemetry sinks. In-memory sink is always attached (when the telemetry is enabled) with 10 second interval and 1 minute retention. This means that metrics will be aggregated over 10 seconds, and metrics will be kept alive for 1 minute.

To query active metrics (see retention note above) you have to enable API server (`api.enabled = true` in the app.toml). Single API endpoint is exposed: `http://localhost:1317/metrics?format={text|prometheus}` (or port `1318` in v2) , the default being `text`.

## Emitting metrics

If telemetry is enabled via configuration, a single global metrics collector is registered via the
[go-metrics](https://github.com/hashicorp/go-metrics) library. This allows emitting and collecting
metrics through simple [API](https://github.com/cosmos/cosmos-sdk/blob/v0.50.10/telemetry/wrapper.go). Example:

```go
func EndBlocker(ctx sdk.Context, k keeper.Keeper) {
  start := telemetry.Now()
  defer telemetry.ModuleMeasureSince(types.ModuleName, start, telemetry.MetricKeyEndBlocker)

  // ...
}
```

Developers may use the `telemetry` package directly, which provides wrappers around metric APIs
that include adding useful labels, or they must use the `go-metrics` library directly. It is preferable
to add as much context and adequate dimensionality to metrics as possible, so the `telemetry` package
is advised. Regardless of the package or method used, the Cosmos SDK supports the following metrics
types:

* gauges
* summaries
* counters

## Labels

Certain components of modules will have their name automatically added as a label (e.g. `BeginBlock`).
Operators may also supply the application with a global set of labels that will be applied to all
metrics emitted using the `telemetry` package (e.g. chain-id). Global labels are supplied as a list
of [name, value] tuples.

Example:

```toml
global-labels = [
  ["chain_id", "chain-OfXo4V"],
]
```

## Cardinality

Cardinality is key, specifically label and key cardinality. Cardinality is how many unique values of
something there are. So there is naturally a tradeoff between granularity and how much stress is put
on the telemetry sink in terms of indexing, scrape, and query performance.

Developers should take care to support metrics with enough dimensionality and granularity to be
useful, but not increase the cardinality beyond the sink's limits. A general rule of thumb is to not
exceed a cardinality of 10.

Consider the following examples with enough granularity and adequate cardinality:

* begin/end blocker time
* tx gas used
* block gas used

The following examples expose too much cardinality and may not even prove to be useful:

* transfers between accounts with amount
* voting/deposit amount from unique addresses

## Idempotency

Metrics aren't idempotent, so if a metric is emitted twice, it will be counted twice.
This is important to keep in mind when collecting metrics. If a module is called twice, the metrics will be emitted twice (for instance in `CheckTx`, `SimulateTx` or `DeliverTx`).

## Supported Metrics

| Metric              | Description                                                                    | Unit | Type    |
| ------------------- | ------------------------------------------------------------------------------ | ---- | ------- |
| `tx_count`          | Total number of txs processed via `DeliverTx`                                  | tx   | counter |
| `tx_successful`     | Total number of successful txs processed via `DeliverTx`                       | tx   | counter |
| `tx_failed`         | Total number of failed txs processed via `DeliverTx`                           | tx   | counter |
| `tx_gas_used`       | The total amount of gas used by a tx                                           | gas  | gauge   |
| `tx_gas_wanted`     | The total amount of gas requested by a tx                                      | gas  | gauge   |
| `store_iavl_get`    | Duration of an IAVL `Store#Get` call                                           | ms   | summary |
| `store_iavl_set`    | Duration of an IAVL `Store#Set` call                                           | ms   | summary |
| `store_iavl_has`    | Duration of an IAVL `Store#Has` call                                           | ms   | summary |
| `store_iavl_delete` | Duration of an IAVL `Store#Delete` call                                        | ms   | summary |
| `store_iavl_commit` | Duration of an IAVL `Store#Commit` call                                        | ms   | summary |
| `store_iavl_query`  | Duration of an IAVL `Store#Query` call                                         | ms   | summary |
| `begin_blocker`     | Duration of the `BeginBlock` call per module                                   | ms   | summary |
| `end_blocker`       | Duration of the `EndBlock` call per module                                     | ms   | summary |
| `server_info`       | Information about the server, such as version, commit, and build date, upgrade | -    | gauge   |

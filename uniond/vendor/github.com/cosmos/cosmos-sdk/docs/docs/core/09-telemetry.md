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

To query active metrics (see retention note above) you have to enable API server (`api.enabled = true` in the app.toml). Single API endpoint is exposed: `http://localhost:1317/metrics?format={text|prometheus}`, the default being `text`.

## Emitting metrics

If telemetry is enabled via configuration, a single global metrics collector is registered via the
[go-metrics](https://github.com/armon/go-metrics) library. This allows emitting and collecting
metrics through simple [API](https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/telemetry/wrapper.go). Example:

```go
func EndBlocker(ctx sdk.Context, k keeper.Keeper) {
  defer telemetry.ModuleMeasureSince(types.ModuleName, time.Now(), telemetry.MetricKeyEndBlocker)

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
* amount of tokens minted
* amount of accounts created

The following examples expose too much cardinality and may not even prove to be useful:

* transfers between accounts with amount
* voting/deposit amount from unique addresses

## Supported Metrics

| Metric                          | Description                                                                               | Unit            | Type    |
|:--------------------------------|:------------------------------------------------------------------------------------------|:----------------|:--------|
| `tx_count`                      | Total number of txs processed via `DeliverTx`                                             | tx              | counter |
| `tx_successful`                 | Total number of successful txs processed via `DeliverTx`                                  | tx              | counter |
| `tx_failed`                     | Total number of failed txs processed via `DeliverTx`                                      | tx              | counter |
| `tx_gas_used`                   | The total amount of gas used by a tx                                                      | gas             | gauge   |
| `tx_gas_wanted`                 | The total amount of gas requested by a tx                                                 | gas             | gauge   |
| `tx_msg_send`                   | The total amount of tokens sent in a `MsgSend` (per denom)                                | token           | gauge   |
| `tx_msg_withdraw_reward`        | The total amount of tokens withdrawn in a `MsgWithdrawDelegatorReward` (per denom)        | token           | gauge   |
| `tx_msg_withdraw_commission`    | The total amount of tokens withdrawn in a `MsgWithdrawValidatorCommission` (per denom)    | token           | gauge   |
| `tx_msg_delegate`               | The total amount of tokens delegated in a `MsgDelegate`                                   | token           | gauge   |
| `tx_msg_begin_unbonding`        | The total amount of tokens undelegated in a `MsgUndelegate`                               | token           | gauge   |
| `tx_msg_begin_begin_redelegate` | The total amount of tokens redelegated in a `MsgBeginRedelegate`                          | token           | gauge   |
| `tx_msg_ibc_transfer`           | The total amount of tokens transferred via IBC in a `MsgTransfer` (source or sink chain)  | token           | gauge   |
| `ibc_transfer_packet_receive`   | The total amount of tokens received in a `FungibleTokenPacketData` (source or sink chain) | token           | gauge   |
| `new_account`                   | Total number of new accounts created                                                      | account         | counter |
| `gov_proposal`                  | Total number of governance proposals                                                      | proposal        | counter |
| `gov_vote`                      | Total number of governance votes for a proposal                                           | vote            | counter |
| `gov_deposit`                   | Total number of governance deposits for a proposal                                        | deposit         | counter |
| `staking_delegate`              | Total number of delegations                                                               | delegation      | counter |
| `staking_undelegate`            | Total number of undelegations                                                             | undelegation    | counter |
| `staking_redelegate`            | Total number of redelegations                                                             | redelegation    | counter |
| `ibc_transfer_send`             | Total number of IBC transfers sent from a chain (source or sink)                          | transfer        | counter |
| `ibc_transfer_receive`          | Total number of IBC transfers received to a chain (source or sink)                        | transfer        | counter |
| `ibc_client_create`             | Total number of clients created                                                           | create          | counter |
| `ibc_client_update`             | Total number of client updates                                                            | update          | counter |
| `ibc_client_upgrade`            | Total number of client upgrades                                                           | upgrade         | counter |
| `ibc_client_misbehaviour`       | Total number of client misbehaviours                                                      | misbehaviour    | counter |
| `ibc_connection_open-init`      | Total number of connection `OpenInit` handshakes                                          | handshake       | counter |
| `ibc_connection_open-try`       | Total number of connection `OpenTry` handshakes                                           | handshake       | counter |
| `ibc_connection_open-ack`       | Total number of connection `OpenAck` handshakes                                           | handshake       | counter |
| `ibc_connection_open-confirm`   | Total number of connection `OpenConfirm` handshakes                                       | handshake       | counter |
| `ibc_channel_open-init`         | Total number of channel `OpenInit` handshakes                                             | handshake       | counter |
| `ibc_channel_open-try`          | Total number of channel `OpenTry` handshakes                                              | handshake       | counter |
| `ibc_channel_open-ack`          | Total number of channel `OpenAck` handshakes                                              | handshake       | counter |
| `ibc_channel_open-confirm`      | Total number of channel `OpenConfirm` handshakes                                          | handshake       | counter |
| `ibc_channel_close-init`        | Total number of channel `CloseInit` handshakes                                            | handshake       | counter |
| `ibc_channel_close-confirm`     | Total number of channel `CloseConfirm` handshakes                                         | handshake       | counter |
| `tx_msg_ibc_recv_packet`        | Total number of IBC packets received                                                      | packet          | counter |
| `tx_msg_ibc_acknowledge_packet` | Total number of IBC packets acknowledged                                                  | acknowledgement | counter |
| `ibc_timeout_packet`            | Total number of IBC timeout packets                                                       | timeout         | counter |
| `store_iavl_get`                | Duration of an IAVL `Store#Get` call                                                      | ms              | summary |
| `store_iavl_set`                | Duration of an IAVL `Store#Set` call                                                      | ms              | summary |
| `store_iavl_has`                | Duration of an IAVL `Store#Has` call                                                      | ms              | summary |
| `store_iavl_delete`             | Duration of an IAVL `Store#Delete` call                                                   | ms              | summary |
| `store_iavl_commit`             | Duration of an IAVL `Store#Commit` call                                                   | ms              | summary |
| `store_iavl_query`              | Duration of an IAVL `Store#Query` call                                                    | ms              | summary |

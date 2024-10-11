# ADR 011: Monitoring

## Changelog

08-06-2018: Initial draft
11-06-2018: Reorg after @xla comments
13-06-2018: Clarification about usage of labels

## Context

In order to bring more visibility into Tendermint, we would like it to report
metrics and, maybe later, traces of transactions and RPC queries. See
https://github.com/tendermint/tendermint/issues/986.

A few solutions were considered:

1. [Prometheus](https://prometheus.io)
   a) Prometheus API
   b) [go-kit metrics package](https://github.com/go-kit/kit/tree/master/metrics) as an interface plus Prometheus
   c) [telegraf](https://github.com/influxdata/telegraf)
   d) new service, which will listen to events emitted by pubsub and report metrics
2. [OpenCensus](https://opencensus.io/introduction/)

### 1. Prometheus

Prometheus seems to be the most popular product out there for monitoring. It has
a Go client library, powerful queries, alerts.

**a) Prometheus API**

We can commit to using Prometheus in Tendermint, but I think Tendermint users
should be free to choose whatever monitoring tool they feel will better suit
their needs (if they don't have existing one already). So we should try to
abstract interface enough so people can switch between Prometheus and other
similar tools.

**b) go-kit metrics package as an interface**

metrics package provides a set of uniform interfaces for service
instrumentation and offers adapters to popular metrics packages:

https://godoc.org/github.com/go-kit/kit/metrics#pkg-subdirectories

Comparing to Prometheus API, we're losing customisability and control, but gaining
freedom in choosing any instrument from the above list given we will extract
metrics creation into a separate function (see "providers" in node/node.go).

**c) telegraf**

Unlike already discussed options, telegraf does not require modifying Tendermint
source code. You create something called an input plugin, which polls
Tendermint RPC every second and calculates the metrics itself.

While it may sound good, but some metrics we want to report are not exposed via
RPC or pubsub, therefore can't be accessed externally.

**d) service, listening to pubsub**

Same issue as the above.

### 2. opencensus

opencensus provides both metrics and tracing, which may be important in the
future. It's API looks different from go-kit and Prometheus, but looks like it
covers everything we need.

Unfortunately, OpenCensus go client does not define any
interfaces, so if we want to abstract away metrics we
will need to write interfaces ourselves.

### List of metrics

|     | Name                                 | Type   | Description                                                                   |
| --- | ------------------------------------ | ------ | ----------------------------------------------------------------------------- |
| A   | consensus_height                     | Gauge  |                                                                               |
| A   | consensus_validators                 | Gauge  | Number of validators who signed                                               |
| A   | consensus_validators_power           | Gauge  | Total voting power of all validators                                          |
| A   | consensus_missing_validators         | Gauge  | Number of validators who did not sign                                         |
| A   | consensus_missing_validators_power   | Gauge  | Total voting power of the missing validators                                  |
| A   | consensus_byzantine_validators       | Gauge  | Number of validators who tried to double sign                                 |
| A   | consensus_byzantine_validators_power | Gauge  | Total voting power of the byzantine validators                                |
| A   | consensus_block_interval             | Timing | Time between this and last block (Block.Header.Time)                          |
|     | consensus_block_time                 | Timing | Time to create a block (from creating a proposal to commit)                   |
|     | consensus_time_between_blocks        | Timing | Time between committing last block and (receiving proposal creating proposal) |
| A   | consensus_rounds                     | Gauge  | Number of rounds                                                              |
|     | consensus_prevotes                   | Gauge  |                                                                               |
|     | consensus_precommits                 | Gauge  |                                                                               |
|     | consensus_prevotes_total_power       | Gauge  |                                                                               |
|     | consensus_precommits_total_power     | Gauge  |                                                                               |
| A   | consensus_num_txs                    | Gauge  |                                                                               |
| A   | mempool_size                         | Gauge  |                                                                               |
| A   | consensus_total_txs                  | Gauge  |                                                                               |
| A   | consensus_block_size                 | Gauge  | In bytes                                                                      |
| A   | p2p_peers                            | Gauge  | Number of peers node's connected to                                           |

`A` - will be implemented in the fist place.

**Proposed solution**

## Status

Implemented

## Consequences

### Positive

Better visibility, support of variety of monitoring backends

### Negative

One more library to audit, messing metrics reporting code with business domain.

### Neutral

-

---
order: 1
parent:
  title: Tendermint Core QA Results v0.37.x
  description: This is a report on the results obtained when running TM v0.37.x on testnets
  order: 4
---

# Tendermint Core QA Results v0.37.x

## Table of Contents
- [Issues discovered](#issues-discovered)
- [200 Node Testnet](#200-node-testnet)
  - [Finding the Saturation Point](#finding-the-saturation-point)
  - [Examining latencies](#examining-latencies)
  - [Prometheus Metrics on the Chosen Experiment](#prometheus-metrics-on-the-chosen-experiment)
    - [Mempool Size](#mempool-size)
    - [Peers](#peers)
    - [Consensus Rounds per Height](#consensus-rounds-per-height)
    - [Blocks Produced per Minute, Transactions Processed per Minute](#blocks-produced-per-minute-transactions-processed-per-minute)
    - [Memory Resident Set Size](#memory-resident-set-size)
    - [CPU utilization](#cpu-utilization)
  - [Test Result](#test-result)
- [Rotating Node Testnet](#rotating-node-testnet)
  - [Latencies](#latencies)
  - [Prometheus Metrics](#prometheus-metrics)
    - [Blocks and Transactions per minute](#blocks-and-transactions-per-minute)
    - [Peers](#peers-1)
    - [Memory Resident Set Size](#memory-resident-set-size-1)
    - [CPU utilization](#cpu-utilization-1)
  - [Test Result](#test-result-1)

## Issues discovered

During this iteration of the QA process, the following issues were found:

* (critical, fixed) [\#9533] - This bug caused full nodes to sometimes get stuck
  when blocksyncing, requiring a manual restart to unblock them. Importantly,
  this bug was also present in v0.34.x and the fix was also backported in
  [\#9534].
* (critical, fixed) [\#9539] - `loadtime` is very likely to include more than
  one "=" character in transactions, with is rejected by the e2e application.
* (critical, fixed) [\#9581] - Absent prometheus label makes CometBFT crash
  when enabling Prometheus metric collection
* (non-critical, not fixed) [\#9548] - Full nodes can go over 50 connected
  peers, which is not intended by the default configuration.
* (non-critical, not fixed) [\#9537] - With the default mempool cache setting,
  duplicated transactions are not rejected when gossipped and eventually flood
  all mempools. The 200 node testnets were thus run with a value of 200000 (as
  opposed to the default 10000)

## 200 Node Testnet

### Finding the Saturation Point

The first goal is to identify the saturation point and compare it with the baseline (v0.34.x).
For further details, see [this paragraph](CometBFT-QA-34.md#finding-the-saturation-point)
in the baseline version.

The following table summarizes the results for v0.37.x, for the different experiments
(extracted from file [`v037_report_tabbed.txt`](img37/200nodes_tm037/v037_report_tabbed.txt)).

The X axis of this table is `c`, the number of connections created by the load runner process to the target node.
The Y axis of this table is `r`, the rate or number of transactions issued per second.

|        |  c=1  |  c=2  |  c=4  |
| :---   | ----: | ----: | ----: |
| r=25   |  2225 | 4450  | 8900  |
| r=50   |  4450 | 8900  | 17800 |
| r=100  |  8900 | 17800 | 35600 |
| r=200  | 17800 | 35600 | 38660 |

For comparison, this is the table with the baseline version.

|        |  c=1  |  c=2  |  c=4  |
| :---   | ----: | ----: | ----: |
| r=25   |  2225 | 4450  | 8900  |
| r=50   |  4450 | 8900  | 17800 |
| r=100  |  8900 | 17800 | 35400 |
| r=200  | 17800 | 35600 | 37358 |

The saturation point is beyond the diagonal:

* `r=200,c=2`
* `r=100,c=4`

which is at the same place as the baseline. For more details on the saturation point, see
[this paragraph](CometBFT-QA-34.md#finding-the-saturation-point) in the baseline version.

The experiment chosen to examine Prometheus metrics is the same as in the baseline:
**`r=200,c=2`**.

The load runner's CPU load was negligible (near 0) when running `r=200,c=2`.

### Examining latencies

The method described [here](method.md) allows us to plot the latencies of transactions
for all experiments.

![all-latencies](img37/200nodes_tm037/v037_200node_latencies.png)

The data seen in the plot is similar to that of the baseline.

![all-latencies-bl](img34/v034_200node_latencies.png)

Therefore, for further details on these plots,
see [this paragraph](CometBFT-QA-34.md#examining-latencies) in the baseline version.

The following plot summarizes average latencies versus overall throughputs
across different numbers of WebSocket connections to the node into which
transactions are being loaded.

![latency-vs-throughput](img37/200nodes_tm037/v037_latency_throughput.png)

This is similar to that of the baseline plot:

![latency-vs-throughput-bl](img34/v034_latency_throughput.png)

### Prometheus Metrics on the Chosen Experiment

As mentioned [above](#finding-the-saturation-point), the chosen experiment is `r=200,c=2`.
This section further examines key metrics for this experiment extracted from Prometheus data.

#### Mempool Size

The mempool size, a count of the number of transactions in the mempool, was shown to be stable and homogeneous
at all full nodes. It did not exhibit any unconstrained growth.
The plot below shows the evolution over time of the cumulative number of transactions inside all full nodes' mempools
at a given time.

![mempool-cumulative](img37/200nodes_tm037/v037_r200c2_mempool_size.png)

The plot below shows evolution of the average over all full nodes, which oscillate between 1500 and 2000 outstanding transactions.

![mempool-avg](img37/200nodes_tm037/v037_r200c2_mempool_size_avg.png)

The peaks observed coincide with the moments when some nodes reached round 1 of consensus (see below).

**These plots yield similar results to the baseline**:

![mempool-cumulative-bl](img34/v034_r200c2_mempool_size.png)

![mempool-avg-bl](img34/v034_r200c2_mempool_size_avg.png)

#### Peers

The number of peers was stable at all nodes.
It was higher for the seed nodes (around 140) than for the rest (between 16 and 78).

![peers](img37/200nodes_tm037/v037_r200c2_peers.png)

Just as in the baseline, the fact that non-seed nodes reach more than 50 peers is due to #9548.

**This plot yields similar results to the baseline**:

![peers-bl](img34/v034_r200c2_peers.png)

#### Consensus Rounds per Height

Most heights took just one round, but some nodes needed to advance to round 1 at some point.

![rounds](img37/200nodes_tm037/v037_r200c2_rounds.png)

**This plot yields slightly better results than the baseline**:

![rounds-bl](img34/v034_r200c2_rounds.png)

#### Blocks Produced per Minute, Transactions Processed per Minute

The blocks produced per minute are the gradient of this plot.

![heights](img37/200nodes_tm037/v037_r200c2_heights.png)

Over a period of 2 minutes, the height goes from 477 to 524.
This results in an average of 23.5 blocks produced per minute.

The transactions processed per minute are the gradient of this plot.

![total-txs](img37/200nodes_tm037/v037_r200c2_total-txs.png)

Over a period of 2 minutes, the total goes from 64525 to 100125 transactions,
resulting in 17800 transactions per minute. However, we can see in the plot that
all transactions in the load are process long before the two minutes.
If we adjust the time window when transactions are processed (approx. 90 seconds),
we obtain 23733 transactions per minute.

**These plots yield similar results to the baseline**:

![heights-bl](img34/v034_r200c2_heights.png)

![total-txs](img34/v034_r200c2_total-txs.png)

#### Memory Resident Set Size

Resident Set Size of all monitored processes is plotted below.

![rss](img37/200nodes_tm037/v037_r200c2_rss.png)

The average over all processes oscillates around 380 MiB and does not demonstrate unconstrained growth.

![rss-avg](img37/200nodes_tm037/v037_r200c2_rss_avg.png)

**These plots yield similar results to the baseline**:

![rss-bl](img34/v034_r200c2_rss.png)

![rss-avg-bl](img34/v034_r200c2_rss_avg.png)

#### CPU utilization

The best metric from Prometheus to gauge CPU utilization in a Unix machine is `load1`,
as it usually appears in the
[output of `top`](https://www.digitalocean.com/community/tutorials/load-average-in-linux).

![load1](img37/200nodes_tm037/v037_r200c2_load1.png)

It is contained below 5 on most nodes.

**This plot yields similar results to the baseline**:

![load1](img34/v034_r200c2_load1.png)

### Test Result

**Result: PASS**

Date: 2022-10-14

Version: 1cf9d8e276afe8595cba960b51cd056514965fd1

## Rotating Node Testnet

We use the same load as in the baseline: `c=4,r=800`.

Just as in the baseline tests, the version of CometBFT used for these tests is affected by #9539.
See this paragraph in the [baseline report](CometBFT-QA-34.md#rotating-node-testnet) for further details.
Finally, note that this setup allows for a fairer comparison between this version and the baseline.

### Latencies

The plot of all latencies can be seen here.

![rotating-all-latencies](img37/200nodes_tm037/v037_rotating_latencies.png)

Which is similar to the baseline.

![rotating-all-latencies-bl](img34/v034_rotating_latencies_uniq.png)

Note that we are comparing against the baseline plot with _unique_
transactions. This is because the problem with duplicate transactions
detected during the baseline experiment did not show up for `v0.37`,
which is _not_ proof that the problem is not present in `v0.37`.

### Prometheus Metrics

The set of metrics shown here match those shown on the baseline (`v0.34`) for the same experiment.
We also show the baseline results for comparison.

#### Blocks and Transactions per minute

The blocks produced per minute are the gradient of this plot.

![rotating-heights](img37/200nodes_tm037/v037_rotating_heights.png)

Over a period of 4446 seconds, the height goes from 5 to 3323.
This results in an average of 45 blocks produced per minute,
which is similar to the baseline, shown below.

![rotating-heights-bl](img34/v034_rotating_heights.png)

The following two plots show only the heights reported by ephemeral nodes.
The second plot is the baseline plot for comparison.

![rotating-heights-ephe](img37/200nodes_tm037/v037_rotating_heights_ephe.png)

![rotating-heights-ephe-bl](img34/v034_rotating_heights_ephe.png)

By the length of the segments, we can see that ephemeral nodes in `v0.37`
catch up slightly faster.

The transactions processed per minute are the gradient of this plot.

![rotating-total-txs](img37/200nodes_tm037/v037_rotating_total-txs.png)

Over a period of 3852 seconds, the total goes from 597 to 267298 transactions in one of the validators,
resulting in 4154 transactions per minute, which is slightly lower than the baseline,
although the baseline had to deal with duplicate transactions.

For comparison, this is the baseline plot.

![rotating-total-txs-bl](img34/v034_rotating_total-txs.png)

#### Peers

The plot below shows the evolution of the number of peers throughout the experiment.

![rotating-peers](img37/200nodes_tm037/v037_rotating_peers.png)

This is the baseline plot, for comparison.

![rotating-peers-bl](img34/v034_rotating_peers.png)

The plotted values and their evolution are comparable in both plots.

For further details on these plots, see the baseline report.

#### Memory Resident Set Size

The average Resident Set Size (RSS) over all processes looks slightly more stable
on `v0.37` (first plot) than on the baseline (second plot).

![rotating-rss-avg](img37/200nodes_tm037/v037_rotating_rss_avg.png)

![rotating-rss-avg-bl](img34/v034_rotating_rss_avg.png)

The memory taken by the validators and the ephemeral nodes when they are up is comparable (not shown in the plots),
just as observed in the baseline.

#### CPU utilization

The plot shows metric `load1` for all nodes.

![rotating-load1](img37/200nodes_tm037/v037_rotating_load1.png)

![rotating-load1-bl](img34/v034_rotating_load1.png)

In both cases, it is contained under 5 most of the time, which is considered normal load.
The green line in the `v0.37` plot and the purple line in the baseline plot (`v0.34`)
correspond to the validators receiving all transactions, via RPC, from the load runner process.
In both cases, they oscillate around 5 (normal load). The main difference is that other
nodes are generally less loaded in `v0.37`.

### Test Result

**Result: PASS**

Date: 2022-10-10

Version: 155110007b9d8b83997a799016c1d0844c8efbaf

[\#9533]: https://github.com/tendermint/tendermint/pull/9533
[\#9534]: https://github.com/tendermint/tendermint/pull/9534
[\#9539]: https://github.com/tendermint/tendermint/issues/9539
[\#9548]: https://github.com/tendermint/tendermint/issues/9548
[\#9537]: https://github.com/tendermint/tendermint/issues/9537
[\#9581]: https://github.com/tendermint/tendermint/issues/9581

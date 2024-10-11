---
order: 2
---
# Time

CometBFT provides a Byzantine fault-tolerant source of time.

Time in CometBFT is defined with the [`Time`][spec-time] field of the
block [`Header`][spec-header].

## Properties

The Time produced by CometBFT satisfies the following properties:

- **Time Monotonicity**: time is monotonically increasing.  More precisely, given
  two block headers `H1` of height `h1` and `H2` of height `h2`,
  it is guaranteed that if `h2 > h1` then `H2.Time > H1.Time`.

- **Byzantine Fault Tolerance**: malicious nodes or nodes with inaccurate clocks should not be able
  to arbitrarily increase or decrease the block Time.
  In other words, the Time of blocks should be defined by correct nodes.

In addition, the Time produced by CometBFT is expected, by external observers, to provide:

- **Relation to real time**: block times bear some resemblance to real time.
  In other words, block times should represent, within some reasonable accuracy,
  the actual clock time at which blocks were produced.
  More formally, lets `t` be the clock time at which a block with header `H`
  was first proposed.
  Then there exists a, possibly unknown but reasonably small, bound `ACCURACY`
  so that `|H.Time - t| < ACCURACY`.

## Implementations

CometBFT implements two algorithms for computing block times:

- [BFT Time][bft-time]: the algorithm adopted in versions up to `v0.38.x`;
  available, in legacy mode, in version `v1.x`.

- [Proposer-Based Timestamps (PBTS)][pbts-spec]: introduced in version `v1.x`,
  as a replacement for BFT Time.

Users are strongly encouraged to adopt PBTS in new chains or switch to PBTS
when upgrading existing chains.

### Comparison

The table below compares BFT Time and PBTS algorithms in terms of the above enumerated properties:

| Algorithm | Time Monotonicity | Byzantine Fault Tolerance         | Relation to real time                                                                         |
|-----------|:-----------------:|:---------------------------------:|-----------------------------------------------------------------------------------------------|
| BFT Time  | Guaranteed        | Tolerates `< 1/3` Byzantine nodes | Best effort and **not guaranteed**.                                                           |
| PBTS      | Guaranteed        | Tolerates `< 2/3` Byzantine nodes | Guaranteed with `ACCURACY` determined by the consensus parameters `PRECISION` and `MSGDELAY`. |

Note that by Byzantine nodes we consider both malicious nodes, that purposely
try to increase or decrease block times, and nodes that produce or propose
inaccurate block times because they rely on inaccurate local clocks.

For more details, refer to the specification of [BFT Time][bft-time] and [Proposer-Based Timestamps][pbts-spec].

## Adopting PBTS

The Proposer-Based Timestamp (PBTS) algorithm is the recommended algorithm for
producing block times.

As of CometBFT `v1.x`, however, PBTS is not enabled by default, neither for new
chains using default values for genesis parameters, nor for chains upgrading to
newer CometBFT versions, for backwards compatibility reasons.

Enabling PBTS requires configuring some consensus parameters:

- From `SynchronyParams`, the `Precision` and `MessageDelay` parameters.
  They correspond, respectively, to the `PRECISION` and `MSGDELAY` parameters 
  adopted in the PBTS specification.
- From `FeatureParams`, the `PbtsEnableHeight` parameter, which defines the
  height from which PBTS will be adopted.
  While it is set to `0` (default) or in heights previous to
  `PbtsEnableHeight`, BFT Time is adopted.

Refer to the [consensus parameters specification][spec-params] for more details,
or to the [PBTS user documentation]() for a more pragmatic description of the
algorithm and recommendations on how to properly configure its parameters.

[spec-time]: ../core/data_structures.md#time
[spec-header]: ../core/data_structures.md#header
[bft-time]: ./bft-time.md
[pbts-spec]: ./proposer-based-timestamp/README.md
[spec-params]: ../core/data_structures.md#consensusparams
[pbts-doc]: https://github.com/cometbft/cometbft/blob/feature/pbts/docs/explanation/core/proposer-based-timestamps.md

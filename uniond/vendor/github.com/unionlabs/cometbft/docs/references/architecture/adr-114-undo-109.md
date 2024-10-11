# ADR 114: Partly Undo ADR 109 (Reduce Go API Surface)

## Changelog

- 2024-04-25: First draft (@adizere)

## Status

Accepted (PR [\#2897])

## Context

In [ADR 109] we have decided to internalize numerous Go APIs following the research and due diligence
in that ADR. This will take effect with CometBFT v1 release.

Prior to releasing v1 RC, we have found that our diligence was insufficient and several Go modules
that we internalize with ADR 109 would either (i) force a difficult upgrade on users or, worse, would
(ii) provoke some users to fork CometBFT.

The question in the present ADR is how to deal with the potential damage our internalized APIs will create
on users' codebases.

## Alternative Approaches

The following alternative approaches were considered.

1. Do nothing. This approach will make the upgrade to v1 very difficult and potentially lead to new forks of Comet.
2. Fully undo ADR 109. This approach will minimize disruption with v1 release, but will bring the CometBFT codebase
    into a state prior to the implementation of that ADR, i.e., if we do breaking changes in non-critical
    modules that will require major version bumps, which will encourage stagnation and slow uptake of new releases.

## Decision

We will partly undo ADR 109, by selectively re-exporting (i.e., make public) certain modules. For modules `state`
and `store`, we have made them public ([\#2610]) because this blocked the SDK upgrade to CometBFT v1.

To select additional modules that we will make public (again) we will follow this high-level strategy:

1. Identify all `/internal` modules that are being imported by open-source projects using a tool
    such as https://www.sourcegraph.com/search.
2. For each of these modules, categorize them by importance. There are 3 levels: _high, medium, low_.
    By 'importance' we mean "important for current or later modularization work in CometBFT."
3. For modules that have _high_ importance, we will:
   1. make the public,
   2. mark the module as deprecated,
   3. establish communication with the team(s) using that module to find a way in `v2` to make the package
       internal again with minimal user disruption.
4. For modules with _medium_ importance, we will:
   1. make them public,
   2. mark them as deprecated; the rationale is that these modules being public is unlikely to block us in
       the future, and if we find they do block us, we will internalize them in `v2` and follow the same
       approach as for _high_ importance.
5. For modules with _low_ importance, we will:
   1. If there is _no_ project using that module, then we keep it private, as decided in [ADR 109].
   2. If there are projects using the module, then there are two sub-cases to consider:
      - i) If the APIs in that module contain Comet-specific features, then we'll make the module
          public; the rationale is that otherwise we would encourage users to fork Comet.
      - ii) If the APIs in that module comprise general-purpose features, then keep the module private;
          the rationale is that such modules have replacements and users will find it easy to replace
          them (e.g. rand number generation, file manipulation, synchronization primitives).

We will present these decisions to the community call, and we will err on the side of
_exposing more_ (i.e., making public) rather than retaining modules as private when there is
ambiguity around the decision for a certain module.

## Detailed Design

### Module Inventory

The following table contains our research, categorization by importance, and decision for each module
in the current `internal` -- as of [v1.0.0-alpha.2] -- directory of CometBFT.

Column legend:
* Comet internal module name: The name of the module
* Decision: The decision we are taking (either make public, or keep private) for this module
* \# Repositories affected \(non-forks\): Count of how many public, open-source projects we have identified that are using APIs from this module
* Affected files: How many files (among the affected repositories, both forks and non-forks) would be affected if we make this module private; this is rough measure of the impact -- or "damage" -- of making the module private
* Importance: Our assessment of how important is it that we make this module (eventually) private
* URL: The public source of data we have used to research the data in this table

| Comet internal module name | Decision           | \# Repositories affected \(non-forks\) | Affected files | Importance | URL               |
|:---------------------------|:-------------------|:---------------------------------------|:---------------|:-----------|:------------------|
| timer                      | keep private       | 0                                      | 7              | low        | [timer-url]       |
| progressbar                | keep private       | 0                                      | 4              | low        | [progressbar-url] |
| inspect                    | keep private       | 0                                      | 0              | low        | [inspect-url]     |
| fail                       | keep private       | 0                                      | 10             | low        | [fail-url]        |
| events                     | keep private       | 0                                      | 10             | low        | [events-url]      |
| cmap                       | keep private       | 0                                      | 10             | low        | [cmap-url]        |
| autofile                   | keep private       | 0                                      | 10             | low        | [autofile-url]    |
| async                      | keep private       | 0                                      | 8              | low        | [async-url]       |
| flowrate                   | keep private       | 0                                      | 10             | low        | [flowrate-url]    |
| bits                       | keep private       | 0                                      | 26             | low        | [bits-url]        |
| blocksync                  | keep private       | 0                                      | 6              | low        | [blocksync-url]   |
| clist                      | keep private       | 0                                      | 24             | low        | [clist-url]       |
| indexer                    | keep private       | 0                                      | 0              | low        | [indexer-url]     |
| net                        | keep private       | 1                                      | 40             | low        | [net-url]         |
| statesync                  | **🧹 make public** | 1                                      | 16             | medium     | [statesync-url]   |
| evidence                   | keep private       | 1                                      | 26             | high       | [evidence-url]    |
| consensus                  | keep private       | 1                                      | 64             | high       | [consensus-url]   |
| protoio                    | **🧹 make public** | 3                                      | 44             | low        | [protoio-url]     |
| sync                       | **🧹 make public** | 3                                      | 172            | low        | [sync-url]        |
| tempfile                   | keep private       | 4                                      | 16             | low        | [tempfile-url]    |
| strings                    | keep private       | 4                                      | 14             | low        | [strings-url]     |
| service                    | **🧹 make public** | 6                                      | 156            | low        | [service-url]     |
| os                         | keep private       | 7                                      | 262            | low        | [os-url]          |
| rand                       | keep private       | 7                                      | 317            | low        | [rand-url]        |
| pubsub                     | **🧹 make public** | 7                                      | 169            | medium     | [pubsub-url]      |

#### Remarks on the table

For `evidence` and `consensus`: There is a single project we have identified using APIs from these modules,
specifically <https://github.com/forbole/juno>. The maintainers of this project have agreed it is not a problem
for them if we keep the two modules private.

### Summary

To summarize, these modules will remain public in v1 and marked as deprecated:
- `statesync`
- `protoio`
- `sync`
- `service`
- `pubsub`

For these four modules which are becoming private in v1, we need to be extra-careful by helping
users transition to other general-purpose libraries:
- `tempfile`
- `strings`
- `os`
- `rand`

## Consequences

### Positive

- A smaller, more manageable Go API surface area.
- Less aggressive progression towards the goals set out in ADR 109.

### Negative

- Some power users may experience breakages. If absolutely necessary, certain packages
  can be moved back out of the `internal` directory in subsequent minor
  releases.

[\#2897]: https://github.com/cometbft/cometbft/pull/2897
[\#2610]: https://github.com/cometbft/cometbft/issues/2610
[ADR 109]: adr-109-reduce-go-api-surface.md
[v1.0.0-alpha.2]: https://github.com/cometbft/cometbft/releases/tag/v1.0.0-alpha.2
[timer-url]: https://sourcegraph.com/search?q=context:global+lang:Go+"github.com/cometbft/cometbft/libs/timer"&patternType=keyword&sm=0
[progressbar-url]: https://sourcegraph.com/search?q=context:global+lang:Go+"github.com/cometbft/cometbft/libs/progressbar"&patternType=keyword&sm=0
[inspect-url]: https://sourcegraph.com/search?q=context:global+lang:Go+"github.com/cometbft/cometbft/inspect"&patternType=keyword&sm=0
[fail-url]: https://sourcegraph.com/search?q=context:global+lang:Go+"github.com/cometbft/cometbft/libs/fail"&patternType=keyword&sm=0
[events-url]: https://sourcegraph.com/search?q=context:global+lang:Go+"github.com/cometbft/cometbft/libs/events"&patternType=keyword&sm=0
[cmap-url]: https://sourcegraph.com/search?q=context:global+lang:Go+"github.com/cometbft/cometbft/libs/cmap"&patternType=keyword&sm=0
[autofile-url]: https://sourcegraph.com/search?q=context:global+lang:Go+"github.com/cometbft/cometbft/libs/autofile"&patternType=keyword&sm=0
[async-url]: https://sourcegraph.com/search?q=context:global+lang:Go+"github.com/cometbft/cometbft/libs/async"&patternType=keyword&sm=0
[flowrate-url]: https://sourcegraph.com/search?q=context:global+lang:Go+"github.com/cometbft/cometbft/libs/flowrate"&patternType=keyword&sm=0
[bits-url]: https://sourcegraph.com/search?q=context:global+lang:Go+%22github.com/cometbft/cometbft/libs/bits%22&patternType=keyword&sm=0
[blocksync-url]: https://sourcegraph.com/search?q=context:global+lang:Go+%22github.com/cometbft/cometbft/blocksync%22&patternType=keyword&sm=0
[clist-url]: https://sourcegraph.com/search?q=context:global+lang:Go+%22github.com/cometbft/cometbft/libs/clist%22&patternType=keyword&sm=0
[net-url]: https://sourcegraph.com/search?q=context:global+lang:Go+%22github.com/cometbft/cometbft/libs/net%22&patternType=keyword&sm=0
[statesync-url]: https://sourcegraph.com/search?q=context:global+lang:Go+%22github.com/cometbft/cometbft/statesync%22&patternType=keyword&sm=0
[evidence-url]: https://sourcegraph.com/search?q=context:global+lang:Go+%22github.com/cometbft/cometbft/evidence%22&patternType=keyword&sm=0
[consensus-url]: https://sourcegraph.com/search?q=context:global+lang:Go+%22github.com/cometbft/cometbft/consensus%22&patternType=keyword&sm=0
[indexer-url]: https://sourcegraph.com/search?q=context:global+lang:Go+%22github.com/cometbft/cometbft/indexer%22&patternType=keyword&sm=0
[protoio-url]: https://sourcegraph.com/search?q=context:global+lang:Go+"github.com/cometbft/cometbft/libs/protoio"&patternType=keyword&sm=0
[sync-url]: https://sourcegraph.com/search?q=context:global+lang:Go+%22github.com/cometbft/cometbft/libs/sync%22&patternType=keyword&sm=0
[tempfile-url]: https://sourcegraph.com/search?q=context:global+lang:Go+"github.com/cometbft/cometbft/libs/tempfile"&patternType=keyword&sm=0
[strings-url]: https://sourcegraph.com/search?q=context:global+lang:Go+%22github.com/cometbft/cometbft/libs/strings%22&patternType=keyword&sm=0
[service-url]: https://sourcegraph.com/search?q=context:global+lang:Go+%22github.com/cometbft/cometbft/libs/service%22&patternType=keyword&sm=0
[os-url]: https://sourcegraph.com/search?q=context:global+lang:Go+%22github.com/cometbft/cometbft/libs/os%22&patternType=keyword&sm=0
[rand-url]: https://sourcegraph.com/search?q=context:global+lang:Go+%22github.com/cometbft/cometbft/libs/rand%22&patternType=keyword&sm=0
[pubsub-url]: https://sourcegraph.com/search?q=context:global+lang:Go+%22github.com/cometbft/cometbft/libs/pubsub%22&patternType=keyword&sm=0

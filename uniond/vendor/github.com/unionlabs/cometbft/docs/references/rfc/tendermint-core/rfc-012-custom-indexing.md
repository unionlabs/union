# RFC 012: Event Indexing Revisited

## Changelog

- 11-Feb-2022: Add terminological notes.
- 10-Feb-2022: Updated from review feedback.
- 07-Feb-2022: Initial draft (@creachadair)

## Abstract

A Tendermint node allows ABCI events associated with block and transaction
processing to be "indexed" into persistent storage.  The original Tendermint
implementation provided a fixed, built-in [proprietary indexer][kv-index] for
such events.

In response to user requests to customize indexing, [ADR 065][adr065]
introduced an "event sink" interface that allows developers (at least in
theory) to plug in alternative index storage.

Although ADR-065 was a good first step toward customization, its implementation
model does not satisfy all the user requirements.  Moreover, this approach
leaves some existing technical issues with indexing unsolved.

This RFC documents these concerns, and discusses some potential approaches to
solving them.  This RFC does _not_ propose a specific technical decision. It is
meant to unify and focus some of the disparate discussions of the topic.


## Background

We begin with some important terminological context.  The term "event" in
Tendermint can be confusing, as the same word is used for multiple related but
distinct concepts:

1. **ABCI Events** refer to the key-value metadata attached to blocks and
   transactions by the application. These values are represented by the ABCI
   `Event` protobuf message type.

2. **Consensus Events** refer to the data published by the Tendermint node to
   its pubsub bus in response to various consensus state transitions and other
   important activities, such as round updates, votes, transaction delivery,
   and block completion.

This confusion is compounded because some "consensus event" values also have
"ABCI event" metadata attached to them. Notably, block and transaction items
typically have ABCI metadata assigned by the application.

Indexers and RPC clients subscribed to the pubsub bus receive **consensus
events**, but they identify which ones to care about using query expressions
that match against the **ABCI events** associated with them.

In the discussion that follows, we will use the term **event item** to refer to
a datum published to or received from the pubsub bus, and **ABCI event** or
**event metadata** to refer to the key/value annotations.

**Indexing** in this context means recording the association between certain
ABCI metadata and the blocks or transactions they're attached to. The ABCI
metadata typically carry application-specific details like sender and recipient
addresses, category tags, and so forth, that are not part of consensus but are
used by UI tools to find and display transactions of interest.

The consensus node records the blocks and transactions as part of its block
store, but does not persist the application metadata. Metadata persistence is
the task of the indexer, which can be (optionally) enabled by the node
operator.

### History

The [original indexer][kv-index] built in to Tendermint stored index data in an
embedded [`tm-db` database][tmdb] with a proprietary key layout.
In [ADR 065][adr065], we noted that this implementation has both performance
and scaling problems under load.  Moreover, the only practical way to query the
index data is via the [query filter language][query] used for event
subscription.  [Issue #1161][i1161] appears to be a motivational context for that ADR.

To mitigate both of these concerns, we introduced the [`EventSink`][esink]
interface, combining the original transaction and block indexer interfaces
along with some service plumbing.  Using this interface, a developer can plug
in an indexer that uses a more efficient storage engine, and provides a more
expressive query language.  As a proof-of-concept, we built a [PostgreSQL event
sink][psql] that exports data to a [PostgreSQL database][postgres].

Although this approach addressed some of the immediate concerns, there are
several issues for custom indexing that have not been fully addressed. Here we
will discuss them in more detail.

For further context, including links to user reports and related work, see also
the [Pluggable custom event indexing tracking issue][i7135] issue.

### Issue 1: Tight Coupling

The `EventSink` interface supports multiple implementations, but plugging in
implementations still requires tight integration with the node. In particular:

- Any custom indexer must either be written in Go and compiled in to the
  Tendermint binary, or the developer must write a Go shim to communicate with
  the implementation and build that into the Tendermint binary.

- This means to support a custom indexer, it either has to be integrated into
  the Tendermint core repository, or every installation that uses that indexer
  must fetch or build a patched version of Tendermint.

The problem with integrating indexers into Tendermint Core is that every user
of Tendermint Core takes a dependency on all supported indexers, including
those they never use. Even if the unused code is disabled with build tags,
users have to remember to do this or potentially be exposed to security issues
that may arise in any of the custom indexers. This is a risk for Tendermint,
which is a trust-critical component of all applications built on it.

The problem with _not_ integrating indexers into Tendermint Core is that any
developer who wants to use a particular indexer must now fetch or build a
patched version of the core code that includes the custom indexer. Besides
being inconvenient, this makes it harder for users to upgrade their node, since
they need to either re-apply their patches directly or wait for an intermediary
to do it for them.

Even for developers who have written their applications in Go and link with the
consensus node directly (e.g., using the [Cosmos SDK][sdk]), these issues add a
potentially significant complication to the build process.

### Issue 2: Legacy Compatibility

The `EventSink` interface retains several limitations of the original
proprietary indexer. These include:

- The indexer has no control over which event items are reported. Only the
  exact block and transaction events that were reported to the original indexer
  are reported to a custom indexer.

- The interface requires the implementation to define methods for the legacy
  search and query API. This requirement comes from the integration with the
  [event subscription RPC API][event-rpc], but actually supporting these
  methods is not trivial.

At present, only the original KV indexer implements the query methods. Even the
proof-of-concept PostgreSQL implementation simply reports errors for all calls
to these methods.

Even for a plugin written in Go, implementing these methods "correctly" would
require parsing and translating the custom query language over whatever storage
platform the indexer uses.

For a plugin _not_ written in Go, even beyond the cost of integration the
developer would have to re-implement the entire query language.

### Issue 3: Indexing Delays Consensus

Within the node, indexing hooks in to the same internal pubsub dispatcher that
is used to export event items to the [event subscription RPC API][event-rpc].
In contrast with RPC subscribers, however, indexing is a "privileged"
subscriber: If an RPC subscriber is "too slow", the node may terminate the
subscription and disconnect the client. That means that RPC subscribers may
lose (miss) event items.  The indexer, however, is "unbuffered", and the
publisher will never drop or disconnect from it. If the indexer is slow, the
publisher will block until it returns, to ensure that no event items are lost.

In practice, this means that the performance of the indexer has a direct effect
on the performance of the consensus node: If the indexer is slow or stalls, it
will slow or halt the progress of consensus. Users have already reported this
problem even with the built-in indexer (see, for example, [#7247][i7247]).
Extending this concern to arbitrary user-defined custom indexers gives that
risk a much larger surface area.


## Discussion

It is not possible to simultaneously guarantee that publishing event items will
not delay consensus, and also that all event items of interest are always
completely indexed.

Therefore, our choice is between eliminating delay (and minimizing loss) or
eliminating loss (and minimizing delay).  Currently, we take the second
approach, which has led to user complaints about consensus delays due to
indexing and subscription overhead.

- If we agree that consensus performance supersedes index completeness, our
  design choices are to constrain the likelihood and frequency of missing event
  items.

- If we decide that consensus performance is more important than index
  completeness, our option is to minimize overhead on the event delivery path
  and document that indexer plugins constrain the rate of consensus.

Since we have user reports requesting both properties, we have to choose one or
the other.  Since the primary job of the consensus engine is to correctly,
robustly, reliablly, and efficiently replicate application state across the
network, I believe the correct choice is to favor consensus performance.

An important consideration for this decision is that a node does not index
application metadata separately: If indexing is disabled, there is no built-in
mechanism to go back and replay or reconstruct the data that an indexer would
have stored. The node _does_ store the blockchain itself (i.e., the blocks and
their transactions), so potentially some use cases currently handled by the
indexer could be handled by the node. For example, allowing clients to ask
whether a given transaction ID has been committed to a block could in principle
be done without an indexer, since it does not depend on application metadata.

Inevitably, a question will arise whether we could implement both strategies
and toggle between them with a flag. That would be a worst-case scenario,
requiring us to maintain the complexity of two very-different operational
concerns.  If our goal is that Tendermint should be as simple, efficient, and
trustworthy as possible, there is not a strong case for making these options
configurable: We should pick a side and commit to it.

### Design Principles

Although there is no unique "best" solution to the issues described above,
there are some specific principles that a solution should include:

1. **A custom indexer should not require integration into Tendermint core.** A
   developer or node operator can create, build, deploy, and use a custom
   indexer with a stock build of the Tendermint consensus node.

2. **Custom indexers cannot stall consensus.** An indexer that is slow or
   stalls cannot slow down or prevent core consensus from making progress.

   The plugin interface must give node operators control over the tolerances
   for acceptable indexer performance, and the means to detect when indexers
   are falling outside those tolerances, but indexer failures should "fail
   safe" with respect to consensus (even if that means the indexer may miss
   some data, in sufficiently-extreme circumstances).

3. **Custom indexers control which event items they index.** A custom indexer
   is not limited to only the current transaction and block events, but can
   observe any event item published by the node.

4. **Custom indexing is forward-compatible.** Adding new event item types or
   metadata to the consensus node should not require existing custom indexers
   to be rebuilt or modified, unless they want to take advantage of the new
   data.

5. **Indexers are responsible for answering queries.** An indexer plugin is not
   required to support the legacy query filter language, nor to be compatible
   with the legacy RPC endpoints for accessing them.  Any APIs for clients to
   query a custom index are the responsibility of the indexer, not the node.

### Open Questions

Given the constraints outlined above, there are important design questions we
must answer to guide any specific changes:

1. **What is an acceptable probability that, given sufficiently extreme
   operational issues, an indexer might miss some number of events?**

   There are two parts to this question: One is what constitutes an extreme
   operational problem, the other is how likely we are to miss some number of
   events items.

   - If the consensus is that no event item must ever be missed, no matter how
     bad the operational circumstances, then we _must_ accept that indexing can
     slow or halt consensus arbitrarily. It is impossible to guarantee complete
     index coverage without potentially unbounded delays.

   - Otherwise, how much data can we afford to lose and how often? For example,
     if we can ensure no event item will be lost unless the indexer halts for
     at least five minutes, is that acceptable? What probabilities and time
     ranges are reasonable for real production environments?

2. **What level of operational overhead is acceptable to impose on node
   operators to support indexing?**

   Are node operators willing to configure and run custom indexers as sidecar
   type processes alongside a node? How much indexer setup above and beyond the
   work of setting up the underlying node in isolation is tractable in
   production networks?

   The answer to this question also informs the question of whether we should
   keep an "in-process" indexing option, and to what extent that option needs
   to satisfy the suggested design principles.

   Relatedly, to what extent do we need to be concerned about the cost of
   encoding and sending event items to an external process (e.g., as JSON blobs
   or protobuf wire messages)? Given that the node already encodes event items
   as JSON for subscription purposes, the overhead would be negligible for the
   node itself, but the indexer would have to decode to process the results.

3. **What (if any) query APIs does the consensus node need to export,
   independent of the indexer implementation?**

   One typical example is whether the node should be able to answer queries
   like "is this transaction ID in a block?" Currently, a node cannot answer
   this query _unless_ it runs the built-in KV indexer. Does the node need to
   continue to support that query even for nodes that disable the KV indexer,
   or which use a custom indexer?

### Informal Design Intent

The design principles described above implicate several components of the
Tendermint node, beyond just the indexer. In the context of [ADR 075][adr075],
we are re-working the RPC event subscription API to improve some of the UX
issues discussed above for RPC clients. It is our expectation that a solution
for pluggable custom indexing will take advantage of some of the same work.

On that basis, the design approach I am considering for custom indexing looks
something like this (subject to refinement):

1. A custom indexer runs as a separate process from the node.

2. The indexer subscribes to event items via the ADR 075 events API.

   This means indexers would receive event payloads as JSON rather than
   protobuf, but since we already have to support JSON encoding for the RPC
   interface anyway, that should not increase complexity for the node.

3. The existing PostgreSQL indexer gets reworked to have this form, and no
   longer built as part of the Tendermint core binary.

   We can retain the code in the core repository as a proof-of-concept, or
   perhaps create a separate repository with contributed indexers and move it
   there.

4. (Possibly) Deprecate and remove the legacy KV indexer, or disable it by
   default.  If we decide to remove it, we can also remove the legacy RPC
   endpoints for querying the KV indexer.

   If we plan to do this, we should also investigate providing a way for
   clients to query whether a given transaction ID has landed in a block.  That
   serves a common need, and currently _only_ works if the KV indexer is
   enabled, but could be addressed more simply using the other data a node
   already has stored, without having to answer more general queries.


## References

- [ADR 065: Custom Event Indexing][adr065]
- [ADR 075: RPC Event Subscription Interface][adr075]
- [Cosmos SDK][sdk]
- [Event subscription RPC][event-rpc]
- [KV transaction indexer][kv-index]
- [Pluggable custom event indexing][i7135] (#7135)
- [PostgreSQL event sink][psql]
    - [PostgreSQL database][postgres]
- [Query filter language][query]
- [Stream events to postgres for indexing][i1161] (#1161)
- [Unbuffered event subscription slow down the consensus][i7247] (#7247)
- [`EventSink` interface][esink]
- [`tm-db` library][tmdb]

[adr065]: https://github.com/tendermint/tendermint/blob/main/docs/architecture/adr-065-custom-event-indexing.md
[adr075]: https://github.com/tendermint/tendermint/blob/main/docs/architecture/adr-075-rpc-subscription.md
[esink]: https://pkg.go.dev/github.com/tendermint/tendermint/internal/state/indexer#EventSink
[event-rpc]: https://docs.tendermint.com/v0.34/rpc/#/Websocket/subscribe
[i1161]: https://github.com/tendermint/tendermint/issues/1161
[i7135]: https://github.com/tendermint/tendermint/issues/7135
[i7247]: https://github.com/tendermint/tendermint/issues/7247
[kv-index]: https://github.com/tendermint/tendermint/blob/main/state/indexer/block/kv
[postgres]: https://postgresql.org/
[psql]: https://github.com/tendermint/tendermint/tree/main/state/indexer/sink/psql
[query]: https://pkg.go.dev/github.com/tendermint/tendermint/internal/pubsub/query/syntax
[sdk]: https://github.com/cosmos/cosmos-sdk
[tmdb]: https://pkg.go.dev/github.com/tendermint/tm-db#DB

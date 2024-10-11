# ADR 073: Built-in In-process Indexer

## Changelog

* 2024-06-03: Initial draft (@aaronc)

## Status

Accepted Not Implemented

## Abstract

This ADR proposes developing a built-in query indexer framework for Cosmos SDK applications that leverages `collections` and `orm` schemas to index on-chain state and events into a PostgreSQL database, or another database if applications prefer. This indexer should be designed to be run in-process with the Cosmos SDK node with guaranteed delivery and provide a full-featured query interface for clients.

## Context

Historically in Cosmos SDK applications, the needs of clients have been intertwined with the needs of state machine
logic. While the state machine logic itself may only require certain pieces of state data, if client applications
need additional data, this data has usually been added to the state. One example is adding additional secondary
indexes and query endpoints that are only needed for client queries, such as the bank reverse denom index. This
can add to state bloat and complicate the state machine logic.

While Cosmos SDK applications try their best to support client needs, the query experience provided by the gRPC
queries provided by modules is always sub-par compared to a real database dedicated to queries. On-chain queries
tend to be slow, and it is impossible to optimize query performance by adding additional indexes without burdening
on-chain state. Most sophisticated client applications build their own off-chain indexers to provide the queries they
need, but these are often highly application specific and not reusable. The Cosmos SDK has attempted to provide some
additional infrastructure to support indexers such as [ADR 038: State Listening](./adr-038-state-listening.md), but
this still requires the creation of lots of custom infrastructure.

Now that the [collections](./adr-062-collections-state-layer.md) and [orm](./adr-055-orm.md) frameworks exist and many SDK modules have been migrated to use them, we can do better by providing a built-in indexer that is easy to deploy, handles 90% of client query needs out of the box, and is easy to extend for the remaining 10% of needs.

In considering the design of built-in support, in discussions we have considered the needs of UI application developers, node operators and application and module developers. The following user stories have been identified as _ideal_ features, with the understanding that the SDK can likely only provide a subset of these features, but should aim for the best balance.

When considering the needs of developers building UI applications, the following desirable features have been identified:

* U1: quickly querying the current on-chain state with any sort of filtering, sorting and pagination the client desires with the ability to optimize queries with custom indexes.
* U2: data should be available in client friendly formats, for instance even if addresses are stored as bytes in state, clients should be available to view and query bech32 encoded addresses.
* U3: state that was pruned from on-chain state to save space but is still otherwise valid (such as completed proposals and votes) should be queryable.
* U4: a full log of changes for each entity should be available, including: 
    * U4.1: semantic information about each change (i.e. events), i.e. not just balance went up or down, but why (transfer, withdraw rewards, etc.)
    * U4.2: links to the block, transaction and message which produced the change
    * U4.3: the ability to see the version of the entity at the time of the change
* U5: the query index should be consistent, meaning that UI applications should not need to be concerned with the possibility that the index has missed indexing some blocks or events

When considering the needs of node operators, the following desirable features have been identified:

* N1: nodes shouldn't _need_ to store indexes or events that are only needed for client queries, i.e. it should be possible to run a lean node
* N2: it should be possible to quickly spin up a node with the query index without tons of custom infrastructure and coding, i.e. this should be strictly configuration change and a bit of devops. It should be possible to configure the index to contain just state, just events, or both.
* N3: it should be possible to build up an almost complete query index starting from any point in time without necessarily needing to replay a full chain's history (with the understanding that some historical data may be missing)

Finally, when considering the needs of application and module developers, the following desirable features have been identified:

* A1: enabling support for query indexing should require minimal tweaking to app boilerplate and configurable by node operators (off by default).
* A2: it should be possible to index applications built with older versions of the SDK (including v0.47 and v0.50, possibly earlier) without major changes.
* A3: module developers should mostly just need to use the collections and orm frameworks to get their data indexed, with minimal additional work.
* A4: there should be hooks available to module developers for extending the built-in indexing functionality.
* A5: the built-in indexer framework should allow newer modules to entirely or mostly skip building custom gRPC queries to satisfy client needs and modules should no longer need to add indexes only for client use, i.e. the indexer framework should essentially be a complete replacement for the gRPC query system (with inter-module query needs being considered a separate concern).

## Decision

We have decided to build a built-in query indexer for Cosmos SDK applications that is structured as two components:

1. a state decoder that takes data from `collections` and `orm` and provides an indexable version of that data to an actual indexer
2. a PostgreSQL-based indexer implementation that can be run in-process with the Cosmos SDK node

This infrastructure should be built upon the existing [ADR 038: State Listening](./adr-038-state-listening.md) functionality as much as possible and integrating it into existing apps should require minimal changes. These components should work with any SDK application that is built against v0.47 or v0.50 and possibly earlier versions of the SDK.

### State Decoder

The state decoder framework should expose a way for modules using `collections` or `orm` to expose their state schemas so that the state decoder can take data exposed by state listening and decode it into logical packets which can consumed by an indexer. It should define an interface that an indexer implements to consume these packets. This framework should be designed to run in-process within a Cosmos SDK node with guaranteed delivery and consistency (satisfying `U5`). While concurrency should be used to optimize performance, there should be a guarantee that if a block is committed, that it was also indexed. This framework should also allow indexers to consume block, transaction, and event data and optionally index these.

At its core, the state decoder framework shouldn't make any assumptions about what database the indexer is targeting. While a built-in indexer will be provided (discussed below), a developer may choose to write an indexer targeting another database and the state decoder framework should be flexible enough to support this.

The state decoder should provide hooks for handling custom data types (to support `U2`), in particular addresses so that indexers can store these in their bech32 string format when they are actually stored as bytes in state.

Some changes to `collections` will be needed in order to expose for decoding functionality and saner naming of map keys and values. These can likely be made in a non-breaking way and any features that allow for better key and value naming would be opt-in. For both `collections` and `orm`, we will need a lightweight way to expose these schemas on `AppModule`s.

To support `U3`, `collections` and `orm` can add "prune" methods that allow the indexer framework to distinguish pruning from deletion so that the query index could for instance retain historical proposals and votes from `x/gov` while these get deleted in state. Alternatively, configuration flags could be used to instruct the indexer to retain certain types of data - these could be configured at the module or node level.

In order to support indexing from any height (`N3`), the state decoder will need the ability to read the full state at the height at which indexing started and also keep track of which blocks have been indexed.

### PostgreSQL Indexer

PostgreSQL has been chosen as the target database for the built-in indexer component because it is widely used, has a rich feature set and is a favorite choice in the open-source community. It is easy to deploy and there are fully managed hosting services that can be used. In addition, the PostgreSQL community has a number of mature frameworks that expose a complete REST or GraphQL query interface for clients with zero code such as [PostgREST](https://postgrest.org/), [PostGraphile](https://www.graphile.org/postgraphile/), [Hasura](https://hasura.io) and [Supabase](https://supabase.com). By combining a PostgreSQL database with one of these "Backend as a Service" frameworks, Web 2.0 client applications can basically be built without any special backend code other than the database schema itself. We can take advantage of this functionality to provide a full-featured query interface for web clients with minimal effort as soon as we can get the data into PostgreSQL.

The PostgreSQL indexer should provide sane default mappings of all `collections` and `orm` types to SQL tables. It should also allow for hooks in modules to write migrations in the SQL schema whenever there are migrations in a module's state so that these can stay in sync.  

Blocks, transactions and events should be stored as rows in PostgreSQL tables when this is enabled by the node operator. This data should come directly from the state decoder framework without needing to go through Comet.

For a full batteries included, client friendly query experience, a GraphQL endpoint should be exposed in the HTTP server for any PostgreSQL database that has the [Supabase pg_graphql](https://github.com/supabase/pg_graphql) extension enabled. `pg_graphql` will expose rich GraphQL queries for all PostgreSQL tables with zero code that support filtering, pagination, sorting and traversing foreign key references. (Support for defining foreign keys with `collections` and `orm` could be added in the future to take advantage of this). In addition, a [GraphiQL](https://github.com/graphql/graphiql) query explorer endpoint can be exposed to simplify client development.

With this setup, a node operator would only need to 1) setup a PostgreSQL database with the `pg_graphql` extension and 2) enable the query indexer in the configuration in order to provide a full-featured query experience to clients. Because PostgreSQL is a full-featured database, node operators can enable any sort of custom indexes or views that are needed for their specific application with no need for this to affect the state machine or any other nodes.


## Alternatives

The following alternatives were considered:

* support any SQL database not just PostgreSQL using a framework like [GORM](https://gorm.io/). While this would be more flexible, it would be slower, require heavy usage of golang reflection and might limit how much we can take advantage of PostgreSQL's unique features for little benefit (the assumption being that most users would choose PostgreSQL anyway and or be happy enough that we made that choice).
* don't support any specific database, but just build the decoder framework. While this would simplify our efforts in the short-term, it still doesn't provide a full-featured solution and requires others to build out the key infrastructure similar to [ADR 038](adr-038-state-listening.md). This limbo state would not allow the SDK to definitely make key optimizations to state layout and simple the task of module development in a definitive way by providing a full replacement for gRPC client queries.
* target a database with full historical query support like [Datomic](https://www.datomic.com). This requires a bunch of custom infrastructure and would expose a powerful, yet unfamiliar query language to users.
* advocate an event sourcing design and build an event sourcing based indexer which would recompute state based on the event log. This is also discussed more below and is considered a complementary idea that can provide better support for historical change logs. Considering event sourcing as a full alternative to a state-based indexer, however, would require a lot of module refactoring, likely custom code, and wouldn't take advantage of the work we've already done in supporting state schemas through `collections` and `orm`.
* build a full-featured out-of-process indexer based on ADR 038 and changelog files. This was another design initially considered, but it requires additional infrastructure and processes. In particular, it also requires a full decodable schema for `collections` which at the moment is fairly complex. It is easier to use the `collections` schemas already in the binary to do indexing rather than create a whole schema definition language for a separate process to consume. Also we want to provide a more batteries-included experience for users and in particular satisfy `N2`. If creating a full query index is easier, it makes everyone's life easier.
* build a GraphQL client on top of the existing state store. This was considered, but it would be slow and not provide the full-featured query experience that a real database can provide. It would still require client only indexes in state, it would be hard to configure custom indexes and views, and would likely require building or reusing a full query planner. In the end, using a real database is easier to build and provides a better experience for clients.

## Consequences

### Backwards Compatibility

We believe that these features can be built to target SDK versions v0.47, v0.50 and possibly earlier without breaking changes. 

### Positive

Considering the user stories identified in the context section, we believe that the proposed design can meet all the user stories identified, except `U4`. Overall, the proposed design should provide a full query experience that is in most ways better than what is provided by the existing gRPC query infrastructure, is easy to deploy and manage, and easy to extend for custom needs. It also simplifies the job of writing a module because module developers mostly do not need to worry about writing query endpoints or other client concerns besides making sure that the design and naming of `collections` and `orm` schemas is client friendly.

Also, because we are separating the design into decoder and indexer components, it should be possible to write indexers targeting other databases besides PostgreSQL using the decoder framework. While the built-in PostgreSQL indexer should provide a good battery-included experience for most users, this design also supports users wanting to target other databases.

### Negative

If module developers choose to deprecate support for some or all gRPC queries then this will be a breaking change for clients. The resulting query experience should be better, but it will require significant changes. This ADR doesn't advocate deprecating these queries, but it could encourage developers to go in that direction. To mitigate this concern, we encourage module authors to consider the client developer impact as they make decisions about deleting existing indexes and queries.

Also, this design does impose the requirement that module developers use `collections` and `orm` to get their data indexed. While we believe that these frameworks are the best way to structure state in the SDK, some developers may not want to refactor their code to use them. This design does not provide a way to index state that is not structured with `collections` or `orm`, although that support could be added in the future to the decoder framework.

Furthermore, this design does require that node operators that want to support the new infrastructure run a PostgreSQL database. While we believe that this will actually be better from an infrastructure perspective - PostgreSQL should be able to serve queries significantly faster than the state machine can - it does require additional infrastructure and knowledge for query node operators.

### Neutral

Regarding `U4`, if event indexing is enabled, then it could be argued that `U4.1` is met, but whether this is _actually_ met depends heavily on how well modules structure their events. `U4.2` and `U4.3` likely require a full archive node which is out of scope of this design. One alternative which satisfies `U4.2` and `U4.3` would be targeting a database with historical data, such as [Datomic](https://www.datomic.com). However, this requires some pretty custom infrastructure and exposes a query interface which is unfamiliar for most users. Also, if events aren't properly structured, `U4.1` still really isn't met. A simpler alternative would be for module developers to follow an event sourcing design more closely so that historical state for any entity could be derived from the history of events. This event sourcing could even be done in client applications themselves by querying all the events relevant to an entity (such as a balance). This topic may be covered in more detail in a separate document in the future and may come down to best practices combined, with maybe a bit of framework support. However, in general satisfying `U4` (other than support event indexing) is mostly concerned out of the scope of the design, because it is either much more complex from an infrastructure perspective (full archive node or custom database like Datomic) or easy to solve with good event design.

## Further Discussions

Further discussions can take place on GitHub as needed.

## References

* [ADR 038: State Listening](./adr-038-state-listening.md)
* [ADR 055: ORM](./adr-055-orm.md)
* [ADR 062: Collections](./adr-062-collections-state-layer.md)

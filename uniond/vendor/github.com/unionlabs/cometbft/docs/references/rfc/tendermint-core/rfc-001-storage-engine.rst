===========================================
RFC 001: Storage Engines and Database Layer
===========================================

Changelog
---------

- 2021-04-19: Initial Draft (gist)
- 2021-09-02: Migrated to RFC folder, with some updates  

Abstract
--------

The aspect of Tendermint that's responsible for persistence and storage (often
"the database" internally) represents a bottle neck in the architecture of the
platform, that the 0.36 release presents a good opportunity to correct. The
current storage engine layer provides a great deal of flexibility that is
difficult for users to leverage or benefit from, while also making it harder
for Tendermint Core developers to deliver improvements on storage engine. This
RFC discusses the possible improvements to this layer of the system.

Background
----------

Tendermint has a very thin common wrapper that makes Tendermint itself
(largely) agnostic to the data storage layer (within the realm of the popular
key-value/embedded databases.) This flexibility is not particularly useful:
the benefits of a specific database engine in the context of Tendermint is not
particularly well understood, and the maintenance burden for multiple backends
is not commensurate with the benefit provided. Additionally, because the data
storage layer is handled generically, and most tests run with an in-memory
framework, it's difficult to take advantage of any higher-level features of a
database engine.

Ideally, developers within Tendermint will be able to interact with persisted
data via an interface that can function, approximately like an object
store, and this storage interface will be able to accommodate all existing
persistence workloads (e.g. block storage, local peer management information
like the "address book", crash-recovery log like the WAL.) In addition to
providing a more ergonomic interface and new semantics, by selecting a single
storage engine tendermint can use native durability and atomicity features of
the storage engine and simplify its own implementations. 

Data Access Patterns
~~~~~~~~~~~~~~~~~~~~

Tendermint's data access patterns have the following characteristics:

- aggregate data size often exceeds memory.

- data is rarely mutated after it's written for most data (e.g. blocks), but
  small amounts of working data is persisted by nodes and is frequently
  mutated (e.g. peer information, validator information.)

- read patterns can be quite random.

- crash resistance and crash recovery, provided by write-ahead-logs (in
  consensus, and potentially for the mempool) should allow the system to
  resume work after an unexpected shut down.

Project Goals
~~~~~~~~~~~~~

As we think about replacing the current persistence layer, we should consider
the following high level goals: 

- drop dependencies on storage engines that have a CGo dependency.

- encapsulate data format and data storage from higher-level services
  (e.g. reactors) within tendermint.

- select a storage engine that does not incur any additional operational
  complexity (e.g. database should be embedded.)

- provide database semantics with sufficient ACID, snapshots, and
  transactional support.

Open Questions
~~~~~~~~~~~~~~

The following questions remain:

- what kind of data-access concurrency does tendermint require?

- would tendermint users SDK/etc. benefit from some shared database
  infrastructure?
  
  - In earlier conversations it seemed as if the SDK has selected Badger and
    RocksDB for their storage engines, and it might make sense to be able to
    (optionally) pass a handle to a Badger instance between the libraries in
    some cases.

- what are typical data sizes, and what kinds of memory sizes can we expect
  operators to be able to provide?

- in addition to simple persistence, what kind of additional semantics would
  tendermint like to enjoy (e.g. transactional semantics, unique constraints,
  indexes, in-place-updates, etc.)?

Decision Framework
~~~~~~~~~~~~~~~~~~

Given the constraint of removing the CGo dependency, the decision is between
"badger" and "boltdb" (in the form of the etcd/CoreOS fork,) as low level. On
top of this and somewhat orthogonally, we must also decide on the interface to
the database and how the larger application will have to interact with the
database layer. Users of the data layer shouldn't ever need to interact with
raw byte slices from the database, and should mostly have the experience of
interacting with Go-types.

Badger is more consistently developed and has a broader feature set than
Bolt. At the same time, Badger is likely more memory intensive and may have
more overhead in terms of open file handles given it's model. At first glance,
Badger is the obvious choice: it's actively developed and it has a lot of
features that could be useful. Bolt is not without some benefits: it's stable
and is maintained by the etcd folks, it's simpler model (single memory mapped
file, etc,) may be easier to reason about.

I propose that we consider the following specific questions about storage
engines:

- does Badger's evolving development, which may result in data file format
  changes in the future, and could restrict our access to using the latest
  version of the library between major upgrades, present a problem?

- do we do we have goals/concerns about memory footprint that Badger may
  prevent us from hitting, particularly as data sets grow over time?

- what kind of additional tooling might we need/like to build (dump/restore,
  etc.)?

- do we want to run unit/integration tests against a data files on disk rather
  than relying exclusively on the memory database?

Project Scope
~~~~~~~~~~~~~

This project will consist of the following aspects:

- selecting a storage engine, and modifying the tendermint codebase to
  disallow any configuration of the storage engine outside of the tendermint. 

- remove the dependency on the current tm-db interfaces and replace with some
  internalized, safe, and ergonomic interface for data persistence with all
  required database semantics.

- update core tendermint code to use the new interface and data tools.

Next Steps
~~~~~~~~~~

- circulate the RFC, and discuss options with appropriate stakeholders. 
  
- write brief ADR to summarize decisions around technical decisions reached
  during the RFC phase. 

References
----------

- `bolddb <https://github.com/etcd-io/bbolt>`_
- `badger <https://github.com/dgraph-io/badger>`_
- `badgerdb overview <https://dbdb.io/db/badgerdb>`_
- `botldb overview <https://dbdb.io/db/boltdb>`_
- `boltdb vs badger <https://tech.townsourced.com/post/boltdb-vs-badger>`_
- `bolthold <https://github.com/timshannon/bolthold>`_
- `badgerhold <https://github.com/timshannon/badgerhold>`_
- `Pebble <https://github.com/cockroachdb/pebble>`_
- `SDK Issue Regarding IVAL <https://github.com/cosmos/cosmos-sdk/issues/7100>`_
- `SDK Discussion about SMT/IVAL <https://github.com/cosmos/cosmos-sdk/discussions/8297>`_

Discussion
----------

- All things being equal, my tendency would be to use badger, with badgerhold
  (if that makes sense) for its ergonomics and indexing capabilities, which
  will require some small selection of wrappers for better write transaction
  support. This is a weakly held tendency/belief and I think it would be
  useful for the RFC process to build consensus (or not) around this basic
  assumption.

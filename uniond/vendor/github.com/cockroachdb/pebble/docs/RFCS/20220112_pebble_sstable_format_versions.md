- Feature Name: Pebble SSTable Format Versions
- Status: completed
- Start Date: 2022-01-12
- Authors: Nick Travers
- RFC PR: https://github.com/cockroachdb/pebble/pull/1450
- Pebble Issues:
  https://github.com/cockroachdb/pebble/issues/1409
  https://github.com/cockroachdb/pebble/issues/1339
- Cockroach Issues:

# Summary

To safely support changes to the SSTable structure, a new versioning scheme
under a Pebble magic number is proposed.

This RFC also outlines the relationship between the SSTable format version and
the existing Pebble format major version, in addition to how the two are to
be used in Cockroach for safely enabling new table format versions.

# Motivation

Pebble currently uses a "format major version" scheme for the store (or DB)
that indicates which Pebble features should be enabled when the store is first
opened, before any SSTables are opened. The versions indicate points of
backwards incompatibility for a store. For example, the introduction of the
`SetWithDelete` key kind is gated behind a version, as is block property
collection. This format major version scheme was introduced in
[#1227](https://github.com/cockroachdb/pebble/issues/1227).

While Pebble can use the format major version to infer how to load and
interpret data in the LSM, the SSTables that make up the store itself have
their own notion of a "version". This "SSTable version" (also referred to as a
"table format") is written to the footer (or trailing section) of each SSTable
file and determines how the file is to be interpreted by Pebble. As of the time
of writing, Pebble supports two table formats - LevelDB's format, and RocksDB's
v2 format. Pebble inherited the latter as the default table format as it was
the version that RocksDB used at the time Pebble was being developed, and
remained the default to allow for a simpler migration path from Cockroach
clusters that were originally using RocksDB as the storage engine. The
RocksDBv2 table format adds various features on top of the LevelDB format,
including a two-level index, configurable checksum algorithms, and an explicit
versioning scheme to allow for the introduction of changes, amongst other
features.

While the RocksDBv2 SSTable format has been sufficient for Pebble's needs since
inception, new Pebble features and potential backports from RocksDB itself
require that the SSTable format evolve over time and therefore that the table
format be updated. As the majority of new features added over time will be
specific to Pebble, it does not make sense to repurpose the RocksDB format
versions that exist upstream for use with Pebble features (at the time of
writing, RocksDB had added versions 3 and 4 on top of the version 2 in use by
Pebble). A new Pebble-specific table format scheme is proposed.

In the context of a distributed system such as Cockroach, certain SSTable
features are backwards incompatible (e.g. the block property collection and
filtering feature extends the RocksDBv2 SSTable block index format to encoded
various block properties, which is a breaking change). Participants must
_first_ ensure that their stores have the code-level features available to read
and write these newer SSTables (indicated by Pebble's format major version).
Once all stores agree that they are running the minimum Pebble format major
version and will not roll back (e.g. Cockroach cluster version finalization),
SSTables can be written and read using more recent table formats. The Pebble
"format major version" and "table format version" are therefore no longer
independent - the former implies an upper bound on the latter.

Additionally, certain SSTable generation operations are independent of a
specific Pebble instance. For example, SSTable construction for the purposes of
backup and restore generates SSTables that are stored external to a specific
Pebble store (e.g. in cloud storage) can be used at a later point in time to
restore a store. SSTables constructed for such purposes must be carefully
versioned to ensure compatibility with existing clusters that may run with a
mixture of Pebble versions.

As a real-world example of the need for the above, consider two Cockroach nodes
each with a Pebble store, one at version A, the other at version B (version A
(newer) > B (older)). Store A constructs an SSTable for an external backup
containing a newer block index format (for block property collection). This
SSTable is then imported in to store B. Store B fails to read the SSTable as it
is not running with a format major version recent enough make sense of the
newer index format. The two stores require a method for agreeing on a minimum
supported table format.

The remainder of this document outlines a new table format for Pebble. This new
table format will be used for new table-level features such as block properties
and range keys (see
[#1339](https://github.com/cockroachdb/pebble/issues/1339)), but also for
backporting table-level features from RocksDB that would be useful to Pebble
(e.g. version 3 avoids encoding sequence numbers in the index, and version 4
uses delta encoding for the block offsets in the index, both of which are
useful for Pebble).

# Technical design

## Pebble magic number

The last 8 bytes of an SSTable is referred to as the "magic number".

LevelDB uses the first 8 bytes of the SHA1 hash of the string
`http://code.google.com/p/leveldb/` for the magic number.

RocksDB uses its own magic number, which indicates the use of a slightly
different table layout - the footer (the name for the end of an SSTable) is
slightly larger to accommodate a 32-bit version number and 8 bits for a
checksum type to be used for all blocks in the SSTable.

A new 8-byte magic number will be introduced for Pebble:

```
\xf0\x9f\xaa\xb3\xf0\x9f\xaa\xb3 // 🪳🪳
```

## Pebble version scheme

Tables with a Pebble magic number will use a dedicated versioning scheme,
starting with version `1`. No new versions other than version `2` will be
supported for tables containing the RocksDB magic number.

The choice of switching to a Pebble versioning scheme starting `1` simplifies
the implementation. Essentially all existing Pebble stores are managed via
Cockroach, and were either previously using RocksDB and migrated to Pebble, or
were created with Pebble stores. In both situations the table format used is
RocksDB v2.

Given that Pebble has not needed (and likely will not need) to support other
RocksDB table formats, it is reasonable to introduce a new magic number for
Pebble and reset the version counter to v1.

The following initial versions will correspond to the following new Pebble
features, that have yet to be introduced to Cockroach clusters as of the time
of writing:

- Version 1: block property collectors (block properties are encoded into the
  block index)
- Version 2: range keys (a new block is present in the table for range keys).

Subsequent alterations to the SSTable format should only increment the _Pebble
version number_. It should be noted that backported RocksDB table format
features (e.g. RocksDB versions 3 and 4) would use a different version number,
within the Pebble version sequence. While possibly confusing, the RocksDB
features are being "adopted" by Pebble, rather than directly ported, so a
Pebble specific version number is appropriate.

An alternative would be to allow RocksDB table format features to be backported
into Pebble under their existing RocksDB magic number, _alongside_
Pebble-specific features. The complexity required to determine the set of
characteristics to read and write to each SSTable would increase with such a
scheme, compared to the simpler "linear history" approach described above,
where new features simply ratchet the Pebble table format version number.

## Footer format

The footer format for SSTables with Pebble magic numbers _will remain the same_
as the RocksDB footer format - specifically, the trailing 53-bytes of the
SSTable consisting of the following fields with the given indices,
little-endian encoded:

- `0`: Checksum type
- `1-20`: Meta-index block handle
- `21-40`: Index block handle
- `41-44`: Version number
- `45-52`: Magic number

## Changes / additions to `sstable.TableFormat`

The `sstable.TableFormat` enum is a `uint32` representation of the tuple
`(magic number, format version). The current values are:

```go
type TableFormat uint32

const (
  TableFormatRocksDBv2 TableFormat = iota
  TableFormatLevelDB
)
```

It should be noted that this enum is _not_ persisted in the SSTable. It is
purely an internal type that represents the tuple that simplifies a number of
version checks when reading / writing an SSTable. The values are free to
change, provided care is taken with default values and existing usage.

The existing `sstable.TableFormat` will be altered to reflect the "linear"
nature of the version history. New versions will be added with the next value
in the sequence.

```go
const (
	TableFormatUnspecified TableFormat = iota
  TableFormatLevelDB    // The original LevelDB table format.
  TableFormatRocksDBv2  // The current default table format.
	TableFormatPebblev1   // Block properties.
	TableFormatPebblev2   // Range keys.
  ...
  TableFormatPebbleDBvN
)
```

The introduction of `TableFormatUnspecified` can be used to ensure that where a
`sstable.TableFormat` is _not_ specified, Pebble can select a suitable default
for writing the table (most likely based on the format major version in use by
the store; more in the next section).

## Interaction with the format major version

The `FormatMajorVersion` type is used to determine the set of features the
store supports.

A Pebble store may be read-from / written-to by a Pebble binary that supports
newer features, with more recent Pebble format major versions. These newer
features could include the ability to read and write more recent SSTables.
While the store _could_ read and write SSTables at the most recent version the
binary supports, it is not safe to do so, for reasons outlined earlier.

The format major version will have a "maximum table format version" associated
with it that indicates the maximum `sstable.TableFormat` that can be safely
handled by the store.

When introducing a new _table format_ version, it should be gated behind an
associated `FormatMajorVersion` that has the new table format as its "maximum
table format version".

For example:

```go
// Existing verisons.
FormatDefault.MaxTableFormat()                       // sstable.TableFormatRocksDBv2
...
FormatSetWithDelete.MaxTableFormat()                 // sstable.TableFormatRocksDBv2
// Proposed versions with Pebble version scheme.
FormatBlockPropertyCollector.MaxTableFormat()        // sstable.TableFormatPebbleDBv1
FormatRangeKeys.MaxTableFormat()                     // sstable.TableFormatPebbleDBv2
```

## Usage in Cockroach

The introduction of new SSTable format versions needs to be carefully
coordinated between stores to ensure there are no incompatibilities (i.e. newer
store writes an SSTable that cannot be understood by other stores).

It is only safe to use a new table format when all nodes in a cluster have been
finalized. A newer Cockroach node, with newer Pebble code, should continue to
write SSTables with a table format version equal to or less than the smallest
table format version across all nodes in the cluster. Once the cluster version
has been finalized, and `(*DB).RatchetFormatMajorVersion(FormatMajorVersion)`
has been called, nodes are free to write SSTables at newer table format
versions.

At runtime, Pebble exposes a `(*DB).FormatMajorVersion()` method, which may be
used to determine the current format major version of the store, and hence, the
associated table format version.

In addition to the above, there are situations where SSTables are created for
consumption at a later point in time, independent of any Pebble store -
specifically backup and restore. Currently, Cockroach uses two functions in
`pkg/sstable` to construct SSTables for both ingestion and backup
([here](https://github.com/cockroachdb/cockroach/blob/20eaf0b415f1df361246804e5d1d80c7a20a8eb6/pkg/storage/sst_writer.go#L57)
and
[here](https://github.com/cockroachdb/cockroach/blob/20eaf0b415f1df361246804e5d1d80c7a20a8eb6/pkg/storage/sst_writer.go#L78)).
Both will need to be updated to take into account the cluster version to ensure
that SSTables with newer versions are only written once the cluster version has
been finalized.

### Cluster version migration sequencing

Cockroach uses cluster versions as a guarantee that all nodes in a cluster are
running at a particular binary version, with a particular set of features
enabled. The Pebble store is ratcheted as the cluster version passes certain
versions that correspond to new Pebble functionality. Care must be taken to
prevent subtle race conditions while the cluster version is being updated
across all nodes in a cluster.

Consider a cluster at cluster version `n-1` with corresponding Pebble format
major version `A`. A new cluster version `n` introduces a new Pebble format
major version `B` with new table level features. One by one, nodes will bump
their format major versions from `A` to `B` as they are upgraded to cluster
version `n`. There exists a period of time where nodes in a cluster are split
between cluster versions `n-1` and `n`, and Pebble format major versions `A`
and `B`. If version `B` introduces SSTable level features that nodes with
stores at format major version `A` do not yet understand, there exists the risk
for runtime incompatibilities.

To guard against the window of incompatibility, _two_ cluster versions are
employed when bumping Pebble format major versions that correspond to new
SSTable level features. The first cluster verison is uesd to synchronize all
stores at the same Pebble format major version (and therefore table format
version). The second cluster version is used as a feature gate that enables
Cockroach nodes to make use of the newer table format, relying on the guarantee
that if a node is at version `n + 1`, then all other nodes in the cluster must
all be at least at version `n`, and therefore have Pebble stores at format
major version `B`.

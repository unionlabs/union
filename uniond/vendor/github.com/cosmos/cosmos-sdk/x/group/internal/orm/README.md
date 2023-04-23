# Group ORM

The orm package provides a framework for creating relational database tables with primary and secondary keys.

## Contents

* [Table](#table)
    * [AutoUInt64Table](#autouint64table)
    * [PrimaryKeyTable](#primarykeytable)
        * [PrimaryKeyed](#primarykeyed)
        * [Key codec](#key-codec)
* [Secondary Index](#secondary-index)
    * [MultiKeyIndex](#multikeyindex)
    * [UniqueIndex](#uniqueindex)
* [Iterator and Pagination](#iterator-and-pagination)
    * [Iterator](#iterator)
    * [Pagination](#pagination)

## Table

A table can be built given a `codec.ProtoMarshaler` model type, a prefix to access the underlying prefix store used to store table data as well as a `Codec` for marshalling/unmarshalling.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/group/internal/orm/table.go#L30-L36
```

In the prefix store, entities should be stored by an unique identifier called `RowID` which can be based either on an `uint64` auto-increment counter, string or dynamic size bytes.
Regular CRUD operations can be performed on a table, these methods take a `sdk.KVStore` as parameter to get the table prefix store.

The `table` struct does not:

* enforce uniqueness of the `RowID`
* enforce prefix uniqueness of keys, i.e. not allowing one key to be a prefix of another
* optimize Gas usage conditions

The `table` struct is private, so that we only have custom tables built on top of it, that do satisfy these requirements.

`table` provides methods for exporting (using a [`PrefixScan` `Iterator`](03_iterator_pagination.md#iterator)) and importing genesis data. For the import to be successful, objects have to be aware of their primary key by implementing the [`PrimaryKeyed`](#primarykeyed) interface.

### AutoUInt64Table

`AutoUInt64Table` is a table type with an auto incrementing `uint64` ID.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/group/internal/orm/auto_uint64.go#L15-L18
```

It's based on the `Sequence` struct which is a persistent unique key generator based on a counter encoded using 8 byte big endian.

### PrimaryKeyTable

`PrimaryKeyTable` provides simpler object style orm methods where are persisted and loaded with a reference to their unique primary key.

#### PrimaryKeyed

The model provided for creating a `PrimaryKeyTable` should implement the `PrimaryKeyed` interface:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/group/internal/orm/primary_key.go#L30-L44
```

`PrimaryKeyFields()` method returns the list of key parts for a given object.
The primary key parts can be []byte, string, and `uint64` types.

#### Key codec

Key parts, except the last part, follow these rules:

* []byte is encoded with a single byte length prefix (which means the max []byte length is 255)
* strings are null-terminated
* `uint64` are encoded using 8 byte big endian.

## Secondary Index

Secondary indexes can be used on `Indexable` [tables](01_table.md). Indeed, those tables implement the `Indexable` interface that provides a set of functions that can be called by indexes to register and interact with the tables, like callback functions that are called on entries creation, update or deletion to create, update or remove corresponding entries in the table secondary indexes.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/group/internal/orm/types.go#L88-L93
```

### MultiKeyIndex

A `MultiKeyIndex` is an index where multiple entries can point to the same underlying object.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/group/internal/orm/index.go#L26-L32
```

Internally, it uses an `Indexer` that manages the persistence of the index based on searchable keys and create/update/delete operations.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/group/internal/orm/index.go#L15-L20
```

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/group/internal/orm/indexer.go#L15-L19
```

The currently used implementation of an `indexer`, `Indexer`, relies on an `IndexerFunc` that should be provided when instantiating the index. Based on the source object, this function returns one or multiple index keys as `[]interface{}`. Such secondary index keys should be bytes, string or `uint64` in order to be handled properly by the [key codec](01_table.md#key-codec) which defines specific encoding for those types.
In the index prefix store, the keys are built based on the source object's `RowID` and its secondary index key(s) using the key codec and the values are set as empty bytes.

### UniqueIndex

As opposed to `MultiKeyIndex`, `UniqueIndex` is an index where duplicate keys are prohibited.

## Iterator and Pagination

Both [tables](01_table.md) and [secondary indexes](02_secondary_index.md) support iterating over a domain of keys, through `PrefixScan` or `ReversePrefixScan`, as well pagination.

### Iterator

An `Iterator` allows iteration through a sequence of key value pairs.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/group/internal/orm/types.go#L77-L85
```

Tables rely on a `typeSafeIterator` that is used by `PrefixScan` and `ReversePrefixScan` `table` methods to iterate through a range of `RowID`s.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/group/internal/orm/table.go#L287-L291
```

Secondary indexes rely on an `indexIterator` that can strip the `RowID` from the full index key in order to get the underlying value in the table prefix store.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/group/internal/orm/index.go#L233-L239
```

Under the hood, both use a prefix store `Iterator` (alias for tm-db `Iterator`).

### Pagination

The `Paginate` function does pagination given an [`Iterator`](#iterator) and a `query.PageRequest`, and returns a `query.PageResponse`.
It unmarshals the results into the provided dest interface that should be a pointer to a slice of models.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/group/internal/orm/iterator.go#L102-L220
```

Secondary indexes have a `GetPaginated` method that returns an `Iterator` for the given searched secondary index key, starting from the `query.PageRequest` key if provided. It's important to note that this `query.PageRequest` key should be a `RowID` (that could have been returned by a previous paginated request). The returned `Iterator` can then be used with the `Paginate` function and the same `query.PageRequest`.

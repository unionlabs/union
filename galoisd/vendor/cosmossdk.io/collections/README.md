# Collections

Collections is a library meant to simplify the experience with respect to module state handling.

Cosmos SDK modules handle their state using the `KVStore` interface. The problem with working with
`KVStore` is that it forces you to think of state as a bytes KV pairings when in reality the majority of
state comes from complex concrete golang objects (strings, ints, structs, etc.).

Collections allows you to work with state as if they were normal golang objects and removes the need
for you to think of your state as raw bytes in your code.

It also allows you to migrate your existing state without causing any state breakage that forces you into
tedious and complex chain state migrations.

## Installation

To install collections in your cosmos-sdk chain project, run the following command:

```shell
go get cosmossdk.io/collections
```

## Core types

Collections offers 5 different APIs to work with state, which will be explored in the next sections, these APIs are:
- ``Map``: to work with typed arbitrary KV pairings.
- ``KeySet``: to work with just typed keys
- ``Item``: to work with just one typed value
- ``Sequence``: which is a monotonically increasing number.
- ``IndexedMap``: which combines ``Map`` and `KeySet` to provide a `Map` with indexing capabilities.

## Preliminary components

Before exploring the different collections types and their capability it is necessary to introduce
the three components that every collection shares. In fact when instantiating a collection type by doing, for example,
```collections.NewMap/collections.NewItem/...``` you will find yourself having to pass them some common arguments.

For example, in code:

```go
package collections

import (
    "cosmossdk.io/collections"
    storetypes "cosmossdk.io/store/types"
    sdk "github.com/cosmos/cosmos-sdk/types"
)

var AllowListPrefix = collections.NewPrefix(0)

type Keeper struct {
	Schema    collections.Schema
	AllowList collections.KeySet[string]
}

func NewKeeper(storeKey *storetypes.KVStoreKey) Keeper {
	sb := collections.NewSchemaBuilder(sdk.OpenKVStore(storeKey))

	return Keeper{
		AllowList: collections.NewKeySet(sb, AllowListPrefix, "allow_list", collections.StringKey),
	}
}

```

Let's analyse the shared arguments, what they do, and why we need them.

### SchemaBuilder

The first argument passed is the ``SchemaBuilder``

`SchemaBuilder` is a structure that keeps track of all the state of a module, it is not required by the collections
 to deal with state but it offers a dynamic and reflective way for clients to explore a module's state.

We instantiate a ``SchemaBuilder`` by passing it a function that given the modules store key returns the module's specific store.

We then need to pass the schema builder to every collection type we instantiate in our keeper, in our case the `AllowList`.

### Prefix

The second argument passed to our ``KeySet`` is a `collections.Prefix`, a prefix represents a partition of the module's `KVStore`
where all the state of a specific collection will be saved. 

Since a module can have multiple collections, the following is expected:
- module params will become a `collections.Item`
- the `AllowList` is a `collections.KeySet`

We don't want a collection to write over the state of the other collection so we pass it a prefix, which defines a storage
partition owned by the collection.

If you already built modules, the prefix translates to the items you were creating in your ``types/keys.go`` file, example: https://github.com/cosmos/cosmos-sdk/blob/main/x/feegrant/key.go#L27

your old:
```go
var (
	// FeeAllowanceKeyPrefix is the set of the kvstore for fee allowance data
	// - 0x00<allowance_key_bytes>: allowance
	FeeAllowanceKeyPrefix = []byte{0x00}

	// FeeAllowanceQueueKeyPrefix is the set of the kvstore for fee allowance keys data
	// - 0x01<allowance_prefix_queue_key_bytes>: <empty value>
	FeeAllowanceQueueKeyPrefix = []byte{0x01}
)
```
becomes:
```go
var (
	// FeeAllowanceKeyPrefix is the set of the kvstore for fee allowance data
	// - 0x00<allowance_key_bytes>: allowance
	FeeAllowanceKeyPrefix = collections.NewPrefix(0)

	// FeeAllowanceQueueKeyPrefix is the set of the kvstore for fee allowance keys data
	// - 0x01<allowance_prefix_queue_key_bytes>: <empty value>
	FeeAllowanceQueueKeyPrefix = collections.NewPrefix(1)
)
```

#### Rules

``collections.NewPrefix`` accepts either `uint8`, `string` or `[]bytes` it's good practice to use an always increasing `uint8`for disk space efficiency.

A collection **MUST NOT** share the same prefix as another collection in the same module, and a collection prefix **MUST NEVER** start with the same prefix as another, examples:

```go
prefix1 := collections.NewPrefix("prefix")
prefix2 := collections.NewPrefix("prefix") // THIS IS BAD!
```

```go
prefix1 := collections.NewPrefix("a")
prefix2 := collections.NewPrefix("aa") // prefix2 starts with the same as prefix1: BAD!!!
```
### Human-Readable Name

The third parameter we pass to a collection is a string, which is a human-readable name.
It is needed to make the role of a collection understandable by clients who have no clue about
what a module is storing in state.

#### Rules

Each collection in a module **MUST** have a unique humanised name.

## Key and Value Codecs

A collection is generic over the type you can use as keys or values.
This makes collections dumb, but also means that hypothetically we can store everything
that can be a go type into a collection. We are not bounded to any type of encoding (be it proto, json or whatever)

So a collection needs to be given a way to understand how to convert your keys and values to bytes.
This is achieved through ``KeyCodec`` and `ValueCodec`, which are arguments that you pass to your
collections when you're instantiating them using the ```collections.NewMap/collections.NewItem/...```
instantiation functions.

NOTE: Generally speaking you will never be required to implement your own ``Key/ValueCodec`` as
the SDK and collections libraries already come with default, safe and fast implementation of those.
You might need to implement them only if you're migrating to collections and there are state layout incompatibilities.

Let's explore an example:

````go
package collections

import (
	"cosmossdk.io/collections"
	storetypes "cosmossdk.io/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var IDsPrefix = collections.NewPrefix(0)

type Keeper struct {
	Schema    collections.Schema
	IDs   collections.Map[string, uint64]
}

func NewKeeper(storeKey *storetypes.KVStoreKey) Keeper {
	sb := collections.NewSchemaBuilder(sdk.OpenKVStore(storeKey))

	return Keeper{
		IDs: collections.NewMap(sb, IDsPrefix, "ids", collections.StringKey, collections.Uint64Value),
	}
}
````

We're now instantiating a map where the key is string and the value is `uint64`.
We already know the first three arguments of the ``NewMap`` function.

The fourth parameter is our `KeyCodec`, we know that the ``Map`` has `string` as key so we pass it a `KeyCodec` that handles strings as keys.

The fifth parameter is our `ValueCodec`, we know that the `Map` as a `uint64` as value so we pass it a `ValueCodec` that handles uint64.

Collections already comes with all the required implementations for golang primitive types.

Let's make another example, this falls closer to what we build using cosmos SDK, let's say we want
to create a `collections.Map` that maps account addresses to their base account. So we want to map an `sdk.AccAddress` to an `auth.BaseAccount` (which is a proto):

```go
package collections

import (
	"cosmossdk.io/collections"
	storetypes "cosmossdk.io/store/types"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

var AccountsPrefix = collections.NewPrefix(0)

type Keeper struct {
	Schema    collections.Schema
	Accounts   collections.Map[sdk.AccAddress, authtypes.BaseAccount]
}

func NewKeeper(storeKey *storetypes.KVStoreKey, cdc codec.BinaryCodec) Keeper {
	sb := collections.NewSchemaBuilder(sdk.OpenKVStore(storeKey))
	return Keeper{
		Accounts: collections.NewMap(sb, AccountsPrefix, "accounts",
			sdk.AccAddressKey, codec.CollValue[authtypes.BaseAccount](cdc)),
	}
}
```

As we can see here since our `collections.Map` maps `sdk.AccAddress` to `authtypes.BaseAccount`,
we use the `sdk.AccAddressKey` which is the `KeyCodec` implementation for `AccAddress` and we use `codec.CollValue` to
encode our proto type `BaseAccount`.

Generally speaking you will always find the respective key and value codecs for types in the `go.mod` path you're using
to import that type. If you want to encode proto values refer to the codec `codec.CollValue` function, which allows you
to encode any type implement the `proto.Message` interface.

## Map

We analyse the first and most important collection type, the ``collections.Map``.
This is the type that everything else builds on top of.

### Use case

A `collections.Map` is used to map arbitrary keys with arbitrary values.

### Example

It's easier to explain a `collections.Map` capabilities through an example:

```go
package collections

import (
	"cosmossdk.io/collections"
	storetypes "cosmossdk.io/store/types"
	"fmt"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

var AccountsPrefix = collections.NewPrefix(0)

type Keeper struct {
	Schema    collections.Schema
	Accounts   collections.Map[sdk.AccAddress, authtypes.BaseAccount]
}

func NewKeeper(storeKey *storetypes.KVStoreKey, cdc codec.BinaryCodec) Keeper {
	sb := collections.NewSchemaBuilder(sdk.OpenKVStore(storeKey))
	return Keeper{
		Accounts: collections.NewMap(sb, AccountsPrefix, "accounts",
			sdk.AccAddressKey, codec.CollValue[authtypes.BaseAccount](cdc)),
	}
}

func (k Keeper) CreateAccount(ctx sdk.Context, addr sdk.AccAddress, account authtypes.BaseAccount) error {
	has, err := k.Accounts.Has(ctx, addr)
	if err != nil {
		return err
	}
	if has {
		return fmt.Errorf("account already exists: %s", addr)
	}
	
	err = k.Accounts.Set(ctx, addr, account)
	if err != nil {
		return err
	}
	return nil
}

func (k Keeper) GetAccount(ctx sdk.Context, addr sdk.AccAddress) (authtypes.BaseAccount, error) {
	acc, err := k.Accounts.Get(ctx, addr)
	if err != nil {
		return authtypes.BaseAccount{}, err
	}
	
	return acc,	nil
}

func (k Keeper) RemoveAccount(ctx sdk.Context, addr sdk.AccAddress) error {
	err := k.Accounts.Remove(ctx, addr)
	if err != nil {
		return err
	}
	return nil
}
```

#### Set method

Set maps with the provided `AccAddress` (the key) to the `auth.BaseAccount` (the value).

Under the hood the `collections.Map` will convert the key and value to bytes using the [key and value codec](README.md#key-and-value-codecs).
It will prepend to our bytes key the [prefix](README.md#prefix) and store it in the KVStore of the module.

#### Has method

The has method reports if the provided key exists in the store.

#### Get method

The get method accepts the `AccAddress` and returns the associated `auth.BaseAccount` if it exists, otherwise it errors.

#### Remove method

The remove method accepts the `AccAddress` and removes it from the store. It won't report errors
if it does not exist, to check for existence before removal use the ``Has`` method.

#### Iteration

Iteration has a separate section.

## KeySet

The second type of collection is `collections.KeySet`, as the word suggests it maintains
only a set of keys without values.

#### Implementation curiosity

A `collections.KeySet` is just a `collections.Map` with a `key` but no value.
The value internally is always the same and is represented as an empty byte slice ```[]byte{}```.

### Example

As always we explore the collection type through an example:

```go
package collections

import (
	"cosmossdk.io/collections"
	storetypes "cosmossdk.io/store/types"
	"fmt"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var ValidatorsSetPrefix = collections.NewPrefix(0)

type Keeper struct {
	Schema        collections.Schema
	ValidatorsSet collections.KeySet[sdk.ValAddress]
}

func NewKeeper(storeKey *storetypes.KVStoreKey) Keeper {
	sb := collections.NewSchemaBuilder(sdk.OpenKVStore(storeKey))
	return Keeper{
		ValidatorsSet: collections.NewKeySet(sb, ValidatorsSetPrefix, "validators_set", sdk.ValAddressKey),
	}
}

func (k Keeper) AddValidator(ctx sdk.Context, validator sdk.ValAddress) error {
	has, err := k.ValidatorsSet.Has(ctx, validator)
	if err != nil {
		return err
	}
	if has {
		return fmt.Errorf("validator already in set: %s", validator)
	}
	
	err = k.ValidatorsSet.Set(ctx, validator)
	if err != nil {
		return err
	}
	
	return nil
}

func (k Keeper) RemoveValidator(ctx sdk.Context, validator sdk.ValAddress) error {
	err := k.ValidatorsSet.Remove(ctx, validator)
	if err != nil {
		return err
	}
	return nil
}
```
The first difference we notice is that `KeySet` needs use to specify only one type parameter: the key (`sdk.ValAddress` in this case).
The second difference we notice is that `KeySet` in its `NewKeySet` function does not require
us to specify a `ValueCodec` but only a `KeyCodec`. This is because a `KeySet` only saves keys and not values.

Let's explore the methods.

#### Has method

Has allows us to understand if a key is present in the `collections.KeySet` or not, functions in the same way as `collections.Map.Has
`

#### Set method

Set inserts the provided key in the `KeySet`.

#### Remove method

Remove removes the provided key from the `KeySet`, it does not error if the key does not exist,
if existence check before removal is required it needs to be coupled with the `Has` method.

## Item

The third type of collection is the `collections.Item`.
It stores only one single item, it's useful for example for parameters, there's only one instance
of parameters in state always.

#### implementation curiosity

A `collections.Item` is just a `collections.Map` with no key but just a value.
The key is the prefix of the collection!

### Example

```go
package collections

import (
	"cosmossdk.io/collections"
	storetypes "cosmossdk.io/store/types"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
)

var ParamsPrefix = collections.NewPrefix(0)

type Keeper struct {
	Schema        collections.Schema
	Params collections.Item[stakingtypes.Params]
}

func NewKeeper(storeKey *storetypes.KVStoreKey, cdc codec.BinaryCodec) Keeper {
	sb := collections.NewSchemaBuilder(sdk.OpenKVStore(storeKey))
	return Keeper{
		Params: collections.NewItem(sb, ParamsPrefix, "params", codec.CollValue[stakingtypes.Params](cdc)),
	}
}

func (k Keeper) UpdateParams(ctx sdk.Context, params stakingtypes.Params) error {
	err := k.Params.Set(ctx, params)
	if err != nil {
		return err
	}
	return nil
}

func (k Keeper) GetParams(ctx sdk.Context) (stakingtypes.Params, error) {
	return k.Params.Get(ctx)
}
```

The first key difference we notice is that we specify only one type parameter, which is the value we're storing.
The second key difference is that we don't specify the `KeyCodec`, since we store only one item we already know the key
and the fact that it is constant.

## Iteration

One of the key features of the ``KVStore`` is iterating over keys.

Collections which deal with keys (so `Map`, `KeySet` and `IndexedMap`) allow you to iterate
over keys in a safe and typed way. They all share the same API, the only difference being
that ``KeySet`` returns a different type of `Iterator` because `KeySet` only deals with keys.

:::note

Every collection shares the same `Iterator` semantics.

:::

Let's have a look at the `Map.Iterate` method:

```go
func (m Map[K, V]) Iterate(ctx context.Context, ranger Ranger[K]) (Iterator[K, V], error) 
```

It accepts a `collections.Ranger[K]`, which is an API that instructs map on how to iterate over keys.
As always we don't need to implement anything here as `collections` already provides some generic `Ranger` implementers
that expose all you need to work with ranges.

### Example

We have a `collections.Map` that maps accounts using `uint64` IDs.

```go
package collections

import (
	"cosmossdk.io/collections"
	storetypes "cosmossdk.io/store/types"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

var AccountsPrefix = collections.NewPrefix(0)

type Keeper struct {
	Schema   collections.Schema
	Accounts collections.Map[uint64, authtypes.BaseAccount]
}

func NewKeeper(storeKey *storetypes.KVStoreKey, cdc codec.BinaryCodec) Keeper {
	sb := collections.NewSchemaBuilder(sdk.OpenKVStore(storeKey))
	return Keeper{
		Accounts: collections.NewMap(sb, AccountsPrefix, "accounts", collections.Uint64Key, codec.CollValue[authtypes.BaseAccount](cdc)),
	}
}

func (k Keeper) GetAllAccounts(ctx sdk.Context) ([]authtypes.BaseAccount, error) {
	// passing a nil Ranger equals to: iterate over every possible key
	iter, err := k.Accounts.Iterate(ctx, nil)
	if err != nil {
		return nil, err
	}
	accounts, err := iter.Values()
	if err != nil {
		return nil, err
	}

	return accounts, err
}

func (k Keeper) IterateAccountsBetween(ctx sdk.Context, start, end uint64) ([]authtypes.BaseAccount, error) {
	// The collections.Range API offers a lot of capabilities
	// like defining where the iteration starts or ends.
	rng := new(collections.Range[uint64]).
		StartInclusive(start).
		EndExclusive(end).
		Descending()

	iter, err := k.Accounts.Iterate(ctx, rng)
	if err != nil {
		return nil, err
	}
	accounts, err := iter.Values()
	if err != nil {
		return nil, err
	}

	return accounts, nil
}

func (k Keeper) IterateAccounts(ctx sdk.Context, do func(id uint64, acc authtypes.BaseAccount) (stop bool)) error {
	iter, err := k.Accounts.Iterate(ctx, nil)
	if err != nil {
		return err
	}
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		kv, err := iter.KeyValue()
		if err != nil {
			return err
		}

		if do(kv.Key, kv.Value) {
			break
		}
	}
	return nil
}
```

Let's analyse each method in the example and how it makes use of the `Iterate` and the returned `Iterator` API.

#### GetAllAccounts

In `GetAllAccounts` we pass to our `Iterate` a nil `Ranger`. This means that the returned `Iterator` will include
all the existing keys within the collection.

Then we use some the `Values` method from the returned `Iterator` API to collect all the values into a slice.

`Iterator` offers other methods such as `Keys()` to collect only the keys and not the values and `KeyValues` to collect
all the keys and values.


#### IterateAccountsBetween

Here we make use of the `collections.Range` helper to specialise our range.
We make it start in a point through `StartInclusive` and end in the other with `EndExclusive`, then
we instruct it to report us results in reverse order through `Descending`

Then we pass the range instruction to `Iterate` and get an `Iterator`, which will contain only the results
we specified in the range.

Then we use again th `Values` method of the `Iterator` to collect all the results.

`collections.Range` also offers a `Prefix` API which is not appliable to all keys types,
for example uint64 cannot be prefix because it is of constant size, but a `string` key
can be prefixed.

#### IterateAccounts

Here we showcase how to lazily collect values from an Iterator. 

:::note

`Keys/Values/KeyValues` fully consume and close the `Iterator`, here we need to explicitly do a `defer iterator.Close()` call.

:::

`Iterator` also exposes a `Value` and `Key` method to collect only the current value or key, if collecting both is not needed.

:::note

For this `callback` pattern, collections expose a `Walk` API.

:::

## Composite keys

So far we've worked only with simple keys, like `uint64`, the account address, etc.
There are some more complex cases in, which we need to deal with composite keys.

A key is composite when it is composed of multiple keys, for example bank balances as stored as the composite key
`(AccAddress, string)` where the first part is the address holding the coins and the second part is the denom.

Example, let's say address `BOB` holds `10atom,15osmo`, this is how it is stored in state:

```
(bob, atom) => 10
(bob, osmos) => 15
```

Now this allows to efficiently get a specific denom balance of an address, by simply `getting` `(address, denom)`, or getting all the balances
of an address by prefixing over `(address)`.

Let's see now how we can work with composite keys using collections.

### Example

In our example we will show-case how we can use collections when we are dealing with balances, similar to bank,
a balance is a mapping between `(address, denom) => math.Int` the composite key in our case is `(address, denom)`.

## Instantiation of a composite key collection

```go
package collections

import (
	"cosmossdk.io/collections"
	"cosmossdk.io/math"
	storetypes "cosmossdk.io/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)


var BalancesPrefix = collections.NewPrefix(1)

type Keeper struct {
	Schema   collections.Schema
	Balances collections.Map[collections.Pair[sdk.AccAddress, string], math.Int]
}

func NewKeeper(storeKey *storetypes.KVStoreKey) Keeper {
	sb := collections.NewSchemaBuilder(sdk.OpenKVStore(storeKey))
	return Keeper{
		Balances: collections.NewMap(
			sb, BalancesPrefix, "balances",
			collections.PairKeyCodec(sdk.AccAddressKey, collections.StringKey),
			math.IntValue,
		),
	}
}
```

#### The Map Key definition

First of all we can see that in order to define a composite key of two elements we use the `collections.Pair` type:
````go
collections.Map[collections.Pair[sdk.AccAddress, string], math.Int]
````

`collections.Pair` defines a key composed of two other keys, in our case the first part is `sdk.AccAddress`, the second
part is `string`.

#### The Key Codec instantiation

The arguments to instantiate are always the same, the only thing that changes is how we instantiate
the ``KeyCodec``, since this key is composed of two keys we use `collections.PairKeyCodec`, which generates
a `KeyCodec` composed of two key codecs. The first one will encode the first part of the key, the second one will
encode the second part of the key.


### Working with composite key collections

Let's expand on the example we used before:

````go
var BalancesPrefix = collections.NewPrefix(1)

type Keeper struct {
	Schema   collections.Schema
	Balances collections.Map[collections.Pair[sdk.AccAddress, string], math.Int]
}

func NewKeeper(storeKey *storetypes.KVStoreKey) Keeper {
	sb := collections.NewSchemaBuilder(sdk.OpenKVStore(storeKey))
	return Keeper{
		Balances: collections.NewMap(
			sb, BalancesPrefix, "balances",
			collections.PairKeyCodec(sdk.AccAddressKey, collections.StringKey),
			math.IntValue,
		),
	}
}

func (k Keeper) SetBalance(ctx sdk.Context, address sdk.AccAddress, denom string, amount math.Int) error {
	key := collections.Join(address, denom)
	return k.Balances.Set(ctx, key, amount)
}

func (k Keeper) GetBalance(ctx sdk.Context, address sdk.AccAddress, denom string) (math.Int, error) {
	return k.Balances.Get(ctx, collections.Join(address, denom))
}

func (k Keeper) GetAllAddressBalances(ctx sdk.Context, address sdk.AccAddress) (sdk.Coins, error) {
	balances := sdk.NewCoins()

	rng := collections.NewPrefixedPairRange[sdk.AccAddress, string](address)

	iter, err := k.Balances.Iterate(ctx, rng)
	if err != nil {
		return nil, err
	}

	kvs, err := iter.KeyValues()
	if err != nil {
		return nil, err
	}

	for _, kv := range kvs {
		balances = balances.Add(sdk.NewCoin(kv.Key.K2(), kv.Value))
	}
	return balances, nil
}

func (k Keeper) GetAllAddressBalancesBetween(ctx sdk.Context, address sdk.AccAddress, startDenom, endDenom string) (sdk.Coins, error) {
    rng := collections.NewPrefixedPairRange[sdk.AccAddress, string](address).
        StartInclusive(startDenom).
        EndInclusive(endDenom)

    iter, err := k.Balances.Iterate(ctx, rng)
    if err != nil {
        return nil, err
	}
    ...
}
````

#### SetBalance

As we can see here we're setting the balance of an address for a specific denom.
We use the `collections.Join` function to generate the composite key.
`collections.Join` returns a `collections.Pair` (which is the key of our `collections.Map`)

`collections.Pair` contains the two keys we have joined, it also exposes two methods: `K1` to fetch the 1st part of the
key and `K2` to fetch the second part.

As always, we use the `collections.Map.Set` method to map the composite key to our value (`math.Int`in this case)

#### GetBalance

To get a value in composite key collection, we simply use `collections.Join` to compose the key.

#### GetAllAddressBalances

We use `collections.PrefixedPairRange` to iterate over all the keys starting with the provided address.
Concretely the iteration will report all the balances belonging to the provided address.

The first part is that we instantiate a `PrefixedPairRange`, which is a `Ranger` implementer aimed to help
in `Pair` keys iterations.

```go
	rng := collections.NewPrefixedPairRange[sdk.AccAddress, string](address)
```

As we can see here we're passing the type parameters of the `collections.Pair` because golang type inference
with respect to generics is not as permissive as other languages, so we need to explitly say what are the types of the pair key.

#### GetAllAddressesBalancesBetween

This showcases how we can further specialise our range to limit the results further, by specifying
the range between the second part of the key (in our case the denoms, which are strings).

## IndexedMap

`collections.IndexedMap` is a collection that uses under the hood a `collections.Map`, and has a struct, which contains the indexes that we need to define.

### Example

Let's say we have an `auth.BaseAccount` struct which looks like the following:

```go
type BaseAccount struct {
	AccountNumber uint64     `protobuf:"varint,3,opt,name=account_number,json=accountNumber,proto3" json:"account_number,omitempty"`
	Sequence      uint64     `protobuf:"varint,4,opt,name=sequence,proto3" json:"sequence,omitempty"`
}
```

First of all, when we save our accounts in state we map them using a primary key `sdk.AccAddress`.
If it were to be a `collections.Map` it would be `collections.Map[sdk.AccAddres, authtypes.BaseAccount]`.

Then we also want to be able to get an account not only by its `sdk.AccAddress`, but also by its `AccountNumber`.

So we can say we want to create an `Index` that maps our `BaseAccount` to its `AccountNumber`.

We also know that this `Index` is unique. Unique means that there can only be one `BaseAccount` that maps to a specific
`AccountNumber`.

First of all, we start by defining the object that contains our index:

```go
var AccountsNumberIndexPrefix = collections.NewPrefix(1)

type AccountsIndexes struct {
	Number *indexes.Unique[uint64, sdk.AccAddress, authtypes.BaseAccount]
}

func (a AccountsIndexes) IndexesList() []collections.Index[sdk.AccAddress, authtypes.BaseAccount] {
	return []collections.Index[sdk.AccAddress, authtypes.BaseAccount]{a.Number}
}

func NewAccountIndexes(sb *collections.SchemaBuilder) AccountsIndexes {
	return AccountsIndexes{
		Number: indexes.NewUnique(
			sb, AccountsNumberIndexPrefix, "accounts_by_number",
			collections.Uint64Key, sdk.AccAddressKey,
			func(_ sdk.AccAddress, v authtypes.BaseAccount) (uint64, error) {
				return v.AccountNumber, nil
			},
		),
	}
}
```

We create an `AccountIndexes` struct which contains a field: `Number`. This field represents our `AccountNumber` index.
`AccountNumber` is a field of `authtypes.BaseAccount` and it's a `uint64`.

Then we can see in our `AccountIndexes` struct the `Number` field is defined as:

```go
*indexes.Unique[uint64, sdk.AccAddress, authtypes.BaseAccount]
```

Where the first type parameter is `uint64`, which is the field type of our index.
The second type parameter is the primary key `sdk.AccAddress`
And the third type parameter is the actual object we're storing `authtypes.BaseAccount`.

Then we implement a function called `IndexesList` on our `AccountIndexes` struct, this will be used
by the `IndexedMap` to keep the underlying map in sync with the indexes, in our case `Number`.
This function just needs to return the slice of indexes contained in the struct.

Then we create a `NewAccountIndexes` function that instantiates and returns the `AccountsIndexes` struct.

The function takes a `SchemaBuilder`. Then we instantiate our `indexes.Unique`, let's analyse the arguments we pass to
`indexes.NewUnique`.

#### Instantiating a `indexes.Unique`

The first three arguments, we already know them, they are: `SchemaBuilder`, `Prefix` which is our index prefix (the partition
where index keys relationship for the `Number` index will be maintained), and the human name for the `Number` index.

The second argument is a `collections.Uint64Key` which is a key codec to deal with `uint64` keys, we pass that because
the key we're trying to index is a `uint64` key (the account number), and then we pass as fifth argument the primary key codec,
which in our case is `sdk.AccAddress` (remember: we're mapping `sdk.AccAddress` => `BaseAccount`).

Then as last parameter we pass a function that: given the `BaseAccount` returns its `AccountNumber`.

After this we can proceed instantiating our `IndexedMap`.

```go
var AccountsPrefix = collections.NewPrefix(0)

type Keeper struct {
	Schema   collections.Schema
	Accounts *collections.IndexedMap[sdk.AccAddress, authtypes.BaseAccount, AccountsIndexes]
}

func NewKeeper(storeKey *storetypes.KVStoreKey, cdc codec.BinaryCodec) Keeper {
	sb := collections.NewSchemaBuilder(sdk.OpenKVStore(storeKey))
	return Keeper{
		Accounts: collections.NewIndexedMap(
			sb, AccountsPrefix, "accounts",
			sdk.AccAddressKey, codec.CollValue[authtypes.BaseAccount](cdc),
			NewAccountIndexes(sb),
		),
	}
}
```

As we can see here what we do, for now, is the same thing as we did for `collections.Map`.
We pass it the `SchemaBuilder`, the `Prefix` where we plan to store the mapping between `sdk.AccAddress` and `authtypes.BaseAccount`,
the human name and the respective `sdk.AccAddress` key codec and `authtypes.BaseAccount` value codec.

Then we pass the instantiation of our `AccountIndexes` through `NewAccountIndexes`.

Full example:

```go
package docs

import (
	"cosmossdk.io/collections"
	"cosmossdk.io/collections/indexes"
	storetypes "cosmossdk.io/store/types"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

var AccountsNumberIndexPrefix = collections.NewPrefix(1)

type AccountsIndexes struct {
	Number *indexes.Unique[uint64, sdk.AccAddress, authtypes.BaseAccount]
}

func (a AccountsIndexes) IndexesList() []collections.Index[sdk.AccAddress, authtypes.BaseAccount] {
	return []collections.Index[sdk.AccAddress, authtypes.BaseAccount]{a.Number}
}

func NewAccountIndexes(sb *collections.SchemaBuilder) AccountsIndexes {
	return AccountsIndexes{
		Number: indexes.NewUnique(
			sb, AccountsNumberIndexPrefix, "accounts_by_number",
			collections.Uint64Key, sdk.AccAddressKey,
			func(_ sdk.AccAddress, v authtypes.BaseAccount) (uint64, error) {
				return v.AccountNumber, nil
			},
		),
	}
}

var AccountsPrefix = collections.NewPrefix(0)

type Keeper struct {
	Schema   collections.Schema
	Accounts *collections.IndexedMap[sdk.AccAddress, authtypes.BaseAccount, AccountsIndexes]
}

func NewKeeper(storeKey *storetypes.KVStoreKey, cdc codec.BinaryCodec) Keeper {
	sb := collections.NewSchemaBuilder(sdk.OpenKVStore(storeKey))
	return Keeper{
		Accounts: collections.NewIndexedMap(
			sb, AccountsPrefix, "accounts",
			sdk.AccAddressKey, codec.CollValue[authtypes.BaseAccount](cdc),
			NewAccountIndexes(sb),
		),
	}
}
```

### Working with IndexedMaps

Whilst instantiating `collections.IndexedMap` is tedious, working with them is extremely smooth.

Let's take the full example, and expand it with some use-cases.

```go
package docs

import (
	"cosmossdk.io/collections"
	"cosmossdk.io/collections/indexes"
	storetypes "cosmossdk.io/store/types"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

var AccountsNumberIndexPrefix = collections.NewPrefix(1)

type AccountsIndexes struct {
	Number *indexes.Unique[uint64, sdk.AccAddress, authtypes.BaseAccount]
}

func (a AccountsIndexes) IndexesList() []collections.Index[sdk.AccAddress, authtypes.BaseAccount] {
	return []collections.Index[sdk.AccAddress, authtypes.BaseAccount]{a.Number}
}

func NewAccountIndexes(sb *collections.SchemaBuilder) AccountsIndexes {
	return AccountsIndexes{
		Number: indexes.NewUnique(
			sb, AccountsNumberIndexPrefix, "accounts_by_number",
			collections.Uint64Key, sdk.AccAddressKey,
			func(_ sdk.AccAddress, v authtypes.BaseAccount) (uint64, error) {
				return v.AccountNumber, nil
			},
		),
	}
}

var AccountsPrefix = collections.NewPrefix(0)

type Keeper struct {
	Schema   collections.Schema
	Accounts *collections.IndexedMap[sdk.AccAddress, authtypes.BaseAccount, AccountsIndexes]
}

func NewKeeper(storeKey *storetypes.KVStoreKey, cdc codec.BinaryCodec) Keeper {
	sb := collections.NewSchemaBuilder(sdk.OpenKVStore(storeKey))
	return Keeper{
		Accounts: collections.NewIndexedMap(
			sb, AccountsPrefix, "accounts",
			sdk.AccAddressKey, codec.CollValue[authtypes.BaseAccount](cdc),
			NewAccountIndexes(sb),
		),
	}
}

func (k Keeper) CreateAccount(ctx sdk.Context, addr sdk.AccAddress) error {
	nextAccountNumber := k.getNextAccountNumber()
	
	newAcc := authtypes.BaseAccount{
		AccountNumber: nextAccountNumber,
		Sequence:      0,
	}
	
	return k.Accounts.Set(ctx, addr, newAcc)
}

func (k Keeper) RemoveAccount(ctx sdk.Context, addr sdk.AccAddress) error {
	return k.Accounts.Remove(ctx, addr)
} 

func (k Keeper) GetAccountByNumber(ctx sdk.Context, accNumber uint64) (sdk.AccAddress, authtypes.BaseAccount, error) {
	accAddress, err := k.Accounts.Indexes.Number.MatchExact(ctx, accNumber)
	if err != nil {
		return nil, authtypes.BaseAccount{}, err
	}
	
	acc, err := k.Accounts.Get(ctx, accAddress)
	return accAddress, acc, nil
}

func (k Keeper) GetAccountsByNumber(ctx sdk.Context, startAccNum, endAccNum uint64) ([]authtypes.BaseAccount, error) {
	rng := new(collections.Range[uint64]).
		StartInclusive(startAccNum).
		EndInclusive(endAccNum)
	
	iter, err := k.Accounts.Indexes.Number.Iterate(ctx, rng)
	if err != nil {
		return nil, err
	}
	
	return indexes.CollectValues(ctx, k.Accounts, iter)
}


func (k Keeper) getNextAccountNumber() uint64 {
	return 0
}
```

## Collections with interfaces as values

Although cosmos-sdk is shifting away from the usage of interface registry, there are still some places where it is used.
In order to support old code, we have to support collections with interface values.

The generic `codec.CollValue` is not able to handle interface values, so we need to use a special type `codec.CollValueInterface`.
`codec.CollValueInterface` takes a `codec.BinaryCodec` as an argument, and uses it to marshal and unmarshal values as interfaces.
The `codec.CollValueInterface` lives in the `codec` package, whose import path is `github.com/cosmos/cosmos-sdk/codec`.

### Instantiating Collections with interface values

In order to instantiate a collection with interface values, we need to use `codec.CollValueInterface` instead of `codec.CollValue`.

```go
package example

import (
    "cosmossdk.io/collections"
    storetypes "cosmossdk.io/store/types"
    "github.com/cosmos/cosmos-sdk/codec"
    sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

var AccountsPrefix = collections.NewPrefix(0)

type Keeper struct {
    Schema   collections.Schema
    Accounts *collections.Map[sdk.AccAddress, sdk.AccountI]
}

func NewKeeper(cdc codec.BinaryCodec, storeKey *storetypes.KVStoreKey) Keeper {
    sb := collections.NewSchemaBuilder(sdk.OpenKVStore(storeKey))
    return Keeper{
        Accounts: collections.NewMap(
            sb, AccountsPrefix, "accounts",
            sdk.AccAddressKey, codec.CollInterfaceValue[sdk.AccountI](cdc),
        ),
    }
}

func (k Keeper) SaveBaseAccount(ctx sdk.Context, account authtypes.BaseAccount) error {
    return k.Accounts.Set(ctx, account.GetAddress(), account)
}

func (k Keeper) SaveModuleAccount(ctx sdk.Context, account authtypes.ModuleAccount) error {
    return k.Accounts.Set(ctx, account.GetAddress(), account)
}

func (k Keeper) GetAccount(ctx sdk.context, addr sdk.AccAddress) (sdk.AccountI, error) {
    return k.Accounts.Get(ctx, addr)
}
```

## Triple key

The `collections.Triple` is a special type of key composed of three keys, it's identical to `collections.Pair`.

Let's see an example.

```go
package example

import (
 "context"

 "cosmossdk.io/collections"
 storetypes "cosmossdk.io/store/types"
 "github.com/cosmos/cosmos-sdk/codec"
)

type AccAddress = string
type ValAddress = string

type Keeper struct {
 // let's simulate we have redelegations which are stored as a triple key composed of
 // the delegator, the source validator and the destination validator.
 Redelegations collections.KeySet[collections.Triple[AccAddress, ValAddress, ValAddress]]
}

func NewKeeper(storeKey *storetypes.KVStoreKey) Keeper {
 sb := collections.NewSchemaBuilder(sdk.OpenKVStore(storeKey))
 return Keeper{
  Redelegations: collections.NewKeySet(sb, collections.NewPrefix(0), "redelegations", collections.TripleKeyCodec(collections.StringKey, collections.StringKey, collections.StringKey)
 }
}

// RedelegationsByDelegator iterates over all the redelegations of a given delegator and calls onResult providing
// each redelegation from source validator towards the destination validator.
func (k Keeper) RedelegationsByDelegator(ctx context.Context, delegator AccAddress, onResult func(src, dst ValAddress) (stop bool, err error)) error {
 rng := collections.NewPrefixedTripleRange[AccAddress, ValAddress, ValAddress](delegator)
 return k.Redelegations.Walk(ctx, rng, func(key collections.Triple[AccAddress, ValAddress, ValAddress]) (stop bool, err error) {
  return onResult(key.K2(), key.K3())
 })
}

// RedelegationsByDelegatorAndValidator iterates over all the redelegations of a given delegator and its source validator and calls onResult for each
// destination validator.
func (k Keeper) RedelegationsByDelegatorAndValidator(ctx context.Context, delegator AccAddress, validator ValAddress, onResult func(dst ValAddress) (stop bool, err error)) error {
 rng := collections.NewSuperPrefixedTripleRange[AccAddress, ValAddress, ValAddress](delegator, validator)
 return k.Redelegations.Walk(ctx, rng, func(key collections.Triple[AccAddress, ValAddress, ValAddress]) (stop bool, err error) {
  return onResult(key.K3())
 })
}
```

## Advanced Usages

### Alternative Value Codec

The `codec.AltValueCodec` allows a collection to decode values using a different codec than the one used to encode them.
Basically it enables to decode two different byte representations of the same concrete value.
It can be used to lazily migrate values from one bytes representation to another, as long as the new representation is
not able to decode the old one.

A concrete example can be found in `x/bank` where the balance was initially stored as `Coin` and then migrated to `Int`.

```go

var BankBalanceValueCodec = codec.NewAltValueCodec(sdk.IntValue, func(b []byte) (sdk.Int, error) {
    coin := sdk.Coin{}
    err := coin.Unmarshal(b)
    if err != nil {
        return sdk.Int{}, err
    }
    return coin.Amount, nil
})
```

The above example shows how to create an `AltValueCodec` that can decode both `sdk.Int` and `sdk.Coin` values. The provided 
decoder function will be used as a fallback in case the default decoder fails. When the value will be encoded back into state
it will use the default encoder. This allows to lazily migrate values to a new bytes representation.
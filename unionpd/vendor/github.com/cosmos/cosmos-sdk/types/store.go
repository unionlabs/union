package types

import (
	fmt "fmt"
	"sort"
	"strings"

	"github.com/cosmos/cosmos-sdk/store/types"
	"github.com/cosmos/cosmos-sdk/types/kv"
)

type (
	Store                     = types.Store
	Committer                 = types.Committer
	CommitStore               = types.CommitStore
	Queryable                 = types.Queryable
	MultiStore                = types.MultiStore
	CacheMultiStore           = types.CacheMultiStore
	CommitMultiStore          = types.CommitMultiStore
	MultiStorePersistentCache = types.MultiStorePersistentCache
	KVStore                   = types.KVStore
	Iterator                  = types.Iterator
)

// StoreDecoderRegistry defines each of the modules store decoders. Used for ImportExport
// simulation.
type StoreDecoderRegistry map[string]func(kvA, kvB kv.Pair) string

// Iterator over all the keys with a certain prefix in ascending order
func KVStorePrefixIterator(kvs KVStore, prefix []byte) Iterator {
	return types.KVStorePrefixIterator(kvs, prefix)
}

// Iterator over all the keys with a certain prefix in descending order.
func KVStoreReversePrefixIterator(kvs KVStore, prefix []byte) Iterator {
	return types.KVStoreReversePrefixIterator(kvs, prefix)
}

// KVStorePrefixIteratorPaginated returns iterator over items in the selected page.
// Items iterated and skipped in ascending order.
func KVStorePrefixIteratorPaginated(kvs KVStore, prefix []byte, page, limit uint) Iterator {
	return types.KVStorePrefixIteratorPaginated(kvs, prefix, page, limit)
}

// KVStoreReversePrefixIteratorPaginated returns iterator over items in the selected page.
// Items iterated and skipped in descending order.
func KVStoreReversePrefixIteratorPaginated(kvs KVStore, prefix []byte, page, limit uint) Iterator {
	return types.KVStoreReversePrefixIteratorPaginated(kvs, prefix, page, limit)
}

// DiffKVStores compares two KVstores and returns all the key/value pairs
// that differ from one another. It also skips value comparison for a set of provided prefixes
func DiffKVStores(a KVStore, b KVStore, prefixesToSkip [][]byte) (kvAs, kvBs []kv.Pair) {
	return types.DiffKVStores(a, b, prefixesToSkip)
}

// assertNoCommonPrefix will panic if there are two keys: k1 and k2 in keys, such that
// k1 is a prefix of k2
func assertNoPrefix(keys []string) {
	sorted := make([]string, len(keys))
	copy(sorted, keys)
	sort.Strings(sorted)
	for i := 1; i < len(sorted); i++ {
		if strings.HasPrefix(sorted[i], sorted[i-1]) {
			panic(fmt.Sprint("Potential key collision between KVStores:", sorted[i], " - ", sorted[i-1]))
		}
	}
}

// NewKVStoreKey returns a new pointer to a KVStoreKey.
func NewKVStoreKey(name string) *types.KVStoreKey {
	return types.NewKVStoreKey(name)
}

// NewKVStoreKeys returns a map of new  pointers to KVStoreKey's.
// The function will panic if there is a potential conflict in names (see `assertNoPrefix`
// function for more details).
func NewKVStoreKeys(names ...string) map[string]*types.KVStoreKey {
	assertNoPrefix(names)
	keys := make(map[string]*types.KVStoreKey, len(names))
	for _, n := range names {
		keys[n] = NewKVStoreKey(n)
	}

	return keys
}

// Constructs new TransientStoreKey
// Must return a pointer according to the ocap principle
func NewTransientStoreKey(name string) *types.TransientStoreKey {
	return types.NewTransientStoreKey(name)
}

// NewTransientStoreKeys constructs a new map of TransientStoreKey's
// Must return pointers according to the ocap principle
// The function will panic if there is a potential conflict in names (see `assertNoPrefix`
// function for more details).
func NewTransientStoreKeys(names ...string) map[string]*types.TransientStoreKey {
	assertNoPrefix(names)
	keys := make(map[string]*types.TransientStoreKey)
	for _, n := range names {
		keys[n] = NewTransientStoreKey(n)
	}

	return keys
}

// NewMemoryStoreKeys constructs a new map matching store key names to their
// respective MemoryStoreKey references.
// The function will panic if there is a potential conflict in names (see `assertNoPrefix`
// function for more details).
func NewMemoryStoreKeys(names ...string) map[string]*types.MemoryStoreKey {
	assertNoPrefix(names)
	keys := make(map[string]*types.MemoryStoreKey)
	for _, n := range names {
		keys[n] = types.NewMemoryStoreKey(n)
	}

	return keys
}

// PrefixEndBytes returns the []byte that would end a
// range query for all []byte with a certain prefix
// Deals with last byte of prefix being FF without overflowing
func PrefixEndBytes(prefix []byte) []byte {
	return types.PrefixEndBytes(prefix)
}

// InclusiveEndBytes returns the []byte that would end a
// range query such that the input would be included
func InclusiveEndBytes(inclusiveBytes []byte) (exclusiveBytes []byte) {
	return types.InclusiveEndBytes(inclusiveBytes)
}

//----------------------------------------

// key-value result for iterator queries
type KVPair = types.KVPair

//----------------------------------------

// TraceContext contains TraceKVStore context data. It will be written with
// every trace operation.
type TraceContext = types.TraceContext

// --------------------------------------

type (
	Gas       = types.Gas
	GasMeter  = types.GasMeter
	GasConfig = types.GasConfig
)

type (
	ErrorOutOfGas    = types.ErrorOutOfGas
	ErrorGasOverflow = types.ErrorGasOverflow
)

func NewGasMeter(limit Gas) GasMeter {
	return types.NewGasMeter(limit)
}

func NewInfiniteGasMeter() GasMeter {
	return types.NewInfiniteGasMeter()
}

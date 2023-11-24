package maps

import (
	"encoding/binary"

	"github.com/cometbft/cometbft/crypto/merkle"
	"github.com/cometbft/cometbft/crypto/tmhash"
	cmtprotocrypto "github.com/cometbft/cometbft/proto/tendermint/crypto"

	"cosmossdk.io/store/internal/kv"
	"cosmossdk.io/store/internal/tree"
)

// merkleMap defines a merkle-ized tree from a map. Leave values are treated as
// hash(key) | hash(value). Leaves are sorted before Merkle hashing.
type merkleMap struct {
	kvs    kv.Pairs
	sorted bool
}

func newMerkleMap() *merkleMap {
	return &merkleMap{
		kvs:    kv.Pairs{},
		sorted: false,
	}
}

// Set creates a kv.Pair from the provided key and value. The value is hashed prior
// to creating a kv.Pair. The created kv.Pair is appended to the MerkleMap's slice
// of kv.Pairs. Whenever called, the MerkleMap must be resorted.
func (sm *merkleMap) set(key string, value []byte) {
	byteKey := []byte(key)
	assertValidKey(byteKey)

	sm.sorted = false

	// The value is hashed, so you can check for equality with a cached value (say)
	// and make a determination to fetch or not.
	vhash := tmhash.Sum(value)

	sm.kvs.Pairs = append(sm.kvs.Pairs, kv.Pair{
		Key:   byteKey,
		Value: vhash,
	})
}

// Hash returns the merkle root of items sorted by key. Note, it is unstable.
func (sm *merkleMap) hash() []byte {
	sm.sort()
	return hashKVPairs(sm.kvs)
}

func (sm *merkleMap) sort() {
	if sm.sorted {
		return
	}

	sm.kvs.Sort()
	sm.sorted = true
}

// hashKVPairs hashes a kvPair and creates a merkle tree where the leaves are
// byte slices.
func hashKVPairs(kvs kv.Pairs) []byte {
	kvsH := make([][]byte, len(kvs.Pairs))
	for i, kvp := range kvs.Pairs {
		kvsH[i] = KVPair(kvp).Bytes()
	}

	return tree.HashFromByteSlices(kvsH)
}

// ---------------------------------------------

// Merkle tree from a map.
// Leaves are `hash(key) | hash(value)`.
// Leaves are sorted before Merkle hashing.
type simpleMap struct {
	Kvs    kv.Pairs
	sorted bool
}

func newSimpleMap() *simpleMap {
	return &simpleMap{
		Kvs:    kv.Pairs{},
		sorted: false,
	}
}

// Set creates a kv pair of the key and the hash of the value,
// and then appends it to SimpleMap's kv pairs.
func (sm *simpleMap) Set(key string, value []byte) {
	byteKey := []byte(key)
	assertValidKey(byteKey)
	sm.sorted = false

	// The value is hashed, so you can
	// check for equality with a cached value (say)
	// and make a determination to fetch or not.
	vhash := tmhash.Sum(value)

	sm.Kvs.Pairs = append(sm.Kvs.Pairs, kv.Pair{
		Key:   byteKey,
		Value: vhash,
	})
}

// Hash Merkle root hash of items sorted by key
// (UNSTABLE: and by value too if duplicate key).
func (sm *simpleMap) Hash() []byte {
	sm.Sort()
	return hashKVPairs(sm.Kvs)
}

func (sm *simpleMap) Sort() {
	if sm.sorted {
		return
	}
	sm.Kvs.Sort()
	sm.sorted = true
}

// Returns a copy of sorted KVPairs.
// NOTE these contain the hashed key and value.
func (sm *simpleMap) KVPairs() kv.Pairs {
	sm.Sort()
	kvs := kv.Pairs{
		Pairs: make([]kv.Pair, len(sm.Kvs.Pairs)),
	}

	copy(kvs.Pairs, sm.Kvs.Pairs)
	return kvs
}

//----------------------------------------

// A local extension to KVPair that can be hashed.
// Key and value are length prefixed and concatenated,
// then hashed.
type KVPair kv.Pair

// NewKVPair takes in a key and value and creates a kv.Pair
// wrapped in the local extension KVPair
func NewKVPair(key, value []byte) KVPair {
	return KVPair(kv.Pair{
		Key:   key,
		Value: value,
	})
}

// Bytes returns key || value, with both the
// key and value length prefixed.
func (kv KVPair) Bytes() []byte {
	// In the worst case:
	// * 8 bytes to Uvarint encode the length of the key
	// * 8 bytes to Uvarint encode the length of the value
	// So preallocate for the worst case, which will in total
	// be a maximum of 14 bytes wasted, if len(key)=1, len(value)=1,
	// but that's going to rare.
	buf := make([]byte, 8+len(kv.Key)+8+len(kv.Value))

	// Encode the key, prefixed with its length.
	nlk := binary.PutUvarint(buf, uint64(len(kv.Key)))
	nk := copy(buf[nlk:], kv.Key)

	// Encode the value, prefixing with its length.
	nlv := binary.PutUvarint(buf[nlk+nk:], uint64(len(kv.Value)))
	nv := copy(buf[nlk+nk+nlv:], kv.Value)

	return buf[:nlk+nk+nlv+nv]
}

// HashFromMap computes a merkle tree from sorted map and returns the merkle
// root.
func HashFromMap(m map[string][]byte) []byte {
	mm := newMerkleMap()
	for k, v := range m {
		mm.set(k, v)
	}

	return mm.hash()
}

// ProofsFromMap generates proofs from a map. The keys/values of the map will be used as the keys/values
// in the underlying key-value pairs.
// The keys are sorted before the proofs are computed.
func ProofsFromMap(m map[string][]byte) ([]byte, map[string]*cmtprotocrypto.Proof, []string) {
	sm := newSimpleMap()
	for k, v := range m {
		sm.Set(k, v)
	}

	sm.Sort()
	kvs := sm.Kvs
	kvsBytes := make([][]byte, len(kvs.Pairs))
	for i, kvp := range kvs.Pairs {
		kvsBytes[i] = KVPair(kvp).Bytes()
	}

	rootHash, proofList := merkle.ProofsFromByteSlices(kvsBytes)
	proofs := make(map[string]*cmtprotocrypto.Proof)
	keys := make([]string, len(proofList))

	for i, kvp := range kvs.Pairs {
		proofs[string(kvp.Key)] = proofList[i].ToProto()
		keys[i] = string(kvp.Key)
	}

	return rootHash, proofs, keys
}

func assertValidKey(key []byte) {
	if len(key) == 0 {
		panic("key is nil")
	}
}

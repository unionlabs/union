package maps

import (
	// "encoding/binary"
	"hash"
	"math/bits"

	"github.com/cometbft/cometbft/crypto/merkle"
	"github.com/cometbft/cometbft/crypto/tmhash"
	cmtprotocrypto "github.com/cometbft/cometbft/proto/tendermint/crypto"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr/mimc"

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
	// vhash := tmhash.Sum(value)

	if len(value) != 32 {
		panic(value)
	}

	sm.kvs.Pairs = append(sm.kvs.Pairs, kv.Pair{
		Key:   byteKey,
		Value: value,
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
	// vhash := tmhash.Sum(value)

	if len(value) != 32 {
		panic(value)
	}

	sm.Kvs.Pairs = append(sm.Kvs.Pairs, kv.Pair{
		Key:   byteKey,
		Value: value,
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
	// buf := make([]byte, 8+len(kv.Key)+8+len(kv.Value))
	// // Encode the key, prefixed with its length.
	// nlk := binary.PutUvarint(buf, uint64(len(kv.Key)))
	// nk := copy(buf[nlk:], kv.Key)

	// // Encode the value, prefixing with its length.
	// nlv := binary.PutUvarint(buf[nlk+nk:], uint64(len(kv.Value)))
	// nv := copy(buf[nlk+nk+nlv:], kv.Value)

	// return buf[:nlk+nk+nlv+nv]

	// NOTE(aeryz): value is already prehashed with sha256, we just need to hash
	// the key here

	var buf [64]byte

	copy(buf[0:32], tmhash.Sum(kv.Key))
	copy(buf[32:64], kv.Value)

	return buf[:]

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

	rootHash, proofList := ProofsFromByteSlices(kvsBytes)
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

func ProofsFromByteSlices(items [][]byte) (rootHash []byte, proofs []*merkle.Proof) {
	trails, rootSPN := trailsFromByteSlices(items)
	rootHash = rootSPN.Hash
	proofs = make([]*merkle.Proof, len(items))
	for i, trail := range trails {
		proofs[i] = &merkle.Proof{
			Total:    int64(len(items)),
			Index:    int64(i),
			LeafHash: trail.Hash,
			Aunts:    trail.FlattenAunts(),
		}
	}
	return
}

func trailsFromByteSlices(items [][]byte) (trails []*merkle.ProofNode, root *merkle.ProofNode) {
	// Recursive impl.
	switch len(items) {
	case 0:
		return []*merkle.ProofNode{}, &merkle.ProofNode{emptyHash(), nil, nil, nil}
	case 1:
		trail := &merkle.ProofNode{leafHash(items[0]), nil, nil, nil}
		return []*merkle.ProofNode{trail}, trail
	default:
		k := getSplitPoint(int64(len(items)))
		lefts, leftRoot := trailsFromByteSlices(items[:k])
		rights, rightRoot := trailsFromByteSlices(items[k:])
		rootHash := innerHash(leftRoot.Hash, rightRoot.Hash)
		root := &merkle.ProofNode{rootHash, nil, nil, nil}
		leftRoot.Parent = root
		leftRoot.Right = rightRoot
		rightRoot.Parent = root
		rightRoot.Left = leftRoot
		return append(lefts, rights...), root
	}
}

// getSplitPoint returns the largest power of 2 less than length
func getSplitPoint(length int64) int64 {
	if length < 1 {
		panic("Trying to split a tree with size < 1")
	}
	uLength := uint(length)
	bitlen := bits.Len(uLength)
	k := int64(1 << uint(bitlen-1))
	if k == length {
		k >>= 1
	}
	return k
}

// TODO: make these have a large predefined capacity
var (
	leafPrefixValue  = 0
	innerPrefixValue = 1
	leafPrefix       = []byte{0}
	innerPrefix      = []byte{1}
)

// returns tmhash(<empty>)
func emptyHash() []byte {
	// TODO(aeryz): Make this constant
	var bytes [32]byte
	h := mimc.NewMiMC()
	_, err := h.Write(bytes[:])
	if err != nil {
		panic(err)
	}
	return h.Sum(nil)
	// return tmhash.Sum([]byte{})
}

// returns tmhash(0x00 || leaf)
func leafHash(leaf []byte) []byte {
	var bytes [16]byte
	h := mimc.NewMiMC()
	paddedMiMC(h, bytes[:])

	// This is sha256'ed key and sha256'ed value, instead of doing 4x16, we do 2x32 by ignoring the MSB
	var bytes32 [32]byte
	copy(bytes32[1:32], leaf[1:32])
	h.Write(bytes32[:])
	copy(bytes32[1:32], leaf[33:64])
	h.Write(bytes32[:])

	return h.Sum(nil)
}

// returns tmhash(0x01 || left || right)
func innerHash(left []byte, right []byte) []byte {
	// TODO(aeryz): check endianness
	var bytes [16]byte
	bytes[0] = byte(innerPrefixValue)

	h := mimc.NewMiMC()
	paddedMiMC(h, bytes[:])
	// Left and right are directly written since they are MiMC
	h.Write(left[:])
	h.Write(right[:])
	// paddedMiMC(h, left)
	// paddedMiMC(h, right)
	return h.Sum(nil)
}

func paddedMiMC(h hash.Hash, data []byte) {
	for cursor := 0; cursor < len(data); cursor += 16 {
		var bytesBlock [32]byte
		inbytes := data[cursor : cursor+16]
		copy(bytesBlock[16:32], inbytes)
		_, err := h.Write(bytesBlock[:])
		if err != nil {
			panic(err)
		}
	}
}

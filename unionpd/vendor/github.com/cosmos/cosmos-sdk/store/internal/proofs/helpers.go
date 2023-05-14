package proofs

import (
	"sort"

	"github.com/cometbft/cometbft/libs/rand"
	tmcrypto "github.com/cometbft/cometbft/proto/tendermint/crypto"
	"golang.org/x/exp/maps"

	sdkmaps "github.com/cosmos/cosmos-sdk/store/internal/maps"
)

// SimpleResult contains a merkle.SimpleProof along with all data needed to build the confio/proof
type SimpleResult struct {
	Key      []byte
	Value    []byte
	Proof    *tmcrypto.Proof
	RootHash []byte
}

// GenerateRangeProof makes a tree of size and returns a range proof for one random element
//
// returns a range proof and the root hash of the tree
func GenerateRangeProof(size int, loc Where) *SimpleResult {
	data := BuildMap(size)
	root, proofs, allkeys := sdkmaps.ProofsFromMap(data)

	key := GetKey(allkeys, loc)
	proof := proofs[key]

	res := &SimpleResult{
		Key:      []byte(key),
		Value:    toValue(key),
		Proof:    proof,
		RootHash: root,
	}
	return res
}

// Where selects a location for a key - Left, Right, or Middle
type Where int

const (
	Left Where = iota
	Right
	Middle
)

func SortedKeys(data map[string][]byte) []string {
	keys := maps.Keys(data)
	sort.Strings(keys)
	return keys
}

func CalcRoot(data map[string][]byte) []byte {
	root, _, _ := sdkmaps.ProofsFromMap(data)
	return root
}

// GetKey this returns a key, on Left/Right/Middle
func GetKey(allkeys []string, loc Where) string {
	if loc == Left {
		return allkeys[0]
	}
	if loc == Right {
		return allkeys[len(allkeys)-1]
	}
	// select a random index between 1 and allkeys-2
	idx := rand.NewRand().Int()%(len(allkeys)-2) + 1
	return allkeys[idx]
}

// GetNonKey returns a missing key - Left of all, Right of all, or in the Middle
func GetNonKey(allkeys []string, loc Where) string {
	if loc == Left {
		return string([]byte{1, 1, 1, 1})
	}
	if loc == Right {
		return string([]byte{0xff, 0xff, 0xff, 0xff})
	}
	// otherwise, next to an existing key (copy before mod)
	key := GetKey(allkeys, loc)
	key = key[:len(key)-2] + string([]byte{255, 255})
	return key
}

func toValue(key string) []byte {
	return []byte("value_for_" + key)
}

// BuildMap creates random key/values and stores in a map,
// returns a list of all keys in sorted order
func BuildMap(size int) map[string][]byte {
	data := make(map[string][]byte)
	// insert lots of info and store the bytes
	for i := 0; i < size; i++ {
		key := rand.Str(20)
		data[key] = toValue(key)
	}
	return data
}

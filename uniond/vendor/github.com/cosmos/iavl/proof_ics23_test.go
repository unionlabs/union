package iavl

import (
	"bytes"
	"crypto/rand"
	mrand "math/rand"
	"sort"
	"testing"

	ics23 "github.com/cosmos/ics23/go"
	"github.com/stretchr/testify/require"

	dbm "github.com/cosmos/iavl/db"
)

func TestGetMembership(t *testing.T) {
	cases := map[string]struct {
		size int
		loc  Where
	}{
		"small left":   {size: 100, loc: Left},
		"small middle": {size: 100, loc: Middle},
		"small right":  {size: 100, loc: Right},
		"big left":     {size: 5431, loc: Left},
		"big middle":   {size: 5431, loc: Middle},
		"big right":    {size: 5431, loc: Right},
	}

	for name, tc := range cases {
		tc := tc
		t.Run(name, func(t *testing.T) {
			tree, allkeys, err := BuildTree(tc.size, 0)
			require.NoError(t, err, "Creating tree: %+v", err)

			key := GetKey(allkeys, tc.loc)
			val, err := tree.Get(key)
			require.NoError(t, err)
			proof, err := tree.GetMembershipProof(key)
			require.NoError(t, err, "Creating Proof: %+v", err)

			root := tree.WorkingHash()
			valid := ics23.VerifyMembership(ics23.IavlSpec, root, proof, key, val)
			require.True(t, valid, "Membership Proof Invalid")
		})
	}
}

func TestGetNonMembership(t *testing.T) {
	cases := map[string]struct {
		size int
		loc  Where
	}{
		"small left":   {size: 100, loc: Left},
		"small middle": {size: 100, loc: Middle},
		"small right":  {size: 100, loc: Right},
		"big left":     {size: 5431, loc: Left},
		"big middle":   {size: 5431, loc: Middle},
		"big right":    {size: 5431, loc: Right},
	}

	performTest := func(tree *MutableTree, allKeys [][]byte, loc Where) {
		key := GetNonKey(allKeys, loc)

		proof, err := tree.GetNonMembershipProof(key)
		require.NoError(t, err, "Creating Proof: %+v", err)

		root := tree.WorkingHash()
		valid := ics23.VerifyNonMembership(ics23.IavlSpec, root, proof, key)
		require.True(t, valid, "Non Membership Proof Invalid")
	}

	for name, tc := range cases {
		tc := tc
		t.Run("fast-"+name, func(t *testing.T) {
			tree, allkeys, err := BuildTree(tc.size, 0)
			require.NoError(t, err, "Creating tree: %+v", err)
			// Save version to enable fast cache
			_, _, err = tree.SaveVersion()
			require.NoError(t, err)

			isFastCacheEnabled, err := tree.IsFastCacheEnabled()
			require.NoError(t, err)
			require.True(t, isFastCacheEnabled)

			performTest(tree, allkeys, tc.loc)
		})

		t.Run("regular-"+name, func(t *testing.T) {
			tree, allkeys, err := BuildTree(tc.size, 0)
			require.NoError(t, err, "Creating tree: %+v", err)
			isFastCacheEnabled, err := tree.IsFastCacheEnabled()
			require.NoError(t, err)
			require.False(t, isFastCacheEnabled)

			performTest(tree, allkeys, tc.loc)
		})
	}
}

func BenchmarkGetNonMembership(b *testing.B) {
	cases := []struct {
		size int
		loc  Where
	}{
		{size: 100, loc: Left},
		{size: 100, loc: Middle},
		{size: 100, loc: Right},
		{size: 5431, loc: Left},
		{size: 5431, loc: Middle},
		{size: 5431, loc: Right},
	}

	performTest := func(tree *MutableTree, allKeys [][]byte, loc Where) {
		key := GetNonKey(allKeys, loc)

		proof, err := tree.GetNonMembershipProof(key)
		require.NoError(b, err, "Creating Proof: %+v", err)

		b.StopTimer()
		root := tree.WorkingHash()
		valid := ics23.VerifyNonMembership(ics23.IavlSpec, root, proof, key)
		require.True(b, valid)
		b.StartTimer()
	}

	b.Run("fast", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			b.StopTimer()
			caseIdx := mrand.Intn(len(cases))
			tc := cases[caseIdx]

			tree, allkeys, err := BuildTree(tc.size, 100000)
			require.NoError(b, err, "Creating tree: %+v", err)
			// Save version to enable fast cache
			_, _, err = tree.SaveVersion()
			require.NoError(b, err)

			isFastCacheEnabled, err := tree.IsFastCacheEnabled()
			require.NoError(b, err)
			require.True(b, isFastCacheEnabled)
			b.StartTimer()
			performTest(tree, allkeys, tc.loc)
		}
	})

	b.Run("regular", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			b.StopTimer()
			caseIdx := mrand.Intn(len(cases))
			tc := cases[caseIdx]

			tree, allkeys, err := BuildTree(tc.size, 100000)
			require.NoError(b, err, "Creating tree: %+v", err)
			isFastCacheEnabled, err := tree.IsFastCacheEnabled()
			require.NoError(b, err)
			require.False(b, isFastCacheEnabled)

			b.StartTimer()
			performTest(tree, allkeys, tc.loc)
		}
	})
}

// Test Helpers

// Where selects a location for a key - Left, Right, or Middle
type Where int

const (
	Left Where = iota
	Right
	Middle
)

// GetKey this returns a key, on Left/Right/Middle
func GetKey(allkeys [][]byte, loc Where) []byte {
	if loc == Left {
		return allkeys[0]
	}
	if loc == Right {
		return allkeys[len(allkeys)-1]
	}
	// select a random index between 1 and allkeys-2
	idx := mrand.Int()%(len(allkeys)-2) + 1
	return allkeys[idx]
}

// GetNonKey returns a missing key - Left of all, Right of all, or in the Middle
func GetNonKey(allkeys [][]byte, loc Where) []byte {
	if loc == Left {
		return []byte{0, 0, 0, 1}
	}
	if loc == Right {
		return []byte{0xff, 0xff, 0xff, 0xff}
	}
	// otherwise, next to an existing key (copy before mod)
	key := append([]byte{}, GetKey(allkeys, loc)...)
	key[len(key)-2] = 255
	key[len(key)-1] = 255
	return key
}

// BuildTree creates random key/values and stores in tree
// returns a list of all keys in sorted order
func BuildTree(size int, cacheSize int) (itree *MutableTree, keys [][]byte, err error) {
	tree := NewMutableTree(dbm.NewMemDB(), cacheSize, false, NewNopLogger())

	// insert lots of info and store the bytes
	keys = make([][]byte, size)
	for i := 0; i < size; i++ {
		key := make([]byte, 4)
		// create random 4 byte key
		rand.Read(key) //nolint:errcheck
		value := "value_for_key:" + string(key)
		_, err = tree.Set(key, []byte(value))
		if err != nil {
			return nil, nil, err
		}
		keys[i] = key
	}
	sort.Slice(keys, func(i, j int) bool {
		return bytes.Compare(keys[i], keys[j]) < 0
	})

	return tree, keys, nil
}

// sink is kept as a global to ensure that value checks and assignments to it can't be
// optimized away, and this will help us ensure that benchmarks successfully run.
var sink interface{}

func BenchmarkConvertLeafOp(b *testing.B) {
	versions := []int64{
		0,
		1,
		100,
		127,
		128,
		1 << 29,
		-0,
		-1,
		-100,
		-127,
		-128,
		-1 << 29,
	}

	b.ReportAllocs()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		for _, version := range versions {
			sink = convertLeafOp(version)
		}
	}
	if sink == nil {
		b.Fatal("Benchmark wasn't run")
	}
	sink = nil
}

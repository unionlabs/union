// nolint: errcheck
package iavl

import (
	"encoding/hex"
	mrand "math/rand"
	"sort"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	dbm "github.com/cosmos/iavl/db"
	iavlrand "github.com/cosmos/iavl/internal/rand"
)

func TestBasic(t *testing.T) {
	tree := getTestTree(0)
	up, err := tree.Set([]byte("1"), []byte("one"))
	require.NoError(t, err)
	if up {
		t.Error("Did not expect an update (should have been create)")
	}
	up, err = tree.Set([]byte("2"), []byte("two"))
	require.NoError(t, err)
	if up {
		t.Error("Did not expect an update (should have been create)")
	}
	up, err = tree.Set([]byte("2"), []byte("TWO"))
	require.NoError(t, err)
	if !up {
		t.Error("Expected an update")
	}
	up, err = tree.Set([]byte("5"), []byte("five"))
	require.NoError(t, err)
	if up {
		t.Error("Did not expect an update (should have been create)")
	}

	// Test 0x00
	{
		key := []byte{0x00}
		expected := ""

		idx, val, err := tree.GetWithIndex(key)
		require.NoError(t, err)
		if val != nil {
			t.Error("Expected no value to exist")
		}
		if idx != 0 {
			t.Errorf("Unexpected idx %x", idx)
		}
		if string(val) != expected {
			t.Errorf("Unexpected value %s", val)
		}

		val, _ = tree.Get(key)
		if val != nil {
			t.Error("Fast method - expected no value to exist")
		}
		if string(val) != expected {
			t.Errorf("Fast method - Unexpected value %s", val)
		}
	}

	// Test "1"
	{
		key := []byte("1")
		expected := "one"

		idx, val, err := tree.GetWithIndex(key)
		require.NoError(t, err)
		if val == nil {
			t.Error("Expected value to exist")
		}
		if idx != 0 {
			t.Errorf("Unexpected idx %x", idx)
		}
		if string(val) != expected {
			t.Errorf("Unexpected value %s", val)
		}

		val, err = tree.Get(key)
		require.NoError(t, err)
		if val == nil {
			t.Error("Fast method - expected value to exist")
		}
		if string(val) != expected {
			t.Errorf("Fast method - Unexpected value %s", val)
		}
	}

	// Test "2"
	{
		key := []byte("2")
		expected := "TWO"

		idx, val, err := tree.GetWithIndex(key)
		require.NoError(t, err)
		if val == nil {
			t.Error("Expected value to exist")
		}
		if idx != 1 {
			t.Errorf("Unexpected idx %x", idx)
		}
		if string(val) != expected {
			t.Errorf("Unexpected value %s", val)
		}

		val, _ = tree.Get(key)
		if val == nil {
			t.Error("Fast method - expected value to exist")
		}
		if string(val) != expected {
			t.Errorf("Fast method - Unexpected value %s", val)
		}
	}

	// Test "4"
	{
		key := []byte("4")
		expected := ""

		idx, val, err := tree.GetWithIndex(key)
		require.NoError(t, err)
		if val != nil {
			t.Error("Expected no value to exist")
		}
		if idx != 2 {
			t.Errorf("Unexpected idx %x", idx)
		}
		if string(val) != expected {
			t.Errorf("Unexpected value %s", val)
		}

		val, _ = tree.Get(key)
		if val != nil {
			t.Error("Fast method - expected no value to exist")
		}
		if string(val) != expected {
			t.Errorf("Fast method - Unexpected value %s", val)
		}
	}

	// Test "6"
	{
		key := []byte("6")
		expected := ""

		idx, val, err := tree.GetWithIndex(key)
		require.NoError(t, err)
		if val != nil {
			t.Error("Expected no value to exist")
		}
		if idx != 3 {
			t.Errorf("Unexpected idx %x", idx)
		}
		if string(val) != expected {
			t.Errorf("Unexpected value %s", val)
		}

		val, _ = tree.Get(key)
		if val != nil {
			t.Error("Fast method - expected no value to exist")
		}
		if string(val) != expected {
			t.Errorf("Fast method - Unexpected value %s", val)
		}
	}
}

func TestUnit(t *testing.T) {
	expectSet := func(tree *MutableTree, i int, repr string) {
		tree.SaveVersion()
		updated, err := tree.Set(i2b(i), []byte{})
		require.NoError(t, err)
		// ensure node was added & structure is as expected.
		if updated || P(tree.root, tree.ImmutableTree) != repr {
			t.Fatalf("Adding %v to %v:\nExpected         %v\nUnexpectedly got %v updated:%v",
				i, P(tree.lastSaved.root, tree.lastSaved), repr, P(tree.root, tree.ImmutableTree), updated)
		}
		tree.ImmutableTree = tree.lastSaved.clone()
	}

	expectRemove := func(tree *MutableTree, i int, repr string) {
		tree.SaveVersion()
		value, removed, err := tree.Remove(i2b(i))
		require.NoError(t, err)
		// ensure node was added & structure is as expected.
		if len(value) != 0 || !removed || P(tree.root, tree.ImmutableTree) != repr {
			t.Fatalf("Removing %v from %v:\nExpected         %v\nUnexpectedly got %v value:%v removed:%v",
				i, P(tree.lastSaved.root, tree.lastSaved), repr, P(tree.root, tree.ImmutableTree), value, removed)
		}
		tree.ImmutableTree = tree.lastSaved.clone()
	}

	// Test Set cases:

	// Case 1:
	t1, err := T(N(4, 20))

	require.NoError(t, err)
	expectSet(t1, 8, "((4 8) 20)")
	expectSet(t1, 25, "(4 (20 25))")

	t2, err := T(N(4, N(20, 25)))

	require.NoError(t, err)
	expectSet(t2, 8, "((4 8) (20 25))")
	expectSet(t2, 30, "((4 20) (25 30))")

	t3, err := T(N(N(1, 2), 6))

	require.NoError(t, err)
	expectSet(t3, 4, "((1 2) (4 6))")
	expectSet(t3, 8, "((1 2) (6 8))")

	t4, err := T(N(N(1, 2), N(N(5, 6), N(7, 9))))

	require.NoError(t, err)
	expectSet(t4, 8, "(((1 2) (5 6)) ((7 8) 9))")
	expectSet(t4, 10, "(((1 2) (5 6)) (7 (9 10)))")

	// Test Remove cases:

	t10, err := T(N(N(1, 2), 3))

	require.NoError(t, err)
	expectRemove(t10, 2, "(1 3)")
	expectRemove(t10, 3, "(1 2)")

	t11, err := T(N(N(N(1, 2), 3), N(4, 5)))

	require.NoError(t, err)
	expectRemove(t11, 4, "((1 2) (3 5))")
	expectRemove(t11, 3, "((1 2) (4 5))")
}

func TestRemove(_ *testing.T) {
	keyLen, dataLen := 16, 40

	size := 10000
	t1 := getTestTree(size)

	// insert a bunch of random nodes
	keys := make([][]byte, size)
	l := int32(len(keys))
	for i := 0; i < size; i++ {
		key := iavlrand.RandBytes(keyLen)
		t1.Set(key, iavlrand.RandBytes(dataLen))
		keys[i] = key
	}

	for i := 0; i < 10; i++ {
		step := 50 * i
		// remove a bunch of existing keys (may have been deleted twice)
		for j := 0; j < step; j++ {
			key := keys[mrand.Int31n(l)]
			t1.Remove(key)
		}
		t1.SaveVersion()
	}
}

func TestIntegration(t *testing.T) {
	type record struct {
		key   string
		value string
	}

	records := make([]*record, 400)
	tree := getTestTree(0)

	randomRecord := func() *record {
		return &record{randstr(20), randstr(20)}
	}

	for i := range records {
		r := randomRecord()
		records[i] = r
		updated, err := tree.Set([]byte(r.key), []byte{})
		require.NoError(t, err)
		if updated {
			t.Error("should have not been updated")
		}
		updated, err = tree.Set([]byte(r.key), []byte(r.value))
		require.NoError(t, err)
		if !updated {
			t.Error("should have been updated")
		}
		if tree.Size() != int64(i+1) {
			t.Error("size was wrong", tree.Size(), i+1)
		}
	}

	for _, r := range records {
		has, err := tree.Has([]byte(r.key))
		require.NoError(t, err)
		if !has {
			t.Error("Missing key", r.key)
		}

		has, err = tree.Has([]byte(randstr(12)))
		require.NoError(t, err)
		if has {
			t.Error("Table has extra key")
		}

		val, err := tree.Get([]byte(r.key))
		require.NoError(t, err)
		if string(val) != r.value {
			t.Error("wrong value")
		}
	}

	for i, x := range records {
		if val, removed, err := tree.Remove([]byte(x.key)); err != nil {
			require.NoError(t, err)
		} else if !removed {
			t.Error("Wasn't removed")
		} else if string(val) != x.value {
			t.Error("Wrong value")
		}
		for _, r := range records[i+1:] {
			has, err := tree.Has([]byte(r.key))
			require.NoError(t, err)
			if !has {
				t.Error("Missing key", r.key)
			}

			has, err = tree.Has([]byte(randstr(12)))
			require.NoError(t, err)
			if has {
				t.Error("Table has extra key")
			}

			val, err := tree.Get([]byte(r.key))
			require.NoError(t, err)
			if string(val) != r.value {
				t.Error("wrong value")
			}
		}
		if tree.Size() != int64(len(records)-(i+1)) {
			t.Error("size was wrong", tree.Size(), (len(records) - (i + 1)))
		}
	}
}

func TestIterateRange(t *testing.T) {
	type record struct {
		key   string
		value string
	}

	records := []record{
		{"abc", "123"},
		{"low", "high"},
		{"fan", "456"},
		{"foo", "a"},
		{"foobaz", "c"},
		{"good", "bye"},
		{"foobang", "d"},
		{"foobar", "b"},
		{"food", "e"},
		{"foml", "f"},
	}
	keys := make([]string, len(records))
	for i, r := range records {
		keys[i] = r.key
	}
	sort.Strings(keys)

	tree := getTestTree(0)

	// insert all the data
	for _, r := range records {
		updated, err := tree.Set([]byte(r.key), []byte(r.value))
		require.NoError(t, err)
		if updated {
			t.Error("should have not been updated")
		}
	}
	// test traversing the whole node works... in order
	viewed := []string{}
	tree.Iterate(func(key []byte, _ []byte) bool {
		viewed = append(viewed, string(key))
		return false
	})
	if len(viewed) != len(keys) {
		t.Error("not the same number of keys as expected")
	}
	for i, v := range viewed {
		if v != keys[i] {
			t.Error("Keys out of order", v, keys[i])
		}
	}

	trav := traverser{}
	tree.IterateRange([]byte("foo"), []byte("goo"), true, trav.view)
	expectTraverse(t, trav, "foo", "food", 5)

	trav = traverser{}
	tree.IterateRange([]byte("aaa"), []byte("abb"), true, trav.view)
	expectTraverse(t, trav, "", "", 0)

	trav = traverser{}
	tree.IterateRange(nil, []byte("flap"), true, trav.view)
	expectTraverse(t, trav, "abc", "fan", 2)

	trav = traverser{}
	tree.IterateRange([]byte("foob"), nil, true, trav.view)
	expectTraverse(t, trav, "foobang", "low", 6)

	trav = traverser{}
	tree.IterateRange([]byte("very"), nil, true, trav.view)
	expectTraverse(t, trav, "", "", 0)

	// make sure it doesn't include end
	trav = traverser{}
	tree.IterateRange([]byte("fooba"), []byte("food"), true, trav.view)
	expectTraverse(t, trav, "foobang", "foobaz", 3)

	// make sure backwards also works... (doesn't include end)
	trav = traverser{}
	tree.IterateRange([]byte("fooba"), []byte("food"), false, trav.view)
	expectTraverse(t, trav, "foobaz", "foobang", 3)

	// make sure backwards also works...
	trav = traverser{}
	tree.IterateRange([]byte("g"), nil, false, trav.view)
	expectTraverse(t, trav, "low", "good", 2)
}

func TestPersistence(t *testing.T) {
	db := dbm.NewMemDB()

	// Create some random key value pairs
	records := make(map[string]string)
	for i := 0; i < 10000; i++ {
		records[randstr(20)] = randstr(20)
	}

	// Construct some tree and save it
	t1 := NewMutableTree(db, 0, false, NewNopLogger())
	for key, value := range records {
		t1.Set([]byte(key), []byte(value))
	}
	t1.SaveVersion()

	// Load a tree
	t2 := NewMutableTree(db, 0, false, NewNopLogger())
	t2.Load()
	for key, value := range records {
		t2value, err := t2.Get([]byte(key))
		require.NoError(t, err)
		if string(t2value) != value {
			t.Fatalf("Invalid value. Expected %v, got %v", value, t2value)
		}
	}
}

func TestProof(t *testing.T) {
	// Construct some random tree
	tree := getTestTree(100)
	for i := 0; i < 10; i++ {
		key, value := randstr(20), randstr(20)
		tree.Set([]byte(key), []byte(value))
	}

	// Persist the items so far
	tree.SaveVersion()

	// Add more items so it's not all persisted
	for i := 0; i < 10; i++ {
		key, value := randstr(20), randstr(20)
		tree.Set([]byte(key), []byte(value))
	}

	// Now for each item, construct a proof and verify
	tree.Iterate(func(key []byte, value []byte) bool {
		proof, err := tree.GetMembershipProof(key)
		assert.NoError(t, err)
		assert.Equal(t, value, proof.GetExist().Value)
		res, err := tree.VerifyMembership(proof, key)
		assert.NoError(t, err)
		value2, err := tree.ImmutableTree.Get(key)
		assert.NoError(t, err)
		if value2 != nil {
			assert.True(t, res)
		} else {
			assert.False(t, res)
		}
		return false
	})
}

func TestTreeProof(t *testing.T) {
	db := dbm.NewMemDB()
	tree := NewMutableTree(db, 100, false, NewNopLogger())
	hash := tree.Hash()
	assert.Equal(t, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", hex.EncodeToString(hash))

	// should get false for proof with nil root
	_, err := tree.GetProof([]byte("foo"))
	require.Error(t, err)

	// insert lots of info and store the bytes
	keys := make([][]byte, 200)
	for i := 0; i < 200; i++ {
		key := randstr(20)
		tree.Set([]byte(key), []byte(key))
		keys[i] = []byte(key)
	}

	tree.SaveVersion()

	// query random key fails
	_, err = tree.GetMembershipProof([]byte("foo"))
	assert.Error(t, err)

	// valid proof for real keys
	for _, key := range keys {
		proof, err := tree.GetMembershipProof(key)
		if assert.NoError(t, err) {
			require.Nil(t, err, "Failed to read proof from bytes: %v", err)
			assert.Equal(t, key, proof.GetExist().Value)
			res, err := tree.VerifyMembership(proof, key)
			require.NoError(t, err)
			require.True(t, res)
		}
	}
}

package metamorphic

import (
	"bytes"
	"testing"

	"github.com/cockroachdb/pebble/internal/randvar"
	"github.com/stretchr/testify/require"
)

func TestObjKey(t *testing.T) {
	testCases := []struct {
		key  objKey
		want string
	}{
		{
			key:  makeObjKey(makeObjID(dbTag, 0), []byte("foo")),
			want: "db:foo",
		},
		{
			key:  makeObjKey(makeObjID(batchTag, 1), []byte("bar")),
			want: "batch1:bar",
		},
	}

	for _, tc := range testCases {
		t.Run("", func(t *testing.T) {
			require.Equal(t, tc.want, tc.key.String())
		})
	}
}

func TestGlobalStateIndicatesEligibleForSingleDelete(t *testing.T) {
	key := makeObjKey(makeObjID(dbTag, 0), []byte("foo"))
	testCases := []struct {
		meta keyMeta
		want bool
	}{
		{
			meta: keyMeta{
				objKey: key,
			},
			want: false,
		},
		{
			meta: keyMeta{
				objKey: key,
				sets:   1,
			},
			want: true,
		},
		{
			meta: keyMeta{
				objKey: key,
				sets:   2,
			},
			want: false,
		},
		{
			meta: keyMeta{
				objKey: key,
				sets:   1,
				merges: 1,
			},
			want: false,
		},
		{
			meta: keyMeta{
				objKey: key,
				sets:   1,
				dels:   1,
			},
			want: false,
		},
		{
			meta: keyMeta{
				objKey:    key,
				sets:      1,
				singleDel: true,
			},
			want: false,
		},
	}

	for _, tc := range testCases {
		k := newKeyManager()
		t.Run("", func(t *testing.T) {
			k.globalKeysMap[string(key.key)] = &tc.meta
			require.Equal(t, tc.want, k.globalStateIndicatesEligibleForSingleDelete(key.key))
		})
	}
}

func TestKeyMeta_MergeInto(t *testing.T) {
	testCases := []struct {
		existing keyMeta
		toMerge  keyMeta
		expected keyMeta
	}{
		{
			existing: keyMeta{
				sets:      1,
				merges:    0,
				singleDel: false,
			},
			toMerge: keyMeta{
				sets:      0,
				merges:    0,
				singleDel: true,
			},
			expected: keyMeta{
				sets:      1,
				merges:    0,
				singleDel: true,
				updateOps: []keyUpdate{
					{deleted: true, metaTimestamp: 0},
				},
			},
		},
		{
			existing: keyMeta{
				sets:   3,
				merges: 1,
				dels:   7,
			},
			toMerge: keyMeta{
				sets:   4,
				merges: 2,
				dels:   8,
				del:    true,
			},
			expected: keyMeta{
				sets:   7,
				merges: 3,
				dels:   15,
				del:    true,
				updateOps: []keyUpdate{
					{deleted: true, metaTimestamp: 1},
				},
			},
		},
		{
			existing: keyMeta{
				sets:   3,
				merges: 1,
				dels:   7,
				del:    true,
			},
			toMerge: keyMeta{
				sets:   1,
				merges: 0,
				dels:   8,
				del:    false,
			},
			expected: keyMeta{
				sets:   4,
				merges: 1,
				dels:   15,
				del:    false,
				updateOps: []keyUpdate{
					{deleted: false, metaTimestamp: 2},
				},
			},
		},
	}

	keyManager := newKeyManager()
	for _, tc := range testCases {
		t.Run("", func(t *testing.T) {
			tc.toMerge.mergeInto(keyManager, &tc.existing)
			require.Equal(t, tc.expected, tc.existing)
		})
	}
}

func TestKeyManager_AddKey(t *testing.T) {
	m := newKeyManager()
	require.Empty(t, m.globalKeys)

	k1 := []byte("foo")
	require.True(t, m.addNewKey(k1))
	require.Len(t, m.globalKeys, 1)
	require.Len(t, m.globalKeyPrefixes, 1)
	require.Contains(t, m.globalKeys, k1)
	require.Contains(t, m.globalKeyPrefixes, k1)
	require.False(t, m.addNewKey(k1))
	require.True(t, m.prefixExists([]byte("foo")))
	require.False(t, m.prefixExists([]byte("bar")))

	k2 := []byte("bar")
	require.True(t, m.addNewKey(k2))
	require.Len(t, m.globalKeys, 2)
	require.Len(t, m.globalKeyPrefixes, 2)
	require.Contains(t, m.globalKeys, k2)
	require.Contains(t, m.globalKeyPrefixes, k2)
	require.True(t, m.prefixExists([]byte("bar")))
	k3 := []byte("bax@4")
	require.True(t, m.addNewKey(k3))
	require.Len(t, m.globalKeys, 3)
	require.Len(t, m.globalKeyPrefixes, 3)
	require.Contains(t, m.globalKeys, k3)
	require.Contains(t, m.globalKeyPrefixes, []byte("bax"))
	require.True(t, m.prefixExists([]byte("bax")))
	k4 := []byte("foo@6")
	require.True(t, m.addNewKey(k4))
	require.Len(t, m.globalKeys, 4)
	require.Len(t, m.globalKeyPrefixes, 3)
	require.Contains(t, m.globalKeys, k4)
	require.True(t, m.prefixExists([]byte("foo")))

	require.Equal(t, [][]byte{
		[]byte("bar"), []byte("bax"), []byte("foo"),
	}, m.prefixes())
}

func TestKeyManager_GetOrInit(t *testing.T) {
	id := makeObjID(batchTag, 1)
	key := []byte("foo")
	o := makeObjKey(id, key)

	m := newKeyManager()
	require.NotContains(t, m.byObjKey, o.String())
	require.NotContains(t, m.byObj, id)
	require.Contains(t, m.byObj, makeObjID(dbTag, 0)) // Always contains the DB key.

	meta1 := m.getOrInit(id, key)
	require.Contains(t, m.byObjKey, o.String())
	require.Contains(t, m.byObj, id)

	// Idempotent.
	meta2 := m.getOrInit(id, key)
	require.Equal(t, meta1, meta2)
}

func TestKeyManager_Contains(t *testing.T) {
	id := makeObjID(dbTag, 0)
	key := []byte("foo")

	m := newKeyManager()
	require.False(t, m.contains(id, key))

	m.getOrInit(id, key)
	require.True(t, m.contains(id, key))
}

func TestKeyManager_MergeInto(t *testing.T) {
	fromID := makeObjID(batchTag, 1)
	toID := makeObjID(dbTag, 0)

	m := newKeyManager()

	// Two keys in "from".
	a := m.getOrInit(fromID, []byte("foo"))
	a.sets = 1
	b := m.getOrInit(fromID, []byte("bar"))
	b.merges = 2

	// One key in "to", with same value as a key in "from", that will be merged.
	m.getOrInit(toID, []byte("foo"))

	// Before, there are two sets.
	require.Len(t, m.byObj[fromID], 2)
	require.Len(t, m.byObj[toID], 1)

	m.mergeKeysInto(fromID, toID)

	// Keys in "from" sets are moved to "to" set.
	require.Len(t, m.byObj[toID], 2)

	// Key "foo" was merged into "to".
	foo := m.getOrInit(toID, []byte("foo"))
	require.Equal(t, 1, foo.sets) // value was merged.

	// Key "bar" was merged into "to".
	bar := m.getOrInit(toID, []byte("bar"))
	require.Equal(t, 2, bar.merges) // value was unchanged.

	// Keys in "from" sets are removed from maps.
	require.NotContains(t, m.byObjKey, makeObjKey(fromID, a.key))
	require.NotContains(t, m.byObjKey, makeObjKey(fromID, b.key))
	require.NotContains(t, m.byObj, fromID)
}

type seqFn func(t *testing.T, k *keyManager)

func updateForOp(op op) seqFn {
	return func(t *testing.T, k *keyManager) {
		k.update(op)
	}
}

func addKey(key []byte, expected bool) seqFn {
	return func(t *testing.T, k *keyManager) {
		require.Equal(t, expected, k.addNewKey(key))
	}
}

func eligibleRead(key []byte, val bool) seqFn {
	return func(t *testing.T, k *keyManager) {
		require.Equal(t, val, contains(key, k.eligibleReadKeys()))
	}
}

func eligibleWrite(key []byte, val bool) seqFn {
	return func(t *testing.T, k *keyManager) {
		require.Equal(t, val, contains(key, k.eligibleWriteKeys()))
	}
}

func eligibleSingleDelete(key []byte, val bool, id objID) seqFn {
	return func(t *testing.T, k *keyManager) {
		require.Equal(t, val, contains(key, k.eligibleSingleDeleteKeys(id)))
	}
}

func contains(key []byte, keys [][]byte) bool {
	for _, k := range keys {
		if bytes.Equal(key, k) {
			return true
		}
	}
	return false
}

func TestKeyManager(t *testing.T) {
	var (
		id1  = makeObjID(batchTag, 0)
		id2  = makeObjID(batchTag, 1)
		key1 = []byte("foo")
	)

	testCases := []struct {
		description string
		ops         []seqFn
		wantPanic   bool
	}{
		{
			description: "set, single del, on db",
			ops: []seqFn{
				addKey(key1, true),
				addKey(key1, false),
				eligibleRead(key1, true),
				eligibleWrite(key1, true),
				eligibleSingleDelete(key1, false, dbObjID),
				eligibleSingleDelete(key1, false, id1),
				updateForOp(&setOp{writerID: dbObjID, key: key1}),
				eligibleRead(key1, true),
				eligibleWrite(key1, true),
				eligibleSingleDelete(key1, true, dbObjID),
				eligibleSingleDelete(key1, true, id1),
				updateForOp(&singleDeleteOp{writerID: dbObjID, key: key1}),
				eligibleRead(key1, true),
				eligibleWrite(key1, true),
				eligibleSingleDelete(key1, false, dbObjID),
			},
		},
		{
			description: "set, single del, on batch",
			ops: []seqFn{
				addKey(key1, true),
				updateForOp(&setOp{writerID: id1, key: key1}),
				eligibleRead(key1, true),
				eligibleWrite(key1, true),
				eligibleSingleDelete(key1, false, dbObjID),
				eligibleSingleDelete(key1, true, id1),
				eligibleSingleDelete(key1, false, id2),
				updateForOp(&singleDeleteOp{writerID: id1, key: key1}),
				eligibleRead(key1, true),
				eligibleWrite(key1, false),
				eligibleSingleDelete(key1, false, dbObjID),
				eligibleSingleDelete(key1, false, id1),
				updateForOp(&applyOp{batchID: id1, writerID: dbObjID}),
				eligibleWrite(key1, true),
				eligibleSingleDelete(key1, false, dbObjID),
			},
		},
		{
			description: "set on db, single del on batch",
			ops: []seqFn{
				addKey(key1, true),
				updateForOp(&setOp{writerID: dbObjID, key: key1}),
				eligibleWrite(key1, true),
				eligibleSingleDelete(key1, true, dbObjID),
				eligibleSingleDelete(key1, true, id1),
				updateForOp(&singleDeleteOp{writerID: id1, key: key1}),
				eligibleWrite(key1, false),
				eligibleSingleDelete(key1, false, dbObjID),
				eligibleSingleDelete(key1, false, id1),
				updateForOp(&applyOp{batchID: id1, writerID: dbObjID}),
				eligibleWrite(key1, true),
				eligibleSingleDelete(key1, false, dbObjID),
				updateForOp(&setOp{writerID: dbObjID, key: key1}),
				eligibleSingleDelete(key1, true, dbObjID),
				eligibleSingleDelete(key1, true, id1),
			},
		},
		{
			description: "set, del, set, single del, on db",
			ops: []seqFn{
				addKey(key1, true),
				updateForOp(&setOp{writerID: dbObjID, key: key1}),
				updateForOp(&deleteOp{writerID: dbObjID, key: key1}),
				eligibleWrite(key1, true),
				eligibleSingleDelete(key1, false, dbObjID),
				updateForOp(&setOp{writerID: dbObjID, key: key1}),
				eligibleWrite(key1, true),
				eligibleSingleDelete(key1, true, dbObjID),
				updateForOp(&singleDeleteOp{writerID: dbObjID, key: key1}),
				eligibleWrite(key1, true),
				eligibleSingleDelete(key1, false, dbObjID),
			},
		},
		{
			description: "set, del, set, del, on batches",
			ops: []seqFn{
				addKey(key1, true),
				updateForOp(&setOp{writerID: id1, key: key1}),
				updateForOp(&deleteOp{writerID: id1, key: key1}),
				updateForOp(&setOp{writerID: id1, key: key1}),
				eligibleWrite(key1, true),
				eligibleSingleDelete(key1, false, id1),
				updateForOp(&applyOp{batchID: id1, writerID: dbObjID}),
				eligibleWrite(key1, true),
				// Not eligible for single del since the set count is 2.
				eligibleSingleDelete(key1, false, dbObjID),
				updateForOp(&setOp{writerID: dbObjID, key: key1}),
				// Not eligible for single del since the set count is 3.
				eligibleSingleDelete(key1, false, dbObjID),
				updateForOp(&deleteOp{writerID: id2, key: key1}),
				updateForOp(&applyOp{batchID: id2, writerID: dbObjID}),
				// Set count is 0.
				eligibleSingleDelete(key1, false, dbObjID),
				// Set count is 1.
				updateForOp(&setOp{writerID: dbObjID, key: key1}),
				eligibleSingleDelete(key1, true, dbObjID),
			},
		},
		{
			description: "set, merge, del, set, single del, on db",
			ops: []seqFn{
				addKey(key1, true),
				updateForOp(&setOp{writerID: dbObjID, key: key1}),
				eligibleSingleDelete(key1, true, dbObjID),
				updateForOp(&mergeOp{writerID: dbObjID, key: key1}),
				eligibleSingleDelete(key1, false, dbObjID),
				updateForOp(&deleteOp{writerID: dbObjID, key: key1}),
				eligibleWrite(key1, true),
				eligibleSingleDelete(key1, false, dbObjID),
				updateForOp(&setOp{writerID: dbObjID, key: key1}),
				eligibleWrite(key1, true),
				eligibleSingleDelete(key1, true, dbObjID),
				updateForOp(&singleDeleteOp{writerID: dbObjID, key: key1}),
				eligibleWrite(key1, true),
				eligibleSingleDelete(key1, false, dbObjID),
			},
		},
		{
			description: "set, del on db, set, single del on batch",
			ops: []seqFn{
				addKey(key1, true),
				updateForOp(&setOp{writerID: dbObjID, key: key1}),
				eligibleSingleDelete(key1, true, dbObjID),
				updateForOp(&deleteOp{writerID: dbObjID, key: key1}),
				eligibleWrite(key1, true),
				eligibleSingleDelete(key1, false, dbObjID),
				updateForOp(&setOp{writerID: id1, key: key1}),
				eligibleWrite(key1, true),
				eligibleSingleDelete(key1, false, dbObjID),
				eligibleSingleDelete(key1, true, id1),
				updateForOp(&singleDeleteOp{writerID: id1, key: key1}),
				eligibleWrite(key1, false),
				eligibleSingleDelete(key1, false, id1),
				eligibleSingleDelete(key1, false, dbObjID),
				updateForOp(&applyOp{batchID: id1, writerID: dbObjID}),
				eligibleWrite(key1, true),
				eligibleSingleDelete(key1, false, dbObjID),
				updateForOp(&setOp{writerID: dbObjID, key: key1}),
				eligibleSingleDelete(key1, true, dbObjID),
			},
		},
	}

	for _, tc := range testCases {
		t.Run(tc.description, func(t *testing.T) {
			m := newKeyManager()
			tFunc := func() {
				for _, op := range tc.ops {
					op(t, m)
				}
			}
			if tc.wantPanic {
				require.Panics(t, tFunc)
			} else {
				tFunc()
			}
		})
	}
}

func TestOpWrittenKeys(t *testing.T) {
	for name, info := range methods {
		t.Run(name, func(t *testing.T) {
			// Any operations that exist in methods but are not handled in
			// opWrittenKeys will result in a panic, failing the subtest.
			opWrittenKeys(info.constructor())
		})
	}
}

func TestLoadPrecedingKeys(t *testing.T) {
	rng := randvar.NewRand()
	cfg := defaultConfig()
	km := newKeyManager()
	ops := generate(rng, 1000, cfg, km)

	cfg2 := defaultConfig()
	km2 := newKeyManager()
	loadPrecedingKeys(t, ops, &cfg2, km2)

	// NB: We can't assert equality, because the original run may not have
	// ever used the max of the distribution.
	require.Greater(t, cfg2.writeSuffixDist.Max(), uint64(1))

	// NB: We can't assert equality, because the original run may have generated
	// keys that it didn't end up using in operations.
	require.Subset(t, km.globalKeys, km2.globalKeys)
	require.Subset(t, km.globalKeyPrefixes, km2.globalKeyPrefixes)
}

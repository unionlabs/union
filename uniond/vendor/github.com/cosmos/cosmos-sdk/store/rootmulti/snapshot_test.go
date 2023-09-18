package rootmulti_test

import (
	"crypto/sha256"
	"encoding/binary"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"math/rand"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	dbm "github.com/cometbft/cometbft-db"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cosmos/cosmos-sdk/snapshots"
	snapshottypes "github.com/cosmos/cosmos-sdk/snapshots/types"
	"github.com/cosmos/cosmos-sdk/store/iavl"
	"github.com/cosmos/cosmos-sdk/store/rootmulti"
	"github.com/cosmos/cosmos-sdk/store/types"
)

func newMultiStoreWithGeneratedData(db dbm.DB, stores uint8, storeKeys uint64) *rootmulti.Store {
	multiStore := rootmulti.NewStore(db, log.NewNopLogger())
	r := rand.New(rand.NewSource(49872768940)) // Fixed seed for deterministic tests

	keys := []*types.KVStoreKey{}
	for i := uint8(0); i < stores; i++ {
		key := types.NewKVStoreKey(fmt.Sprintf("store%v", i))
		multiStore.MountStoreWithDB(key, types.StoreTypeIAVL, nil)
		keys = append(keys, key)
	}
	multiStore.LoadLatestVersion()

	for _, key := range keys {
		store := multiStore.GetCommitKVStore(key).(*iavl.Store)
		for i := uint64(0); i < storeKeys; i++ {
			k := make([]byte, 8)
			v := make([]byte, 1024)
			binary.BigEndian.PutUint64(k, i)
			_, err := r.Read(v)
			if err != nil {
				panic(err)
			}
			store.Set(k, v)
		}
	}

	multiStore.Commit()
	multiStore.LoadLatestVersion()

	return multiStore
}

func newMultiStoreWithMixedMounts(db dbm.DB) *rootmulti.Store {
	store := rootmulti.NewStore(db, log.NewNopLogger())
	store.MountStoreWithDB(types.NewKVStoreKey("iavl1"), types.StoreTypeIAVL, nil)
	store.MountStoreWithDB(types.NewKVStoreKey("iavl2"), types.StoreTypeIAVL, nil)
	store.MountStoreWithDB(types.NewKVStoreKey("iavl3"), types.StoreTypeIAVL, nil)
	store.MountStoreWithDB(types.NewTransientStoreKey("trans1"), types.StoreTypeTransient, nil)
	store.LoadLatestVersion()

	return store
}

func newMultiStoreWithMixedMountsAndBasicData(db dbm.DB) *rootmulti.Store {
	store := newMultiStoreWithMixedMounts(db)
	store1 := store.GetStoreByName("iavl1").(types.CommitKVStore)
	store2 := store.GetStoreByName("iavl2").(types.CommitKVStore)
	trans1 := store.GetStoreByName("trans1").(types.KVStore)

	store1.Set([]byte("a"), []byte{1})
	store1.Set([]byte("b"), []byte{1})
	store2.Set([]byte("X"), []byte{255})
	store2.Set([]byte("A"), []byte{101})
	trans1.Set([]byte("x1"), []byte{91})
	store.Commit()

	store1.Set([]byte("b"), []byte{2})
	store1.Set([]byte("c"), []byte{3})
	store2.Set([]byte("B"), []byte{102})
	store.Commit()

	store2.Set([]byte("C"), []byte{103})
	store2.Delete([]byte("X"))
	trans1.Set([]byte("x2"), []byte{92})
	store.Commit()

	return store
}

func assertStoresEqual(t *testing.T, expect, actual types.CommitKVStore, msgAndArgs ...interface{}) {
	assert.Equal(t, expect.LastCommitID(), actual.LastCommitID())
	expectIter := expect.Iterator(nil, nil)
	expectMap := map[string][]byte{}
	for ; expectIter.Valid(); expectIter.Next() {
		expectMap[string(expectIter.Key())] = expectIter.Value()
	}
	require.NoError(t, expectIter.Error())

	actualIter := expect.Iterator(nil, nil)
	actualMap := map[string][]byte{}
	for ; actualIter.Valid(); actualIter.Next() {
		actualMap[string(actualIter.Key())] = actualIter.Value()
	}
	require.NoError(t, actualIter.Error())

	assert.Equal(t, expectMap, actualMap, msgAndArgs...)
}

func TestMultistoreSnapshot_Checksum(t *testing.T) {
	// Chunks from different nodes must fit together, so all nodes must produce identical chunks.
	// This checksum test makes sure that the byte stream remains identical. If the test fails
	// without having changed the data (e.g. because the Protobuf or zlib encoding changes),
	// snapshottypes.CurrentFormat must be bumped.
	store := newMultiStoreWithGeneratedData(dbm.NewMemDB(), 5, 10000)
	version := uint64(store.LastCommitID().Version)

	testcases := []struct {
		format      uint32
		chunkHashes []string
	}{
		{1, []string{
			"503e5b51b657055b77e88169fadae543619368744ad15f1de0736c0a20482f24",
			"e1a0daaa738eeb43e778aefd2805e3dd720798288a410b06da4b8459c4d8f72e",
			"aa048b4ee0f484965d7b3b06822cf0772cdcaad02f3b1b9055e69f2cb365ef3c",
			"7921eaa3ed4921341e504d9308a9877986a879fe216a099c86e8db66fcba4c63",
			"a4a864e6c02c9fca5837ec80dc84f650b25276ed7e4820cf7516ced9f9901b86",
			"980925390cc50f14998ecb1e87de719ca9dd7e72f5fefbe445397bf670f36c31",
		}},
	}
	for _, tc := range testcases {
		tc := tc
		t.Run(fmt.Sprintf("Format %v", tc.format), func(t *testing.T) {
			ch := make(chan io.ReadCloser)
			go func() {
				streamWriter := snapshots.NewStreamWriter(ch)
				defer streamWriter.Close()
				require.NotNil(t, streamWriter)
				err := store.Snapshot(version, streamWriter)
				require.NoError(t, err)
			}()
			hashes := []string{}
			hasher := sha256.New()
			for chunk := range ch {
				hasher.Reset()
				_, err := io.Copy(hasher, chunk)
				require.NoError(t, err)
				hashes = append(hashes, hex.EncodeToString(hasher.Sum(nil)))
			}
			assert.Equal(t, tc.chunkHashes, hashes,
				"Snapshot output for format %v has changed", tc.format)
		})
	}
}

func TestMultistoreSnapshot_Errors(t *testing.T) {
	store := newMultiStoreWithMixedMountsAndBasicData(dbm.NewMemDB())

	testcases := map[string]struct {
		height     uint64
		expectType error
	}{
		"0 height":       {0, nil},
		"unknown height": {9, nil},
	}
	for name, tc := range testcases {
		tc := tc
		t.Run(name, func(t *testing.T) {
			err := store.Snapshot(tc.height, nil)
			require.Error(t, err)
			if tc.expectType != nil {
				assert.True(t, errors.Is(err, tc.expectType))
			}
		})
	}
}

func TestMultistoreSnapshotRestore(t *testing.T) {
	source := newMultiStoreWithMixedMountsAndBasicData(dbm.NewMemDB())
	target := newMultiStoreWithMixedMounts(dbm.NewMemDB())
	version := uint64(source.LastCommitID().Version)
	require.EqualValues(t, 3, version)
	dummyExtensionItem := snapshottypes.SnapshotItem{
		Item: &snapshottypes.SnapshotItem_Extension{
			Extension: &snapshottypes.SnapshotExtensionMeta{
				Name:   "test",
				Format: 1,
			},
		},
	}

	chunks := make(chan io.ReadCloser, 100)
	go func() {
		streamWriter := snapshots.NewStreamWriter(chunks)
		require.NotNil(t, streamWriter)
		defer streamWriter.Close()
		err := source.Snapshot(version, streamWriter)
		require.NoError(t, err)
		// write an extension metadata
		err = streamWriter.WriteMsg(&dummyExtensionItem)
		require.NoError(t, err)
	}()

	streamReader, err := snapshots.NewStreamReader(chunks)
	require.NoError(t, err)
	nextItem, err := target.Restore(version, snapshottypes.CurrentFormat, streamReader)
	require.NoError(t, err)
	require.Equal(t, *dummyExtensionItem.GetExtension(), *nextItem.GetExtension())

	assert.Equal(t, source.LastCommitID(), target.LastCommitID())
	for _, key := range source.StoreKeysByName() {
		sourceStore := source.GetStoreByName(key.Name()).(types.CommitKVStore)
		targetStore := target.GetStoreByName(key.Name()).(types.CommitKVStore)
		switch sourceStore.GetStoreType() {
		case types.StoreTypeTransient:
			assert.False(t, targetStore.Iterator(nil, nil).Valid(),
				"transient store %v not empty", key.Name())
		default:
			assertStoresEqual(t, sourceStore, targetStore, "store %q not equal", key.Name())
		}
	}
}

func benchmarkMultistoreSnapshot(b *testing.B, stores uint8, storeKeys uint64) {
	b.Skip("Noisy with slow setup time, please see https://github.com/cosmos/cosmos-sdk/issues/8855.")

	b.ReportAllocs()
	b.StopTimer()
	source := newMultiStoreWithGeneratedData(dbm.NewMemDB(), stores, storeKeys)
	version := source.LastCommitID().Version
	require.EqualValues(b, 1, version)
	b.StartTimer()

	for i := 0; i < b.N; i++ {
		target := rootmulti.NewStore(dbm.NewMemDB(), log.NewNopLogger())
		for _, key := range source.StoreKeysByName() {
			target.MountStoreWithDB(key, types.StoreTypeIAVL, nil)
		}
		err := target.LoadLatestVersion()
		require.NoError(b, err)
		require.EqualValues(b, 0, target.LastCommitID().Version)

		chunks := make(chan io.ReadCloser)
		go func() {
			streamWriter := snapshots.NewStreamWriter(chunks)
			require.NotNil(b, streamWriter)
			err := source.Snapshot(uint64(version), streamWriter)
			require.NoError(b, err)
		}()
		for reader := range chunks {
			_, err := io.Copy(io.Discard, reader)
			require.NoError(b, err)
			err = reader.Close()
			require.NoError(b, err)
		}
	}
}

func benchmarkMultistoreSnapshotRestore(b *testing.B, stores uint8, storeKeys uint64) {
	b.Skip("Noisy with slow setup time, please see https://github.com/cosmos/cosmos-sdk/issues/8855.")

	b.ReportAllocs()
	b.StopTimer()
	source := newMultiStoreWithGeneratedData(dbm.NewMemDB(), stores, storeKeys)
	version := uint64(source.LastCommitID().Version)
	require.EqualValues(b, 1, version)
	b.StartTimer()

	for i := 0; i < b.N; i++ {
		target := rootmulti.NewStore(dbm.NewMemDB(), log.NewNopLogger())
		for _, key := range source.StoreKeysByName() {
			target.MountStoreWithDB(key, types.StoreTypeIAVL, nil)
		}
		err := target.LoadLatestVersion()
		require.NoError(b, err)
		require.EqualValues(b, 0, target.LastCommitID().Version)

		chunks := make(chan io.ReadCloser)
		go func() {
			writer := snapshots.NewStreamWriter(chunks)
			require.NotNil(b, writer)
			err := source.Snapshot(version, writer)
			require.NoError(b, err)
		}()
		reader, err := snapshots.NewStreamReader(chunks)
		require.NoError(b, err)
		_, err = target.Restore(version, snapshottypes.CurrentFormat, reader)
		require.NoError(b, err)
		require.Equal(b, source.LastCommitID(), target.LastCommitID())
	}
}

func BenchmarkMultistoreSnapshot100K(b *testing.B) {
	benchmarkMultistoreSnapshot(b, 10, 10000)
}

func BenchmarkMultistoreSnapshot1M(b *testing.B) {
	benchmarkMultistoreSnapshot(b, 10, 100000)
}

func BenchmarkMultistoreSnapshotRestore100K(b *testing.B) {
	benchmarkMultistoreSnapshotRestore(b, 10, 10000)
}

func BenchmarkMultistoreSnapshotRestore1M(b *testing.B) {
	benchmarkMultistoreSnapshotRestore(b, 10, 100000)
}

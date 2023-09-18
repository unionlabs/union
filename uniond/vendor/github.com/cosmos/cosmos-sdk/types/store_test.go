package types_test

import (
	"testing"

	dbm "github.com/cometbft/cometbft-db"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/store/rootmulti"
	"github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

type storeTestSuite struct {
	suite.Suite
}

func TestStoreTestSuite(t *testing.T) {
	suite.Run(t, new(storeTestSuite))
}

func (s *storeTestSuite) SetupSuite() {
	s.T().Parallel()
}

func (s *storeTestSuite) TestPrefixEndBytes() {
	testCases := []struct {
		prefix   []byte
		expected []byte
	}{
		{[]byte{byte(55), byte(255), byte(255), byte(0)}, []byte{byte(55), byte(255), byte(255), byte(1)}},
		{[]byte{byte(55), byte(255), byte(255), byte(15)}, []byte{byte(55), byte(255), byte(255), byte(16)}},
		{[]byte{byte(55), byte(200), byte(255)}, []byte{byte(55), byte(201)}},
		{[]byte{byte(55), byte(255), byte(255)}, []byte{byte(56)}},
		{[]byte{byte(255), byte(255), byte(255)}, nil},
		{[]byte{byte(255)}, nil},
		{nil, nil},
	}

	for _, test := range testCases {
		end := sdk.PrefixEndBytes(test.prefix)
		s.Require().Equal(test.expected, end)
	}
}

func (s *storeTestSuite) TestCommitID() {
	var empty types.CommitID
	s.Require().True(empty.IsZero())

	nonempty := types.CommitID{
		Version: 1,
		Hash:    []byte("testhash"),
	}
	s.Require().False(nonempty.IsZero())
}

func (s *storeTestSuite) TestNewTransientStoreKeys() {
	s.Require().Equal(map[string]*types.TransientStoreKey{}, sdk.NewTransientStoreKeys())
	s.Require().Equal(1, len(sdk.NewTransientStoreKeys("one")))
}

func (s *storeTestSuite) TestNewInfiniteGasMeter() {
	gm := sdk.NewInfiniteGasMeter()
	s.Require().NotNil(gm)
	_, ok := gm.(types.GasMeter)
	s.Require().True(ok)
}

func (s *storeTestSuite) TestStoreTypes() {
	s.Require().Equal(sdk.InclusiveEndBytes([]byte("endbytes")), types.InclusiveEndBytes([]byte("endbytes")))
}

func (s *storeTestSuite) TestDiffKVStores() {
	store1, store2 := s.initTestStores()
	// Two equal stores
	k1, v1 := []byte("k1"), []byte("v1")
	store1.Set(k1, v1)
	store2.Set(k1, v1)

	s.checkDiffResults(store1, store2)

	// delete k1 from store2, which is now empty
	store2.Delete(k1)
	s.checkDiffResults(store1, store2)

	// set k1 in store2, different value than what store1 holds for k1
	v2 := []byte("v2")
	store2.Set(k1, v2)
	s.checkDiffResults(store1, store2)

	// add k2 to store2
	k2 := []byte("k2")
	store2.Set(k2, v2)
	s.checkDiffResults(store1, store2)

	// Reset stores
	store1.Delete(k1)
	store2.Delete(k1)
	store2.Delete(k2)

	// Same keys, different value. Comparisons will be nil as prefixes are skipped.
	prefix := []byte("prefix:")
	k1Prefixed := append(prefix, k1...)
	store1.Set(k1Prefixed, v1)
	store2.Set(k1Prefixed, v2)
	s.checkDiffResults(store1, store2)
}

func (s *storeTestSuite) initTestStores() (types.KVStore, types.KVStore) {
	db := dbm.NewMemDB()
	ms := rootmulti.NewStore(db, log.NewNopLogger())

	key1 := types.NewKVStoreKey("store1")
	key2 := types.NewKVStoreKey("store2")
	s.Require().NotPanics(func() { ms.MountStoreWithDB(key1, types.StoreTypeIAVL, db) })
	s.Require().NotPanics(func() { ms.MountStoreWithDB(key2, types.StoreTypeIAVL, db) })
	s.Require().NoError(ms.LoadLatestVersion())
	return ms.GetKVStore(key1), ms.GetKVStore(key2)
}

func (s *storeTestSuite) checkDiffResults(store1, store2 types.KVStore) {
	kvAs1, kvBs1 := sdk.DiffKVStores(store1, store2, nil)
	kvAs2, kvBs2 := types.DiffKVStores(store1, store2, nil)
	s.Require().Equal(kvAs1, kvAs2)
	s.Require().Equal(kvBs1, kvBs2)
}

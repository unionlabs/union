package rootmulti

import (
	"testing"

	dbm "github.com/cometbft/cometbft-db"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/store/iavl"
	"github.com/cosmos/cosmos-sdk/store/types"
)

func TestVerifyIAVLStoreQueryProof(t *testing.T) {
	// Create main tree for testing.
	db := dbm.NewMemDB()
	iStore, err := iavl.LoadStore(db, log.NewNopLogger(), types.NewKVStoreKey("test"), types.CommitID{}, false, iavl.DefaultIAVLCacheSize, false)
	store := iStore.(*iavl.Store)
	require.Nil(t, err)
	store.Set([]byte("MYKEY"), []byte("MYVALUE"))
	cid := store.Commit()

	// Get Proof
	res := store.Query(abci.RequestQuery{
		Path:  "/key", // required path to get key/value+proof
		Data:  []byte("MYKEY"),
		Prove: true,
	})
	require.NotNil(t, res.ProofOps)

	// Verify proof.
	prt := DefaultProofRuntime()
	err = prt.VerifyValue(res.ProofOps, cid.Hash, "/MYKEY", []byte("MYVALUE"))
	require.Nil(t, err)

	// Verify (bad) proof.
	err = prt.VerifyValue(res.ProofOps, cid.Hash, "/MYKEY_NOT", []byte("MYVALUE"))
	require.NotNil(t, err)

	// Verify (bad) proof.
	err = prt.VerifyValue(res.ProofOps, cid.Hash, "/MYKEY/MYKEY", []byte("MYVALUE"))
	require.NotNil(t, err)

	// Verify (bad) proof.
	err = prt.VerifyValue(res.ProofOps, cid.Hash, "MYKEY", []byte("MYVALUE"))
	require.NotNil(t, err)

	// Verify (bad) proof.
	err = prt.VerifyValue(res.ProofOps, cid.Hash, "/MYKEY", []byte("MYVALUE_NOT"))
	require.NotNil(t, err)

	// Verify (bad) proof.
	err = prt.VerifyValue(res.ProofOps, cid.Hash, "/MYKEY", []byte(nil))
	require.NotNil(t, err)
}

func TestVerifyMultiStoreQueryProof(t *testing.T) {
	// Create main tree for testing.
	db := dbm.NewMemDB()
	store := NewStore(db, log.NewNopLogger())
	iavlStoreKey := types.NewKVStoreKey("iavlStoreKey")

	store.MountStoreWithDB(iavlStoreKey, types.StoreTypeIAVL, nil)
	require.NoError(t, store.LoadVersion(0))

	iavlStore := store.GetCommitStore(iavlStoreKey).(*iavl.Store)
	iavlStore.Set([]byte("MYKEY"), []byte("MYVALUE"))
	cid := store.Commit()

	// Get Proof
	res := store.Query(abci.RequestQuery{
		Path:  "/iavlStoreKey/key", // required path to get key/value+proof
		Data:  []byte("MYKEY"),
		Prove: true,
	})
	require.NotNil(t, res.ProofOps)

	// Verify proof.
	prt := DefaultProofRuntime()
	err := prt.VerifyValue(res.ProofOps, cid.Hash, "/iavlStoreKey/MYKEY", []byte("MYVALUE"))
	require.Nil(t, err)

	// Verify proof.
	err = prt.VerifyValue(res.ProofOps, cid.Hash, "/iavlStoreKey/MYKEY", []byte("MYVALUE"))
	require.Nil(t, err)

	// Verify (bad) proof.
	err = prt.VerifyValue(res.ProofOps, cid.Hash, "/iavlStoreKey/MYKEY_NOT", []byte("MYVALUE"))
	require.NotNil(t, err)

	// Verify (bad) proof.
	err = prt.VerifyValue(res.ProofOps, cid.Hash, "/iavlStoreKey/MYKEY/MYKEY", []byte("MYVALUE"))
	require.NotNil(t, err)

	// Verify (bad) proof.
	err = prt.VerifyValue(res.ProofOps, cid.Hash, "iavlStoreKey/MYKEY", []byte("MYVALUE"))
	require.NotNil(t, err)

	// Verify (bad) proof.
	err = prt.VerifyValue(res.ProofOps, cid.Hash, "/MYKEY", []byte("MYVALUE"))
	require.NotNil(t, err)

	// Verify (bad) proof.
	err = prt.VerifyValue(res.ProofOps, cid.Hash, "/iavlStoreKey/MYKEY", []byte("MYVALUE_NOT"))
	require.NotNil(t, err)

	// Verify (bad) proof.
	err = prt.VerifyValue(res.ProofOps, cid.Hash, "/iavlStoreKey/MYKEY", []byte(nil))
	require.NotNil(t, err)
}

func TestVerifyMultiStoreQueryProofAbsence(t *testing.T) {
	// Create main tree for testing.
	db := dbm.NewMemDB()
	store := NewStore(db, log.NewNopLogger())
	iavlStoreKey := types.NewKVStoreKey("iavlStoreKey")

	store.MountStoreWithDB(iavlStoreKey, types.StoreTypeIAVL, nil)
	err := store.LoadVersion(0)
	require.NoError(t, err)

	iavlStore := store.GetCommitStore(iavlStoreKey).(*iavl.Store)
	iavlStore.Set([]byte("MYKEY"), []byte("MYVALUE"))
	cid := store.Commit() // Commit with empty iavl store.

	// Get Proof
	res := store.Query(abci.RequestQuery{
		Path:  "/iavlStoreKey/key", // required path to get key/value+proof
		Data:  []byte("MYABSENTKEY"),
		Prove: true,
	})
	require.NotNil(t, res.ProofOps)

	// Verify proof.
	prt := DefaultProofRuntime()
	err = prt.VerifyAbsence(res.ProofOps, cid.Hash, "/iavlStoreKey/MYABSENTKEY")
	require.Nil(t, err)

	// Verify (bad) proof.
	prt = DefaultProofRuntime()
	err = prt.VerifyAbsence(res.ProofOps, cid.Hash, "/MYABSENTKEY")
	require.NotNil(t, err)

	// Verify (bad) proof.
	prt = DefaultProofRuntime()
	err = prt.VerifyValue(res.ProofOps, cid.Hash, "/iavlStoreKey/MYABSENTKEY", []byte(""))
	require.NotNil(t, err)
}

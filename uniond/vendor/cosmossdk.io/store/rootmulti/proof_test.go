package rootmulti

import (
	"testing"

	"github.com/stretchr/testify/require"

	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/log"
	"cosmossdk.io/store/iavl"
	"cosmossdk.io/store/metrics"
	"cosmossdk.io/store/types"
)

func TestVerifyIAVLStoreQueryProof(t *testing.T) {
	// Create main tree for testing.
	db := coretesting.NewMemDB()
	iStore, err := iavl.LoadStore(db, log.NewNopLogger(), types.NewKVStoreKey("test"), types.CommitID{}, iavl.DefaultIAVLCacheSize, false, metrics.NewNoOpMetrics())
	store := iStore.(*iavl.Store)
	require.Nil(t, err)
	store.Set([]byte("MYKEY"), []byte("MYVALUE"))
	cid := store.Commit()

	// Get Proof
	res, err := store.Query(&types.RequestQuery{
		Path:  "/key", // required path to get key/value+proof
		Data:  []byte("MYKEY"),
		Prove: true,
	})
	require.NoError(t, err)
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
	db := coretesting.NewMemDB()
	store := NewStore(db, log.NewNopLogger(), metrics.NewNoOpMetrics())
	iavlStoreKey := types.NewKVStoreKey("iavlStoreKey")

	store.MountStoreWithDB(iavlStoreKey, types.StoreTypeIAVL, nil)
	require.NoError(t, store.LoadVersion(0))

	iavlStore := store.GetCommitStore(iavlStoreKey).(*iavl.Store)
	iavlStore.Set([]byte("MYKEY"), []byte("MYVALUE"))
	cid := store.Commit()

	// Get Proof
	res, err := store.Query(&types.RequestQuery{
		Path:  "/iavlStoreKey/key", // required path to get key/value+proof
		Data:  []byte("MYKEY"),
		Prove: true,
	})
	require.NoError(t, err)
	require.NotNil(t, res.ProofOps)

	// Verify proof.
	prt := DefaultProofRuntime()
	err = prt.VerifyValue(res.ProofOps, cid.Hash, "/iavlStoreKey/MYKEY", []byte("MYVALUE"))
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
	db := coretesting.NewMemDB()
	store := NewStore(db, log.NewNopLogger(), metrics.NewNoOpMetrics())
	iavlStoreKey := types.NewKVStoreKey("iavlStoreKey")

	store.MountStoreWithDB(iavlStoreKey, types.StoreTypeIAVL, nil)
	err := store.LoadVersion(0)
	require.NoError(t, err)

	iavlStore := store.GetCommitStore(iavlStoreKey).(*iavl.Store)
	iavlStore.Set([]byte("MYKEY"), []byte("MYVALUE"))
	cid := store.Commit() // Commit with empty iavl store.

	// Get Proof
	res, err := store.Query(&types.RequestQuery{
		Path:  "/iavlStoreKey/key", // required path to get key/value+proof
		Data:  []byte("MYABSENTKEY"),
		Prove: true,
	})
	require.NoError(t, err)
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

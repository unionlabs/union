package db

import (
	"sync"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	dbm "github.com/cometbft/cometbft-db"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	cmtversion "github.com/cometbft/cometbft/api/cometbft/version/v1"
	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/tmhash"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
	"github.com/cometbft/cometbft/types"
	cmttime "github.com/cometbft/cometbft/types/time"
	"github.com/cometbft/cometbft/version"
)

func TestDBKeyLayoutVersioning(t *testing.T) {
	prefix := "TestDBKeyLayoutVersioning"
	db := dbm.NewMemDB()
	dbStore := New(db, prefix)

	// Empty store
	height, err := dbStore.LastLightBlockHeight()
	require.NoError(t, err)
	assert.EqualValues(t, -1, height)

	lb := randLightBlock(int64(1))
	// 1 key
	err = dbStore.SaveLightBlock(lb)
	require.NoError(t, err)

	lbKey := v1LegacyLayout{}.LBKey(int64(1), prefix)

	lbRetrieved, err := db.Get(lbKey)
	require.NoError(t, err)

	var lbpb cmtproto.LightBlock
	err = lbpb.Unmarshal(lbRetrieved)
	require.NoError(t, err)

	lightBlock, err := types.LightBlockFromProto(&lbpb)
	require.NoError(t, err)

	require.Equal(t, lightBlock.AppHash, lb.AppHash)
	require.Equal(t, lightBlock.ConsensusHash, lb.ConsensusHash)

	lbKeyV2 := v2Layout{}.LBKey(1, prefix)

	lbv2, err := db.Get(lbKeyV2)
	require.NoError(t, err)
	require.Equal(t, len(lbv2), 0)

	// test on v2

	prefix = "TestDBKeyLayoutVersioningV2"
	db2 := dbm.NewMemDB()
	dbStore2 := NewWithDBVersion(db2, prefix, "v2")

	// Empty store
	height, err = dbStore2.LastLightBlockHeight()
	require.NoError(t, err)
	assert.EqualValues(t, -1, height)

	// 1 key
	err = dbStore2.SaveLightBlock(lb)
	require.NoError(t, err)

	lbKey = v1LegacyLayout{}.LBKey(int64(1), prefix)
	// No block is found if we look for a key parsed with v1
	lbRetrieved, err = db2.Get(lbKey)
	require.NoError(t, err)
	require.Equal(t, len(lbRetrieved), 0)

	// Key parsed with v2 should find the light block
	lbKeyV2 = v2Layout{}.LBKey(1, prefix)
	lbv2, err = db2.Get(lbKeyV2)
	require.NoError(t, err)

	// Unmarshall the light block bytes
	err = lbpb.Unmarshal(lbv2)
	require.NoError(t, err)

	lightBlock, err = types.LightBlockFromProto(&lbpb)
	require.NoError(t, err)

	require.Equal(t, lightBlock.AppHash, lb.AppHash)
	require.Equal(t, lightBlock.ConsensusHash, lb.ConsensusHash)
}

func TestLast_FirstLightBlockHeight(t *testing.T) {
	dbStore := New(dbm.NewMemDB(), "TestLast_FirstLightBlockHeight")

	// Empty store
	height, err := dbStore.LastLightBlockHeight()
	require.NoError(t, err)
	assert.EqualValues(t, -1, height)

	height, err = dbStore.FirstLightBlockHeight()
	require.NoError(t, err)
	assert.EqualValues(t, -1, height)

	// 1 key
	err = dbStore.SaveLightBlock(randLightBlock(int64(1)))
	require.NoError(t, err)

	height, err = dbStore.LastLightBlockHeight()
	require.NoError(t, err)
	assert.EqualValues(t, 1, height)

	height, err = dbStore.FirstLightBlockHeight()
	require.NoError(t, err)
	assert.EqualValues(t, 1, height)
}

func Test_SaveLightBlockCustomConfig(t *testing.T) {
	dbStore := NewWithDBVersion(dbm.NewMemDB(), "Test_SaveLightBlockAndValidatorSet", "v2")

	// Empty store
	h, err := dbStore.LightBlock(1)
	require.Error(t, err)
	assert.Nil(t, h)

	// 1 key
	err = dbStore.SaveLightBlock(randLightBlock(1))
	require.NoError(t, err)

	size := dbStore.Size()
	assert.Equal(t, uint16(1), size)
	t.Log(size)

	h, err = dbStore.LightBlock(1)
	require.NoError(t, err)
	assert.NotNil(t, h)

	// Empty store
	err = dbStore.DeleteLightBlock(1)
	require.NoError(t, err)

	h, err = dbStore.LightBlock(1)
	require.Error(t, err)
	assert.Nil(t, h)
}

func Test_LightBlockBefore(t *testing.T) {
	dbStore := New(dbm.NewMemDB(), "Test_LightBlockBefore")

	assert.Panics(t, func() {
		_, _ = dbStore.LightBlockBefore(0)
		_, _ = dbStore.LightBlockBefore(100)
	})

	err := dbStore.SaveLightBlock(randLightBlock(int64(2)))
	require.NoError(t, err)

	h, err := dbStore.LightBlockBefore(3)
	require.NoError(t, err)
	if assert.NotNil(t, h) {
		assert.EqualValues(t, 2, h.Height)
	}
}

func Test_Prune(t *testing.T) {
	dbStore := New(dbm.NewMemDB(), "Test_Prune")

	// Empty store
	assert.EqualValues(t, 0, dbStore.Size())
	err := dbStore.Prune(0)
	require.NoError(t, err)

	// One header
	err = dbStore.SaveLightBlock(randLightBlock(2))
	require.NoError(t, err)

	assert.EqualValues(t, 1, dbStore.Size())

	err = dbStore.Prune(1)
	require.NoError(t, err)
	assert.EqualValues(t, 1, dbStore.Size())

	err = dbStore.Prune(0)
	require.NoError(t, err)
	assert.EqualValues(t, 0, dbStore.Size())

	// Multiple headers
	for i := 1; i <= 10; i++ {
		err = dbStore.SaveLightBlock(randLightBlock(int64(i)))
		require.NoError(t, err)
	}

	err = dbStore.Prune(11)
	require.NoError(t, err)
	assert.EqualValues(t, 10, dbStore.Size())

	err = dbStore.Prune(7)
	require.NoError(t, err)
	assert.EqualValues(t, 7, dbStore.Size())
}

func Test_Concurrency(t *testing.T) {
	dbStore := New(dbm.NewMemDB(), "Test_Prune")

	var wg sync.WaitGroup
	for i := 1; i <= 100; i++ {
		wg.Add(1)
		go func(i int64) {
			defer wg.Done()

			err := dbStore.SaveLightBlock(randLightBlock(i))
			require.NoError(t, err)

			_, err = dbStore.LightBlock(i)
			if err != nil {
				t.Log(err)
			}

			_, err = dbStore.LastLightBlockHeight()
			if err != nil {
				t.Log(err)
			}
			_, err = dbStore.FirstLightBlockHeight()
			if err != nil {
				t.Log(err)
			}

			err = dbStore.Prune(2)
			if err != nil {
				t.Log(err)
			}
			_ = dbStore.Size()

			err = dbStore.DeleteLightBlock(1)
			if err != nil {
				t.Log(err)
			}
		}(int64(i))
	}

	wg.Wait()
}

func randLightBlock(height int64) *types.LightBlock {
	vals, _ := types.RandValidatorSet(2, 1)
	return &types.LightBlock{
		SignedHeader: &types.SignedHeader{
			Header: &types.Header{
				Version:            cmtversion.Consensus{Block: version.BlockProtocol, App: 0},
				ChainID:            cmtrand.Str(12),
				Height:             height,
				Time:               cmttime.Now(),
				LastBlockID:        types.BlockID{},
				LastCommitHash:     crypto.CRandBytes(tmhash.Size),
				DataHash:           crypto.CRandBytes(tmhash.Size),
				ValidatorsHash:     crypto.CRandBytes(tmhash.Size),
				NextValidatorsHash: crypto.CRandBytes(tmhash.Size),
				ConsensusHash:      crypto.CRandBytes(tmhash.Size),
				AppHash:            crypto.CRandBytes(tmhash.Size),
				LastResultsHash:    crypto.CRandBytes(tmhash.Size),
				EvidenceHash:       crypto.CRandBytes(tmhash.Size),
				ProposerAddress:    crypto.CRandBytes(crypto.AddressSize),
			},
			Commit: &types.Commit{},
		},
		ValidatorSet: vals,
	}
}

package light_test

import (
	"context"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	dbm "github.com/cometbft/cometbft-db"
	"github.com/cometbft/cometbft/internal/test"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/light"
	"github.com/cometbft/cometbft/light/provider"
	mockp "github.com/cometbft/cometbft/light/provider/mock"
	dbs "github.com/cometbft/cometbft/light/store/db"
	"github.com/cometbft/cometbft/types"
)

const (
	chainID = test.DefaultTestChainID
)

var (
	ctx      = context.Background()
	keys     = genPrivKeys(4)
	vals     = keys.ToValidators(20, 10)
	bTime, _ = time.Parse(time.RFC3339, "2006-01-02T15:04:05Z")
	h1       = keys.GenSignedHeader(chainID, 1, bTime, nil, vals, vals,
		hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys))
	// 3/3 signed.
	vals2 = vals.CopyIncrementProposerPriority(1)
	h2    = keys.GenSignedHeaderLastBlockID(chainID, 2, bTime.Add(30*time.Minute), nil, vals2, vals2,
		hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys), types.BlockID{Hash: h1.Hash()})
	// 3/3 signed.
	vals3 = vals2.CopyIncrementProposerPriority(1)
	h3    = keys.GenSignedHeaderLastBlockID(chainID, 3, bTime.Add(1*time.Hour), nil, vals3, vals3,
		hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys), types.BlockID{Hash: h2.Hash()})
	trustPeriod  = 4 * time.Hour
	trustOptions = light.TrustOptions{
		Period: 4 * time.Hour,
		Height: 1,
		Hash:   h1.Hash(),
	}
	valSet = map[int64]*types.ValidatorSet{
		1: vals,
		2: vals2,
		3: vals3,
		4: vals.CopyIncrementProposerPriority(1),
	}
	headerSet = map[int64]*types.SignedHeader{
		1: h1,
		// interim header (3/3 signed)
		2: h2,
		// last header (3/3 signed)
		3: h3,
	}
	l1       = &types.LightBlock{SignedHeader: h1, ValidatorSet: vals}
	l2       = &types.LightBlock{SignedHeader: h2, ValidatorSet: vals2}
	fullNode = mockp.New(
		chainID,
		headerSet,
		valSet,
	)
	deadNode      = mockp.NewDeadMock(chainID)
	largeFullNode = mockp.New(genMockNode(10, 3, 0, bTime))
)

func TestValidateTrustOptions(t *testing.T) {
	testCases := []struct {
		err bool
		to  light.TrustOptions
	}{
		{
			false,
			trustOptions,
		},
		{
			true,
			light.TrustOptions{
				Period: -1 * time.Hour,
				Height: 1,
				Hash:   h1.Hash(),
			},
		},
		{
			true,
			light.TrustOptions{
				Period: 1 * time.Hour,
				Height: 0,
				Hash:   h1.Hash(),
			},
		},
		{
			true,
			light.TrustOptions{
				Period: 1 * time.Hour,
				Height: 1,
				Hash:   []byte("incorrect hash"),
			},
		},
	}

	for _, tc := range testCases {
		err := tc.to.ValidateBasic()
		if tc.err {
			require.Error(t, err)
		} else {
			require.NoError(t, err)
		}
	}
}

func TestMock(t *testing.T) {
	l, _ := fullNode.LightBlock(ctx, 3)
	assert.Equal(t, int64(3), l.Height)
}

func TestClient_SequentialVerification(t *testing.T) {
	newKeys := genPrivKeys(4)
	newVals := newKeys.ToValidators(10, 1)
	differentVals, _ := types.RandValidatorSet(10, 100)

	testCases := []struct {
		name         string
		otherHeaders map[int64]*types.SignedHeader // all except ^
		vals         map[int64]*types.ValidatorSet
		initErr      bool
		verifyErr    bool
	}{
		{
			"good",
			headerSet,
			valSet,
			false,
			false,
		},
		{
			"bad: different first header",
			map[int64]*types.SignedHeader{
				// different header
				1: keys.GenSignedHeader(chainID, 1, bTime.Add(1*time.Hour), nil, vals, vals,
					hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys)),
			},
			map[int64]*types.ValidatorSet{
				1: vals,
			},
			true,
			false,
		},
		{
			"bad: no first signed header",
			map[int64]*types.SignedHeader{},
			map[int64]*types.ValidatorSet{
				1: differentVals,
			},
			true,
			true,
		},
		{
			"bad: different first validator set",
			map[int64]*types.SignedHeader{
				1: h1,
			},
			map[int64]*types.ValidatorSet{
				1: differentVals,
			},
			true,
			true,
		},
		{
			"bad: 1/3 signed interim header",
			map[int64]*types.SignedHeader{
				// trusted header
				1: h1,
				// interim header (1/3 signed)
				2: keys.GenSignedHeader(chainID, 2, bTime.Add(1*time.Hour), nil, vals, vals,
					hash("app_hash"), hash("cons_hash"), hash("results_hash"), len(keys)-1, len(keys)),
				// last header (3/3 signed)
				3: keys.GenSignedHeader(chainID, 3, bTime.Add(2*time.Hour), nil, vals, vals,
					hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys)),
			},
			valSet,
			false,
			true,
		},
		{
			"bad: 1/3 signed last header",
			map[int64]*types.SignedHeader{
				// trusted header
				1: h1,
				// interim header (3/3 signed)
				2: keys.GenSignedHeader(chainID, 2, bTime.Add(1*time.Hour), nil, vals, vals,
					hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys)),
				// last header (1/3 signed)
				3: keys.GenSignedHeader(chainID, 3, bTime.Add(2*time.Hour), nil, vals, vals,
					hash("app_hash"), hash("cons_hash"), hash("results_hash"), len(keys)-1, len(keys)),
			},
			valSet,
			false,
			true,
		},
		{
			"bad: different validator set at height 3",
			headerSet,
			map[int64]*types.ValidatorSet{
				1: vals,
				2: vals,
				3: newVals,
			},
			false,
			true,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			c, err := light.NewClient(
				ctx,
				chainID,
				trustOptions,
				mockp.New(
					chainID,
					tc.otherHeaders,
					tc.vals,
				),
				[]provider.Provider{mockp.New(
					chainID,
					tc.otherHeaders,
					tc.vals,
				)},
				dbs.New(dbm.NewMemDB(), chainID),
				light.SequentialVerification(),
				light.Logger(log.TestingLogger()),
			)

			if tc.initErr {
				require.Error(t, err)
				return
			}

			require.NoError(t, err)

			_, err = c.VerifyLightBlockAtHeight(ctx, 3, bTime.Add(3*time.Hour))
			if tc.verifyErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestClient_SkippingVerification(t *testing.T) {
	// required for 2nd test case
	newKeys := genPrivKeys(4)
	newVals := newKeys.ToValidators(10, 1)

	// 1/3+ of vals, 2/3- of newVals
	transitKeys := keys.Extend(3)
	transitVals := transitKeys.ToValidators(10, 1)

	testCases := []struct {
		name         string
		otherHeaders map[int64]*types.SignedHeader // all except ^
		vals         map[int64]*types.ValidatorSet
		initErr      bool
		verifyErr    bool
	}{
		{
			"good",
			map[int64]*types.SignedHeader{
				// trusted header
				1: h1,
				// last header (3/3 signed)
				3: h3,
			},
			valSet,
			false,
			false,
		},
		{
			"good, but val set changes by 2/3 (1/3 of vals is still present)",
			map[int64]*types.SignedHeader{
				// trusted header
				1: h1,
				3: transitKeys.GenSignedHeader(chainID, 3, bTime.Add(2*time.Hour), nil, transitVals, transitVals,
					hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(transitKeys)),
			},
			map[int64]*types.ValidatorSet{
				1: vals,
				2: vals,
				3: transitVals,
			},
			false,
			false,
		},
		{
			"good, but val set changes 100% at height 2",
			map[int64]*types.SignedHeader{
				// trusted header
				1: h1,
				// interim header (3/3 signed)
				2: keys.GenSignedHeader(chainID, 2, bTime.Add(1*time.Hour), nil, vals, newVals,
					hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys)),
				// last header (0/4 of the original val set signed)
				3: newKeys.GenSignedHeader(chainID, 3, bTime.Add(2*time.Hour), nil, newVals, newVals,
					hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(newKeys)),
			},
			map[int64]*types.ValidatorSet{
				1: vals,
				2: vals,
				3: newVals,
			},
			false,
			false,
		},
		{
			"bad: last header signed by newVals, interim header has no signers",
			map[int64]*types.SignedHeader{
				// trusted header
				1: h1,
				// last header (0/4 of the original val set signed)
				2: keys.GenSignedHeader(chainID, 2, bTime.Add(1*time.Hour), nil, vals, newVals,
					hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, 0),
				// last header (0/4 of the original val set signed)
				3: newKeys.GenSignedHeader(chainID, 3, bTime.Add(2*time.Hour), nil, newVals, newVals,
					hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(newKeys)),
			},
			map[int64]*types.ValidatorSet{
				1: vals,
				2: vals,
				3: newVals,
			},
			false,
			true,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			c, err := light.NewClient(
				ctx,
				chainID,
				trustOptions,
				mockp.New(
					chainID,
					tc.otherHeaders,
					tc.vals,
				),
				[]provider.Provider{mockp.New(
					chainID,
					tc.otherHeaders,
					tc.vals,
				)},
				dbs.New(dbm.NewMemDB(), chainID),
				light.SkippingVerification(light.DefaultTrustLevel),
				light.Logger(log.TestingLogger()),
			)
			if tc.initErr {
				require.Error(t, err)
				return
			}

			require.NoError(t, err)

			_, err = c.VerifyLightBlockAtHeight(ctx, 3, bTime.Add(3*time.Hour))
			if tc.verifyErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

// start from a large light block to make sure that the pivot height doesn't select a height outside
// the appropriate range.
func TestClientLargeBisectionVerification(t *testing.T) {
	veryLargeFullNode := mockp.New(genMockNode(100, 3, 0, bTime))
	trustedLightBlock, err := veryLargeFullNode.LightBlock(ctx, 5)
	require.NoError(t, err)
	c, err := light.NewClient(
		ctx,
		chainID,
		light.TrustOptions{
			Period: 4 * time.Hour,
			Height: trustedLightBlock.Height,
			Hash:   trustedLightBlock.Hash(),
		},
		veryLargeFullNode,
		[]provider.Provider{veryLargeFullNode},
		dbs.New(dbm.NewMemDB(), chainID),
		light.SkippingVerification(light.DefaultTrustLevel),
	)
	require.NoError(t, err)
	h, err := c.Update(ctx, bTime.Add(100*time.Minute))
	require.NoError(t, err)
	h2, err := veryLargeFullNode.LightBlock(ctx, 100)
	require.NoError(t, err)
	assert.Equal(t, h, h2)
}

func TestClientBisectionBetweenTrustedHeaders(t *testing.T) {
	c, err := light.NewClient(
		ctx,
		chainID,
		light.TrustOptions{
			Period: 4 * time.Hour,
			Height: 1,
			Hash:   h1.Hash(),
		},
		fullNode,
		[]provider.Provider{fullNode},
		dbs.New(dbm.NewMemDB(), chainID),
		light.SkippingVerification(light.DefaultTrustLevel),
	)
	require.NoError(t, err)

	_, err = c.VerifyLightBlockAtHeight(ctx, 3, bTime.Add(2*time.Hour))
	require.NoError(t, err)

	// confirm that the client already doesn't have the light block
	_, err = c.TrustedLightBlock(2)
	require.Error(t, err)

	// verify using bisection the light block between the two trusted light blocks
	_, err = c.VerifyLightBlockAtHeight(ctx, 2, bTime.Add(1*time.Hour))
	require.NoError(t, err)
}

func TestClient_Cleanup(t *testing.T) {
	c, err := light.NewClient(
		ctx,
		chainID,
		trustOptions,
		fullNode,
		[]provider.Provider{fullNode},
		dbs.New(dbm.NewMemDB(), chainID),
		light.Logger(log.TestingLogger()),
	)
	require.NoError(t, err)
	_, err = c.TrustedLightBlock(1)
	require.NoError(t, err)

	err = c.Cleanup()
	require.NoError(t, err)

	// Check no light blocks exist after Cleanup.
	l, err := c.TrustedLightBlock(1)
	require.Error(t, err)
	require.Nil(t, l)
}

// trustedHeader.Height == options.Height.
func TestClientRestoresTrustedHeaderAfterStartup1(t *testing.T) {
	// 1. options.Hash == trustedHeader.Hash
	{
		trustedStore := dbs.New(dbm.NewMemDB(), chainID)
		err := trustedStore.SaveLightBlock(l1)
		require.NoError(t, err)

		c, err := light.NewClient(
			ctx,
			chainID,
			trustOptions,
			fullNode,
			[]provider.Provider{fullNode},
			trustedStore,
			light.Logger(log.TestingLogger()),
		)
		require.NoError(t, err)

		l, err := c.TrustedLightBlock(1)
		require.NoError(t, err)
		assert.NotNil(t, l)
		assert.Equal(t, l.Hash(), h1.Hash())
		assert.Equal(t, l.ValidatorSet.Hash(), h1.ValidatorsHash.Bytes())
	}

	// 2. options.Hash != trustedHeader.Hash
	{
		trustedStore := dbs.New(dbm.NewMemDB(), chainID)
		err := trustedStore.SaveLightBlock(l1)
		require.NoError(t, err)

		// header1 != h1
		header1 := keys.GenSignedHeader(chainID, 1, bTime.Add(1*time.Hour), nil, vals, vals,
			hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys))

		primary := mockp.New(
			chainID,
			map[int64]*types.SignedHeader{
				// trusted header
				1: header1,
			},
			valSet,
		)

		c, err := light.NewClient(
			ctx,
			chainID,
			light.TrustOptions{
				Period: 4 * time.Hour,
				Height: 1,
				Hash:   header1.Hash(),
			},
			primary,
			[]provider.Provider{primary},
			trustedStore,
			light.Logger(log.TestingLogger()),
		)
		require.NoError(t, err)

		l, err := c.TrustedLightBlock(1)
		require.NoError(t, err)
		if assert.NotNil(t, l) {
			assert.Equal(t, l.Hash(), header1.Hash())
			require.NoError(t, l.ValidateBasic(chainID))
		}
	}
}

// trustedHeader.Height < options.Height.
func TestClientRestoresTrustedHeaderAfterStartup2(t *testing.T) {
	// 1. options.Hash == trustedHeader.Hash
	{
		trustedStore := dbs.New(dbm.NewMemDB(), chainID)
		err := trustedStore.SaveLightBlock(l1)
		require.NoError(t, err)

		c, err := light.NewClient(
			ctx,
			chainID,
			light.TrustOptions{
				Period: 4 * time.Hour,
				Height: 2,
				Hash:   h2.Hash(),
			},
			fullNode,
			[]provider.Provider{fullNode},
			trustedStore,
			light.Logger(log.TestingLogger()),
		)
		require.NoError(t, err)

		// Check we still have the 1st header (+header+).
		l, err := c.TrustedLightBlock(1)
		require.NoError(t, err)
		assert.NotNil(t, l)
		assert.Equal(t, l.Hash(), h1.Hash())
		require.NoError(t, l.ValidateBasic(chainID))
	}

	// 2. options.Hash != trustedHeader.Hash
	// This could happen if previous provider was lying to us.
	{
		trustedStore := dbs.New(dbm.NewMemDB(), chainID)
		err := trustedStore.SaveLightBlock(l1)
		require.NoError(t, err)

		// header1 != header
		diffHeader1 := keys.GenSignedHeader(chainID, 1, bTime.Add(1*time.Hour), nil, vals, vals,
			hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys))

		diffHeader2 := keys.GenSignedHeader(chainID, 2, bTime.Add(2*time.Hour), nil, vals, vals,
			hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys))

		primary := mockp.New(
			chainID,
			map[int64]*types.SignedHeader{
				1: diffHeader1,
				2: diffHeader2,
			},
			valSet,
		)

		c, err := light.NewClient(
			ctx,
			chainID,
			light.TrustOptions{
				Period: 4 * time.Hour,
				Height: 2,
				Hash:   diffHeader2.Hash(),
			},
			primary,
			[]provider.Provider{primary},
			trustedStore,
			light.Logger(log.TestingLogger()),
		)
		require.NoError(t, err)

		// Check we no longer have the invalid 1st header (+header+).
		l, err := c.TrustedLightBlock(1)
		require.Error(t, err)
		require.Nil(t, l)
	}
}

// trustedHeader.Height > options.Height.
func TestClientRestoresTrustedHeaderAfterStartup3(t *testing.T) {
	// 1. options.Hash == trustedHeader.Hash
	{
		// load the first three headers into the trusted store
		trustedStore := dbs.New(dbm.NewMemDB(), chainID)
		err := trustedStore.SaveLightBlock(l1)
		require.NoError(t, err)

		err = trustedStore.SaveLightBlock(l2)
		require.NoError(t, err)

		c, err := light.NewClient(
			ctx,
			chainID,
			trustOptions,
			fullNode,
			[]provider.Provider{fullNode},
			trustedStore,
			light.Logger(log.TestingLogger()),
		)
		require.NoError(t, err)

		// Check we still have the 1st light block.
		l, err := c.TrustedLightBlock(1)
		require.NoError(t, err)
		assert.NotNil(t, l)
		assert.Equal(t, l.Hash(), h1.Hash())
		require.NoError(t, l.ValidateBasic(chainID))

		// Check we no longer have 2nd light block.
		l, err = c.TrustedLightBlock(2)
		require.Error(t, err)
		require.Nil(t, l)

		l, err = c.TrustedLightBlock(3)
		require.Error(t, err)
		require.Nil(t, l)
	}

	// 2. options.Hash != trustedHeader.Hash
	// This could happen if previous provider was lying to us.
	{
		trustedStore := dbs.New(dbm.NewMemDB(), chainID)
		err := trustedStore.SaveLightBlock(l1)
		require.NoError(t, err)

		// header1 != header
		header1 := keys.GenSignedHeader(chainID, 1, bTime.Add(1*time.Hour), nil, vals, vals,
			hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys))

		header2 := keys.GenSignedHeader(chainID, 2, bTime.Add(2*time.Hour), nil, vals, vals,
			hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys))
		err = trustedStore.SaveLightBlock(&types.LightBlock{
			SignedHeader: header2,
			ValidatorSet: vals,
		})
		require.NoError(t, err)

		primary := mockp.New(
			chainID,
			map[int64]*types.SignedHeader{
				1: header1,
			},
			valSet,
		)

		c, err := light.NewClient(
			ctx,
			chainID,
			light.TrustOptions{
				Period: 4 * time.Hour,
				Height: 1,
				Hash:   header1.Hash(),
			},
			primary,
			[]provider.Provider{primary},
			trustedStore,
			light.Logger(log.TestingLogger()),
		)
		require.NoError(t, err)

		// Check we have swapped invalid 1st light block (+lightblock+) with correct one (+lightblock2+).
		l, err := c.TrustedLightBlock(1)
		require.NoError(t, err)
		assert.NotNil(t, l)
		assert.Equal(t, l.Hash(), header1.Hash())
		require.NoError(t, l.ValidateBasic(chainID))

		// Check we no longer have invalid 2nd light block (+lightblock2+).
		l, err = c.TrustedLightBlock(2)
		require.Error(t, err)
		require.Nil(t, l)
	}
}

func TestClient_Update(t *testing.T) {
	c, err := light.NewClient(
		ctx,
		chainID,
		trustOptions,
		fullNode,
		[]provider.Provider{fullNode},
		dbs.New(dbm.NewMemDB(), chainID),
		light.Logger(log.TestingLogger()),
	)
	require.NoError(t, err)

	// should result in downloading & verifying header #3
	l, err := c.Update(ctx, bTime.Add(2*time.Hour))
	require.NoError(t, err)
	if assert.NotNil(t, l) {
		assert.EqualValues(t, 3, l.Height)
		require.NoError(t, l.ValidateBasic(chainID))
	}
}

func TestClient_Concurrency(t *testing.T) {
	c, err := light.NewClient(
		ctx,
		chainID,
		trustOptions,
		fullNode,
		[]provider.Provider{fullNode},
		dbs.New(dbm.NewMemDB(), chainID),
		light.Logger(log.TestingLogger()),
	)
	require.NoError(t, err)

	_, err = c.VerifyLightBlockAtHeight(ctx, 2, bTime.Add(2*time.Hour))
	require.NoError(t, err)

	var wg sync.WaitGroup
	for i := 0; i < 100; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()

			// NOTE: Cleanup, Stop, VerifyLightBlockAtHeight and Verify are not supposed
			// to be concurrenly safe.

			assert.Equal(t, chainID, c.ChainID())

			_, err := c.LastTrustedHeight()
			require.NoError(t, err)

			_, err = c.FirstTrustedHeight()
			require.NoError(t, err)

			l, err := c.TrustedLightBlock(1)
			require.NoError(t, err)
			assert.NotNil(t, l)
		}()
	}

	wg.Wait()
}

func TestClientReplacesPrimaryWithWitnessIfPrimaryIsUnavailable(t *testing.T) {
	c, err := light.NewClient(
		ctx,
		chainID,
		trustOptions,
		deadNode,
		[]provider.Provider{fullNode, fullNode},
		dbs.New(dbm.NewMemDB(), chainID),
		light.Logger(log.TestingLogger()),
		light.MaxRetryAttempts(1),
	)

	require.NoError(t, err)
	_, err = c.Update(ctx, bTime.Add(2*time.Hour))
	require.NoError(t, err)

	assert.NotEqual(t, c.Primary(), deadNode)
	assert.Len(t, c.Witnesses(), 2)
}

func TestClient_BackwardsVerification(t *testing.T) {
	{
		trustHeader, _ := largeFullNode.LightBlock(ctx, 6)
		c, err := light.NewClient(
			ctx,
			chainID,
			light.TrustOptions{
				Period: 4 * time.Minute,
				Height: trustHeader.Height,
				Hash:   trustHeader.Hash(),
			},
			largeFullNode,
			[]provider.Provider{largeFullNode},
			dbs.New(dbm.NewMemDB(), chainID),
			light.Logger(log.TestingLogger()),
		)
		require.NoError(t, err)

		// 1) verify before the trusted header using backwards => expect no error
		h, err := c.VerifyLightBlockAtHeight(ctx, 5, bTime.Add(6*time.Minute))
		require.NoError(t, err)
		if assert.NotNil(t, h) {
			assert.EqualValues(t, 5, h.Height)
		}

		// 2) untrusted header is expired but trusted header is not => expect no error
		h, err = c.VerifyLightBlockAtHeight(ctx, 3, bTime.Add(8*time.Minute))
		require.NoError(t, err)
		assert.NotNil(t, h)

		// 3) already stored headers should return the header without error
		h, err = c.VerifyLightBlockAtHeight(ctx, 5, bTime.Add(6*time.Minute))
		require.NoError(t, err)
		assert.NotNil(t, h)

		// 4a) First verify latest header
		_, err = c.VerifyLightBlockAtHeight(ctx, 9, bTime.Add(9*time.Minute))
		require.NoError(t, err)

		// 4b) Verify backwards using bisection => expect no error
		_, err = c.VerifyLightBlockAtHeight(ctx, 7, bTime.Add(9*time.Minute))
		require.NoError(t, err)
		// shouldn't have verified this header in the process
		_, err = c.TrustedLightBlock(8)
		require.Error(t, err)

		// 5) Try bisection method, but closest header (at 7) has expired
		// so expect error
		_, err = c.VerifyLightBlockAtHeight(ctx, 8, bTime.Add(12*time.Minute))
		require.Error(t, err)
	}
	{
		testCases := []struct {
			provider provider.Provider
		}{
			{
				// 7) provides incorrect height
				mockp.New(
					chainID,
					map[int64]*types.SignedHeader{
						1: h1,
						2: keys.GenSignedHeader(chainID, 1, bTime.Add(30*time.Minute), nil, vals, vals,
							hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys)),
						3: h3,
					},
					valSet,
				),
			},
			{
				// 8) provides incorrect hash
				mockp.New(
					chainID,
					map[int64]*types.SignedHeader{
						1: h1,
						2: keys.GenSignedHeader(chainID, 2, bTime.Add(30*time.Minute), nil, vals, vals,
							hash("app_hash2"), hash("cons_hash23"), hash("results_hash30"), 0, len(keys)),
						3: h3,
					},
					valSet,
				),
			},
		}

		for idx, tc := range testCases {
			c, err := light.NewClient(
				ctx,
				chainID,
				light.TrustOptions{
					Period: 1 * time.Hour,
					Height: 3,
					Hash:   h3.Hash(),
				},
				tc.provider,
				[]provider.Provider{tc.provider},
				dbs.New(dbm.NewMemDB(), chainID),
				light.Logger(log.TestingLogger()),
			)
			require.NoError(t, err, idx)

			_, err = c.VerifyLightBlockAtHeight(ctx, 2, bTime.Add(1*time.Hour).Add(1*time.Second))
			require.Error(t, err, idx)
		}
	}
}

func TestClient_NewClientFromTrustedStore(t *testing.T) {
	// 1) Initiate DB and fill with a "trusted" header
	db := dbs.New(dbm.NewMemDB(), chainID)
	err := db.SaveLightBlock(l1)
	require.NoError(t, err)

	c, err := light.NewClientFromTrustedStore(
		chainID,
		trustPeriod,
		deadNode,
		[]provider.Provider{deadNode},
		db,
	)
	require.NoError(t, err)

	// 2) Check light block exists (deadNode is being used to ensure we're not getting
	// it from primary)
	h, err := c.TrustedLightBlock(1)
	require.NoError(t, err)
	assert.EqualValues(t, l1.Height, h.Height)
}

func TestClient_NewClientFromEmptyTrustedStore(t *testing.T) {
	// empty DB
	db := dbs.New(dbm.NewMemDB(), chainID)

	c, err := light.NewClientFromTrustedStore(
		chainID,
		trustPeriod,
		fullNode,
		[]provider.Provider{fullNode},
		db,
	)

	if err == nil {
		assert.NotPanics(t, func() {
			_, _ = c.VerifyLightBlockAtHeight(ctx, 2, bTime)
		})
	} else {
		require.ErrorIs(t, err, light.ErrEmptyTrustedStore)
	}
}

func TestClientRemovesWitnessIfItSendsUsIncorrectHeader(t *testing.T) {
	// different headers hash then primary plus less than 1/3 signed (no fork)
	badProvider1 := mockp.New(
		chainID,
		map[int64]*types.SignedHeader{
			1: h1,
			2: keys.GenSignedHeaderLastBlockID(chainID, 2, bTime.Add(30*time.Minute), nil, vals2, vals2,
				hash("app_hash2"), hash("cons_hash"), hash("results_hash"),
				len(keys), len(keys), types.BlockID{Hash: h1.Hash()}),
		},
		map[int64]*types.ValidatorSet{
			1: vals,
			2: vals2,
		},
	)
	// header is empty
	badProvider2 := mockp.New(
		chainID,
		map[int64]*types.SignedHeader{
			1: h1,
			2: h2,
		},
		map[int64]*types.ValidatorSet{
			1: vals,
			2: vals2,
		},
	)

	lb1, _ := badProvider1.LightBlock(ctx, 2)
	require.NotEqual(t, lb1.Hash(), l1.Hash())

	c, err := light.NewClient(
		ctx,
		chainID,
		trustOptions,
		fullNode,
		[]provider.Provider{badProvider1, badProvider2},
		dbs.New(dbm.NewMemDB(), chainID),
		light.Logger(log.TestingLogger()),
		light.MaxRetryAttempts(1),
	)
	// witness should have behaved properly -> no error
	require.NoError(t, err)
	assert.EqualValues(t, 2, len(c.Witnesses()))

	// witness behaves incorrectly -> removed from list, no error
	l, err := c.VerifyLightBlockAtHeight(ctx, 2, bTime.Add(2*time.Hour))
	require.NoError(t, err)
	assert.EqualValues(t, 1, len(c.Witnesses()))
	// light block should still be verified
	assert.EqualValues(t, 2, l.Height)

	// remaining witnesses don't have light block -> error
	_, err = c.VerifyLightBlockAtHeight(ctx, 3, bTime.Add(2*time.Hour))
	if assert.Error(t, err) { //nolint:testifylint // require.Error doesn't work with the conditional here
		assert.Equal(t, light.ErrFailedHeaderCrossReferencing, err)
	}
	// witness does not have a light block -> left in the list
	assert.EqualValues(t, 1, len(c.Witnesses()))
}

func TestClient_TrustedValidatorSet(t *testing.T) {
	differentVals, _ := types.RandValidatorSet(10, 100)
	badValSetNode := mockp.New(
		chainID,
		map[int64]*types.SignedHeader{
			1: h1,
			// 3/3 signed, but validator set at height 2 below is invalid -> witness
			// should be removed.
			2: keys.GenSignedHeaderLastBlockID(chainID, 2, bTime.Add(30*time.Minute), nil, vals, vals,
				hash("app_hash2"), hash("cons_hash"), hash("results_hash"),
				0, len(keys), types.BlockID{Hash: h1.Hash()}),
			3: h3,
		},
		map[int64]*types.ValidatorSet{
			1: vals,
			2: differentVals,
			3: differentVals,
		},
	)

	c, err := light.NewClient(
		ctx,
		chainID,
		trustOptions,
		fullNode,
		[]provider.Provider{badValSetNode, fullNode},
		dbs.New(dbm.NewMemDB(), chainID),
		light.Logger(log.TestingLogger()),
	)
	require.NoError(t, err)
	assert.Len(t, c.Witnesses(), 2)

	_, err = c.VerifyLightBlockAtHeight(ctx, 2, bTime.Add(2*time.Hour).Add(1*time.Second))
	require.NoError(t, err)
	assert.Len(t, c.Witnesses(), 1)
}

func TestClientPrunesHeadersAndValidatorSets(t *testing.T) {
	c, err := light.NewClient(
		ctx,
		chainID,
		trustOptions,
		fullNode,
		[]provider.Provider{fullNode},
		dbs.New(dbm.NewMemDB(), chainID),
		light.Logger(log.TestingLogger()),
		light.PruningSize(1),
	)
	require.NoError(t, err)
	_, err = c.TrustedLightBlock(1)
	require.NoError(t, err)

	h, err := c.Update(ctx, bTime.Add(2*time.Hour))
	require.NoError(t, err)
	require.Equal(t, int64(3), h.Height)

	_, err = c.TrustedLightBlock(1)
	require.Error(t, err)
}

func TestClientEnsureValidHeadersAndValSets(t *testing.T) {
	emptyValSet := &types.ValidatorSet{
		Validators: nil,
		Proposer:   nil,
	}

	testCases := []struct {
		headers map[int64]*types.SignedHeader
		vals    map[int64]*types.ValidatorSet
		err     bool
	}{
		{
			headerSet,
			valSet,
			false,
		},
		{
			headerSet,
			map[int64]*types.ValidatorSet{
				1: vals,
				2: vals,
				3: nil,
			},
			true,
		},
		{
			map[int64]*types.SignedHeader{
				1: h1,
				2: h2,
				3: nil,
			},
			valSet,
			true,
		},
		{
			headerSet,
			map[int64]*types.ValidatorSet{
				1: vals,
				2: vals,
				3: emptyValSet,
			},
			true,
		},
	}

	for _, tc := range testCases {
		badNode := mockp.New(
			chainID,
			tc.headers,
			tc.vals,
		)
		c, err := light.NewClient(
			ctx,
			chainID,
			trustOptions,
			badNode,
			[]provider.Provider{badNode, badNode},
			dbs.New(dbm.NewMemDB(), chainID),
			light.MaxRetryAttempts(1),
		)
		require.NoError(t, err)

		_, err = c.VerifyLightBlockAtHeight(ctx, 3, bTime.Add(2*time.Hour))
		if tc.err {
			require.Error(t, err)
		} else {
			require.NoError(t, err)
		}
	}
}

func TestClientHandlesContexts(t *testing.T) {
	p := mockp.New(genMockNode(100, 10, 1, bTime))
	genBlock, err := p.LightBlock(ctx, 1)
	require.NoError(t, err)

	// instantiate the light client with a timeout
	ctxTimeOut, cancel := context.WithTimeout(ctx, 10*time.Millisecond)
	defer cancel()
	_, err = light.NewClient(
		ctxTimeOut,
		chainID,
		light.TrustOptions{
			Period: 24 * time.Hour,
			Height: 1,
			Hash:   genBlock.Hash(),
		},
		p,
		[]provider.Provider{p, p},
		dbs.New(dbm.NewMemDB(), chainID),
	)
	require.Error(t, ctxTimeOut.Err())
	require.Error(t, err)
	require.ErrorIs(t, err, context.DeadlineExceeded)

	// instantiate the client for real
	c, err := light.NewClient(
		ctx,
		chainID,
		light.TrustOptions{
			Period: 24 * time.Hour,
			Height: 1,
			Hash:   genBlock.Hash(),
		},
		p,
		[]provider.Provider{p, p},
		dbs.New(dbm.NewMemDB(), chainID),
	)
	require.NoError(t, err)

	// verify a block with a timeout
	ctxTimeOutBlock, cancel := context.WithTimeout(ctx, 10*time.Millisecond)
	defer cancel()
	_, err = c.VerifyLightBlockAtHeight(ctxTimeOutBlock, 100, bTime.Add(100*time.Minute))
	require.Error(t, ctxTimeOutBlock.Err())
	require.Error(t, err)
	require.ErrorIs(t, err, context.DeadlineExceeded)

	// verify a block with a cancel
	ctxCancel, cancel := context.WithCancel(ctx)
	defer cancel()
	time.AfterFunc(10*time.Millisecond, cancel)
	_, err = c.VerifyLightBlockAtHeight(ctxCancel, 100, bTime.Add(100*time.Minute))
	require.Error(t, ctxCancel.Err())
	require.Error(t, err)
	require.ErrorIs(t, err, context.Canceled)
}

// TestClientErrorsDifferentProposerPriorities tests the case where the witness
// sends us a light block with a validator set with different proposer priorities.
func TestClientErrorsDifferentProposerPriorities(t *testing.T) {
	primary := mockp.New(
		chainID,
		map[int64]*types.SignedHeader{
			1: h1,
			2: h2,
		},
		map[int64]*types.ValidatorSet{
			1: vals,
			2: vals2,
		},
	)
	witness := mockp.New(
		chainID,
		map[int64]*types.SignedHeader{
			1: h1,
			2: h2,
		},
		map[int64]*types.ValidatorSet{
			1: vals,
			2: vals,
		},
	)

	// Proposer priorities in vals and vals2 are different.
	// This is because vals2 = vals.CopyIncrementProposerPriority(1)
	require.Equal(t, vals.Hash(), vals2.Hash())
	require.NotEqual(t, vals.ProposerPriorityHash(), vals2.ProposerPriorityHash())

	c, err := light.NewClient(
		ctx,
		chainID,
		trustOptions,
		fullNode,
		[]provider.Provider{primary, witness},
		dbs.New(dbm.NewMemDB(), chainID),
		light.Logger(log.TestingLogger()),
		light.MaxRetryAttempts(1),
	)
	// witness should have behaved properly -> no error
	require.NoError(t, err)
	assert.EqualValues(t, 2, len(c.Witnesses()))

	// witness behaves incorrectly, but we can't prove who's guilty -> error
	_, err = c.VerifyLightBlockAtHeight(ctx, 2, bTime.Add(2*time.Hour))
	require.Error(t, err)

	// witness left in the list
	assert.EqualValues(t, 2, len(c.Witnesses()))
}

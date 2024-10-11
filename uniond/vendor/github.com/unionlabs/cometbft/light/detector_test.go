package light_test

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	dbm "github.com/cometbft/cometbft-db"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/light"
	"github.com/cometbft/cometbft/light/provider"
	mockp "github.com/cometbft/cometbft/light/provider/mock"
	dbs "github.com/cometbft/cometbft/light/store/db"
	"github.com/cometbft/cometbft/types"
)

func TestLightClientAttackEvidence_Lunatic(t *testing.T) {
	// primary performs a lunatic attack
	var (
		latestHeight      = int64(10)
		valSize           = 5
		divergenceHeight  = int64(6)
		primaryHeaders    = make(map[int64]*types.SignedHeader, latestHeight)
		primaryValidators = make(map[int64]*types.ValidatorSet, latestHeight)
	)

	witnessHeaders, witnessValidators, chainKeys := genMockNodeWithKeys(latestHeight, valSize, 2, bTime)
	witness := mockp.New(chainID, witnessHeaders, witnessValidators)
	forgedKeys := chainKeys[divergenceHeight-1].ChangeKeys(3) // we change 3 out of the 5 validators (still 2/5 remain)
	forgedVals := forgedKeys.ToValidators(2, 0)

	for height := int64(1); height <= latestHeight; height++ {
		if height < divergenceHeight {
			primaryHeaders[height] = witnessHeaders[height]
			primaryValidators[height] = witnessValidators[height]
			continue
		}
		primaryHeaders[height] = forgedKeys.GenSignedHeader(chainID, height, bTime.Add(time.Duration(height)*time.Minute),
			nil, forgedVals, forgedVals, hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(forgedKeys))
		primaryValidators[height] = forgedVals
	}
	primary := mockp.New(chainID, primaryHeaders, primaryValidators)

	c, err := light.NewClient(
		ctx,
		chainID,
		light.TrustOptions{
			Period: 4 * time.Hour,
			Height: 1,
			Hash:   primaryHeaders[1].Hash(),
		},
		primary,
		[]provider.Provider{witness},
		dbs.New(dbm.NewMemDB(), chainID),
		light.Logger(log.TestingLogger()),
		light.MaxRetryAttempts(1),
	)
	require.NoError(t, err)

	// Check verification returns an error.
	_, err = c.VerifyLightBlockAtHeight(ctx, 10, bTime.Add(1*time.Hour))
	if assert.Error(t, err) { //nolint:testifylint // require.Error doesn't work with the conditional here
		assert.Equal(t, light.ErrLightClientAttack, err)
	}

	// Check evidence was sent to both full nodes.
	evAgainstPrimary := &types.LightClientAttackEvidence{
		// after the divergence height the valset doesn't change so we expect the evidence to be for height 10
		ConflictingBlock: &types.LightBlock{
			SignedHeader: primaryHeaders[10],
			ValidatorSet: primaryValidators[10],
		},
		CommonHeight: 4,
	}
	assert.True(t, witness.HasEvidence(evAgainstPrimary))

	evAgainstWitness := &types.LightClientAttackEvidence{
		// when forming evidence against witness we learn that the canonical chain continued to change validator sets
		// hence the conflicting block is at 7
		ConflictingBlock: &types.LightBlock{
			SignedHeader: witnessHeaders[7],
			ValidatorSet: witnessValidators[7],
		},
		CommonHeight: 4,
	}
	assert.True(t, primary.HasEvidence(evAgainstWitness))
}

func TestLightClientAttackEvidence_Equivocation(t *testing.T) {
	verificationOptions := map[string]light.Option{
		"sequential": light.SequentialVerification(),
		"skipping":   light.SkippingVerification(light.DefaultTrustLevel),
	}

	for s, verificationOption := range verificationOptions {
		t.Log("==> verification", s)

		// primary performs an equivocation attack
		var (
			latestHeight      = int64(10)
			valSize           = 5
			divergenceHeight  = int64(6)
			primaryHeaders    = make(map[int64]*types.SignedHeader, latestHeight)
			primaryValidators = make(map[int64]*types.ValidatorSet, latestHeight)
		)
		// validators don't change in this network (however we still use a map just for convenience)
		witnessHeaders, witnessValidators, chainKeys := genMockNodeWithKeys(latestHeight+2, valSize, 2, bTime)
		witness := mockp.New(chainID, witnessHeaders, witnessValidators)

		for height := int64(1); height <= latestHeight; height++ {
			if height < divergenceHeight {
				primaryHeaders[height] = witnessHeaders[height]
				primaryValidators[height] = witnessValidators[height]
				continue
			}
			// we don't have a network partition so we will make 4/5 (greater than 2/3) malicious and vote again for
			// a different block (which we do by adding txs)
			primaryHeaders[height] = chainKeys[height].GenSignedHeader(chainID, height,
				bTime.Add(time.Duration(height)*time.Minute), []types.Tx{[]byte("abcd")},
				witnessValidators[height], witnessValidators[height+1], hash("app_hash"),
				hash("cons_hash"), hash("results_hash"), 0, len(chainKeys[height])-1)
			primaryValidators[height] = witnessValidators[height]
		}
		primary := mockp.New(chainID, primaryHeaders, primaryValidators)

		c, err := light.NewClient(
			ctx,
			chainID,
			light.TrustOptions{
				Period: 4 * time.Hour,
				Height: 1,
				Hash:   primaryHeaders[1].Hash(),
			},
			primary,
			[]provider.Provider{witness},
			dbs.New(dbm.NewMemDB(), chainID),
			light.Logger(log.TestingLogger()),
			light.MaxRetryAttempts(1),
			verificationOption,
		)
		require.NoError(t, err)

		// Check verification returns an error.
		_, err = c.VerifyLightBlockAtHeight(ctx, 10, bTime.Add(1*time.Hour))
		if assert.Error(t, err) { //nolint:testifylint // require.Error doesn't work with the conditional here
			assert.Equal(t, light.ErrLightClientAttack, err)
		}

		// Check evidence was sent to both full nodes.
		// Common height should be set to the height of the divergent header in the instance
		// of an equivocation attack and the validator sets are the same as what the witness has
		evAgainstPrimary := &types.LightClientAttackEvidence{
			ConflictingBlock: &types.LightBlock{
				SignedHeader: primaryHeaders[divergenceHeight],
				ValidatorSet: primaryValidators[divergenceHeight],
			},
			CommonHeight: divergenceHeight,
		}
		assert.True(t, witness.HasEvidence(evAgainstPrimary))

		evAgainstWitness := &types.LightClientAttackEvidence{
			ConflictingBlock: &types.LightBlock{
				SignedHeader: witnessHeaders[divergenceHeight],
				ValidatorSet: witnessValidators[divergenceHeight],
			},
			CommonHeight: divergenceHeight,
		}
		assert.True(t, primary.HasEvidence(evAgainstWitness))
	}
}

func TestLightClientAttackEvidence_ForwardLunatic(t *testing.T) {
	// primary performs a lunatic attack but changes the time of the header to
	// something in the future relative to the blockchain
	var (
		latestHeight      = int64(10)
		valSize           = 5
		forgedHeight      = int64(12)
		proofHeight       = int64(11)
		primaryHeaders    = make(map[int64]*types.SignedHeader, forgedHeight)
		primaryValidators = make(map[int64]*types.ValidatorSet, forgedHeight)
	)

	witnessHeaders, witnessValidators, chainKeys := genMockNodeWithKeys(latestHeight, valSize, 2, bTime)

	// primary has the exact same headers except it forges one extra header in the future using keys from 2/5ths of
	// the validators
	for h := range witnessHeaders {
		primaryHeaders[h] = witnessHeaders[h]
		primaryValidators[h] = witnessValidators[h]
	}
	forgedKeys := chainKeys[latestHeight].ChangeKeys(3) // we change 3 out of the 5 validators (still 2/5 remain)
	primaryValidators[forgedHeight] = forgedKeys.ToValidators(2, 0)
	primaryHeaders[forgedHeight] = forgedKeys.GenSignedHeader(
		chainID,
		forgedHeight,
		bTime.Add(time.Duration(latestHeight+1)*time.Minute), // 11 mins
		nil,
		primaryValidators[forgedHeight],
		primaryValidators[forgedHeight],
		hash("app_hash"),
		hash("cons_hash"),
		hash("results_hash"),
		0, len(forgedKeys),
	)

	witness := mockp.New(chainID, witnessHeaders, witnessValidators)
	primary := mockp.New(chainID, primaryHeaders, primaryValidators)

	laggingWitness := witness.Copy(chainID)

	// In order to perform the attack, the primary needs at least one accomplice as a witness to also
	// send the forged block
	accomplice := primary

	c, err := light.NewClient(
		ctx,
		chainID,
		light.TrustOptions{
			Period: 4 * time.Hour,
			Height: 1,
			Hash:   primaryHeaders[1].Hash(),
		},
		primary,
		[]provider.Provider{witness, accomplice},
		dbs.New(dbm.NewMemDB(), chainID),
		light.Logger(log.TestingLogger()),
		light.MaxClockDrift(1*time.Second),
		light.MaxBlockLag(1*time.Second),
	)
	require.NoError(t, err)

	// two seconds later, the supporting withness should receive the header that can be used
	// to prove that there was an attack
	vals := chainKeys[latestHeight].ToValidators(2, 0)
	newLb := &types.LightBlock{
		SignedHeader: chainKeys[latestHeight].GenSignedHeader(
			chainID,
			proofHeight,
			bTime.Add(time.Duration(proofHeight+1)*time.Minute), // 12 mins
			nil,
			vals,
			vals,
			hash("app_hash"),
			hash("cons_hash"),
			hash("results_hash"),
			0, len(chainKeys),
		),
		ValidatorSet: vals,
	}
	go func() {
		time.Sleep(2 * time.Second)
		witness.AddLightBlock(newLb)
	}()

	// Now assert that verification returns an error. We craft the light clients time to be a little ahead of the chain
	// to allow a window for the attack to manifest itself.
	_, err = c.Update(ctx, bTime.Add(time.Duration(forgedHeight)*time.Minute))
	if assert.Error(t, err) { //nolint:testifylint // require.Error doesn't work with the conditional here
		assert.Equal(t, light.ErrLightClientAttack, err)
	}

	// Check evidence was sent to the witness against the full node
	evAgainstPrimary := &types.LightClientAttackEvidence{
		ConflictingBlock: &types.LightBlock{
			SignedHeader: primaryHeaders[forgedHeight],
			ValidatorSet: primaryValidators[forgedHeight],
		},
		CommonHeight: latestHeight,
	}
	assert.True(t, witness.HasEvidence(evAgainstPrimary))

	// We attempt the same call but now the supporting witness has a block which should
	// immediately conflict in time with the primary
	_, err = c.VerifyLightBlockAtHeight(ctx, forgedHeight, bTime.Add(time.Duration(forgedHeight)*time.Minute))
	if assert.Error(t, err) { //nolint:testifylint // require.Error doesn't work with the conditional here
		assert.Equal(t, light.ErrLightClientAttack, err)
	}
	assert.True(t, witness.HasEvidence(evAgainstPrimary))

	// Lastly we test the unfortunate case where the light clients supporting witness doesn't update
	// in enough time
	c, err = light.NewClient(
		ctx,
		chainID,
		light.TrustOptions{
			Period: 4 * time.Hour,
			Height: 1,
			Hash:   primaryHeaders[1].Hash(),
		},
		primary,
		[]provider.Provider{laggingWitness, accomplice},
		dbs.New(dbm.NewMemDB(), chainID),
		light.Logger(log.TestingLogger()),
		light.MaxClockDrift(1*time.Second),
		light.MaxBlockLag(1*time.Second),
	)
	require.NoError(t, err)

	_, err = c.Update(ctx, bTime.Add(time.Duration(forgedHeight)*time.Minute))
	require.NoError(t, err)
}

// 1. Different nodes therefore a divergent header is produced.
// => light client returns an error upon creation because primary and witness
// have a different view.
func TestClientDivergentTraces1(t *testing.T) {
	primary := mockp.New(genMockNode(10, 5, 2, bTime))
	firstBlock, err := primary.LightBlock(ctx, 1)
	require.NoError(t, err)
	witness := mockp.New(genMockNode(10, 5, 2, bTime))

	_, err = light.NewClient(
		ctx,
		chainID,
		light.TrustOptions{
			Height: 1,
			Hash:   firstBlock.Hash(),
			Period: 4 * time.Hour,
		},
		primary,
		[]provider.Provider{witness},
		dbs.New(dbm.NewMemDB(), chainID),
		light.Logger(log.TestingLogger()),
		light.MaxRetryAttempts(1),
	)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "does not match primary")
}

// 2. Two out of three nodes don't respond but the third has a header that matches
// => verification should be successful and all the witnesses should remain.
func TestClientDivergentTraces2(t *testing.T) {
	primary := mockp.New(genMockNode(10, 5, 2, bTime))
	firstBlock, err := primary.LightBlock(ctx, 1)
	require.NoError(t, err)
	c, err := light.NewClient(
		ctx,
		chainID,
		light.TrustOptions{
			Height: 1,
			Hash:   firstBlock.Hash(),
			Period: 4 * time.Hour,
		},
		primary,
		[]provider.Provider{deadNode, deadNode, primary},
		dbs.New(dbm.NewMemDB(), chainID),
		light.Logger(log.TestingLogger()),
		light.MaxRetryAttempts(1),
	)
	require.NoError(t, err)

	_, err = c.VerifyLightBlockAtHeight(ctx, 10, bTime.Add(1*time.Hour))
	require.NoError(t, err)
	assert.Len(t, c.Witnesses(), 3)
}

// 3. witness has the same first header, but different second header
// => creation should succeed, but the verification should fail.
func TestClientDivergentTraces3(t *testing.T) {
	_, primaryHeaders, primaryVals := genMockNode(10, 5, 2, bTime)
	primary := mockp.New(chainID, primaryHeaders, primaryVals)

	firstBlock, err := primary.LightBlock(ctx, 1)
	require.NoError(t, err)

	_, mockHeaders, mockVals := genMockNode(10, 5, 2, bTime)
	mockHeaders[1] = primaryHeaders[1]
	mockVals[1] = primaryVals[1]
	witness := mockp.New(chainID, mockHeaders, mockVals)

	c, err := light.NewClient(
		ctx,
		chainID,
		light.TrustOptions{
			Height: 1,
			Hash:   firstBlock.Hash(),
			Period: 4 * time.Hour,
		},
		primary,
		[]provider.Provider{witness},
		dbs.New(dbm.NewMemDB(), chainID),
		light.Logger(log.TestingLogger()),
		light.MaxRetryAttempts(1),
	)
	require.NoError(t, err)

	_, err = c.VerifyLightBlockAtHeight(ctx, 10, bTime.Add(1*time.Hour))
	require.Error(t, err)
	assert.Len(t, c.Witnesses(), 1)
}

// 4. Witness has a divergent header but can not produce a valid trace to back it up.
// It should be ignored.
func TestClientDivergentTraces4(t *testing.T) {
	_, primaryHeaders, primaryVals := genMockNode(10, 5, 2, bTime)
	primary := mockp.New(chainID, primaryHeaders, primaryVals)

	firstBlock, err := primary.LightBlock(ctx, 1)
	require.NoError(t, err)

	_, mockHeaders, mockVals := genMockNode(10, 5, 2, bTime)
	witness := primary.Copy(chainID)
	witness.AddLightBlock(&types.LightBlock{
		SignedHeader: mockHeaders[10],
		ValidatorSet: mockVals[10],
	})

	c, err := light.NewClient(
		ctx,
		chainID,
		light.TrustOptions{
			Height: 1,
			Hash:   firstBlock.Hash(),
			Period: 4 * time.Hour,
		},
		primary,
		[]provider.Provider{witness},
		dbs.New(dbm.NewMemDB(), chainID),
		light.Logger(log.TestingLogger()),
	)
	require.NoError(t, err)

	_, err = c.VerifyLightBlockAtHeight(ctx, 10, bTime.Add(1*time.Hour))
	require.Error(t, err)
	assert.Len(t, c.Witnesses(), 1)
}

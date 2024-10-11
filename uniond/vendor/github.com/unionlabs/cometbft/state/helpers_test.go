package state_test

import (
	"bytes"
	"context"
	"time"

	dbm "github.com/cometbft/cometbft-db"
	abci "github.com/cometbft/cometbft/abci/types"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/ed25519"
	"github.com/cometbft/cometbft/internal/test"
	"github.com/cometbft/cometbft/proxy"
	sm "github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/types"
	cmttime "github.com/cometbft/cometbft/types/time"
)

type paramsChangeTestCase struct {
	height int64
	params types.ConsensusParams
}

func newTestApp() proxy.AppConns {
	app := &testApp{}
	cc := proxy.NewLocalClientCreator(app)
	return proxy.NewAppConns(cc, proxy.NopMetrics())
}

func makeAndCommitGoodBlock(
	state sm.State,
	height int64,
	lastCommit *types.Commit,
	proposerAddr []byte,
	blockExec *sm.BlockExecutor,
	privVals map[string]types.PrivValidator,
	evidence []types.Evidence,
) (sm.State, types.BlockID, *types.ExtendedCommit, error) {
	// A good block passes
	state, blockID, err := makeAndApplyGoodBlock(state, height, lastCommit, proposerAddr, blockExec, evidence)
	if err != nil {
		return state, types.BlockID{}, nil, err
	}

	// Simulate a lastCommit for this block from all validators for the next height
	commit, err := makeValidCommit(height, blockID, state.Validators, privVals)
	if err != nil {
		return state, types.BlockID{}, nil, err
	}
	return state, blockID, commit, nil
}

func makeAndApplyGoodBlock(state sm.State, height int64, lastCommit *types.Commit, proposerAddr []byte,
	blockExec *sm.BlockExecutor, evidence []types.Evidence,
) (sm.State, types.BlockID, error) {
	block := state.MakeBlock(height, test.MakeNTxs(height, 10), lastCommit, evidence, proposerAddr)
	partSet, err := block.MakePartSet(types.BlockPartSizeBytes)
	if err != nil {
		return state, types.BlockID{}, err
	}

	if err := blockExec.ValidateBlock(state, block); err != nil {
		return state, types.BlockID{}, err
	}
	blockID := types.BlockID{
		Hash:          block.Hash(),
		PartSetHeader: partSet.Header(),
	}
	state, err = blockExec.ApplyBlock(state, blockID, block, block.Height)
	if err != nil {
		return state, types.BlockID{}, err
	}
	return state, blockID, nil
}

func makeBlock(state sm.State, height int64, c *types.Commit) *types.Block {
	return state.MakeBlock(
		height,
		test.MakeNTxs(state.LastBlockHeight, 10),
		c,
		nil,
		state.Validators.GetProposer().Address,
	)
}

func makeValidCommit(
	height int64,
	blockID types.BlockID,
	vals *types.ValidatorSet,
	privVals map[string]types.PrivValidator,
) (*types.ExtendedCommit, error) {
	sigs := make([]types.ExtendedCommitSig, vals.Size())
	votes := make([]*types.Vote, vals.Size())
	for i := 0; i < vals.Size(); i++ {
		_, val := vals.GetByIndex(int32(i))
		vote, err := types.MakeVote(
			privVals[val.Address.String()],
			chainID,
			int32(i),
			height,
			0,
			types.PrecommitType,
			blockID,
			cmttime.Now(),
		)
		if err != nil {
			return nil, err
		}
		sigs[i] = vote.ExtendedCommitSig()
		votes[i] = vote
	}
	return &types.ExtendedCommit{
		Height:             height,
		BlockID:            blockID,
		ExtendedSignatures: sigs,
	}, nil
}

func genValSet(size int) *types.ValidatorSet {
	vals := make([]*types.Validator, size)
	for i := 0; i < size; i++ {
		vals[i] = types.NewValidator(ed25519.GenPrivKey().PubKey(), 10)
	}
	return types.NewValidatorSet(vals)
}

func makeHeaderPartsResponsesValPubKeyChange(
	state sm.State,
	pubkey crypto.PubKey,
) (types.Header, types.BlockID, *abci.FinalizeBlockResponse) {
	block := makeBlock(state, state.LastBlockHeight+1, new(types.Commit))
	abciResponses := &abci.FinalizeBlockResponse{}
	// If the pubkey is new, remove the old and add the new.
	_, val := state.NextValidators.GetByIndex(0)
	if !bytes.Equal(pubkey.Bytes(), val.PubKey.Bytes()) {
		abciResponses.ValidatorUpdates = []abci.ValidatorUpdate{
			abci.NewValidatorUpdate(val.PubKey, 0),
			abci.NewValidatorUpdate(pubkey, 10),
		}
	}

	return block.Header, types.BlockID{Hash: block.Hash(), PartSetHeader: types.PartSetHeader{}}, abciResponses
}

func makeHeaderPartsResponsesValPowerChange(
	state sm.State,
	power int64,
) (types.Header, types.BlockID, *abci.FinalizeBlockResponse) {
	block := makeBlock(state, state.LastBlockHeight+1, new(types.Commit))
	abciResponses := &abci.FinalizeBlockResponse{}

	// If the pubkey is new, remove the old and add the new.
	_, val := state.NextValidators.GetByIndex(0)
	if val.VotingPower != power {
		abciResponses.ValidatorUpdates = []abci.ValidatorUpdate{abci.NewValidatorUpdate(val.PubKey, power)}
	}

	return block.Header, types.BlockID{Hash: block.Hash(), PartSetHeader: types.PartSetHeader{}}, abciResponses
}

func makeHeaderPartsResponsesParams(
	state sm.State,
	params cmtproto.ConsensusParams,
) (types.Header, types.BlockID, *abci.FinalizeBlockResponse) {
	block := makeBlock(state, state.LastBlockHeight+1, new(types.Commit))
	abciResponses := &abci.FinalizeBlockResponse{
		ConsensusParamUpdates: &params,
	}
	return block.Header, types.BlockID{Hash: block.Hash(), PartSetHeader: types.PartSetHeader{}}, abciResponses
}

func randomGenesisDoc() *types.GenesisDoc {
	pubkey := ed25519.GenPrivKey().PubKey()
	return &types.GenesisDoc{
		GenesisTime: cmttime.Now(),
		ChainID:     "abc",
		Validators: []types.GenesisValidator{
			{
				Address: pubkey.Address(),
				PubKey:  pubkey,
				Power:   10,
				Name:    "myval",
			},
		},
		ConsensusParams: types.DefaultConsensusParams(),
	}
}

// ----------------------------------------------------------------------------

type testApp struct {
	abci.BaseApplication

	CommitVotes      []abci.VoteInfo
	Misbehavior      []abci.Misbehavior
	LastTime         time.Time
	ValidatorUpdates []abci.ValidatorUpdate
	AppHash          []byte
}

var _ abci.Application = (*testApp)(nil)

func (app *testApp) FinalizeBlock(_ context.Context, req *abci.FinalizeBlockRequest) (*abci.FinalizeBlockResponse, error) {
	app.CommitVotes = req.DecidedLastCommit.Votes
	app.Misbehavior = req.Misbehavior
	app.LastTime = req.Time
	txResults := make([]*abci.ExecTxResult, len(req.Txs))
	for idx := range req.Txs {
		txResults[idx] = &abci.ExecTxResult{
			Code: abci.CodeTypeOK,
		}
	}

	return &abci.FinalizeBlockResponse{
		ValidatorUpdates: app.ValidatorUpdates,
		ConsensusParamUpdates: &cmtproto.ConsensusParams{
			Version: &cmtproto.VersionParams{
				App: 1,
			},
		},
		TxResults: txResults,
		AppHash:   app.AppHash,
	}, nil
}

func (*testApp) Commit(_ context.Context, _ *abci.CommitRequest) (*abci.CommitResponse, error) {
	return &abci.CommitResponse{RetainHeight: 1}, nil
}

func (*testApp) PrepareProposal(
	_ context.Context,
	req *abci.PrepareProposalRequest,
) (*abci.PrepareProposalResponse, error) {
	txs := make([][]byte, 0, len(req.Txs))
	var totalBytes int64
	for _, tx := range req.Txs {
		if len(tx) == 0 {
			continue
		}
		totalBytes += int64(len(tx))
		if totalBytes > req.MaxTxBytes {
			break
		}
		txs = append(txs, tx)
	}
	return &abci.PrepareProposalResponse{Txs: txs}, nil
}

func (*testApp) ProcessProposal(
	_ context.Context,
	req *abci.ProcessProposalRequest,
) (*abci.ProcessProposalResponse, error) {
	for _, tx := range req.Txs {
		if len(tx) == 0 {
			return &abci.ProcessProposalResponse{Status: abci.PROCESS_PROPOSAL_STATUS_REJECT}, nil
		}
	}
	return &abci.ProcessProposalResponse{Status: abci.PROCESS_PROPOSAL_STATUS_ACCEPT}, nil
}

func makeStateWithParams(nVals, height int, params *types.ConsensusParams, chainID string) (sm.State, dbm.DB, map[string]types.PrivValidator) {
	vals, privVals := test.GenesisValidatorSet(nVals)

	s, _ := sm.MakeGenesisState(&types.GenesisDoc{
		ChainID:         chainID,
		Validators:      vals,
		AppHash:         nil,
		ConsensusParams: params,
	})

	stateDB := dbm.NewMemDB()
	stateStore := sm.NewStore(stateDB, sm.StoreOptions{
		DiscardABCIResponses: false,
	})
	if err := stateStore.Save(s); err != nil {
		panic(err)
	}

	for i := 1; i < height; i++ {
		s.LastBlockHeight++
		s.LastValidators = s.Validators.Copy()
		if err := stateStore.Save(s); err != nil {
			panic(err)
		}
	}

	return s, stateDB, privVals
}

func makeState(nVals, height int, chainID string) (sm.State, dbm.DB, map[string]types.PrivValidator) {
	return makeStateWithParams(nVals, height, test.ConsensusParams(), chainID)
}

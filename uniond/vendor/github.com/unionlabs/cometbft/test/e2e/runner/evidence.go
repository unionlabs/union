package main

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"math/rand"
	"os"
	"path/filepath"
	"time"

	cmtversion "github.com/cometbft/cometbft/api/cometbft/version/v1"
	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/tmhash"
	"github.com/cometbft/cometbft/internal/test"
	cmtjson "github.com/cometbft/cometbft/libs/json"
	"github.com/cometbft/cometbft/privval"
	e2e "github.com/cometbft/cometbft/test/e2e/pkg"
	"github.com/cometbft/cometbft/types"
	"github.com/cometbft/cometbft/version"
)

// 1 in 4 evidence is light client evidence, the rest is duplicate vote evidence.
const lightClientEvidenceRatio = 4

// InjectEvidence takes a running testnet and generates an amount of valid/invalid
// evidence and broadcasts it to a random node through the rpc endpoint `/broadcast_evidence`.
// Evidence is random and can be a mixture of LightClientAttackEvidence and
// DuplicateVoteEvidence.
func InjectEvidence(ctx context.Context, r *rand.Rand, testnet *e2e.Testnet, amount int) error {
	// select a random node
	var targetNode *e2e.Node

	for _, idx := range r.Perm(len(testnet.Nodes)) {
		targetNode = testnet.Nodes[idx]

		if targetNode.Mode == e2e.ModeSeed || targetNode.Mode == e2e.ModeLight {
			targetNode = nil
			continue
		}

		break
	}

	if targetNode == nil {
		return errors.New("could not find node to inject evidence into")
	}

	logger.Info(fmt.Sprintf("Injecting evidence through %v (amount: %d)...", targetNode.Name, amount))

	client, err := targetNode.Client()
	if err != nil {
		return err
	}

	// request the latest block and validator set from the node
	blockRes, err := client.Block(ctx, nil)
	if err != nil {
		return err
	}
	evidenceHeight := blockRes.Block.Height
	waitHeight := blockRes.Block.Height + 3

	nValidators := 100
	valRes, err := client.Validators(ctx, &evidenceHeight, nil, &nValidators)
	if err != nil {
		return err
	}

	valSet, err := types.ValidatorSetFromExistingValidators(valRes.Validators)
	if err != nil {
		return err
	}

	// get the private keys of all the validators in the network
	privVals, err := getPrivateValidatorKeys(testnet)
	if err != nil {
		return err
	}

	// wait for the node to reach the height above the forged height so that
	// it is able to validate the evidence
	_, err = waitForNode(ctx, targetNode, waitHeight, time.Minute)
	if err != nil {
		return err
	}

	var ev types.Evidence
	for i := 0; i < amount; i++ {
		validEv := true
		if i%lightClientEvidenceRatio == 0 {
			validEv = i%(lightClientEvidenceRatio*2) != 0 // Alternate valid and invalid evidence
			ev, err = generateLightClientAttackEvidence(
				ctx, privVals, evidenceHeight, valSet, testnet.Name, blockRes.Block.Time, validEv,
			)
		} else {
			var dve *types.DuplicateVoteEvidence
			dve, err = generateDuplicateVoteEvidence(
				privVals, evidenceHeight, valSet, testnet.Name, blockRes.Block.Time,
			)
			if dve.VoteA.Height < testnet.VoteExtensionsEnableHeight {
				dve.VoteA.Extension = nil
				dve.VoteA.ExtensionSignature = nil
				dve.VoteB.Extension = nil
				dve.VoteB.ExtensionSignature = nil
			}
			ev = dve
		}
		if err != nil {
			return err
		}

		_, err := client.BroadcastEvidence(ctx, ev)
		if !validEv {
			// The tests will count committed evidences later on,
			// and only valid evidences will make it
			amount++
		}
		if validEv != (err == nil) {
			if err == nil {
				return errors.New("submitting invalid evidence didn't return an error")
			}
			return err
		}
		time.Sleep(5 * time.Second / time.Duration(amount))
	}

	// wait for the node to reach the height above the forged height so that
	// it is able to validate the evidence
	_, err = waitForNode(ctx, targetNode, blockRes.Block.Height+2, 30*time.Second)
	if err != nil {
		return err
	}

	logger.Info(fmt.Sprintf("Finished sending evidence (height %d)", blockRes.Block.Height+2))

	return nil
}

func getPrivateValidatorKeys(testnet *e2e.Testnet) ([]types.MockPV, error) {
	privVals := []types.MockPV{}

	for _, node := range testnet.Nodes {
		if node.Mode == e2e.ModeValidator {
			privKeyPath := filepath.Join(testnet.Dir, node.Name, PrivvalKeyFile)
			privKey, err := readPrivKey(privKeyPath)
			if err != nil {
				return nil, err
			}
			// Create mock private validators from the validators private key. MockPV is
			// stateless which means we can double vote and do other funky stuff
			privVals = append(privVals, types.NewMockPVWithParams(privKey, false, false))
		}
	}

	return privVals, nil
}

// creates evidence of a lunatic attack. The height provided is the common height.
// The forged height happens 2 blocks later.
func generateLightClientAttackEvidence(
	ctx context.Context,
	privVals []types.MockPV,
	height int64,
	vals *types.ValidatorSet,
	chainID string,
	evTime time.Time,
	validEvidence bool,
) (*types.LightClientAttackEvidence, error) {
	// forge a random header
	forgedHeight := height + 2
	forgedTime := evTime.Add(1 * time.Second)
	header := makeHeaderRandom(chainID, forgedHeight)
	header.Time = forgedTime

	// add a new bogus validator and remove an existing one to
	// vary the validator set slightly
	pv, conflictingVals, err := mutateValidatorSet(ctx, privVals, vals, !validEvidence)
	if err != nil {
		return nil, err
	}

	header.ValidatorsHash = conflictingVals.Hash()

	// create a commit for the forged header
	blockID := makeBlockID(header.Hash(), 1000, []byte("partshash"))
	voteSet := types.NewVoteSet(chainID, forgedHeight, 0, types.SignedMsgType(2), conflictingVals)
	commit, err := test.MakeCommitFromVoteSet(blockID, voteSet, pv, forgedTime)
	if err != nil {
		return nil, err
	}

	// malleate the last signature of the commit by adding one to its first byte
	if !validEvidence {
		commit.Signatures[len(commit.Signatures)-1].Signature[0]++
	}

	ev := &types.LightClientAttackEvidence{
		ConflictingBlock: &types.LightBlock{
			SignedHeader: &types.SignedHeader{
				Header: header,
				Commit: commit,
			},
			ValidatorSet: conflictingVals,
		},
		CommonHeight:     height,
		TotalVotingPower: vals.TotalVotingPower(),
		Timestamp:        evTime,
	}
	ev.ByzantineValidators = ev.GetByzantineValidators(vals, &types.SignedHeader{
		Header: makeHeaderRandom(chainID, forgedHeight),
	})
	return ev, nil
}

// generateDuplicateVoteEvidence picks a random validator from the val set and
// returns duplicate vote evidence against the validator.
func generateDuplicateVoteEvidence(
	privVals []types.MockPV,
	height int64,
	vals *types.ValidatorSet,
	chainID string,
	time time.Time,
) (*types.DuplicateVoteEvidence, error) {
	privVal, valIdx, err := getRandomValidatorIndex(privVals, vals)
	if err != nil {
		return nil, err
	}
	voteA, err := types.MakeVote(privVal, chainID, valIdx, height, 0, 2, makeRandomBlockID(), time)
	if err != nil {
		return nil, err
	}
	voteB, err := types.MakeVote(privVal, chainID, valIdx, height, 0, 2, makeRandomBlockID(), time)
	if err != nil {
		return nil, err
	}
	ev, err := types.NewDuplicateVoteEvidence(voteA, voteB, time, vals)
	if err != nil {
		return nil, fmt.Errorf("could not generate evidence: %w", err)
	}

	return ev, nil
}

// getRandomValidatorIndex picks a random validator from a slice of mock PrivVals that's
// also part of the validator set, returning the PrivVal and its index in the validator set.
func getRandomValidatorIndex(privVals []types.MockPV, vals *types.ValidatorSet) (types.MockPV, int32, error) {
	for _, idx := range rand.Perm(len(privVals)) {
		pv := privVals[idx]
		valIdx, _ := vals.GetByAddress(pv.PrivKey.PubKey().Address())
		if valIdx >= 0 {
			return pv, valIdx, nil
		}
	}
	return types.MockPV{}, -1, errors.New("no private validator found in validator set")
}

func readPrivKey(keyFilePath string) (crypto.PrivKey, error) {
	keyJSONBytes, err := os.ReadFile(keyFilePath)
	if err != nil {
		return nil, err
	}
	pvKey := privval.FilePVKey{}
	err = cmtjson.Unmarshal(keyJSONBytes, &pvKey)
	if err != nil {
		return nil, fmt.Errorf("error reading PrivValidator key from %v: %w", keyFilePath, err)
	}

	return pvKey.PrivKey, nil
}

func makeHeaderRandom(chainID string, height int64) *types.Header {
	return &types.Header{
		Version:            cmtversion.Consensus{Block: version.BlockProtocol, App: 1},
		ChainID:            chainID,
		Height:             height,
		Time:               time.Now(),
		LastBlockID:        makeBlockID([]byte("headerhash"), 1000, []byte("partshash")),
		LastCommitHash:     crypto.CRandBytes(tmhash.Size),
		DataHash:           crypto.CRandBytes(tmhash.Size),
		ValidatorsHash:     crypto.CRandBytes(tmhash.Size),
		NextValidatorsHash: crypto.CRandBytes(tmhash.Size),
		ConsensusHash:      crypto.CRandBytes(tmhash.Size),
		AppHash:            crypto.CRandBytes(tmhash.Size),
		LastResultsHash:    crypto.CRandBytes(tmhash.Size),
		EvidenceHash:       crypto.CRandBytes(tmhash.Size),
		ProposerAddress:    crypto.CRandBytes(crypto.AddressSize),
	}
}

func makeRandomBlockID() types.BlockID {
	return makeBlockID(crypto.CRandBytes(tmhash.Size), 100, crypto.CRandBytes(tmhash.Size))
}

func makeBlockID(hash []byte, partSetSize uint32, partSetHash []byte) types.BlockID {
	var (
		h   = make([]byte, tmhash.Size)
		psH = make([]byte, tmhash.Size)
	)
	copy(h, hash)
	copy(psH, partSetHash)
	return types.BlockID{
		Hash: h,
		PartSetHeader: types.PartSetHeader{
			Total: partSetSize,
			Hash:  psH,
		},
	}
}

func mutateValidatorSet(
	ctx context.Context,
	privVals []types.MockPV,
	vals *types.ValidatorSet,
	nop bool,
) ([]types.PrivValidator, *types.ValidatorSet, error) {
	newVal, newPrivVal, err := test.Validator(ctx, 10)
	if err != nil {
		return nil, nil, err
	}

	var newVals *types.ValidatorSet
	if nop {
		newVals = types.NewValidatorSet(vals.Copy().Validators)
	} else {
		if vals.Size() > 2 {
			newVals = types.NewValidatorSet(append(vals.Copy().Validators[:vals.Size()-1], newVal))
		} else {
			newVals = types.NewValidatorSet(append(vals.Copy().Validators, newVal))
		}
	}

	// we need to sort the priv validators with the same index as the validator set
	pv := make([]types.PrivValidator, newVals.Size())
	for idx, val := range newVals.Validators {
		found := false
		for _, p := range append(privVals, newPrivVal.(types.MockPV)) {
			if bytes.Equal(p.PrivKey.PubKey().Address(), val.Address) {
				pv[idx] = p
				found = true
				break
			}
		}
		if !found {
			return nil, nil, fmt.Errorf("missing priv validator for %v", val.Address)
		}
	}

	return pv, newVals, nil
}

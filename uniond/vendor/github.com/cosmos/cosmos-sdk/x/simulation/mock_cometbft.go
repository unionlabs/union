package simulation

import (
	"fmt"
	"math/rand"
	"sort"
	"testing"
	"time"

	abci "github.com/cometbft/cometbft/abci/types"
	cryptoenc "github.com/cometbft/cometbft/crypto/encoding"
	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"
)

type mockValidator struct {
	val           abci.ValidatorUpdate
	livenessState int
}

func (mv mockValidator) String() string {
	return fmt.Sprintf("mockValidator{%s power:%v state:%v}",
		mv.val.PubKey.String(),
		mv.val.Power,
		mv.livenessState)
}

type mockValidators map[string]mockValidator

// get mockValidators from abci validators
func newMockValidators(r *rand.Rand, abciVals []abci.ValidatorUpdate, params Params) mockValidators {
	validators := make(mockValidators)

	for _, validator := range abciVals {
		str := fmt.Sprintf("%X", validator.PubKey.GetEd25519())
		liveliness := GetMemberOfInitialState(r, params.InitialLivenessWeightings())

		validators[str] = mockValidator{
			val:           validator,
			livenessState: liveliness,
		}
	}

	return validators
}

// TODO describe usage
func (vals mockValidators) getKeys() []string {
	keys := make([]string, len(vals))
	i := 0

	for key := range vals {
		keys[i] = key
		i++
	}

	sort.Strings(keys)

	return keys
}

// randomProposer picks a random proposer from the current validator set
func (vals mockValidators) randomProposer(r *rand.Rand) []byte {
	keys := vals.getKeys()
	if len(keys) == 0 {
		return nil
	}

	key := keys[r.Intn(len(keys))]

	proposer := vals[key].val
	pk, err := cryptoenc.PubKeyFromProto(proposer.PubKey)
	if err != nil {
		panic(err)
	}

	return pk.Address()
}

// updateValidators mimics CometBFT's update logic.
func updateValidators(
	tb testing.TB,
	r *rand.Rand,
	params Params,
	current map[string]mockValidator,
	updates []abci.ValidatorUpdate,
	event func(route, op, evResult string),
) map[string]mockValidator {
	for _, update := range updates {
		str := fmt.Sprintf("%X", update.PubKey.GetEd25519())

		if update.Power == 0 {
			if _, ok := current[str]; !ok {
				tb.Fatalf("tried to delete a nonexistent validator: %s", str)
			}

			event("end_block", "validator_updates", "kicked")
			delete(current, str)
		} else if _, ok := current[str]; ok {
			// validator already exists
			event("end_block", "validator_updates", "updated")
		} else {
			// Set this new validator
			current[str] = mockValidator{
				update,
				GetMemberOfInitialState(r, params.InitialLivenessWeightings()),
			}
			event("end_block", "validator_updates", "added")
		}
	}

	return current
}

// RandomRequestFinalizeBlock generates a list of signing validators according to
// the provided list of validators, signing fraction, and evidence fraction
func RandomRequestFinalizeBlock(
	r *rand.Rand,
	params Params,
	validators mockValidators,
	pastTimes []time.Time,
	pastVoteInfos [][]abci.VoteInfo,
	event func(route, op, evResult string),
	blockHeight int64,
	time time.Time,
	proposer []byte,
) *abci.RequestFinalizeBlock {
	if len(validators) == 0 {
		return &abci.RequestFinalizeBlock{
			Height:          blockHeight,
			Time:            time,
			ProposerAddress: proposer,
		}
	}

	voteInfos := make([]abci.VoteInfo, len(validators))

	for i, key := range validators.getKeys() {
		mVal := validators[key]
		mVal.livenessState = params.LivenessTransitionMatrix().NextState(r, mVal.livenessState)
		signed := true

		if mVal.livenessState == 1 {
			// spotty connection, 50% probability of success
			// See https://github.com/golang/go/issues/23804#issuecomment-365370418
			// for reasoning behind computing like this
			signed = r.Int63()%2 == 0
		} else if mVal.livenessState == 2 {
			// offline
			signed = false
		}

		if signed {
			event("begin_block", "signing", "signed")
		} else {
			event("begin_block", "signing", "missed")
		}

		pubkey, err := cryptoenc.PubKeyFromProto(mVal.val.PubKey)
		if err != nil {
			panic(err)
		}

		voteInfos[i] = abci.VoteInfo{
			Validator: abci.Validator{
				Address: pubkey.Address(),
				Power:   mVal.val.Power,
			},
			BlockIdFlag: cmtproto.BlockIDFlagCommit,
		}
	}

	// return if no past times
	if len(pastTimes) == 0 {
		return &abci.RequestFinalizeBlock{
			Height:          blockHeight,
			Time:            time,
			ProposerAddress: proposer,
			DecidedLastCommit: abci.CommitInfo{
				Votes: voteInfos,
			},
		}
	}

	// TODO: Determine capacity before allocation
	evidence := make([]abci.Misbehavior, 0)

	for r.Float64() < params.EvidenceFraction() {
		vals := voteInfos
		height := blockHeight
		misbehaviorTime := time
		if r.Float64() < params.PastEvidenceFraction() && height > 1 {
			height = int64(r.Intn(int(height)-1)) + 1 // CometBFT starts at height 1
			// array indices offset by one
			misbehaviorTime = pastTimes[height-1]
			vals = pastVoteInfos[height-1]
		}

		validator := vals[r.Intn(len(vals))].Validator

		var totalVotingPower int64
		for _, val := range vals {
			totalVotingPower += val.Validator.Power
		}

		evidence = append(evidence,
			abci.Misbehavior{
				Type:             abci.MisbehaviorType_DUPLICATE_VOTE,
				Validator:        validator,
				Height:           height,
				Time:             misbehaviorTime,
				TotalVotingPower: totalVotingPower,
			},
		)

		event("begin_block", "evidence", "ok")
	}

	return &abci.RequestFinalizeBlock{
		Height:          blockHeight,
		Time:            time,
		ProposerAddress: proposer,
		DecidedLastCommit: abci.CommitInfo{
			Votes: voteInfos,
		},
		Misbehavior: evidence,
	}
}

package simulation

import (
	"crypto/sha256"
	"fmt"
	"math/rand"
	"sort"
	"testing"
	"time"

	abci "github.com/cometbft/cometbft/abci/types"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
)

// TODO: move this somewhere else
const (
	TruncatedSize = 20
)

type mockValidator struct {
	val           abci.ValidatorUpdate
	livenessState int
}

func (mv mockValidator) String() string {
	return fmt.Sprintf("mockValidator{%s power:%v state:%v}",
		string(mv.val.PubKeyBytes),
		mv.val.Power,
		mv.livenessState)
}

type mockValidators map[string]mockValidator

// get mockValidators from abci validators
func newMockValidators(r *rand.Rand, abciVals []abci.ValidatorUpdate, params Params) mockValidators {
	validators := make(mockValidators)

	for _, validator := range abciVals {
		str := fmt.Sprintf("%X", validator.PubKeyBytes)
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

	return SumTruncated(proposer.PubKeyBytes)
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
	tb.Helper()
	for _, update := range updates {
		str := fmt.Sprintf("%X", update.PubKeyBytes)

		if update.Power == 0 {
			if _, ok := current[str]; !ok {
				tb.Logf("tried to delete a nonexistent validator: %s", str)
				continue
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
) *abci.FinalizeBlockRequest {
	if len(validators) == 0 {
		return &abci.FinalizeBlockRequest{
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

		var commitStatus cmtproto.BlockIDFlag
		if signed {
			event("begin_block", "signing", "signed")
			commitStatus = cmtproto.BlockIDFlagCommit
		} else {
			event("begin_block", "signing", "missed")
			commitStatus = cmtproto.BlockIDFlagAbsent
		}

		voteInfos[i] = abci.VoteInfo{
			Validator: abci.Validator{
				Address: SumTruncated(mVal.val.PubKeyBytes),
				Power:   mVal.val.Power,
			},
			BlockIdFlag: commitStatus,
		}
	}

	// return if no past times
	if len(pastTimes) == 0 {
		return &abci.FinalizeBlockRequest{
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
	// If the evidenceFraction value is to close to 1.0,
	// the following loop will most likely never end
	if params.EvidenceFraction() > 0.9 {
		// Reduce the evidenceFraction to a more sane value
		params.evidenceFraction = 0.9
	}

	totalBlocksProcessed := len(pastTimes)
	startHeight := blockHeight - int64(totalBlocksProcessed) + 1
	for r.Float64() < params.EvidenceFraction() {
		vals := voteInfos
		height := blockHeight
		misbehaviorTime := time
		if r.Float64() < params.PastEvidenceFraction() && totalBlocksProcessed > 1 {
			n := int64(r.Intn(totalBlocksProcessed))
			misbehaviorTime = pastTimes[n]
			vals = pastVoteInfos[n]
			height = startHeight + n
		}

		validator := vals[r.Intn(len(vals))].Validator

		var totalVotingPower int64
		for _, val := range vals {
			totalVotingPower += val.Validator.Power
		}

		evidence = append(evidence,
			abci.Misbehavior{
				Type:             abci.MISBEHAVIOR_TYPE_DUPLICATE_VOTE,
				Validator:        validator,
				Height:           height,
				Time:             misbehaviorTime,
				TotalVotingPower: totalVotingPower,
			},
		)

		event("begin_block", "evidence", "ok")
	}

	return &abci.FinalizeBlockRequest{
		Height:          blockHeight,
		Time:            time,
		ProposerAddress: proposer,
		DecidedLastCommit: abci.CommitInfo{
			Votes: voteInfos,
		},
		Misbehavior: evidence,
	}
}

// SumTruncated returns the first 20 bytes of SHA256 of the bz.
func SumTruncated(bz []byte) []byte {
	hash := sha256.Sum256(bz)
	return hash[:TruncatedSize]
}

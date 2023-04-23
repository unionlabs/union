package simulation

import (
	"fmt"
	"math/rand"
	"sort"
	"testing"
	"time"

	abci "github.com/cometbft/cometbft/abci/types"
	cryptoenc "github.com/cometbft/cometbft/crypto/encoding"
	tmbytes "github.com/cometbft/cometbft/libs/bytes"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
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
func (vals mockValidators) randomProposer(r *rand.Rand) tmbytes.HexBytes {
	keys := vals.getKeys()
	if len(keys) == 0 {
		return nil
	}

	key := keys[r.Intn(len(keys))]

	proposer := vals[key].val
	pk, err := cryptoenc.PubKeyFromProto(proposer.PubKey)
	if err != nil { //nolint:wsl
		panic(err)
	}

	return pk.Address()
}

// updateValidators mimics Tendermint's update logic.
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

// RandomRequestBeginBlock generates a list of signing validators according to
// the provided list of validators, signing fraction, and evidence fraction
func RandomRequestBeginBlock(r *rand.Rand, params Params,
	validators mockValidators, pastTimes []time.Time,
	pastVoteInfos [][]abci.VoteInfo,
	event func(route, op, evResult string), header tmproto.Header,
) abci.RequestBeginBlock {
	if len(validators) == 0 {
		return abci.RequestBeginBlock{
			Header: header,
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
			SignedLastBlock: signed,
		}
	}

	// return if no past times
	if len(pastTimes) == 0 {
		return abci.RequestBeginBlock{
			Header: header,
			LastCommitInfo: abci.CommitInfo{
				Votes: voteInfos,
			},
		}
	}

	// TODO: Determine capacity before allocation
	evidence := make([]abci.Misbehavior, 0)

	for r.Float64() < params.EvidenceFraction() {
		height := header.Height
		time := header.Time
		vals := voteInfos

		if r.Float64() < params.PastEvidenceFraction() && header.Height > 1 {
			height = int64(r.Intn(int(header.Height)-1)) + 1 // Tendermint starts at height 1
			// array indices offset by one
			time = pastTimes[height-1]
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
				Time:             time,
				TotalVotingPower: totalVotingPower,
			},
		)

		event("begin_block", "evidence", "ok")
	}

	return abci.RequestBeginBlock{
		Header: header,
		LastCommitInfo: abci.CommitInfo{
			Votes: voteInfos,
		},
		ByzantineValidators: evidence,
	}
}

package shared

import (
	"errors"
	"fmt"

	"github.com/ethereum/go-ethereum/common/hexutil"
	beaconState "github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
)

var errPayloadHeaderNotFound = errors.New("expected payload header not found")

func BeaconStateFromConsensus(st beaconState.BeaconState) (*BeaconState, error) {
	srcBr := st.BlockRoots()
	br := make([]string, len(srcBr))
	for i, r := range srcBr {
		br[i] = hexutil.Encode(r)
	}
	srcSr := st.StateRoots()
	sr := make([]string, len(srcSr))
	for i, r := range srcSr {
		sr[i] = hexutil.Encode(r)
	}
	srcHr, err := st.HistoricalRoots()
	if err != nil {
		return nil, err
	}
	hr := make([]string, len(srcHr))
	for i, r := range srcHr {
		hr[i] = hexutil.Encode(r)
	}
	srcVotes := st.Eth1DataVotes()
	votes := make([]*Eth1Data, len(srcVotes))
	for i, e := range srcVotes {
		votes[i] = Eth1DataFromConsensus(e)
	}
	srcVals := st.Validators()
	vals := make([]*Validator, len(srcVals))
	for i, v := range srcVals {
		vals[i] = ValidatorFromConsensus(v)
	}
	srcBals := st.Balances()
	bals := make([]string, len(srcBals))
	for i, b := range srcBals {
		bals[i] = fmt.Sprintf("%d", b)
	}
	srcRm := st.RandaoMixes()
	rm := make([]string, len(srcRm))
	for i, m := range srcRm {
		rm[i] = hexutil.Encode(m)
	}
	srcSlashings := st.Slashings()
	slashings := make([]string, len(srcSlashings))
	for i, s := range srcSlashings {
		slashings[i] = fmt.Sprintf("%d", s)
	}
	srcPrevAtts, err := st.PreviousEpochAttestations()
	if err != nil {
		return nil, err
	}
	prevAtts := make([]*PendingAttestation, len(srcPrevAtts))
	for i, a := range srcPrevAtts {
		prevAtts[i] = PendingAttestationFromConsensus(a)
	}
	srcCurrAtts, err := st.CurrentEpochAttestations()
	if err != nil {
		return nil, err
	}
	currAtts := make([]*PendingAttestation, len(srcCurrAtts))
	for i, a := range srcCurrAtts {
		currAtts[i] = PendingAttestationFromConsensus(a)
	}

	return &BeaconState{
		GenesisTime:                 fmt.Sprintf("%d", st.GenesisTime()),
		GenesisValidatorsRoot:       hexutil.Encode(st.GenesisValidatorsRoot()),
		Slot:                        fmt.Sprintf("%d", st.Slot()),
		Fork:                        ForkFromConsensus(st.Fork()),
		LatestBlockHeader:           BeaconBlockHeaderFromConsensus(st.LatestBlockHeader()),
		BlockRoots:                  br,
		StateRoots:                  sr,
		HistoricalRoots:             hr,
		Eth1Data:                    Eth1DataFromConsensus(st.Eth1Data()),
		Eth1DataVotes:               votes,
		Eth1DepositIndex:            fmt.Sprintf("%d", st.Eth1DepositIndex()),
		Validators:                  vals,
		Balances:                    bals,
		RandaoMixes:                 rm,
		Slashings:                   slashings,
		PreviousEpochAttestations:   prevAtts,
		CurrentEpochAttestations:    currAtts,
		JustificationBits:           hexutil.Encode(st.JustificationBits()),
		PreviousJustifiedCheckpoint: CheckpointFromConsensus(st.PreviousJustifiedCheckpoint()),
		CurrentJustifiedCheckpoint:  CheckpointFromConsensus(st.CurrentJustifiedCheckpoint()),
		FinalizedCheckpoint:         CheckpointFromConsensus(st.FinalizedCheckpoint()),
	}, nil
}

func BeaconStateAltairFromConsensus(st beaconState.BeaconState) (*BeaconStateAltair, error) {
	srcBr := st.BlockRoots()
	br := make([]string, len(srcBr))
	for i, r := range srcBr {
		br[i] = hexutil.Encode(r)
	}
	srcSr := st.StateRoots()
	sr := make([]string, len(srcSr))
	for i, r := range srcSr {
		sr[i] = hexutil.Encode(r)
	}
	srcHr, err := st.HistoricalRoots()
	if err != nil {
		return nil, err
	}
	hr := make([]string, len(srcHr))
	for i, r := range srcHr {
		hr[i] = hexutil.Encode(r)
	}
	srcVotes := st.Eth1DataVotes()
	votes := make([]*Eth1Data, len(srcVotes))
	for i, e := range srcVotes {
		votes[i] = Eth1DataFromConsensus(e)
	}
	srcVals := st.Validators()
	vals := make([]*Validator, len(srcVals))
	for i, v := range srcVals {
		vals[i] = ValidatorFromConsensus(v)
	}
	srcBals := st.Balances()
	bals := make([]string, len(srcBals))
	for i, b := range srcBals {
		bals[i] = fmt.Sprintf("%d", b)
	}
	srcRm := st.RandaoMixes()
	rm := make([]string, len(srcRm))
	for i, m := range srcRm {
		rm[i] = hexutil.Encode(m)
	}
	srcSlashings := st.Slashings()
	slashings := make([]string, len(srcSlashings))
	for i, s := range srcSlashings {
		slashings[i] = fmt.Sprintf("%d", s)
	}
	srcPrevPart, err := st.PreviousEpochParticipation()
	if err != nil {
		return nil, err
	}
	prevPart := make([]string, len(srcPrevPart))
	for i, p := range srcPrevPart {
		prevPart[i] = fmt.Sprintf("%d", p)
	}
	srcCurrPart, err := st.CurrentEpochParticipation()
	if err != nil {
		return nil, err
	}
	currPart := make([]string, len(srcCurrPart))
	for i, p := range srcCurrPart {
		currPart[i] = fmt.Sprintf("%d", p)
	}
	srcIs, err := st.InactivityScores()
	if err != nil {
		return nil, err
	}
	is := make([]string, len(srcIs))
	for i, s := range srcIs {
		is[i] = fmt.Sprintf("%d", s)
	}
	currSc, err := st.CurrentSyncCommittee()
	if err != nil {
		return nil, err
	}
	nextSc, err := st.NextSyncCommittee()
	if err != nil {
		return nil, err
	}

	return &BeaconStateAltair{
		GenesisTime:                 fmt.Sprintf("%d", st.GenesisTime()),
		GenesisValidatorsRoot:       hexutil.Encode(st.GenesisValidatorsRoot()),
		Slot:                        fmt.Sprintf("%d", st.Slot()),
		Fork:                        ForkFromConsensus(st.Fork()),
		LatestBlockHeader:           BeaconBlockHeaderFromConsensus(st.LatestBlockHeader()),
		BlockRoots:                  br,
		StateRoots:                  sr,
		HistoricalRoots:             hr,
		Eth1Data:                    Eth1DataFromConsensus(st.Eth1Data()),
		Eth1DataVotes:               votes,
		Eth1DepositIndex:            fmt.Sprintf("%d", st.Eth1DepositIndex()),
		Validators:                  vals,
		Balances:                    bals,
		RandaoMixes:                 rm,
		Slashings:                   slashings,
		PreviousEpochParticipation:  prevPart,
		CurrentEpochParticipation:   currPart,
		JustificationBits:           hexutil.Encode(st.JustificationBits()),
		PreviousJustifiedCheckpoint: CheckpointFromConsensus(st.PreviousJustifiedCheckpoint()),
		CurrentJustifiedCheckpoint:  CheckpointFromConsensus(st.CurrentJustifiedCheckpoint()),
		FinalizedCheckpoint:         CheckpointFromConsensus(st.FinalizedCheckpoint()),
		InactivityScores:            is,
		CurrentSyncCommittee:        SyncCommitteeFromConsensus(currSc),
		NextSyncCommittee:           SyncCommitteeFromConsensus(nextSc),
	}, nil
}

func BeaconStateBellatrixFromConsensus(st beaconState.BeaconState) (*BeaconStateBellatrix, error) {
	srcBr := st.BlockRoots()
	br := make([]string, len(srcBr))
	for i, r := range srcBr {
		br[i] = hexutil.Encode(r)
	}
	srcSr := st.StateRoots()
	sr := make([]string, len(srcSr))
	for i, r := range srcSr {
		sr[i] = hexutil.Encode(r)
	}
	srcHr, err := st.HistoricalRoots()
	if err != nil {
		return nil, err
	}
	hr := make([]string, len(srcHr))
	for i, r := range srcHr {
		hr[i] = hexutil.Encode(r)
	}
	srcVotes := st.Eth1DataVotes()
	votes := make([]*Eth1Data, len(srcVotes))
	for i, e := range srcVotes {
		votes[i] = Eth1DataFromConsensus(e)
	}
	srcVals := st.Validators()
	vals := make([]*Validator, len(srcVals))
	for i, v := range srcVals {
		vals[i] = ValidatorFromConsensus(v)
	}
	srcBals := st.Balances()
	bals := make([]string, len(srcBals))
	for i, b := range srcBals {
		bals[i] = fmt.Sprintf("%d", b)
	}
	srcRm := st.RandaoMixes()
	rm := make([]string, len(srcRm))
	for i, m := range srcRm {
		rm[i] = hexutil.Encode(m)
	}
	srcSlashings := st.Slashings()
	slashings := make([]string, len(srcSlashings))
	for i, s := range srcSlashings {
		slashings[i] = fmt.Sprintf("%d", s)
	}
	srcPrevPart, err := st.PreviousEpochParticipation()
	if err != nil {
		return nil, err
	}
	prevPart := make([]string, len(srcPrevPart))
	for i, p := range srcPrevPart {
		prevPart[i] = fmt.Sprintf("%d", p)
	}
	srcCurrPart, err := st.CurrentEpochParticipation()
	if err != nil {
		return nil, err
	}
	currPart := make([]string, len(srcCurrPart))
	for i, p := range srcCurrPart {
		currPart[i] = fmt.Sprintf("%d", p)
	}
	srcIs, err := st.InactivityScores()
	if err != nil {
		return nil, err
	}
	is := make([]string, len(srcIs))
	for i, s := range srcIs {
		is[i] = fmt.Sprintf("%d", s)
	}
	currSc, err := st.CurrentSyncCommittee()
	if err != nil {
		return nil, err
	}
	nextSc, err := st.NextSyncCommittee()
	if err != nil {
		return nil, err
	}
	execData, err := st.LatestExecutionPayloadHeader()
	if err != nil {
		return nil, err
	}
	srcPayload, ok := execData.Proto().(*enginev1.ExecutionPayloadHeader)
	if !ok {
		return nil, errPayloadHeaderNotFound
	}
	payload, err := ExecutionPayloadHeaderFromConsensus(srcPayload)
	if err != nil {
		return nil, err
	}

	return &BeaconStateBellatrix{
		GenesisTime:                  fmt.Sprintf("%d", st.GenesisTime()),
		GenesisValidatorsRoot:        hexutil.Encode(st.GenesisValidatorsRoot()),
		Slot:                         fmt.Sprintf("%d", st.Slot()),
		Fork:                         ForkFromConsensus(st.Fork()),
		LatestBlockHeader:            BeaconBlockHeaderFromConsensus(st.LatestBlockHeader()),
		BlockRoots:                   br,
		StateRoots:                   sr,
		HistoricalRoots:              hr,
		Eth1Data:                     Eth1DataFromConsensus(st.Eth1Data()),
		Eth1DataVotes:                votes,
		Eth1DepositIndex:             fmt.Sprintf("%d", st.Eth1DepositIndex()),
		Validators:                   vals,
		Balances:                     bals,
		RandaoMixes:                  rm,
		Slashings:                    slashings,
		PreviousEpochParticipation:   prevPart,
		CurrentEpochParticipation:    currPart,
		JustificationBits:            hexutil.Encode(st.JustificationBits()),
		PreviousJustifiedCheckpoint:  CheckpointFromConsensus(st.PreviousJustifiedCheckpoint()),
		CurrentJustifiedCheckpoint:   CheckpointFromConsensus(st.CurrentJustifiedCheckpoint()),
		FinalizedCheckpoint:          CheckpointFromConsensus(st.FinalizedCheckpoint()),
		InactivityScores:             is,
		CurrentSyncCommittee:         SyncCommitteeFromConsensus(currSc),
		NextSyncCommittee:            SyncCommitteeFromConsensus(nextSc),
		LatestExecutionPayloadHeader: payload,
	}, nil
}

func BeaconStateCapellaFromConsensus(st beaconState.BeaconState) (*BeaconStateCapella, error) {
	srcBr := st.BlockRoots()
	br := make([]string, len(srcBr))
	for i, r := range srcBr {
		br[i] = hexutil.Encode(r)
	}
	srcSr := st.StateRoots()
	sr := make([]string, len(srcSr))
	for i, r := range srcSr {
		sr[i] = hexutil.Encode(r)
	}
	srcHr, err := st.HistoricalRoots()
	if err != nil {
		return nil, err
	}
	hr := make([]string, len(srcHr))
	for i, r := range srcHr {
		hr[i] = hexutil.Encode(r)
	}
	srcVotes := st.Eth1DataVotes()
	votes := make([]*Eth1Data, len(srcVotes))
	for i, e := range srcVotes {
		votes[i] = Eth1DataFromConsensus(e)
	}
	srcVals := st.Validators()
	vals := make([]*Validator, len(srcVals))
	for i, v := range srcVals {
		vals[i] = ValidatorFromConsensus(v)
	}
	srcBals := st.Balances()
	bals := make([]string, len(srcBals))
	for i, b := range srcBals {
		bals[i] = fmt.Sprintf("%d", b)
	}
	srcRm := st.RandaoMixes()
	rm := make([]string, len(srcRm))
	for i, m := range srcRm {
		rm[i] = hexutil.Encode(m)
	}
	srcSlashings := st.Slashings()
	slashings := make([]string, len(srcSlashings))
	for i, s := range srcSlashings {
		slashings[i] = fmt.Sprintf("%d", s)
	}
	srcPrevPart, err := st.PreviousEpochParticipation()
	if err != nil {
		return nil, err
	}
	prevPart := make([]string, len(srcPrevPart))
	for i, p := range srcPrevPart {
		prevPart[i] = fmt.Sprintf("%d", p)
	}
	srcCurrPart, err := st.CurrentEpochParticipation()
	if err != nil {
		return nil, err
	}
	currPart := make([]string, len(srcCurrPart))
	for i, p := range srcCurrPart {
		currPart[i] = fmt.Sprintf("%d", p)
	}
	srcIs, err := st.InactivityScores()
	if err != nil {
		return nil, err
	}
	is := make([]string, len(srcIs))
	for i, s := range srcIs {
		is[i] = fmt.Sprintf("%d", s)
	}
	currSc, err := st.CurrentSyncCommittee()
	if err != nil {
		return nil, err
	}
	nextSc, err := st.NextSyncCommittee()
	if err != nil {
		return nil, err
	}
	execData, err := st.LatestExecutionPayloadHeader()
	if err != nil {
		return nil, err
	}
	srcPayload, ok := execData.Proto().(*enginev1.ExecutionPayloadHeaderCapella)
	if !ok {
		return nil, errPayloadHeaderNotFound
	}
	payload, err := ExecutionPayloadHeaderCapellaFromConsensus(srcPayload)
	if err != nil {
		return nil, err
	}
	srcHs, err := st.HistoricalSummaries()
	if err != nil {
		return nil, err
	}
	hs := make([]*HistoricalSummary, len(srcHs))
	for i, s := range srcHs {
		hs[i] = HistoricalSummaryFromConsensus(s)
	}
	nwi, err := st.NextWithdrawalIndex()
	if err != nil {
		return nil, err
	}
	nwvi, err := st.NextWithdrawalValidatorIndex()
	if err != nil {
		return nil, err
	}

	return &BeaconStateCapella{
		GenesisTime:                  fmt.Sprintf("%d", st.GenesisTime()),
		GenesisValidatorsRoot:        hexutil.Encode(st.GenesisValidatorsRoot()),
		Slot:                         fmt.Sprintf("%d", st.Slot()),
		Fork:                         ForkFromConsensus(st.Fork()),
		LatestBlockHeader:            BeaconBlockHeaderFromConsensus(st.LatestBlockHeader()),
		BlockRoots:                   br,
		StateRoots:                   sr,
		HistoricalRoots:              hr,
		Eth1Data:                     Eth1DataFromConsensus(st.Eth1Data()),
		Eth1DataVotes:                votes,
		Eth1DepositIndex:             fmt.Sprintf("%d", st.Eth1DepositIndex()),
		Validators:                   vals,
		Balances:                     bals,
		RandaoMixes:                  rm,
		Slashings:                    slashings,
		PreviousEpochParticipation:   prevPart,
		CurrentEpochParticipation:    currPart,
		JustificationBits:            hexutil.Encode(st.JustificationBits()),
		PreviousJustifiedCheckpoint:  CheckpointFromConsensus(st.PreviousJustifiedCheckpoint()),
		CurrentJustifiedCheckpoint:   CheckpointFromConsensus(st.CurrentJustifiedCheckpoint()),
		FinalizedCheckpoint:          CheckpointFromConsensus(st.FinalizedCheckpoint()),
		InactivityScores:             is,
		CurrentSyncCommittee:         SyncCommitteeFromConsensus(currSc),
		NextSyncCommittee:            SyncCommitteeFromConsensus(nextSc),
		LatestExecutionPayloadHeader: payload,
		NextWithdrawalIndex:          fmt.Sprintf("%d", nwi),
		NextWithdrawalValidatorIndex: fmt.Sprintf("%d", nwvi),
		HistoricalSummaries:          hs,
	}, nil
}

func BeaconStateDenebFromConsensus(st beaconState.BeaconState) (*BeaconStateDeneb, error) {
	srcBr := st.BlockRoots()
	br := make([]string, len(srcBr))
	for i, r := range srcBr {
		br[i] = hexutil.Encode(r)
	}
	srcSr := st.StateRoots()
	sr := make([]string, len(srcSr))
	for i, r := range srcSr {
		sr[i] = hexutil.Encode(r)
	}
	srcHr, err := st.HistoricalRoots()
	if err != nil {
		return nil, err
	}
	hr := make([]string, len(srcHr))
	for i, r := range srcHr {
		hr[i] = hexutil.Encode(r)
	}
	srcVotes := st.Eth1DataVotes()
	votes := make([]*Eth1Data, len(srcVotes))
	for i, e := range srcVotes {
		votes[i] = Eth1DataFromConsensus(e)
	}
	srcVals := st.Validators()
	vals := make([]*Validator, len(srcVals))
	for i, v := range srcVals {
		vals[i] = ValidatorFromConsensus(v)
	}
	srcBals := st.Balances()
	bals := make([]string, len(srcBals))
	for i, b := range srcBals {
		bals[i] = fmt.Sprintf("%d", b)
	}
	srcRm := st.RandaoMixes()
	rm := make([]string, len(srcRm))
	for i, m := range srcRm {
		rm[i] = hexutil.Encode(m)
	}
	srcSlashings := st.Slashings()
	slashings := make([]string, len(srcSlashings))
	for i, s := range srcSlashings {
		slashings[i] = fmt.Sprintf("%d", s)
	}
	srcPrevPart, err := st.PreviousEpochParticipation()
	if err != nil {
		return nil, err
	}
	prevPart := make([]string, len(srcPrevPart))
	for i, p := range srcPrevPart {
		prevPart[i] = fmt.Sprintf("%d", p)
	}
	srcCurrPart, err := st.CurrentEpochParticipation()
	if err != nil {
		return nil, err
	}
	currPart := make([]string, len(srcCurrPart))
	for i, p := range srcCurrPart {
		currPart[i] = fmt.Sprintf("%d", p)
	}
	srcIs, err := st.InactivityScores()
	if err != nil {
		return nil, err
	}
	is := make([]string, len(srcIs))
	for i, s := range srcIs {
		is[i] = fmt.Sprintf("%d", s)
	}
	currSc, err := st.CurrentSyncCommittee()
	if err != nil {
		return nil, err
	}
	nextSc, err := st.NextSyncCommittee()
	if err != nil {
		return nil, err
	}
	execData, err := st.LatestExecutionPayloadHeader()
	if err != nil {
		return nil, err
	}
	srcPayload, ok := execData.Proto().(*enginev1.ExecutionPayloadHeaderDeneb)
	if !ok {
		return nil, errPayloadHeaderNotFound
	}
	payload, err := ExecutionPayloadHeaderDenebFromConsensus(srcPayload)
	if err != nil {
		return nil, err
	}
	srcHs, err := st.HistoricalSummaries()
	if err != nil {
		return nil, err
	}
	hs := make([]*HistoricalSummary, len(srcHs))
	for i, s := range srcHs {
		hs[i] = HistoricalSummaryFromConsensus(s)
	}
	nwi, err := st.NextWithdrawalIndex()
	if err != nil {
		return nil, err
	}
	nwvi, err := st.NextWithdrawalValidatorIndex()
	if err != nil {
		return nil, err
	}

	return &BeaconStateDeneb{
		GenesisTime:                  fmt.Sprintf("%d", st.GenesisTime()),
		GenesisValidatorsRoot:        hexutil.Encode(st.GenesisValidatorsRoot()),
		Slot:                         fmt.Sprintf("%d", st.Slot()),
		Fork:                         ForkFromConsensus(st.Fork()),
		LatestBlockHeader:            BeaconBlockHeaderFromConsensus(st.LatestBlockHeader()),
		BlockRoots:                   br,
		StateRoots:                   sr,
		HistoricalRoots:              hr,
		Eth1Data:                     Eth1DataFromConsensus(st.Eth1Data()),
		Eth1DataVotes:                votes,
		Eth1DepositIndex:             fmt.Sprintf("%d", st.Eth1DepositIndex()),
		Validators:                   vals,
		Balances:                     bals,
		RandaoMixes:                  rm,
		Slashings:                    slashings,
		PreviousEpochParticipation:   prevPart,
		CurrentEpochParticipation:    currPart,
		JustificationBits:            hexutil.Encode(st.JustificationBits()),
		PreviousJustifiedCheckpoint:  CheckpointFromConsensus(st.PreviousJustifiedCheckpoint()),
		CurrentJustifiedCheckpoint:   CheckpointFromConsensus(st.CurrentJustifiedCheckpoint()),
		FinalizedCheckpoint:          CheckpointFromConsensus(st.FinalizedCheckpoint()),
		InactivityScores:             is,
		CurrentSyncCommittee:         SyncCommitteeFromConsensus(currSc),
		NextSyncCommittee:            SyncCommitteeFromConsensus(nextSc),
		LatestExecutionPayloadHeader: payload,
		NextWithdrawalIndex:          fmt.Sprintf("%d", nwi),
		NextWithdrawalValidatorIndex: fmt.Sprintf("%d", nwvi),
		HistoricalSummaries:          hs,
	}, nil
}

package state_native

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	testtmpl "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/testing"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

func TestBeaconState_SlotDataRace_Phase0(t *testing.T) {
	testtmpl.VerifyBeaconStateSlotDataRace(t, func() (state.BeaconState, error) {
		return InitializeFromProtoPhase0(&ethpb.BeaconState{Slot: 1})
	})
}

func TestBeaconState_SlotDataRace_Altair(t *testing.T) {
	testtmpl.VerifyBeaconStateSlotDataRace(t, func() (state.BeaconState, error) {
		return InitializeFromProtoAltair(&ethpb.BeaconStateAltair{Slot: 1})
	})
}

func TestBeaconState_SlotDataRace_Bellatrix(t *testing.T) {
	testtmpl.VerifyBeaconStateSlotDataRace(t, func() (state.BeaconState, error) {
		return InitializeFromProtoBellatrix(&ethpb.BeaconStateBellatrix{Slot: 1})
	})
}

func TestBeaconState_SlotDataRace_Capella(t *testing.T) {
	testtmpl.VerifyBeaconStateSlotDataRace(t, func() (state.BeaconState, error) {
		return InitializeFromProtoCapella(&ethpb.BeaconStateCapella{Slot: 1})
	})
}

func TestBeaconState_SlotDataRace_Deneb(t *testing.T) {
	testtmpl.VerifyBeaconStateSlotDataRace(t, func() (state.BeaconState, error) {
		return InitializeFromProtoDeneb(&ethpb.BeaconStateDeneb{Slot: 1})
	})
}

func TestBeaconState_MatchCurrentJustifiedCheckpt_Phase0(t *testing.T) {
	testtmpl.VerifyBeaconStateMatchCurrentJustifiedCheckptNative(
		t,
		func(cp *ethpb.Checkpoint) (state.BeaconState, error) {
			return InitializeFromProtoPhase0(&ethpb.BeaconState{CurrentJustifiedCheckpoint: cp})
		},
	)
}

func TestBeaconState_MatchCurrentJustifiedCheckpt_Altair(t *testing.T) {
	testtmpl.VerifyBeaconStateMatchCurrentJustifiedCheckptNative(
		t,
		func(cp *ethpb.Checkpoint) (state.BeaconState, error) {
			return InitializeFromProtoAltair(&ethpb.BeaconStateAltair{CurrentJustifiedCheckpoint: cp})
		},
	)
}

func TestBeaconState_MatchCurrentJustifiedCheckpt_Bellatrix(t *testing.T) {
	testtmpl.VerifyBeaconStateMatchCurrentJustifiedCheckptNative(
		t,
		func(cp *ethpb.Checkpoint) (state.BeaconState, error) {
			return InitializeFromProtoBellatrix(&ethpb.BeaconStateBellatrix{CurrentJustifiedCheckpoint: cp})
		},
	)
}

func TestBeaconState_MatchCurrentJustifiedCheckpt_Capella(t *testing.T) {
	testtmpl.VerifyBeaconStateMatchCurrentJustifiedCheckptNative(
		t,
		func(cp *ethpb.Checkpoint) (state.BeaconState, error) {
			return InitializeFromProtoCapella(&ethpb.BeaconStateCapella{CurrentJustifiedCheckpoint: cp})
		},
	)
}

func TestBeaconState_MatchPreviousJustifiedCheckpt_Phase0(t *testing.T) {
	testtmpl.VerifyBeaconStateMatchPreviousJustifiedCheckptNative(
		t,
		func(cp *ethpb.Checkpoint) (state.BeaconState, error) {
			return InitializeFromProtoPhase0(&ethpb.BeaconState{PreviousJustifiedCheckpoint: cp})
		},
	)
}

func TestBeaconState_MatchPreviousJustifiedCheckpt_Altair(t *testing.T) {
	testtmpl.VerifyBeaconStateMatchPreviousJustifiedCheckptNative(
		t,
		func(cp *ethpb.Checkpoint) (state.BeaconState, error) {
			return InitializeFromProtoAltair(&ethpb.BeaconStateAltair{PreviousJustifiedCheckpoint: cp})
		},
	)
}

func TestBeaconState_MatchPreviousJustifiedCheckpt_Bellatrix(t *testing.T) {
	testtmpl.VerifyBeaconStateMatchPreviousJustifiedCheckptNative(
		t,
		func(cp *ethpb.Checkpoint) (state.BeaconState, error) {
			return InitializeFromProtoBellatrix(&ethpb.BeaconStateBellatrix{PreviousJustifiedCheckpoint: cp})
		},
	)
}

func TestBeaconState_MatchPreviousJustifiedCheckpt_Capella(t *testing.T) {
	testtmpl.VerifyBeaconStateMatchPreviousJustifiedCheckptNative(
		t,
		func(cp *ethpb.Checkpoint) (state.BeaconState, error) {
			return InitializeFromProtoCapella(&ethpb.BeaconStateCapella{PreviousJustifiedCheckpoint: cp})
		},
	)
}

func TestBeaconState_ValidatorByPubkey_Phase0(t *testing.T) {
	testtmpl.VerifyBeaconStateValidatorByPubkey(t, func() (state.BeaconState, error) {
		return InitializeFromProtoPhase0(&ethpb.BeaconState{})
	})
}

func TestBeaconState_ValidatorByPubkey_Altair(t *testing.T) {
	testtmpl.VerifyBeaconStateValidatorByPubkey(t, func() (state.BeaconState, error) {
		return InitializeFromProtoAltair(&ethpb.BeaconStateAltair{})
	})
}

func TestBeaconState_ValidatorByPubkey_Bellatrix(t *testing.T) {
	testtmpl.VerifyBeaconStateValidatorByPubkey(t, func() (state.BeaconState, error) {
		return InitializeFromProtoBellatrix(&ethpb.BeaconStateBellatrix{})
	})
}

func TestBeaconState_ValidatorByPubkey_Capella(t *testing.T) {
	testtmpl.VerifyBeaconStateValidatorByPubkey(t, func() (state.BeaconState, error) {
		return InitializeFromProtoCapella(&ethpb.BeaconStateCapella{})
	})
}

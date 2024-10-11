package state_native_test

import (
	"bytes"
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	statenative "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestInitializeFromProto_Phase0(t *testing.T) {
	testState, _ := util.DeterministicGenesisState(t, 64)
	pbState, err := statenative.ProtobufBeaconStatePhase0(testState.ToProtoUnsafe())
	require.NoError(t, err)
	type test struct {
		name  string
		state *ethpb.BeaconState
		error string
	}
	initTests := []test{
		{
			name:  "nil state",
			state: nil,
			error: "received nil state",
		},
		{
			name: "nil validators",
			state: &ethpb.BeaconState{
				Slot:       4,
				Validators: nil,
			},
		},
		{
			name:  "empty state",
			state: &ethpb.BeaconState{},
		},
		{
			name:  "full state",
			state: pbState,
		},
	}
	for _, tt := range initTests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := statenative.InitializeFromProtoUnsafePhase0(tt.state)
			if tt.error != "" {
				assert.ErrorContains(t, tt.error, err)
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func TestInitializeFromProto_Altair(t *testing.T) {
	type test struct {
		name  string
		state *ethpb.BeaconStateAltair
		error string
	}
	initTests := []test{
		{
			name:  "nil state",
			state: nil,
			error: "received nil state",
		},
		{
			name: "nil validators",
			state: &ethpb.BeaconStateAltair{
				Slot:       4,
				Validators: nil,
			},
		},
		{
			name:  "empty state",
			state: &ethpb.BeaconStateAltair{},
		},
	}
	for _, tt := range initTests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := statenative.InitializeFromProtoAltair(tt.state)
			if tt.error != "" {
				require.ErrorContains(t, tt.error, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestInitializeFromProto_Bellatrix(t *testing.T) {
	type test struct {
		name  string
		state *ethpb.BeaconStateBellatrix
		error string
	}
	initTests := []test{
		{
			name:  "nil state",
			state: nil,
			error: "received nil state",
		},
		{
			name: "nil validators",
			state: &ethpb.BeaconStateBellatrix{
				Slot:       4,
				Validators: nil,
			},
		},
		{
			name:  "empty state",
			state: &ethpb.BeaconStateBellatrix{},
		},
	}
	for _, tt := range initTests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := statenative.InitializeFromProtoBellatrix(tt.state)
			if tt.error != "" {
				require.ErrorContains(t, tt.error, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestInitializeFromProto_Capella(t *testing.T) {
	type test struct {
		name  string
		state *ethpb.BeaconStateCapella
		error string
	}
	initTests := []test{
		{
			name:  "nil state",
			state: nil,
			error: "received nil state",
		},
		{
			name: "nil validators",
			state: &ethpb.BeaconStateCapella{
				Slot:       4,
				Validators: nil,
			},
		},
		{
			name:  "empty state",
			state: &ethpb.BeaconStateCapella{},
		},
	}
	for _, tt := range initTests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := statenative.InitializeFromProtoCapella(tt.state)
			if tt.error != "" {
				require.ErrorContains(t, tt.error, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestInitializeFromProto_Deneb(t *testing.T) {
	type test struct {
		name  string
		state *ethpb.BeaconStateDeneb
		error string
	}
	initTests := []test{
		{
			name:  "nil state",
			state: nil,
			error: "received nil state",
		},
		{
			name: "nil validators",
			state: &ethpb.BeaconStateDeneb{
				Slot:       4,
				Validators: nil,
			},
		},
		{
			name:  "empty state",
			state: &ethpb.BeaconStateDeneb{},
		},
	}
	for _, tt := range initTests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := statenative.InitializeFromProtoDeneb(tt.state)
			if tt.error != "" {
				require.ErrorContains(t, tt.error, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestInitializeFromProtoUnsafe_Phase0(t *testing.T) {
	testState, _ := util.DeterministicGenesisState(t, 64)
	pbState, err := statenative.ProtobufBeaconStatePhase0(testState.ToProtoUnsafe())
	require.NoError(t, err)
	type test struct {
		name  string
		state *ethpb.BeaconState
		error string
	}
	initTests := []test{
		{
			name: "nil validators",
			state: &ethpb.BeaconState{
				Slot:       4,
				Validators: nil,
			},
		},
		{
			name:  "empty state",
			state: &ethpb.BeaconState{},
		},
		{
			name:  "full state",
			state: pbState,
		},
	}
	for _, tt := range initTests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := statenative.InitializeFromProtoUnsafePhase0(tt.state)
			if tt.error != "" {
				assert.ErrorContains(t, tt.error, err)
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func TestInitializeFromProtoUnsafe_Altair(t *testing.T) {
	type test struct {
		name  string
		state *ethpb.BeaconStateAltair
		error string
	}
	initTests := []test{
		{
			name: "nil validators",
			state: &ethpb.BeaconStateAltair{
				Slot:       4,
				Validators: nil,
			},
		},
		{
			name:  "empty state",
			state: &ethpb.BeaconStateAltair{},
		},
	}
	for _, tt := range initTests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := statenative.InitializeFromProtoUnsafeAltair(tt.state)
			if tt.error != "" {
				assert.ErrorContains(t, tt.error, err)
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func TestInitializeFromProtoUnsafe_Bellatrix(t *testing.T) {
	type test struct {
		name  string
		state *ethpb.BeaconStateBellatrix
		error string
	}
	initTests := []test{
		{
			name: "nil validators",
			state: &ethpb.BeaconStateBellatrix{
				Slot:       4,
				Validators: nil,
			},
		},
		{
			name:  "empty state",
			state: &ethpb.BeaconStateBellatrix{},
		},
	}
	for _, tt := range initTests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := statenative.InitializeFromProtoUnsafeBellatrix(tt.state)
			if tt.error != "" {
				assert.ErrorContains(t, tt.error, err)
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func TestInitializeFromProtoUnsafe_Capella(t *testing.T) {
	type test struct {
		name  string
		state *ethpb.BeaconStateCapella
		error string
	}
	initTests := []test{
		{
			name: "nil validators",
			state: &ethpb.BeaconStateCapella{
				Slot:       4,
				Validators: nil,
			},
		},
		{
			name:  "empty state",
			state: &ethpb.BeaconStateCapella{},
		},
	}
	for _, tt := range initTests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := statenative.InitializeFromProtoUnsafeCapella(tt.state)
			if tt.error != "" {
				assert.ErrorContains(t, tt.error, err)
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func TestInitializeFromProtoUnsafe_Deneb(t *testing.T) {
	type test struct {
		name  string
		state *ethpb.BeaconStateDeneb
		error string
	}
	initTests := []test{
		{
			name: "nil validators",
			state: &ethpb.BeaconStateDeneb{
				Slot:       4,
				Validators: nil,
			},
		},
		{
			name:  "empty state",
			state: &ethpb.BeaconStateDeneb{},
		},
	}
	for _, tt := range initTests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := statenative.InitializeFromProtoUnsafeDeneb(tt.state)
			if tt.error != "" {
				assert.ErrorContains(t, tt.error, err)
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func TestBeaconState_HashTreeRoot(t *testing.T) {
	testState, _ := util.DeterministicGenesisState(t, 64)

	type test struct {
		name        string
		stateModify func(beaconState state.BeaconState) (state.BeaconState, error)
		error       string
	}
	initTests := []test{
		{
			name: "unchanged state",
			stateModify: func(beaconState state.BeaconState) (state.BeaconState, error) {
				return beaconState, nil
			},
			error: "",
		},
		{
			name: "different slot",
			stateModify: func(beaconState state.BeaconState) (state.BeaconState, error) {
				if err := beaconState.SetSlot(5); err != nil {
					return nil, err
				}
				return beaconState, nil
			},
			error: "",
		},
		{
			name: "different validator balance",
			stateModify: func(beaconState state.BeaconState) (state.BeaconState, error) {
				val, err := beaconState.ValidatorAtIndex(5)
				if err != nil {
					return nil, err
				}
				val.EffectiveBalance = params.BeaconConfig().MaxEffectiveBalance - params.BeaconConfig().EffectiveBalanceIncrement
				if err := beaconState.UpdateValidatorAtIndex(5, val); err != nil {
					return nil, err
				}
				return beaconState, nil
			},
			error: "",
		},
	}

	var err error
	var oldHTR []byte
	for _, tt := range initTests {
		t.Run(tt.name, func(t *testing.T) {
			testState, err = tt.stateModify(testState)
			assert.NoError(t, err)
			root, err := testState.HashTreeRoot(context.Background())
			if err == nil && tt.error != "" {
				t.Errorf("Expected error, expected %v, received %v", tt.error, err)
			}
			pbState, err := statenative.ProtobufBeaconStatePhase0(testState.ToProtoUnsafe())
			require.NoError(t, err)
			genericHTR, err := pbState.HashTreeRoot()
			if err == nil && tt.error != "" {
				t.Errorf("Expected error, expected %v, received %v", tt.error, err)
			}
			assert.DeepNotEqual(t, []byte{}, root[:], "Received empty hash tree root")
			assert.DeepEqual(t, genericHTR[:], root[:], "Expected hash tree root to match generic")
			if len(oldHTR) != 0 && bytes.Equal(root[:], oldHTR) {
				t.Errorf("Expected HTR to change, received %#x == old %#x", root, oldHTR)
			}
			oldHTR = root[:]
		})
	}
}

func BenchmarkBeaconState(b *testing.B) {
	testState, _ := util.DeterministicGenesisState(b, 16000)
	pbState, err := statenative.ProtobufBeaconStatePhase0(testState.ToProtoUnsafe())
	require.NoError(b, err)

	b.Run("Vectorized SHA256", func(b *testing.B) {
		st, err := statenative.InitializeFromProtoUnsafePhase0(pbState)
		require.NoError(b, err)
		_, err = st.HashTreeRoot(context.Background())
		assert.NoError(b, err)
	})

	b.Run("Current SHA256", func(b *testing.B) {
		_, err := pbState.HashTreeRoot()
		require.NoError(b, err)
	})
}

func TestBeaconState_HashTreeRoot_FieldTrie(t *testing.T) {
	testState, _ := util.DeterministicGenesisState(t, 64)

	type test struct {
		name        string
		stateModify func(state.BeaconState) (state.BeaconState, error)
		error       string
	}
	initTests := []test{
		{
			name: "unchanged state",
			stateModify: func(beaconState state.BeaconState) (state.BeaconState, error) {
				return beaconState, nil
			},
			error: "",
		},
		{
			name: "different slot",
			stateModify: func(beaconState state.BeaconState) (state.BeaconState, error) {
				if err := beaconState.SetSlot(5); err != nil {
					return nil, err
				}
				return beaconState, nil
			},
			error: "",
		},
		{
			name: "different validator balance",
			stateModify: func(beaconState state.BeaconState) (state.BeaconState, error) {
				val, err := beaconState.ValidatorAtIndex(5)
				if err != nil {
					return nil, err
				}
				val.EffectiveBalance = params.BeaconConfig().MaxEffectiveBalance - params.BeaconConfig().EffectiveBalanceIncrement
				if err := beaconState.UpdateValidatorAtIndex(5, val); err != nil {
					return nil, err
				}
				return beaconState, nil
			},
			error: "",
		},
	}

	var err error
	var oldHTR []byte
	for _, tt := range initTests {
		t.Run(tt.name, func(t *testing.T) {
			testState, err = tt.stateModify(testState)
			assert.NoError(t, err)
			root, err := testState.HashTreeRoot(context.Background())
			if err == nil && tt.error != "" {
				t.Errorf("Expected error, expected %v, received %v", tt.error, err)
			}
			pbState, err := statenative.ProtobufBeaconStatePhase0(testState.ToProtoUnsafe())
			require.NoError(t, err)
			genericHTR, err := pbState.HashTreeRoot()
			if err == nil && tt.error != "" {
				t.Errorf("Expected error, expected %v, received %v", tt.error, err)
			}
			assert.DeepNotEqual(t, []byte{}, root[:], "Received empty hash tree root")
			assert.DeepEqual(t, genericHTR[:], root[:], "Expected hash tree root to match generic")
			if len(oldHTR) != 0 && bytes.Equal(root[:], oldHTR) {
				t.Errorf("Expected HTR to change, received %#x == old %#x", root, oldHTR)
			}
			oldHTR = root[:]
		})
	}
}

func TestBeaconState_AppendValidator_DoesntMutateCopy(t *testing.T) {
	st0, err := util.NewBeaconState()
	require.NoError(t, err)
	st1 := st0.Copy()
	originalCount := st1.NumValidators()

	val := &ethpb.Validator{Slashed: true}
	assert.NoError(t, st0.AppendValidator(val))
	assert.Equal(t, originalCount, st1.NumValidators(), "st1 NumValidators mutated")
	_, ok := st1.ValidatorIndexByPubkey(bytesutil.ToBytes48(val.PublicKey))
	assert.Equal(t, false, ok, "Expected no validator index to be present in st1 for the newly inserted pubkey")
}

func TestBeaconState_ValidatorMutation_Phase0(t *testing.T) {
	testState, _ := util.DeterministicGenesisState(t, 400)
	pbState, err := statenative.ProtobufBeaconStatePhase0(testState.ToProtoUnsafe())
	require.NoError(t, err)
	testState, err = statenative.InitializeFromProtoPhase0(pbState)
	require.NoError(t, err)

	_, err = testState.HashTreeRoot(context.Background())
	require.NoError(t, err)

	// Reset tries
	require.NoError(t, testState.UpdateValidatorAtIndex(200, new(ethpb.Validator)))
	_, err = testState.HashTreeRoot(context.Background())
	require.NoError(t, err)

	newState1 := testState.Copy()
	_ = testState.Copy()

	require.NoError(t, testState.UpdateValidatorAtIndex(15, &ethpb.Validator{
		PublicKey:                  make([]byte, 48),
		WithdrawalCredentials:      make([]byte, 32),
		EffectiveBalance:           1111,
		Slashed:                    false,
		ActivationEligibilityEpoch: 1112,
		ActivationEpoch:            1114,
		ExitEpoch:                  1116,
		WithdrawableEpoch:          1117,
	}))

	rt, err := testState.HashTreeRoot(context.Background())
	require.NoError(t, err)
	pbState, err = statenative.ProtobufBeaconStatePhase0(testState.ToProtoUnsafe())
	require.NoError(t, err)

	copiedTestState, err := statenative.InitializeFromProtoPhase0(pbState)
	require.NoError(t, err)

	rt2, err := copiedTestState.HashTreeRoot(context.Background())
	require.NoError(t, err)

	assert.Equal(t, rt, rt2)

	require.NoError(t, newState1.UpdateValidatorAtIndex(150, &ethpb.Validator{
		PublicKey:                  make([]byte, 48),
		WithdrawalCredentials:      make([]byte, 32),
		EffectiveBalance:           2111,
		Slashed:                    false,
		ActivationEligibilityEpoch: 2112,
		ActivationEpoch:            2114,
		ExitEpoch:                  2116,
		WithdrawableEpoch:          2117,
	}))

	rt, err = newState1.HashTreeRoot(context.Background())
	require.NoError(t, err)
	pbState, err = statenative.ProtobufBeaconStatePhase0(newState1.ToProtoUnsafe())
	require.NoError(t, err)

	copiedTestState, err = statenative.InitializeFromProtoPhase0(pbState)
	require.NoError(t, err)

	rt2, err = copiedTestState.HashTreeRoot(context.Background())
	require.NoError(t, err)

	assert.Equal(t, rt, rt2)
}

func TestBeaconState_ValidatorMutation_Altair(t *testing.T) {
	testState, _ := util.DeterministicGenesisStateAltair(t, 400)
	pbState, err := statenative.ProtobufBeaconStateAltair(testState.ToProtoUnsafe())
	require.NoError(t, err)
	testState, err = statenative.InitializeFromProtoAltair(pbState)
	require.NoError(t, err)

	_, err = testState.HashTreeRoot(context.Background())
	require.NoError(t, err)

	// Reset tries
	require.NoError(t, testState.UpdateValidatorAtIndex(200, new(ethpb.Validator)))
	_, err = testState.HashTreeRoot(context.Background())
	require.NoError(t, err)

	newState1 := testState.Copy()
	_ = testState.Copy()

	require.NoError(t, testState.UpdateValidatorAtIndex(15, &ethpb.Validator{
		PublicKey:                  make([]byte, 48),
		WithdrawalCredentials:      make([]byte, 32),
		EffectiveBalance:           1111,
		Slashed:                    false,
		ActivationEligibilityEpoch: 1112,
		ActivationEpoch:            1114,
		ExitEpoch:                  1116,
		WithdrawableEpoch:          1117,
	}))

	rt, err := testState.HashTreeRoot(context.Background())
	require.NoError(t, err)
	pbState, err = statenative.ProtobufBeaconStateAltair(testState.ToProtoUnsafe())
	require.NoError(t, err)

	copiedTestState, err := statenative.InitializeFromProtoAltair(pbState)
	require.NoError(t, err)

	rt2, err := copiedTestState.HashTreeRoot(context.Background())
	require.NoError(t, err)

	assert.Equal(t, rt, rt2)

	require.NoError(t, newState1.UpdateValidatorAtIndex(150, &ethpb.Validator{
		PublicKey:                  make([]byte, 48),
		WithdrawalCredentials:      make([]byte, 32),
		EffectiveBalance:           2111,
		Slashed:                    false,
		ActivationEligibilityEpoch: 2112,
		ActivationEpoch:            2114,
		ExitEpoch:                  2116,
		WithdrawableEpoch:          2117,
	}))

	rt, err = newState1.HashTreeRoot(context.Background())
	require.NoError(t, err)
	pbState, err = statenative.ProtobufBeaconStateAltair(newState1.ToProtoUnsafe())
	require.NoError(t, err)

	copiedTestState, err = statenative.InitializeFromProtoAltair(pbState)
	require.NoError(t, err)

	rt2, err = copiedTestState.HashTreeRoot(context.Background())
	require.NoError(t, err)

	assert.Equal(t, rt, rt2)
}

func TestBeaconState_ValidatorMutation_Bellatrix(t *testing.T) {
	testState, _ := util.DeterministicGenesisStateBellatrix(t, 400)
	pbState, err := statenative.ProtobufBeaconStateBellatrix(testState.ToProtoUnsafe())
	require.NoError(t, err)
	testState, err = statenative.InitializeFromProtoBellatrix(pbState)
	require.NoError(t, err)

	_, err = testState.HashTreeRoot(context.Background())
	require.NoError(t, err)

	// Reset tries
	require.NoError(t, testState.UpdateValidatorAtIndex(200, new(ethpb.Validator)))
	_, err = testState.HashTreeRoot(context.Background())
	require.NoError(t, err)

	newState1 := testState.Copy()
	_ = testState.Copy()

	require.NoError(t, testState.UpdateValidatorAtIndex(15, &ethpb.Validator{
		PublicKey:                  make([]byte, 48),
		WithdrawalCredentials:      make([]byte, 32),
		EffectiveBalance:           1111,
		Slashed:                    false,
		ActivationEligibilityEpoch: 1112,
		ActivationEpoch:            1114,
		ExitEpoch:                  1116,
		WithdrawableEpoch:          1117,
	}))

	rt, err := testState.HashTreeRoot(context.Background())
	require.NoError(t, err)
	pbState, err = statenative.ProtobufBeaconStateBellatrix(testState.ToProtoUnsafe())
	require.NoError(t, err)

	copiedTestState, err := statenative.InitializeFromProtoBellatrix(pbState)
	require.NoError(t, err)

	rt2, err := copiedTestState.HashTreeRoot(context.Background())
	require.NoError(t, err)

	assert.Equal(t, rt, rt2)

	require.NoError(t, newState1.UpdateValidatorAtIndex(150, &ethpb.Validator{
		PublicKey:                  make([]byte, 48),
		WithdrawalCredentials:      make([]byte, 32),
		EffectiveBalance:           2111,
		Slashed:                    false,
		ActivationEligibilityEpoch: 2112,
		ActivationEpoch:            2114,
		ExitEpoch:                  2116,
		WithdrawableEpoch:          2117,
	}))

	rt, err = newState1.HashTreeRoot(context.Background())
	require.NoError(t, err)
	pbState, err = statenative.ProtobufBeaconStateBellatrix(newState1.ToProtoUnsafe())
	require.NoError(t, err)

	copiedTestState, err = statenative.InitializeFromProtoBellatrix(pbState)
	require.NoError(t, err)

	rt2, err = copiedTestState.HashTreeRoot(context.Background())
	require.NoError(t, err)

	assert.Equal(t, rt, rt2)
}

func TestBeaconState_InitializeInactivityScoresCorrectly_Deneb(t *testing.T) {
	resetCfg := features.InitWithReset(&features.Flags{
		EnableExperimentalState: true,
	})
	defer resetCfg()
	st, _ := util.DeterministicGenesisStateDeneb(t, 200)
	_, err := st.HashTreeRoot(context.Background())
	require.NoError(t, err)
	ic, err := st.InactivityScores()
	require.NoError(t, err)
	ic[10] = 10000
	ic[100] = 1000

	err = st.SetInactivityScores(ic)
	require.NoError(t, err)

	_, err = st.HashTreeRoot(context.Background())
	require.NoError(t, err)

	ic[150] = 2390239
	err = st.SetInactivityScores(ic)
	require.NoError(t, err)
	rt, err := st.HashTreeRoot(context.Background())
	require.NoError(t, err)

	copiedSt, ok := st.ToProtoUnsafe().(*ethpb.BeaconStateDeneb)
	if !ok {
		t.Error("not ok")
	}
	newSt, err := statenative.InitializeFromProtoUnsafeDeneb(copiedSt)
	require.NoError(t, err)

	newRt, err := newSt.HashTreeRoot(context.Background())
	require.NoError(t, err)

	require.DeepSSZEqual(t, rt, newRt)
}

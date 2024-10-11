package state_native_test

import (
	"context"
	"testing"

	coreState "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/rand"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func FuzzPhase0StateHashTreeRoot(f *testing.F) {
	gState, _ := util.DeterministicGenesisState(f, 100)
	output, err := gState.MarshalSSZ()
	assert.NoError(f, err)
	randPool := make([]byte, 100)
	_, err = rand.NewDeterministicGenerator().Read(randPool)
	assert.NoError(f, err)
	f.Add(randPool, uint64(10))
	f.Fuzz(func(t *testing.T, diffBuffer []byte, slotsToTransition uint64) {
		stateSSZ := bytesutil.SafeCopyBytes(output)
		for i := 0; i < len(diffBuffer); i += 9 {
			if i+8 >= len(diffBuffer) {
				return
			}
			num := bytesutil.BytesToUint64BigEndian(diffBuffer[i : i+8])
			num %= uint64(len(diffBuffer))
			// Perform a XOR on the byte of the selected index.
			stateSSZ[num] ^= diffBuffer[i+8]
		}
		pbState := &ethpb.BeaconState{}
		err := pbState.UnmarshalSSZ(stateSSZ)
		if err != nil {
			return
		}
		nativeState, err := native.InitializeFromProtoPhase0(pbState)
		assert.NoError(t, err)

		slotsToTransition %= 100
		stateObj, err := native.InitializeFromProtoUnsafePhase0(pbState)
		assert.NoError(t, err)
		for stateObj.Slot() < primitives.Slot(slotsToTransition) {
			stateObj, err = coreState.ProcessSlots(context.Background(), stateObj, stateObj.Slot()+1)
			assert.NoError(t, err)
			stateObj.Copy()

			nativeState, err = coreState.ProcessSlots(context.Background(), nativeState, nativeState.Slot()+1)
			assert.NoError(t, err)
			nativeState.Copy()
		}
		assert.NoError(t, err)
		// Perform a cold HTR calculation by initializing a new state.
		innerState, ok := stateObj.ToProtoUnsafe().(*ethpb.BeaconState)
		assert.Equal(t, true, ok, "inner state is a not a beacon state proto")
		newState, err := native.InitializeFromProtoUnsafePhase0(innerState)
		assert.NoError(t, err)

		newRt, newErr := newState.HashTreeRoot(context.Background())
		rt, err := stateObj.HashTreeRoot(context.Background())
		nativeRt, nativeErr := nativeState.HashTreeRoot(context.Background())

		assert.Equal(t, newErr != nil, err != nil)
		assert.Equal(t, newErr != nil, nativeErr != nil)
		if err == nil {
			assert.Equal(t, rt, newRt)
			assert.Equal(t, rt, nativeRt)
		}

		newSSZ, newErr := newState.MarshalSSZ()
		stateObjSSZ, err := stateObj.MarshalSSZ()
		nativeSSZ, nativeErr := nativeState.MarshalSSZ()
		assert.Equal(t, newErr != nil, err != nil)
		assert.Equal(t, newErr != nil, nativeErr != nil)
		if err == nil {
			assert.DeepEqual(t, newSSZ, stateObjSSZ)
			assert.DeepEqual(t, newSSZ, nativeSSZ)
		}
	})
}

func FuzzAltairStateHashTreeRoot(f *testing.F) {
	gState, _ := util.DeterministicGenesisStateAltair(f, 100)
	output, err := gState.MarshalSSZ()
	assert.NoError(f, err)
	randPool := make([]byte, 100)
	_, err = rand.NewDeterministicGenerator().Read(randPool)
	assert.NoError(f, err)
	f.Add(randPool, uint64(10))
	f.Fuzz(func(t *testing.T, diffBuffer []byte, slotsToTransition uint64) {
		stateSSZ := bytesutil.SafeCopyBytes(output)
		for i := 0; i < len(diffBuffer); i += 9 {
			if i+8 >= len(diffBuffer) {
				return
			}
			num := bytesutil.BytesToUint64BigEndian(diffBuffer[i : i+8])
			num %= uint64(len(diffBuffer))
			// Perform a XOR on the byte of the selected index.
			stateSSZ[num] ^= diffBuffer[i+8]
		}
		pbState := &ethpb.BeaconStateAltair{}
		err := pbState.UnmarshalSSZ(stateSSZ)
		if err != nil {
			return
		}
		nativeState, err := native.InitializeFromProtoAltair(pbState)
		if err != nil {
			return
		}

		slotsToTransition %= 100
		stateObj, err := native.InitializeFromProtoUnsafeAltair(pbState)
		assert.NoError(t, err)
		for stateObj.Slot() < primitives.Slot(slotsToTransition) {
			stateObj, err = coreState.ProcessSlots(context.Background(), stateObj, stateObj.Slot()+1)
			assert.NoError(t, err)
			stateObj.Copy()

			nativeState, err = coreState.ProcessSlots(context.Background(), nativeState, nativeState.Slot()+1)
			assert.NoError(t, err)
			nativeState.Copy()
		}
		assert.NoError(t, err)
		// Perform a cold HTR calculation by initializing a new state.
		innerState, ok := stateObj.ToProtoUnsafe().(*ethpb.BeaconStateAltair)
		assert.Equal(t, true, ok, "inner state is a not a beacon state altair proto")
		newState, err := native.InitializeFromProtoUnsafeAltair(innerState)
		assert.NoError(t, err)

		newRt, newErr := newState.HashTreeRoot(context.Background())
		rt, err := stateObj.HashTreeRoot(context.Background())
		nativeRt, nativeErr := nativeState.HashTreeRoot(context.Background())
		assert.Equal(t, newErr != nil, err != nil)
		assert.Equal(t, newErr != nil, nativeErr != nil)
		if err == nil {
			assert.Equal(t, rt, newRt)
			assert.Equal(t, rt, nativeRt)
		}

		newSSZ, newErr := newState.MarshalSSZ()
		stateObjSSZ, err := stateObj.MarshalSSZ()
		nativeSSZ, nativeErr := nativeState.MarshalSSZ()
		assert.Equal(t, newErr != nil, err != nil)
		assert.Equal(t, newErr != nil, nativeErr != nil)
		if err == nil {
			assert.DeepEqual(t, newSSZ, stateObjSSZ)
			assert.DeepEqual(t, newSSZ, nativeSSZ)
		}
	})
}

func FuzzBellatrixStateHashTreeRoot(f *testing.F) {
	gState, _ := util.DeterministicGenesisStateBellatrix(f, 100)
	output, err := gState.MarshalSSZ()
	assert.NoError(f, err)
	randPool := make([]byte, 100)
	_, err = rand.NewDeterministicGenerator().Read(randPool)
	assert.NoError(f, err)
	f.Add(randPool, uint64(10))
	f.Fuzz(func(t *testing.T, diffBuffer []byte, slotsToTransition uint64) {
		stateSSZ := bytesutil.SafeCopyBytes(output)
		for i := 0; i < len(diffBuffer); i += 9 {
			if i+8 >= len(diffBuffer) {
				return
			}
			num := bytesutil.BytesToUint64BigEndian(diffBuffer[i : i+8])
			num %= uint64(len(diffBuffer))
			// Perform a XOR on the byte of the selected index.
			stateSSZ[num] ^= diffBuffer[i+8]
		}
		pbState := &ethpb.BeaconStateBellatrix{}
		err := pbState.UnmarshalSSZ(stateSSZ)
		if err != nil {
			return
		}
		nativeState, err := native.InitializeFromProtoBellatrix(pbState)
		if err != nil {
			return
		}

		slotsToTransition %= 100
		stateObj, err := native.InitializeFromProtoUnsafeBellatrix(pbState)
		assert.NoError(t, err)
		for stateObj.Slot() < primitives.Slot(slotsToTransition) {
			stateObj, err = coreState.ProcessSlots(context.Background(), stateObj, stateObj.Slot()+1)
			assert.NoError(t, err)
			stateObj.Copy()

			nativeState, err = coreState.ProcessSlots(context.Background(), nativeState, nativeState.Slot()+1)
			assert.NoError(t, err)
			nativeState.Copy()
		}
		assert.NoError(t, err)
		// Perform a cold HTR calculation by initializing a new state.
		innerState, ok := stateObj.ToProtoUnsafe().(*ethpb.BeaconStateBellatrix)
		assert.Equal(t, true, ok, "inner state is a not a beacon state bellatrix proto")
		newState, err := native.InitializeFromProtoUnsafeBellatrix(innerState)
		assert.NoError(t, err)

		newRt, newErr := newState.HashTreeRoot(context.Background())
		rt, err := stateObj.HashTreeRoot(context.Background())
		nativeRt, nativeErr := nativeState.HashTreeRoot(context.Background())
		assert.Equal(t, newErr != nil, err != nil)
		assert.Equal(t, newErr != nil, nativeErr != nil)
		if err == nil {
			assert.Equal(t, rt, newRt)
			assert.Equal(t, rt, nativeRt)
		}

		newSSZ, newErr := newState.MarshalSSZ()
		stateObjSSZ, err := stateObj.MarshalSSZ()
		nativeSSZ, nativeErr := nativeState.MarshalSSZ()
		assert.Equal(t, newErr != nil, err != nil)
		assert.Equal(t, newErr != nil, nativeErr != nil)
		if err == nil {
			assert.DeepEqual(t, newSSZ, stateObjSSZ)
			assert.DeepEqual(t, newSSZ, nativeSSZ)
		}
	})
}

func FuzzCapellaStateHashTreeRoot(f *testing.F) {
	gState, _ := util.DeterministicGenesisStateCapella(f, 100)
	output, err := gState.MarshalSSZ()
	assert.NoError(f, err)
	randPool := make([]byte, 100)
	_, err = rand.NewDeterministicGenerator().Read(randPool)
	assert.NoError(f, err)
	f.Add(randPool, uint64(10))
	f.Fuzz(func(t *testing.T, diffBuffer []byte, slotsToTransition uint64) {
		stateSSZ := bytesutil.SafeCopyBytes(output)
		for i := 0; i < len(diffBuffer); i += 9 {
			if i+8 >= len(diffBuffer) {
				return
			}
			num := bytesutil.BytesToUint64BigEndian(diffBuffer[i : i+8])
			num %= uint64(len(diffBuffer))
			// Perform a XOR on the byte of the selected index.
			stateSSZ[num] ^= diffBuffer[i+8]
		}
		pbState := &ethpb.BeaconStateCapella{}
		err := pbState.UnmarshalSSZ(stateSSZ)
		if err != nil {
			return
		}
		nativeState, err := native.InitializeFromProtoCapella(pbState)
		if err != nil {
			return
		}

		slotsToTransition %= 100
		stateObj, err := native.InitializeFromProtoUnsafeCapella(pbState)
		assert.NoError(t, err)
		for stateObj.Slot() < primitives.Slot(slotsToTransition) {
			stateObj, err = coreState.ProcessSlots(context.Background(), stateObj, stateObj.Slot()+1)
			assert.NoError(t, err)
			stateObj.Copy()

			nativeState, err = coreState.ProcessSlots(context.Background(), nativeState, nativeState.Slot()+1)
			assert.NoError(t, err)
			nativeState.Copy()
		}
		assert.NoError(t, err)
		// Perform a cold HTR calculation by initializing a new state.
		innerState, ok := stateObj.ToProtoUnsafe().(*ethpb.BeaconStateCapella)
		assert.Equal(t, true, ok, "inner state is a not a beacon state capella proto")
		newState, err := native.InitializeFromProtoUnsafeCapella(innerState)
		assert.NoError(t, err)

		newRt, newErr := newState.HashTreeRoot(context.Background())
		rt, err := stateObj.HashTreeRoot(context.Background())
		nativeRt, nativeErr := nativeState.HashTreeRoot(context.Background())
		assert.Equal(t, newErr != nil, err != nil)
		assert.Equal(t, newErr != nil, nativeErr != nil)
		if err == nil {
			assert.Equal(t, rt, newRt)
			assert.Equal(t, rt, nativeRt)
		}

		newSSZ, newErr := newState.MarshalSSZ()
		stateObjSSZ, err := stateObj.MarshalSSZ()
		nativeSSZ, nativeErr := nativeState.MarshalSSZ()
		assert.Equal(t, newErr != nil, err != nil)
		assert.Equal(t, newErr != nil, nativeErr != nil)
		if err == nil {
			assert.DeepEqual(t, newSSZ, stateObjSSZ)
			assert.DeepEqual(t, newSSZ, nativeSSZ)
		}
	})
}

func FuzzDenebStateHashTreeRoot(f *testing.F) {
	gState, _ := util.DeterministicGenesisStateDeneb(f, 100)
	output, err := gState.MarshalSSZ()
	assert.NoError(f, err)
	randPool := make([]byte, 100)
	_, err = rand.NewDeterministicGenerator().Read(randPool)
	assert.NoError(f, err)
	f.Add(randPool, uint64(10))
	f.Fuzz(func(t *testing.T, diffBuffer []byte, slotsToTransition uint64) {
		stateSSZ := bytesutil.SafeCopyBytes(output)
		for i := 0; i < len(diffBuffer); i += 9 {
			if i+8 >= len(diffBuffer) {
				return
			}
			num := bytesutil.BytesToUint64BigEndian(diffBuffer[i : i+8])
			num %= uint64(len(diffBuffer))
			// Perform a XOR on the byte of the selected index.
			stateSSZ[num] ^= diffBuffer[i+8]
		}
		pbState := &ethpb.BeaconStateDeneb{}
		err := pbState.UnmarshalSSZ(stateSSZ)
		if err != nil {
			return
		}
		nativeState, err := native.InitializeFromProtoDeneb(pbState)
		if err != nil {
			return
		}

		slotsToTransition %= 100
		stateObj, err := native.InitializeFromProtoUnsafeDeneb(pbState)
		assert.NoError(t, err)
		for stateObj.Slot() < primitives.Slot(slotsToTransition) {
			stateObj, err = coreState.ProcessSlots(context.Background(), stateObj, stateObj.Slot()+1)
			assert.NoError(t, err)
			stateObj.Copy()

			nativeState, err = coreState.ProcessSlots(context.Background(), nativeState, nativeState.Slot()+1)
			assert.NoError(t, err)
			nativeState.Copy()
		}
		assert.NoError(t, err)
		// Perform a cold HTR calculation by initializing a new state.
		innerState, ok := stateObj.ToProtoUnsafe().(*ethpb.BeaconStateDeneb)
		assert.Equal(t, true, ok, "inner state is a not a beacon state deneb proto")
		newState, err := native.InitializeFromProtoUnsafeDeneb(innerState)
		assert.NoError(t, err)

		newRt, newErr := newState.HashTreeRoot(context.Background())
		rt, err := stateObj.HashTreeRoot(context.Background())
		nativeRt, nativeErr := nativeState.HashTreeRoot(context.Background())
		assert.Equal(t, newErr != nil, err != nil)
		assert.Equal(t, newErr != nil, nativeErr != nil)
		if err == nil {
			assert.Equal(t, rt, newRt)
			assert.Equal(t, rt, nativeRt)
		}

		newSSZ, newErr := newState.MarshalSSZ()
		stateObjSSZ, err := stateObj.MarshalSSZ()
		nativeSSZ, nativeErr := nativeState.MarshalSSZ()
		assert.Equal(t, newErr != nil, err != nil)
		assert.Equal(t, newErr != nil, nativeErr != nil)
		if err == nil {
			assert.DeepEqual(t, newSSZ, stateObjSSZ)
			assert.DeepEqual(t, newSSZ, nativeSSZ)
		}
	})
}

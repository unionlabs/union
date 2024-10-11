package stateutil_test

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stateutil"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestReturnTrieLayer_OK(t *testing.T) {
	newState, _ := util.DeterministicGenesisState(t, 32)
	root, err := stateutil.RootsArrayHashTreeRoot(newState.BlockRoots(), uint64(params.BeaconConfig().SlotsPerHistoricalRoot))
	require.NoError(t, err)
	roots := retrieveBlockRoots(newState)
	layers, err := stateutil.ReturnTrieLayer(roots, uint64(len(roots)))
	assert.NoError(t, err)
	newRoot := *layers[len(layers)-1][0]
	assert.Equal(t, root, newRoot)

	layers, err = stateutil.ReturnTrieLayer(roots, uint64(len(roots)))
	assert.NoError(t, err)
	lastRoot := *layers[len(layers)-1][0]
	assert.Equal(t, root, lastRoot)
}

func BenchmarkReturnTrieLayer_NormalAlgorithm(b *testing.B) {
	newState, _ := util.DeterministicGenesisState(b, 32)
	root, err := stateutil.RootsArrayHashTreeRoot(newState.BlockRoots(), uint64(params.BeaconConfig().SlotsPerHistoricalRoot))
	require.NoError(b, err)
	roots := retrieveBlockRoots(newState)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		layers, err := stateutil.ReturnTrieLayer(roots, uint64(len(roots)))
		assert.NoError(b, err)
		newRoot := *layers[len(layers)-1][0]
		assert.Equal(b, root, newRoot)
	}
}

func BenchmarkReturnTrieLayer_VectorizedAlgorithm(b *testing.B) {
	newState, _ := util.DeterministicGenesisState(b, 32)
	root, err := stateutil.RootsArrayHashTreeRoot(newState.BlockRoots(), uint64(params.BeaconConfig().SlotsPerHistoricalRoot))
	require.NoError(b, err)
	roots := retrieveBlockRoots(newState)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		layers, err := stateutil.ReturnTrieLayer(roots, uint64(len(roots)))
		assert.NoError(b, err)
		newRoot := *layers[len(layers)-1][0]
		assert.Equal(b, root, newRoot)
	}
}

func TestReturnTrieLayerVariable_OK(t *testing.T) {
	newState, _ := util.DeterministicGenesisState(t, 32)
	root, err := stateutil.ValidatorRegistryRoot(newState.Validators())
	require.NoError(t, err)
	validators := newState.Validators()
	roots := make([][32]byte, 0, len(validators))
	for _, val := range validators {
		rt, err := stateutil.ValidatorRootWithHasher(val)
		require.NoError(t, err)
		roots = append(roots, rt)
	}
	layers := stateutil.ReturnTrieLayerVariable(roots, params.BeaconConfig().ValidatorRegistryLimit)
	newRoot := *layers[len(layers)-1][0]
	newRoot, err = stateutil.AddInMixin(newRoot, uint64(len(validators)))
	require.NoError(t, err)
	assert.Equal(t, root, newRoot)

	layers = stateutil.ReturnTrieLayerVariable(roots, params.BeaconConfig().ValidatorRegistryLimit)
	lastRoot := *layers[len(layers)-1][0]
	lastRoot, err = stateutil.AddInMixin(lastRoot, uint64(len(validators)))
	require.NoError(t, err)
	assert.Equal(t, root, lastRoot)

}

func BenchmarkReturnTrieLayerVariable_NormalAlgorithm(b *testing.B) {
	newState, _ := util.DeterministicGenesisState(b, 16000)
	root, err := stateutil.ValidatorRegistryRoot(newState.Validators())
	require.NoError(b, err)
	validators := newState.Validators()
	roots := make([][32]byte, 0, len(validators))
	for _, val := range validators {
		rt, err := stateutil.ValidatorRootWithHasher(val)
		require.NoError(b, err)
		roots = append(roots, rt)
	}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		layers := stateutil.ReturnTrieLayerVariable(roots, params.BeaconConfig().ValidatorRegistryLimit)
		newRoot := *layers[len(layers)-1][0]
		newRoot, err = stateutil.AddInMixin(newRoot, uint64(len(validators)))
		require.NoError(b, err)
		assert.Equal(b, root, newRoot)
	}
}

func BenchmarkReturnTrieLayerVariable_VectorizedAlgorithm(b *testing.B) {

	newState, _ := util.DeterministicGenesisState(b, 16000)
	root, err := stateutil.ValidatorRegistryRoot(newState.Validators())
	require.NoError(b, err)
	validators := newState.Validators()
	roots := make([][32]byte, 0, len(validators))
	for _, val := range validators {
		rt, err := stateutil.ValidatorRootWithHasher(val)
		require.NoError(b, err)
		roots = append(roots, rt)
	}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		layers := stateutil.ReturnTrieLayerVariable(roots, params.BeaconConfig().ValidatorRegistryLimit)
		newRoot := *layers[len(layers)-1][0]
		newRoot, err = stateutil.AddInMixin(newRoot, uint64(len(validators)))
		require.NoError(b, err)
		assert.Equal(b, root, newRoot)
	}
}

func TestRecomputeFromLayer_FixedSizedArray(t *testing.T) {
	newState, _ := util.DeterministicGenesisState(t, 32)
	roots := retrieveBlockRoots(newState)

	layers, err := stateutil.ReturnTrieLayer(roots, uint64(len(roots)))
	require.NoError(t, err)

	changedIdx := []uint64{24, 41}
	changedRoots := [][32]byte{{'A', 'B', 'C'}, {'D', 'E', 'F'}}
	require.NoError(t, newState.UpdateBlockRootAtIndex(changedIdx[0], changedRoots[0]))
	require.NoError(t, newState.UpdateBlockRootAtIndex(changedIdx[1], changedRoots[1]))

	expectedRoot, err := stateutil.RootsArrayHashTreeRoot(newState.BlockRoots(), uint64(params.BeaconConfig().SlotsPerHistoricalRoot))
	require.NoError(t, err)
	root, _, err := stateutil.RecomputeFromLayer(changedRoots, changedIdx, layers)
	require.NoError(t, err)
	assert.Equal(t, expectedRoot, root)
}

func TestRecomputeFromLayer_VariableSizedArray(t *testing.T) {
	newState, _ := util.DeterministicGenesisState(t, 32)
	validators := newState.Validators()
	roots := make([][32]byte, 0, len(validators))
	for _, val := range validators {
		rt, err := stateutil.ValidatorRootWithHasher(val)
		require.NoError(t, err)
		roots = append(roots, rt)
	}
	layers := stateutil.ReturnTrieLayerVariable(roots, params.BeaconConfig().ValidatorRegistryLimit)

	changedIdx := []uint64{2, 29}
	val1, err := newState.ValidatorAtIndex(10)
	require.NoError(t, err)
	val2, err := newState.ValidatorAtIndex(11)
	require.NoError(t, err)
	val1.Slashed = true
	val1.ExitEpoch = 20

	val2.Slashed = true
	val2.ExitEpoch = 40

	changedVals := []*ethpb.Validator{val1, val2}
	require.NoError(t, newState.UpdateValidatorAtIndex(primitives.ValidatorIndex(changedIdx[0]), changedVals[0]))
	require.NoError(t, newState.UpdateValidatorAtIndex(primitives.ValidatorIndex(changedIdx[1]), changedVals[1]))

	expectedRoot, err := stateutil.ValidatorRegistryRoot(newState.Validators())
	require.NoError(t, err)
	roots = make([][32]byte, 0, len(changedVals))
	for _, val := range changedVals {
		rt, err := stateutil.ValidatorRootWithHasher(val)
		require.NoError(t, err)
		roots = append(roots, rt)
	}
	root, _, err := stateutil.RecomputeFromLayerVariable(roots, changedIdx, layers)
	require.NoError(t, err)
	root, err = stateutil.AddInMixin(root, uint64(len(validators)))
	require.NoError(t, err)
	assert.Equal(t, expectedRoot, root)
}

func TestMerkleizeTrieLeaves_BadHashLayer(t *testing.T) {
	hashLayer := make([][32]byte, 12)
	layers := make([][][32]byte, 20)
	_, _, err := stateutil.MerkleizeTrieLeaves(layers, hashLayer)
	assert.ErrorContains(t, "hash layer is a non power of 2", err)
}

func retrieveBlockRoots(b state.BeaconState) [][32]byte {
	blockRts := b.BlockRoots()
	roots := make([][32]byte, 0, len(blockRts))
	for _, rt := range blockRts {
		roots = append(roots, bytesutil.ToBytes32(rt))
	}
	return roots
}

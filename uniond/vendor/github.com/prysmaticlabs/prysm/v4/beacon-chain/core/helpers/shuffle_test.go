package helpers

import (
	"fmt"
	"reflect"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/container/slice"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestShuffleList_InvalidValidatorCount(t *testing.T) {
	ClearCache()

	maxShuffleListSize = 20
	list := make([]primitives.ValidatorIndex, 21)
	if _, err := ShuffleList(list, [32]byte{123, 125}); err == nil {
		t.Error("Shuffle should have failed when validator count exceeds ModuloBias")
		maxShuffleListSize = 1 << 40
	}
	maxShuffleListSize = 1 << 40
}

func TestShuffleList_OK(t *testing.T) {
	ClearCache()

	var list1 []primitives.ValidatorIndex
	seed1 := [32]byte{1, 128, 12}
	seed2 := [32]byte{2, 128, 12}
	for i := 0; i < 10; i++ {
		list1 = append(list1, primitives.ValidatorIndex(i))
	}

	list2 := make([]primitives.ValidatorIndex, len(list1))
	copy(list2, list1)

	list1, err := ShuffleList(list1, seed1)
	assert.NoError(t, err, "Shuffle failed with")

	list2, err = ShuffleList(list2, seed2)
	assert.NoError(t, err, "Shuffle failed with")

	if reflect.DeepEqual(list1, list2) {
		t.Errorf("2 shuffled lists shouldn't be equal")
	}
	assert.DeepEqual(t, []primitives.ValidatorIndex{0, 7, 8, 6, 3, 9, 4, 5, 2, 1}, list1, "List 1 was incorrectly shuffled got")
	assert.DeepEqual(t, []primitives.ValidatorIndex{0, 5, 2, 1, 6, 8, 7, 3, 4, 9}, list2, "List 2 was incorrectly shuffled got")
}

func TestSplitIndices_OK(t *testing.T) {
	ClearCache()

	var l []uint64
	numValidators := uint64(64000)
	for i := uint64(0); i < numValidators; i++ {
		l = append(l, i)
	}
	split := SplitIndices(l, uint64(params.BeaconConfig().SlotsPerEpoch))
	assert.Equal(t, uint64(params.BeaconConfig().SlotsPerEpoch), uint64(len(split)), "Split list failed due to incorrect length")

	for _, s := range split {
		assert.Equal(t, numValidators/uint64(params.BeaconConfig().SlotsPerEpoch), uint64(len(s)), "Split list failed due to incorrect length")
	}
}

func TestShuffleList_Vs_ShuffleIndex(t *testing.T) {
	ClearCache()

	var list []primitives.ValidatorIndex
	listSize := uint64(1000)
	seed := [32]byte{123, 42}
	for i := primitives.ValidatorIndex(0); uint64(i) < listSize; i++ {
		list = append(list, i)
	}
	shuffledListByIndex := make([]primitives.ValidatorIndex, listSize)
	for i := primitives.ValidatorIndex(0); uint64(i) < listSize; i++ {
		si, err := ShuffledIndex(i, listSize, seed)
		assert.NoError(t, err)
		shuffledListByIndex[si] = i
	}
	shuffledList, err := ShuffleList(list, seed)
	require.NoError(t, err, "Shuffled list error")
	assert.DeepEqual(t, shuffledListByIndex, shuffledList, "Shuffled lists ar not equal")
}

func BenchmarkShuffledIndex(b *testing.B) {
	listSizes := []uint64{4000000, 40000, 400}
	seed := [32]byte{123, 42}
	for _, listSize := range listSizes {
		b.Run(fmt.Sprintf("ShuffledIndex_%d", listSize), func(ib *testing.B) {
			for i := uint64(0); i < uint64(ib.N); i++ {
				_, err := ShuffledIndex(primitives.ValidatorIndex(i%listSize), listSize, seed)
				assert.NoError(b, err)
			}
		})
	}
}

func BenchmarkIndexComparison(b *testing.B) {
	listSizes := []uint64{400000, 40000, 400}
	seed := [32]byte{123, 42}
	for _, listSize := range listSizes {
		b.Run(fmt.Sprintf("Indexwise_ShuffleList_%d", listSize), func(ib *testing.B) {
			for i := 0; i < ib.N; i++ {
				// Simulate a list-shuffle by running shuffle-index listSize times.
				for j := primitives.ValidatorIndex(0); uint64(j) < listSize; j++ {
					_, err := ShuffledIndex(j, listSize, seed)
					assert.NoError(b, err)
				}
			}
		})
	}
}

func BenchmarkShuffleList(b *testing.B) {
	listSizes := []uint64{400000, 40000, 400}
	seed := [32]byte{123, 42}
	for _, listSize := range listSizes {
		testIndices := make([]primitives.ValidatorIndex, listSize)
		for i := uint64(0); i < listSize; i++ {
			testIndices[i] = primitives.ValidatorIndex(i)
		}
		b.Run(fmt.Sprintf("ShuffleList_%d", listSize), func(ib *testing.B) {
			for i := 0; i < ib.N; i++ {
				_, err := ShuffleList(testIndices, seed)
				assert.NoError(b, err)
			}
		})
	}
}

func TestShuffledIndex(t *testing.T) {
	ClearCache()

	var list []primitives.ValidatorIndex
	listSize := uint64(399)
	for i := primitives.ValidatorIndex(0); uint64(i) < listSize; i++ {
		list = append(list, i)
	}
	shuffledList := make([]primitives.ValidatorIndex, listSize)
	unshuffledlist := make([]primitives.ValidatorIndex, listSize)
	seed := [32]byte{123, 42}
	for i := primitives.ValidatorIndex(0); uint64(i) < listSize; i++ {
		si, err := ShuffledIndex(i, listSize, seed)
		assert.NoError(t, err)
		shuffledList[si] = i
	}
	for i := primitives.ValidatorIndex(0); uint64(i) < listSize; i++ {
		ui, err := UnShuffledIndex(i, listSize, seed)
		assert.NoError(t, err)
		unshuffledlist[ui] = shuffledList[i]
	}
	assert.DeepEqual(t, list, unshuffledlist)
}

func TestSplitIndicesAndOffset_OK(t *testing.T) {
	ClearCache()

	var l []uint64
	validators := uint64(64000)
	for i := uint64(0); i < validators; i++ {
		l = append(l, i)
	}
	chunks := uint64(6)
	split := SplitIndices(l, chunks)
	for i := uint64(0); i < chunks; i++ {
		if !reflect.DeepEqual(split[i], l[slice.SplitOffset(uint64(len(l)), chunks, i):slice.SplitOffset(uint64(len(l)), chunks, i+1)]) {
			t.Errorf("Want: %v got: %v", l[slice.SplitOffset(uint64(len(l)), chunks, i):slice.SplitOffset(uint64(len(l)), chunks, i+1)], split[i])
			break
		}
	}
}

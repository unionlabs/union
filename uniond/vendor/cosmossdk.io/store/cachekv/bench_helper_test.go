package cachekv_test

import "crypto/rand"

func randSlice(sliceSize int) []byte {
	bz := make([]byte, sliceSize)
	_, _ = rand.Read(bz)
	return bz
}

func incrementByteSlice(bz []byte) {
	for index := len(bz) - 1; index >= 0; index-- {
		if bz[index] < 255 {
			bz[index]++
			break
		} else {
			bz[index] = 0
		}
	}
}

// Generate many keys starting at startKey, and are in sequential order
func generateSequentialKeys(startKey []byte, numKeys int) [][]byte {
	toReturn := make([][]byte, 0, numKeys)
	cur := make([]byte, len(startKey))
	copy(cur, startKey)
	for i := 0; i < numKeys; i++ {
		newKey := make([]byte, len(startKey))
		copy(newKey, cur)
		toReturn = append(toReturn, newKey)
		incrementByteSlice(cur)
	}
	return toReturn
}

// Generate many random, unsorted keys
func generateRandomKeys(keySize, numKeys int) [][]byte {
	toReturn := make([][]byte, 0, numKeys)
	for i := 0; i < numKeys; i++ {
		newKey := randSlice(keySize)
		toReturn = append(toReturn, newKey)
	}
	return toReturn
}

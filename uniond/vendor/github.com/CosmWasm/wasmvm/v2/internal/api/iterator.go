package api

import (
	"fmt"
	"math"
	"sync"

	"github.com/CosmWasm/wasmvm/v2/types"
)

// frame stores all Iterators for one contract call
type frame []types.Iterator

// iteratorFrames contains one frame for each contract call, indexed by contract call ID.
var (
	iteratorFrames      = make(map[uint64]frame)
	iteratorFramesMutex sync.Mutex
)

// this is a global counter for creating call IDs
var (
	latestCallID      uint64
	latestCallIDMutex sync.Mutex
)

// startCall is called at the beginning of a contract call to create a new frame in iteratorFrames.
// It updates latestCallID for generating a new call ID.
func startCall() uint64 {
	latestCallIDMutex.Lock()
	defer latestCallIDMutex.Unlock()
	latestCallID += 1
	return latestCallID
}

// removeFrame removes the frame with for the given call ID.
// The result can be nil when the frame is not initialized,
// i.e. when startCall() is called but no iterator is stored.
func removeFrame(callID uint64) frame {
	iteratorFramesMutex.Lock()
	defer iteratorFramesMutex.Unlock()

	remove := iteratorFrames[callID]
	delete(iteratorFrames, callID)
	return remove
}

// endCall is called at the end of a contract call to remove one item the iteratorFrames
func endCall(callID uint64) {
	// we pull removeFrame in another function so we don't hold the mutex while cleaning up the removed frame
	remove := removeFrame(callID)
	// free all iterators in the frame when we release it
	for _, iter := range remove {
		iter.Close()
	}
}

// storeIterator will add this to the end of the frame for the given call ID and return
// an iterator ID to reference it.
//
// We assign iterator IDs starting with 1 for historic reasons. This could be changed to 0
// I guess.
func storeIterator(callID uint64, it types.Iterator, frameLenLimit int) (uint64, error) {
	iteratorFramesMutex.Lock()
	defer iteratorFramesMutex.Unlock()

	new_index := len(iteratorFrames[callID])
	if new_index >= frameLenLimit {
		return 0, fmt.Errorf("Reached iterator limit (%d)", frameLenLimit)
	}

	// store at array position `new_index`
	iteratorFrames[callID] = append(iteratorFrames[callID], it)

	iterator_id, ok := indexToIteratorID(new_index)
	if !ok {
		// This error case is not expected to happen since the above code ensures the
		// index is in the range [0, frameLenLimit-1]
		return 0, fmt.Errorf("could not convert index to iterator ID")
	}
	return iterator_id, nil
}

// retrieveIterator will recover an iterator based on its ID.
func retrieveIterator(callID uint64, iteratorID uint64) types.Iterator {
	indexInFrame, ok := iteratorIdToIndex(iteratorID)
	if !ok {
		return nil
	}

	iteratorFramesMutex.Lock()
	defer iteratorFramesMutex.Unlock()
	myFrame := iteratorFrames[callID]
	if myFrame == nil {
		return nil
	}
	if indexInFrame >= len(myFrame) {
		// index out of range
		return nil
	}
	return myFrame[indexInFrame]
}

// iteratorIdToIndex converts an iterator ID to an index in the frame.
// The second value marks if the conversion succeeded.
func iteratorIdToIndex(id uint64) (int, bool) {
	if id < 1 || id > math.MaxInt32 {
		// If success is false, the int value is undefined. We use an arbitrary constant for potential debugging purposes.
		return 777777777, false
	}

	// Int conversion safe because value is in signed 32bit integer range
	return int(id) - 1, true
}

// indexToIteratorID converts an index in the frame to an iterator ID.
// The second value marks if the conversion succeeded.
func indexToIteratorID(index int) (uint64, bool) {
	if index < 0 || index > math.MaxInt32 {
		// If success is false, the return value is undefined. We use an arbitrary constant for potential debugging purposes.
		return 888888888, false
	}

	return uint64(index) + 1, true
}

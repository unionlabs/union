package async

import (
	"errors"
	"runtime"
	"sync"
)

// WorkerResults are the results of a scatter worker.
type WorkerResults struct {
	Offset int
	Extent interface{}
}

// Scatter scatters a computation across multiple goroutines.
// This breaks the task in to a number of chunks and executes those chunks in parallel with the function provided.
// Results returned are collected and presented as a set of WorkerResults, which can be reassembled by the calling function.
// Any error that occurs in the workers will be passed back to the calling function.
func Scatter(inputLen int, sFunc func(int, int, *sync.RWMutex) (interface{}, error)) ([]*WorkerResults, error) {
	if inputLen <= 0 {
		return nil, errors.New("input length must be greater than 0")
	}

	chunkSize := calculateChunkSize(inputLen)
	workers := inputLen / chunkSize
	if inputLen%chunkSize != 0 {
		workers++
	}
	resultCh := make(chan *WorkerResults, workers)
	defer close(resultCh)
	errorCh := make(chan error, workers)
	defer close(errorCh)
	mutex := new(sync.RWMutex)
	for worker := 0; worker < workers; worker++ {
		offset := worker * chunkSize
		entries := chunkSize
		if offset+entries > inputLen {
			entries = inputLen - offset
		}
		go func(offset int, entries int) {
			extent, err := sFunc(offset, entries, mutex)
			if err != nil {
				errorCh <- err
			} else {
				resultCh <- &WorkerResults{
					Offset: offset,
					Extent: extent,
				}
			}
		}(offset, entries)
	}

	// Collect results from workers
	results := make([]*WorkerResults, workers)
	for i := 0; i < workers; i++ {
		select {
		case result := <-resultCh:
			results[i] = result
		case err := <-errorCh:
			return nil, err
		}
	}
	return results, nil
}

// calculateChunkSize calculates a suitable chunk size for the purposes of parallelisation.
func calculateChunkSize(items int) int {
	// Start with a simple even split
	chunkSize := items / runtime.GOMAXPROCS(0)

	// Add 1 if we have leftovers (or if we have fewer items than processors).
	if chunkSize == 0 || items%chunkSize != 0 {
		chunkSize++
	}

	return chunkSize
}

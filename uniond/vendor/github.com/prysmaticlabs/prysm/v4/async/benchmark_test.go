package async_test

import (
	"crypto/rand"
	"crypto/sha256"
	"sync"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/async"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	log "github.com/sirupsen/logrus"
)

var input [][]byte

const (
	benchmarkElements    = 65536
	benchmarkElementSize = 32
	benchmarkHashRuns    = 128
)

func init() {
	input = make([][]byte, benchmarkElements)
	for i := 0; i < benchmarkElements; i++ {
		input[i] = make([]byte, benchmarkElementSize)
		_, err := rand.Read(input[i])
		if err != nil {
			log.WithError(err).Debug("Cannot read from rand")
		}
	}
}

// hash repeatedly hashes the data passed to it
func hash(input [][]byte) [][]byte {
	output := make([][]byte, len(input))
	for i := range input {
		copy(output, input)
		for j := 0; j < benchmarkHashRuns; j++ {
			hash := sha256.Sum256(output[i])
			output[i] = hash[:]
		}
	}
	return output
}

func BenchmarkHash(b *testing.B) {
	for i := 0; i < b.N; i++ {
		hash(input)
	}
}

func BenchmarkHashMP(b *testing.B) {
	output := make([][]byte, len(input))
	for i := 0; i < b.N; i++ {
		workerResults, err := async.Scatter(len(input), func(offset int, entries int, _ *sync.RWMutex) (interface{}, error) {
			return hash(input[offset : offset+entries]), nil
		})
		require.NoError(b, err)
		for _, result := range workerResults {
			copy(output[result.Offset:], result.Extent.([][]byte))
		}
	}
}

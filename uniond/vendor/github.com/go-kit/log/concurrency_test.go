package log_test

import (
	"math"
	"testing"

	"github.com/go-kit/log"
)

// These test are designed to be run with the race detector.

func testConcurrency(t *testing.T, logger log.Logger, total int) {
	n := int(math.Sqrt(float64(total)))
	share := total / n

	errC := make(chan error, n)

	for i := 0; i < n; i++ {
		go func() {
			errC <- spam(logger, share)
		}()
	}

	for i := 0; i < n; i++ {
		err := <-errC
		if err != nil {
			t.Fatalf("concurrent logging error: %v", err)
		}
	}
}

func spam(logger log.Logger, count int) error {
	for i := 0; i < count; i++ {
		err := logger.Log("key", i)
		if err != nil {
			return err
		}
	}
	return nil
}

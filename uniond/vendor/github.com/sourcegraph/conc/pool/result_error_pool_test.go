package pool

import (
	"errors"
	"fmt"
	"strconv"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

func TestResultErrorGroup(t *testing.T) {
	t.Parallel()

	err1 := errors.New("err1")
	err2 := errors.New("err2")

	t.Run("panics on configuration after init", func(t *testing.T) {
		t.Run("before wait", func(t *testing.T) {
			t.Parallel()
			g := NewWithResults[int]().WithErrors()
			g.Go(func() (int, error) { return 0, nil })
			require.Panics(t, func() { g.WithMaxGoroutines(10) })
		})

		t.Run("after wait", func(t *testing.T) {
			t.Parallel()
			g := NewWithResults[int]().WithErrors()
			g.Go(func() (int, error) { return 0, nil })
			_, _ = g.Wait()
			require.Panics(t, func() { g.WithMaxGoroutines(10) })
		})
	})

	t.Run("wait returns no error if no errors", func(t *testing.T) {
		t.Parallel()
		g := NewWithResults[int]().WithErrors()
		g.Go(func() (int, error) { return 1, nil })
		res, err := g.Wait()
		require.NoError(t, err)
		require.Equal(t, []int{1}, res)
	})

	t.Run("wait error if func returns error", func(t *testing.T) {
		t.Parallel()
		g := NewWithResults[int]().WithErrors()
		g.Go(func() (int, error) { return 0, err1 })
		res, err := g.Wait()
		require.Len(t, res, 0) // errored value is ignored
		require.ErrorIs(t, err, err1)
	})

	t.Run("WithCollectErrored", func(t *testing.T) {
		t.Parallel()
		g := NewWithResults[int]().WithErrors().WithCollectErrored()
		g.Go(func() (int, error) { return 0, err1 })
		res, err := g.Wait()
		require.Len(t, res, 1) // errored value is collected
		require.ErrorIs(t, err, err1)
	})

	t.Run("WithFirstError", func(t *testing.T) {
		t.Parallel()
		g := NewWithResults[int]().WithErrors().WithFirstError()
		synchronizer := make(chan struct{})
		g.Go(func() (int, error) {
			<-synchronizer
			// This test has an intrinsic race condition that can be reproduced
			// by adding a `defer time.Sleep(time.Second)` before the `defer
			// close(synchronizer)`. We cannot guarantee that the group processes
			// the return value of the second goroutine before the first goroutine
			// exits in response to synchronizer, so we add a sleep here to make
			// this race condition vanishingly unlikely. Note that this is a race
			// in the test, not in the library.
			time.Sleep(100 * time.Millisecond)
			return 0, err1
		})
		g.Go(func() (int, error) {
			defer close(synchronizer)
			return 0, err2
		})
		res, err := g.Wait()
		require.Len(t, res, 0)
		require.ErrorIs(t, err, err2)
		require.NotErrorIs(t, err, err1)
	})

	t.Run("wait error is all returned errors", func(t *testing.T) {
		t.Parallel()
		g := NewWithResults[int]().WithErrors()
		g.Go(func() (int, error) { return 0, err1 })
		g.Go(func() (int, error) { return 0, nil })
		g.Go(func() (int, error) { return 0, err2 })
		res, err := g.Wait()
		require.Len(t, res, 1)
		require.ErrorIs(t, err, err1)
		require.ErrorIs(t, err, err2)
	})

	t.Run("limit", func(t *testing.T) {
		t.Parallel()
		for _, maxConcurrency := range []int{1, 10, 100} {
			t.Run(strconv.Itoa(maxConcurrency), func(t *testing.T) {
				maxConcurrency := maxConcurrency // copy

				t.Parallel()
				g := NewWithResults[int]().WithErrors().WithMaxGoroutines(maxConcurrency)

				var currentConcurrent atomic.Int64
				taskCount := maxConcurrency * 10
				for i := 0; i < taskCount; i++ {
					g.Go(func() (int, error) {
						cur := currentConcurrent.Add(1)
						if cur > int64(maxConcurrency) {
							return 0, fmt.Errorf("expected no more than %d concurrent goroutine", maxConcurrency)
						}
						time.Sleep(time.Millisecond)
						currentConcurrent.Add(-1)
						return 0, nil
					})
				}
				res, err := g.Wait()
				require.Len(t, res, taskCount)
				require.NoError(t, err)
				require.Equal(t, int64(0), currentConcurrent.Load())
			})
		}
	})
}

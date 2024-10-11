package pool

import (
	"context"
	"errors"
	"fmt"
	"sort"
	"strconv"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestResultContextPool(t *testing.T) {
	t.Parallel()

	err1 := errors.New("err1")
	err2 := errors.New("err2")

	t.Run("panics on configuration after init", func(t *testing.T) {
		t.Run("before wait", func(t *testing.T) {
			t.Parallel()
			g := NewWithResults[int]().WithContext(context.Background())
			g.Go(func(context.Context) (int, error) { return 0, nil })
			require.Panics(t, func() { g.WithMaxGoroutines(10) })
		})

		t.Run("after wait", func(t *testing.T) {
			t.Parallel()
			g := NewWithResults[int]().WithContext(context.Background())
			g.Go(func(context.Context) (int, error) { return 0, nil })
			_, _ = g.Wait()
			require.Panics(t, func() { g.WithMaxGoroutines(10) })
		})
	})

	t.Run("behaves the same as ErrorGroup", func(t *testing.T) {
		t.Parallel()
		bgctx := context.Background()
		t.Run("wait returns no error if no errors", func(t *testing.T) {
			t.Parallel()
			g := NewWithResults[int]().WithContext(bgctx)
			g.Go(func(context.Context) (int, error) { return 0, nil })
			res, err := g.Wait()
			require.Len(t, res, 1)
			require.NoError(t, err)
		})

		t.Run("wait error if func returns error", func(t *testing.T) {
			t.Parallel()
			g := NewWithResults[int]().WithContext(bgctx)
			g.Go(func(context.Context) (int, error) { return 0, err1 })
			res, err := g.Wait()
			require.Len(t, res, 0)
			require.ErrorIs(t, err, err1)
		})

		t.Run("wait error is all returned errors", func(t *testing.T) {
			t.Parallel()
			g := NewWithResults[int]().WithErrors().WithContext(bgctx)
			g.Go(func(context.Context) (int, error) { return 0, err1 })
			g.Go(func(context.Context) (int, error) { return 0, nil })
			g.Go(func(context.Context) (int, error) { return 0, err2 })
			res, err := g.Wait()
			require.Len(t, res, 1)
			require.ErrorIs(t, err, err1)
			require.ErrorIs(t, err, err2)
		})
	})

	t.Run("context cancel propagates", func(t *testing.T) {
		t.Parallel()
		ctx, cancel := context.WithCancel(context.Background())
		g := NewWithResults[int]().WithContext(ctx)
		g.Go(func(ctx context.Context) (int, error) {
			<-ctx.Done()
			return 0, ctx.Err()
		})
		cancel()
		res, err := g.Wait()
		require.Len(t, res, 0)
		require.ErrorIs(t, err, context.Canceled)
	})

	t.Run("WithCancelOnError", func(t *testing.T) {
		t.Parallel()
		g := NewWithResults[int]().WithContext(context.Background()).WithCancelOnError()
		g.Go(func(ctx context.Context) (int, error) {
			<-ctx.Done()
			return 0, ctx.Err()
		})
		g.Go(func(ctx context.Context) (int, error) {
			return 0, err1
		})
		res, err := g.Wait()
		require.Len(t, res, 0)
		require.ErrorIs(t, err, context.Canceled)
		require.ErrorIs(t, err, err1)
	})

	t.Run("WithCancelOnError and panic", func(t *testing.T) {
		t.Parallel()
		p := NewWithResults[int]().
			WithContext(context.Background()).
			WithCancelOnError()
		var cancelledTasks atomic.Int64
		p.Go(func(ctx context.Context) (int, error) {
			<-ctx.Done()
			cancelledTasks.Add(1)
			return 0, ctx.Err()
		})
		p.Go(func(ctx context.Context) (int, error) {
			<-ctx.Done()
			cancelledTasks.Add(1)
			return 0, ctx.Err()
		})
		p.Go(func(ctx context.Context) (int, error) {
			panic("abort!")
		})
		assert.Panics(t, func() { _, _ = p.Wait() })
		assert.EqualValues(t, 2, cancelledTasks.Load())
	})

	t.Run("no WithCancelOnError", func(t *testing.T) {
		t.Parallel()
		g := NewWithResults[int]().WithContext(context.Background())
		g.Go(func(ctx context.Context) (int, error) {
			select {
			case <-ctx.Done():
				return 0, ctx.Err()
			case <-time.After(10 * time.Millisecond):
				return 0, nil
			}
		})
		g.Go(func(ctx context.Context) (int, error) {
			return 0, err1
		})
		res, err := g.Wait()
		require.Len(t, res, 1)
		require.NotErrorIs(t, err, context.Canceled)
		require.ErrorIs(t, err, err1)
	})

	t.Run("WithCollectErrored", func(t *testing.T) {
		t.Parallel()
		g := NewWithResults[int]().WithContext(context.Background()).WithCollectErrored()
		g.Go(func(context.Context) (int, error) { return 0, err1 })
		res, err := g.Wait()
		require.Len(t, res, 1) // errored value is collected
		require.ErrorIs(t, err, err1)
	})

	t.Run("WithFirstError", func(t *testing.T) {
		t.Parallel()
		g := NewWithResults[int]().WithContext(context.Background()).WithFirstError()
		sync := make(chan struct{})
		g.Go(func(ctx context.Context) (int, error) {
			defer close(sync)
			return 0, err1
		})
		g.Go(func(ctx context.Context) (int, error) {
			// This test has a race condition. After the first goroutine
			// completes, this goroutine is woken up because sync is closed.
			// However, this goroutine might be woken up before the error from
			// the first goroutine is registered. To prevent that, we sleep for
			// another 10 milliseconds, giving the other goroutine time to return
			// and register its error before this goroutine returns its error.
			<-sync
			time.Sleep(10 * time.Millisecond)
			return 0, err2
		})
		res, err := g.Wait()
		require.Len(t, res, 0)
		require.ErrorIs(t, err, err1)
		require.NotErrorIs(t, err, err2)
	})

	t.Run("limit", func(t *testing.T) {
		t.Parallel()
		for _, maxConcurrency := range []int{1, 10, 100} {
			t.Run(strconv.Itoa(maxConcurrency), func(t *testing.T) {
				maxConcurrency := maxConcurrency // copy

				t.Parallel()
				ctx := context.Background()
				g := NewWithResults[int]().WithContext(ctx).WithMaxGoroutines(maxConcurrency)

				var currentConcurrent atomic.Int64
				taskCount := maxConcurrency * 10
				expected := make([]int, taskCount)
				for i := 0; i < taskCount; i++ {
					i := i
					expected[i] = i
					g.Go(func(context.Context) (int, error) {
						cur := currentConcurrent.Add(1)
						if cur > int64(maxConcurrency) {
							return 0, fmt.Errorf("expected no more than %d concurrent goroutines", maxConcurrency)
						}
						time.Sleep(time.Millisecond)
						currentConcurrent.Add(-1)
						return i, nil
					})
				}
				res, err := g.Wait()
				sort.Ints(res)
				require.Equal(t, expected, res)
				require.NoError(t, err)
				require.Equal(t, int64(0), currentConcurrent.Load())
			})
		}
	})
}

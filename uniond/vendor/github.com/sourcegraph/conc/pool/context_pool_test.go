package pool

import (
	"context"
	"errors"
	"fmt"
	"strconv"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func ExampleContextPool_WithCancelOnError() {
	p := New().
		WithMaxGoroutines(4).
		WithContext(context.Background()).
		WithCancelOnError()
	for i := 0; i < 3; i++ {
		i := i
		p.Go(func(ctx context.Context) error {
			if i == 2 {
				return errors.New("I will cancel all other tasks!")
			}
			<-ctx.Done()
			return nil
		})
	}
	err := p.Wait()
	fmt.Println(err)
	// Output:
	// I will cancel all other tasks!
}

func TestContextPool(t *testing.T) {
	t.Parallel()

	err1 := errors.New("err1")
	err2 := errors.New("err2")
	bgctx := context.Background()

	t.Run("panics on configuration after init", func(t *testing.T) {
		t.Run("before wait", func(t *testing.T) {
			t.Parallel()
			g := New().WithContext(context.Background())
			g.Go(func(context.Context) error { return nil })
			require.Panics(t, func() { g.WithMaxGoroutines(10) })
		})

		t.Run("after wait", func(t *testing.T) {
			t.Parallel()
			g := New().WithContext(context.Background())
			g.Go(func(context.Context) error { return nil })
			_ = g.Wait()
			require.Panics(t, func() { g.WithMaxGoroutines(10) })
		})
	})

	t.Run("behaves the same as ErrorGroup", func(t *testing.T) {
		t.Parallel()

		t.Run("wait returns no error if no errors", func(t *testing.T) {
			t.Parallel()
			p := New().WithContext(bgctx)
			p.Go(func(context.Context) error { return nil })
			require.NoError(t, p.Wait())
		})

		t.Run("wait errors if func returns error", func(t *testing.T) {
			t.Parallel()
			p := New().WithContext(bgctx)
			p.Go(func(context.Context) error { return err1 })
			require.ErrorIs(t, p.Wait(), err1)
		})

		t.Run("wait error is all returned errors", func(t *testing.T) {
			t.Parallel()
			p := New().WithErrors().WithContext(bgctx)
			p.Go(func(context.Context) error { return err1 })
			p.Go(func(context.Context) error { return nil })
			p.Go(func(context.Context) error { return err2 })
			err := p.Wait()
			require.ErrorIs(t, err, err1)
			require.ErrorIs(t, err, err2)
		})
	})

	t.Run("context error propagates", func(t *testing.T) {
		t.Parallel()

		t.Run("canceled", func(t *testing.T) {
			t.Parallel()
			ctx, cancel := context.WithCancel(bgctx)
			p := New().WithContext(ctx)
			p.Go(func(ctx context.Context) error {
				<-ctx.Done()
				return ctx.Err()
			})
			cancel()
			require.ErrorIs(t, p.Wait(), context.Canceled)
		})

		t.Run("timed out", func(t *testing.T) {
			t.Parallel()
			ctx, cancel := context.WithTimeout(bgctx, time.Millisecond)
			defer cancel()
			p := New().WithContext(ctx)
			p.Go(func(ctx context.Context) error {
				<-ctx.Done()
				return ctx.Err()
			})
			require.ErrorIs(t, p.Wait(), context.DeadlineExceeded)
		})
	})

	t.Run("WithCancelOnError", func(t *testing.T) {
		t.Parallel()
		p := New().WithContext(bgctx).WithCancelOnError()
		p.Go(func(ctx context.Context) error {
			<-ctx.Done()
			return ctx.Err()
		})
		p.Go(func(ctx context.Context) error {
			return err1
		})
		err := p.Wait()
		require.ErrorIs(t, err, context.Canceled)
		require.ErrorIs(t, err, err1)
	})

	t.Run("no WithCancelOnError", func(t *testing.T) {
		t.Parallel()
		p := New().WithContext(bgctx)
		p.Go(func(ctx context.Context) error {
			select {
			case <-ctx.Done():
				return ctx.Err()
			case <-time.After(10 * time.Millisecond):
				return nil
			}
		})
		p.Go(func(ctx context.Context) error {
			return err1
		})
		err := p.Wait()
		require.ErrorIs(t, err, err1)
		require.NotErrorIs(t, err, context.Canceled)
	})

	t.Run("WithFirstError", func(t *testing.T) {
		t.Parallel()
		p := New().WithContext(bgctx).WithFirstError()
		sync := make(chan struct{})
		p.Go(func(ctx context.Context) error {
			defer close(sync)
			return err1
		})
		p.Go(func(ctx context.Context) error {
			// This test has a race condition. After the first goroutine
			// completes, this goroutine is woken up because sync is closed.
			// However, this goroutine might be woken up before the error from
			// the first goroutine is registered. To prevent that, we sleep for
			// another 10 milliseconds, giving the other goroutine time to return
			// and register its error before this goroutine returns its error.
			<-sync
			time.Sleep(10 * time.Millisecond)
			return err2
		})
		err := p.Wait()
		require.ErrorIs(t, err, err1)
		require.NotErrorIs(t, err, err2)
	})

	t.Run("WithFirstError and WithCancelOnError", func(t *testing.T) {
		t.Parallel()
		p := New().WithContext(bgctx).WithFirstError().WithCancelOnError()
		p.Go(func(ctx context.Context) error {
			return err1
		})
		p.Go(func(ctx context.Context) error {
			<-ctx.Done()
			return ctx.Err()
		})
		err := p.Wait()
		require.ErrorIs(t, err, err1)
		require.NotErrorIs(t, err, context.Canceled)
	})

	t.Run("WithCancelOnError and panic", func(t *testing.T) {
		t.Parallel()
		p := New().WithContext(bgctx).WithCancelOnError()
		var cancelledTasks atomic.Int64
		p.Go(func(ctx context.Context) error {
			<-ctx.Done()
			cancelledTasks.Add(1)
			return ctx.Err()
		})
		p.Go(func(ctx context.Context) error {
			<-ctx.Done()
			cancelledTasks.Add(1)
			return ctx.Err()
		})
		p.Go(func(ctx context.Context) error {
			panic("abort!")
		})
		assert.Panics(t, func() { _ = p.Wait() })
		assert.EqualValues(t, 2, cancelledTasks.Load())
	})

	t.Run("limit", func(t *testing.T) {
		t.Parallel()
		for _, maxConcurrent := range []int{1, 10, 100} {
			t.Run(strconv.Itoa(maxConcurrent), func(t *testing.T) {
				maxConcurrent := maxConcurrent // copy

				t.Parallel()
				p := New().WithContext(bgctx).WithMaxGoroutines(maxConcurrent)

				var currentConcurrent atomic.Int64
				for i := 0; i < 100; i++ {
					p.Go(func(context.Context) error {
						cur := currentConcurrent.Add(1)
						if cur > int64(maxConcurrent) {
							return fmt.Errorf("expected no more than %d concurrent goroutine", maxConcurrent)
						}
						time.Sleep(time.Millisecond)
						currentConcurrent.Add(-1)
						return nil
					})
				}
				require.NoError(t, p.Wait())
				require.Equal(t, int64(0), currentConcurrent.Load())
			})
		}
	})
}

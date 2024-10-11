package conc

import (
	"fmt"
	"sync/atomic"
	"testing"

	"github.com/stretchr/testify/require"
)

func ExampleWaitGroup() {
	var count atomic.Int64

	var wg WaitGroup
	for i := 0; i < 10; i++ {
		wg.Go(func() {
			count.Add(1)
		})
	}
	wg.Wait()

	fmt.Println(count.Load())
	// Output:
	// 10
}

func ExampleWaitGroup_WaitAndRecover() {
	var wg WaitGroup

	wg.Go(func() {
		panic("super bad thing")
	})

	recoveredPanic := wg.WaitAndRecover()
	fmt.Println(recoveredPanic.Value)
	// Output:
	// super bad thing
}

func TestWaitGroup(t *testing.T) {
	t.Parallel()

	t.Run("ctor", func(t *testing.T) {
		t.Parallel()
		wg := NewWaitGroup()
		require.IsType(t, &WaitGroup{}, wg)
	})

	t.Run("all spawned run", func(t *testing.T) {
		t.Parallel()
		var count atomic.Int64
		var wg WaitGroup
		for i := 0; i < 100; i++ {
			wg.Go(func() {
				count.Add(1)
			})
		}
		wg.Wait()
		require.Equal(t, count.Load(), int64(100))
	})

	t.Run("panic", func(t *testing.T) {
		t.Parallel()

		t.Run("is propagated", func(t *testing.T) {
			t.Parallel()
			var wg WaitGroup
			wg.Go(func() {
				panic("super bad thing")
			})
			require.Panics(t, wg.Wait)
		})

		t.Run("one is propagated", func(t *testing.T) {
			t.Parallel()
			var wg WaitGroup
			wg.Go(func() {
				panic("super bad thing")
			})
			wg.Go(func() {
				panic("super badder thing")
			})
			require.Panics(t, wg.Wait)
		})

		t.Run("non-panics do not overwrite panic", func(t *testing.T) {
			t.Parallel()
			var wg WaitGroup
			wg.Go(func() {
				panic("super bad thing")
			})
			for i := 0; i < 10; i++ {
				wg.Go(func() {})
			}
			require.Panics(t, wg.Wait)
		})

		t.Run("non-panics run successfully", func(t *testing.T) {
			t.Parallel()
			var wg WaitGroup
			var i atomic.Int64
			wg.Go(func() {
				i.Add(1)
			})
			wg.Go(func() {
				panic("super bad thing")
			})
			wg.Go(func() {
				i.Add(1)
			})
			require.Panics(t, wg.Wait)
			require.Equal(t, int64(2), i.Load())
		})

		t.Run("is caught by waitandrecover", func(t *testing.T) {
			t.Parallel()
			var wg WaitGroup
			wg.Go(func() {
				panic("super bad thing")
			})
			p := wg.WaitAndRecover()
			require.Equal(t, p.Value, "super bad thing")
		})

		t.Run("one is caught by waitandrecover", func(t *testing.T) {
			t.Parallel()
			var wg WaitGroup
			wg.Go(func() {
				panic("super bad thing")
			})
			wg.Go(func() {
				panic("super badder thing")
			})
			p := wg.WaitAndRecover()
			require.NotNil(t, p)
		})

		t.Run("nonpanics run successfully with waitandrecover", func(t *testing.T) {
			t.Parallel()
			var wg WaitGroup
			var i atomic.Int64
			wg.Go(func() {
				i.Add(1)
			})
			wg.Go(func() {
				panic("super bad thing")
			})
			wg.Go(func() {
				i.Add(1)
			})
			p := wg.WaitAndRecover()
			require.Equal(t, p.Value, "super bad thing")
			require.Equal(t, int64(2), i.Load())
		})
	})
}

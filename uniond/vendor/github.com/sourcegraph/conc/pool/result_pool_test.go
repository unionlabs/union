package pool

import (
	"fmt"
	"sort"
	"strconv"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

func ExampleResultPool() {
	p := NewWithResults[int]()
	for i := 0; i < 10; i++ {
		i := i
		p.Go(func() int {
			return i * 2
		})
	}
	res := p.Wait()
	// Result order is nondeterministic, so sort them first
	sort.Ints(res)
	fmt.Println(res)

	// Output:
	// [0 2 4 6 8 10 12 14 16 18]
}

func TestResultGroup(t *testing.T) {
	t.Parallel()

	t.Run("panics on configuration after init", func(t *testing.T) {
		t.Run("before wait", func(t *testing.T) {
			t.Parallel()
			g := NewWithResults[int]()
			g.Go(func() int { return 0 })
			require.Panics(t, func() { g.WithMaxGoroutines(10) })
		})

		t.Run("after wait", func(t *testing.T) {
			t.Parallel()
			g := NewWithResults[int]()
			g.Go(func() int { return 0 })
			require.Panics(t, func() { g.WithMaxGoroutines(10) })
		})
	})

	t.Run("basic", func(t *testing.T) {
		t.Parallel()
		g := NewWithResults[int]()
		expected := []int{}
		for i := 0; i < 100; i++ {
			i := i
			expected = append(expected, i)
			g.Go(func() int {
				return i
			})
		}
		res := g.Wait()
		sort.Ints(res)
		require.Equal(t, expected, res)
	})

	t.Run("limit", func(t *testing.T) {
		t.Parallel()
		for _, maxGoroutines := range []int{1, 10, 100} {
			t.Run(strconv.Itoa(maxGoroutines), func(t *testing.T) {
				g := NewWithResults[int]().WithMaxGoroutines(maxGoroutines)

				var currentConcurrent atomic.Int64
				var errCount atomic.Int64
				taskCount := maxGoroutines * 10
				expected := make([]int, taskCount)
				for i := 0; i < taskCount; i++ {
					i := i
					expected[i] = i
					g.Go(func() int {
						cur := currentConcurrent.Add(1)
						if cur > int64(maxGoroutines) {
							errCount.Add(1)
						}
						time.Sleep(time.Millisecond)
						currentConcurrent.Add(-1)
						return i
					})
				}
				res := g.Wait()
				sort.Ints(res)
				require.Equal(t, expected, res)
				require.Equal(t, int64(0), errCount.Load())
				require.Equal(t, int64(0), currentConcurrent.Load())
			})
		}
	})
}

package stream

import (
	"fmt"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

func ExampleStream() {
	times := []int{20, 52, 16, 45, 4, 80}

	stream := New()
	for _, millis := range times {
		dur := time.Duration(millis) * time.Millisecond
		stream.Go(func() Callback {
			time.Sleep(dur)
			// This will print in the order the tasks were submitted
			return func() { fmt.Println(dur) }
		})
	}
	stream.Wait()

	// Output:
	// 20ms
	// 52ms
	// 16ms
	// 45ms
	// 4ms
	// 80ms
}

func TestStream(t *testing.T) {
	t.Parallel()

	t.Run("simple", func(t *testing.T) {
		t.Parallel()
		s := New()
		var res []int
		for i := 0; i < 5; i++ {
			i := i
			s.Go(func() Callback {
				i *= 2
				return func() {
					res = append(res, i)
				}
			})
		}
		s.Wait()
		require.Equal(t, []int{0, 2, 4, 6, 8}, res)
	})

	t.Run("max goroutines", func(t *testing.T) {
		t.Parallel()
		s := New().WithMaxGoroutines(5)
		var currentTaskCount atomic.Int64
		var currentCallbackCount atomic.Int64
		for i := 0; i < 50; i++ {
			s.Go(func() Callback {
				curr := currentTaskCount.Add(1)
				if curr > 5 {
					t.Fatal("too many concurrent tasks being executed")
				}
				defer currentTaskCount.Add(-1)

				time.Sleep(time.Millisecond)

				return func() {
					curr := currentCallbackCount.Add(1)
					if curr > 1 {
						t.Fatal("too many concurrent callbacks being executed")
					}

					time.Sleep(time.Millisecond)

					defer currentCallbackCount.Add(-1)
				}
			})
		}
		s.Wait()
	})

	t.Run("panic in task is propagated", func(t *testing.T) {
		t.Parallel()
		s := New().WithMaxGoroutines(5)
		s.Go(func() Callback {
			panic("something really bad happened in the task")
		})
		require.Panics(t, s.Wait)
	})

	t.Run("panic in callback is propagated", func(t *testing.T) {
		t.Parallel()
		s := New().WithMaxGoroutines(5)
		s.Go(func() Callback {
			return func() {
				panic("something really bad happened in the callback")
			}
		})
		require.Panics(t, s.Wait)
	})

	t.Run("panic in callback does not block producers", func(t *testing.T) {
		t.Parallel()
		s := New().WithMaxGoroutines(5)
		s.Go(func() Callback {
			return func() {
				panic("something really bad happened in the callback")
			}
		})
		for i := 0; i < 100; i++ {
			s.Go(func() Callback {
				return func() {}
			})
		}
		require.Panics(t, s.Wait)
	})
}

func BenchmarkStream(b *testing.B) {
	b.Run("startup and teardown", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			s := New()
			s.Go(func() Callback { return func() {} })
			s.Wait()
		}
	})

	b.Run("per task", func(b *testing.B) {
		n := 0
		s := New()
		for i := 0; i < b.N; i++ {
			s.Go(func() Callback {
				return func() {
					n += 1
				}
			})
		}
		s.Wait()
	})
}

package grocksdb

import (
	"fmt"
	"sync"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestCOWList(t *testing.T) {
	t.Parallel()

	cl := NewCOWList()
	cl.Append("hello")
	cl.Append("world")
	cl.Append("!")
	require.EqualValues(t, cl.Get(0), "hello")
	require.EqualValues(t, cl.Get(1), "world")
	require.EqualValues(t, cl.Get(2), "!")
}

func TestCOWListMT(t *testing.T) {
	t.Parallel()

	cl := NewCOWList()
	expectedRes := make([]int, 3)
	var wg sync.WaitGroup
	for i := 0; i < 3; i++ {
		wg.Add(1)
		go func(v int) {
			defer wg.Done()
			index := cl.Append(v)
			expectedRes[index] = v
		}(i)
	}
	wg.Wait()
	for i, v := range expectedRes {
		require.EqualValues(t, cl.Get(i), v)
	}
}

func BenchmarkCOWList_Get(b *testing.B) {
	cl := NewCOWList()
	for i := 0; i < 10; i++ {
		cl.Append(fmt.Sprintf("helloworld%d", i))
	}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = cl.Get(i % 10).(string)
	}
}

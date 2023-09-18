package mempool_test

import (
	"testing"

	"github.com/huandu/skiplist"
	"github.com/stretchr/testify/require"
)

type collisionKey struct {
	a int
	b int
}

func TestSkipListCollisions(t *testing.T) {
	integerList := skiplist.New(skiplist.Int)

	integerList.Set(1, 1)
	integerList.Set(2, 2)
	integerList.Set(3, 3)

	k := integerList.Front()
	i := 1
	for k != nil {
		require.Equal(t, i, k.Key())
		require.Equal(t, i, k.Value)
		i++
		k = k.Next()
	}

	// a duplicate key will overwrite the previous value
	integerList.Set(1, 4)
	require.Equal(t, 3, integerList.Len())
	require.Equal(t, 4, integerList.Get(1).Value)

	// prove this again with a compound key
	compoundList := skiplist.New(skiplist.LessThanFunc(func(x, y any) int {
		kx := x.(collisionKey)
		ky := y.(collisionKey)
		if kx.a == ky.a {
			return skiplist.Int.Compare(kx.b, ky.b)
		}
		return skiplist.Int.Compare(kx.a, ky.a)
	}))

	compoundList.Set(collisionKey{a: 1, b: 1}, 1)
	compoundList.Set(collisionKey{a: 1, b: 2}, 2)
	compoundList.Set(collisionKey{a: 1, b: 3}, 3)

	require.Equal(t, 3, compoundList.Len())
	compoundList.Set(collisionKey{a: 1, b: 2}, 4)
	require.Equal(t, 4, compoundList.Get(collisionKey{a: 1, b: 2}).Value)
	compoundList.Set(collisionKey{a: 2, b: 2}, 5)
	require.Equal(t, 4, compoundList.Len())
}

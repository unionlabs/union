package doublylinkedlist

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestAppend(t *testing.T) {
	t.Run("empty list", func(t *testing.T) {
		list := &List[int]{}
		i := 1
		list.Append(NewNode(i))
		require.Equal(t, i, list.len)
		require.NotNil(t, list.first)
		assert.Equal(t, i, list.first.value)
		require.NotNil(t, list.last)
		assert.DeepEqual(t, i, list.last.value)
	})
	t.Run("one node in list", func(t *testing.T) {
		list := &List[int]{}
		old := 1
		i := 2
		list.Append(NewNode(old))
		list.Append(NewNode(i))
		require.Equal(t, 2, list.len)
		require.NotNil(t, list.first)
		assert.DeepEqual(t, old, list.first.value)
		require.NotNil(t, list.first.next)
		assert.Equal(t, i, list.first.next.value)
		require.NotNil(t, list.last)
		assert.DeepEqual(t, i, list.last.value)
		require.NotNil(t, list.last.prev)
		assert.Equal(t, old, list.last.prev.value)
	})
	t.Run("multiple nodes in list", func(t *testing.T) {
		list := &List[int]{}
		first := 1
		second := 2
		i := 3
		list.Append(NewNode(first))
		list.Append(NewNode(second))
		list.Append(NewNode(i))
		require.Equal(t, 3, list.len)
		require.NotNil(t, list.first)
		assert.DeepEqual(t, first, list.first.value)
		require.NotNil(t, list.first.next)
		assert.Equal(t, second, list.first.next.value)
		assert.Equal(t, first, list.first.next.prev.value)
		require.NotNil(t, list.last)
		assert.DeepEqual(t, i, list.last.value)
		require.NotNil(t, list.first.next.next)
		assert.DeepEqual(t, i, list.first.next.next.value)
		require.NotNil(t, list.last.prev)
		assert.Equal(t, second, list.last.prev.value)
	})
}

func TestRemove(t *testing.T) {
	t.Run("one node in list", func(t *testing.T) {
		list := &List[int]{}
		n := NewNode(1)
		list.Append(n)
		list.Remove(n)
		assert.Equal(t, 0, list.len)
		assert.Equal(t, (*Node[int])(nil), list.first)
		assert.Equal(t, (*Node[int])(nil), list.last)
	})
	t.Run("first of multiple nodes", func(t *testing.T) {
		list := &List[int]{}
		first := NewNode(1)
		second := NewNode(2)
		third := NewNode(3)
		list.Append(first)
		list.Append(second)
		list.Append(third)
		list.Remove(first)
		require.Equal(t, 2, list.len)
		require.NotNil(t, list.first)
		assert.Equal(t, 2, list.first.value)
		assert.Equal(t, (*Node[int])(nil), list.first.prev)
	})
	t.Run("last of multiple nodes", func(t *testing.T) {
		list := &List[int]{}
		first := NewNode(1)
		second := NewNode(2)
		third := NewNode(3)
		list.Append(first)
		list.Append(second)
		list.Append(third)
		list.Remove(third)
		require.Equal(t, 2, list.len)
		require.NotNil(t, list.last)
		assert.Equal(t, 2, list.last.value)
		assert.Equal(t, (*Node[int])(nil), list.last.next)
	})
	t.Run("in the middle of multiple nodes", func(t *testing.T) {
		list := &List[int]{}
		first := NewNode(1)
		second := NewNode(2)
		third := NewNode(3)
		list.Append(first)
		list.Append(second)
		list.Append(third)
		list.Remove(second)
		require.Equal(t, 2, list.len)
		require.NotNil(t, list.first)
		require.NotNil(t, list.last)
		assert.Equal(t, 1, list.first.value)
		assert.Equal(t, 3, list.last.value)
		assert.Equal(t, 3, list.first.next.value)
		assert.Equal(t, 1, list.last.prev.value)
	})
	t.Run("not in list", func(t *testing.T) {
		list := &List[int]{}
		first := NewNode(1)
		second := NewNode(2)
		n := NewNode(3)
		list.Append(first)
		list.Append(second)
		list.Remove(n)
		require.Equal(t, 2, list.len)
	})
}

func TestNodeCopy(t *testing.T) {
	first := NewNode(1)
	second := first.Copy()
	v, err := second.Value()
	require.NoError(t, err)
	require.Equal(t, first.value, v)
}

func TestListCopy(t *testing.T) {
	list := &List[int]{}
	first := NewNode(1)
	second := NewNode(2)
	third := NewNode(3)
	list.Append(first)
	list.Append(second)
	list.Append(third)

	copied := list.Copy()
	require.Equal(t, 3, copied.Len())
	m := copied.First()
	for n := list.First(); n != nil; n = n.next {
		nv, err := n.Value()
		require.NoError(t, err)
		mv, err := m.Value()
		require.NoError(t, err)
		require.Equal(t, nv, mv)

		require.NotEqual(t, n, m)
		m, err = m.Next()
		require.NoError(t, err)
	}
}

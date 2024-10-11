# Immutable Tree

### Structure

The Immutable tree struct contains an IAVL version.

```golang
type ImmutableTree struct {
	root                   *Node
	ndb                    *nodeDB
	version                int64
	skipFastStorageUpgrade bool
}
```

Using the root and the nodeDB, the ImmutableTree can retrieve any node that is a part of the IAVL tree at this version.

Users can get information about the IAVL tree by calling getter functions such as `Size()` and `Height()` which will return the tree's size and height by querying the root node's size and height.

### Get

Users can get values by specifying the key or the index of the leaf node they want to get value for.

GetWithIndex by key will return both the index and the value.

```golang
// GetWithIndex returns the index and value of the specified key if it exists, or nil
// and the next index, if it doesn't.
func (t *ImmutableTree) GetWithIndex(key []byte) (int64, []byte, error) {
	if t.root == nil {
		return 0, nil, nil
	}
	return t.root.get(t, key)
}
```

Get by index will return both the key and the value. The index is the index in the list of leaf nodes sorted lexicographically by key. The leftmost leaf has index 0. It's neighbor has index 1 and so on.

```golang
// GetByIndex gets the key and value at the specified index.
func (t *ImmutableTree) GetByIndex(index int64) (key []byte, value []byte) {
	if t.root == nil {
		return nil, nil
	}
	return t.root.getByIndex(t, index)
}
```

### Iterating

Iteration works by traversing from the root node. All iteration functions are provided a callback function `func(key, value []byte) (stop bool`). This callback is called on every leaf node's key and value in order of the iteration. If the callback returns true, then the iteration stops. Otherwise it continues.

Thus the callback is useful both as a way to run some logic on every key-value pair stored in the IAVL and as a way to dynamically stop the iteration.

The `IterateRange` functions allow users to iterate over a specific range and specify if the iteration should be in ascending or descending order.

The API's for Iteration functions are shown below.

```golang
// Iterate iterates over all keys of the tree, in order.
func (t *ImmutableTree) Iterate(fn func(key []byte, value []byte) bool) (stopped bool)

// IterateRange makes a callback for all nodes with key between start and end non-inclusive.
// If either are nil, then it is open on that side (nil, nil is the same as Iterate)
func (t *ImmutableTree) IterateRange(start, end []byte, ascending bool, fn func(key []byte, value []byte) bool) (stopped bool)

// IterateRangeInclusive makes a callback for all nodes with key between start and end inclusive.
// If either are nil, then it is open on that side (nil, nil is the same as Iterate)
func (t *ImmutableTree) IterateRangeInclusive(start, end []byte, ascending bool, fn func(key, value []byte, version int64) bool) (stopped bool)
```

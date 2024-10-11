# Export/Import

A single `ImmutableTree` (i.e. a single version) can be exported via `ImmutableTree.Export()`, returning an iterator over `ExportNode` items. These nodes can be imported into an empty `MutableTree` with `MutableTree.Import()` to recreate an identical tree. The structure of `ExportNode` is:

```go
type ExportNode struct {
	Key     []byte
	Value   []byte
	Version int64
	Height  int8
}
```

This is the minimum amount of data about nodes that can be exported, see the [node documentation](../node/node.md) for comparison. The other node attributes, such as `hash` and `size`, can be derived from this data. Both leaf nodes and inner nodes are exported, since `Version` is part of the hash and inner nodes have different versions than the leaf nodes with the same key.

The order of exported nodes is significant. Nodes are exported by depth-first post-order (LRN) tree traversal. Consider the following tree (with nodes in `key@version=value` format):

```
              d@3
            /     \
        c@3         e@3
       /   \       /   \
     b@3  c@3=3 d@2=4 e@3=5
   /    \
a@1=1  b@3=2

```

This would produce the following export:

```go
[]*ExportNode{
    {Key: []byte("a"), Value: []byte{1}, Version: 1, Height: 0},
    {Key: []byte("b"), Value: []byte{2}, Version: 3, Height: 0},
    {Key: []byte("b"), Value: nil,       Version: 3, Height: 1},
    {Key: []byte("c"), Value: []byte{3}, Version: 3, Height: 0},
    {Key: []byte("c"), Value: nil,       Version: 3, Height: 2},
    {Key: []byte("d"), Value: []byte{4}, Version: 2, Height: 0},
    {Key: []byte("e"), Value: []byte{5}, Version: 3, Height: 0},
    {Key: []byte("e"), Value: nil,       Version: 3, Height: 1},
    {Key: []byte("d"), Value: nil,       Version: 3, Height: 3},
}
```

When importing, the tree must be rebuilt in the same order, such that the missing attributes (e.g. `hash` and `size`) can be generated. This is possible because children are always given before their parents. We can therefore first generate the hash and size of the left and right leaf nodes, and then use these to recursively generate the hash and size of the parent.

One way to do this is to keep a stack of determined children, and then pop those children once we build their parent, which then becomes a new child on the stack. We know that we encounter a parent because its height is higher than the child or children on top of the stack. We need a stack because we may need to recursively build a right branch while holding an determined left child. For the above export this would look like the following (in `key:height=value` format):

```
| Stack           | Import node                                                 |
|-----------------|-------------------------------------------------------------|
|                 | {Key: []byte("a"), Value: []byte{1}, Version: 1, Height: 0} |
| a:0=1           | {Key: []byte("b"), Value: []byte{2}, Version: 3, Height: 0} |
| a:0=1,b:0=2     | {Key: []byte("b"), Value: nil,       Version: 3, Height: 1} |
| b:1             | {Key: []byte("c"), Value: []byte{3}, Version: 3, Height: 0} |
| b:1,c:0=3       | {Key: []byte("c"), Value: nil,       Version: 3, Height: 2} |
| c:2             | {Key: []byte("d"), Value: []byte{4}, Version: 2, Height: 0} |
| c:2,d:0=4       | {Key: []byte("e"), Value: []byte{5}, Version: 3, Height: 0} |
| c:2,d:0=4,e:0=5 | {Key: []byte("e"), Value: nil,       Version: 3, Height: 1} |
| c:2,e:1         | {Key: []byte("d"), Value: nil,       Version: 3, Height: 3} |
| d:3             |                                                             |
```

At the end, there will be a single node left on the stack, which is the root node of the tree.
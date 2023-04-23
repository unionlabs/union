# orderedcode

orderedcode provides a byte encoding of a sequence of typed items.
The resulting bytes can be lexicographically compared to yield the same
ordering as item-wise comparison on the original sequences.

This is particularly useful for specifying the order of rows in a database with
lexicographically ordered string keys, such as Bigtable.

See the package documentation in orderedcode.go for details and examples.

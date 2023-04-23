# btree

[![GoDoc](https://godoc.org/github.com/tidwall/btree?status.svg)](https://godoc.org/github.com/tidwall/btree)

An efficient [B-tree](https://en.wikipedia.org/wiki/B-tree) implementation in Go.

## Features

- Support for [Generics](#generics) (Go 1.18+).
- `Map` and `Set` types for ordered key-value maps and sets,
- Fast bulk loading for pre-ordered data using the `Load()` method.
- `Copy()` method with copy-on-write support.
- Thread-safe operations.
- [Path hinting](PATH_HINT.md) optimization for operations with nearby keys.

## Using

To start using this package, install Go and run:

```sh
$ go get github.com/tidwall/btree
```

## B-tree types

This package includes the following types of B-trees:

- [`btree.Map`](#btreemap):
A fast B-tree for storing ordered key value pairs.
Go 1.18+ 
- [`btree.Set`](#btreeset):
Like `Map`, but only for storing keys.
Go 1.18+
- [`btree.BTreeG`](#btreegeneric):
A feature-rich B-tree for storing data using a custom comparator.
Go 1.18+
- [`btree.BTree`](#btreebtree):
Like `BTreeG` but uses the `interface{}` type for data. Backwards compatible.
Go 1.16+

### btree.Map

```go
// Basic
Set(key, value)    // insert or replace an item
Get(key, value)    // get an existing item
Delete(key)        // delete an item
Len()              // return the number of items in the map

// Iteration
Scan(iter)         // scan items in ascending order
Reverse(iter)      // scan items in descending order
Ascend(key, iter)  // scan items in ascending order that are >= to key
Descend(key, iter) // scan items in descending order that are <= to key.
Iter()             // returns a read-only iterator for for-loops.

// Array-like operations
GetAt(index)       // returns the item at index
DeleteAt(index)    // deletes the item at index

// Bulk-loading
Load(key, value)   // load presorted items into tree
```

#### Example

```go
package main

import (
	"fmt"
	"github.com/tidwall/btree"
)

func main() {
	// create a map
	var users btree.Map[string, string]

	// add some users
	users.Set("user:4", "Andrea")
	users.Set("user:6", "Andy")
	users.Set("user:2", "Andy")
	users.Set("user:1", "Jane")
	users.Set("user:5", "Janet")
	users.Set("user:3", "Steve")

	// Iterate over the maps and print each user
	users.Scan(func(key, value string) bool {
		fmt.Printf("%s %s\n", key, value)
		return true
	})
	fmt.Printf("\n")

	// Delete a couple
	users.Delete("user:5")
	users.Delete("user:1")

	// print the map again
	users.Scan(func(key, value string) bool {
		fmt.Printf("%s %s\n", key, value)
		return true
	})
	fmt.Printf("\n")

	// Output:
	// user:1 Jane
	// user:2 Andy
	// user:3 Steve
	// user:4 Andrea
	// user:5 Janet
	// user:6 Andy
	//
	// user:2 Andy
	// user:3 Steve
	// user:4 Andrea
	// user:6 Andy
}
```

### btree.Set

```go
// Basic
Insert(key)        // insert an item
Contains(key)      // test if item exists
Delete(key)        // delete an item
Len()              // return the number of items in the set

// Iteration
Scan(iter)         // scan items in ascending order
Reverse(iter)      // scan items in descending order
Ascend(key, iter)  // scan items in ascending order that are >= to key
Descend(key, iter) // scan items in descending order that are <= to key.
Iter()             // returns a read-only iterator for for-loops.

// Array-like operations
GetAt(index)       // returns the item at index
DeleteAt(index)    // deletes the item at index

// Bulk-loading
Load(key)          // load presorted item into tree
```

#### Example

```go
package main

import (
	"fmt"
	"github.com/tidwall/btree"
)

func main() {
	// create a set
	var names btree.Set[string]

	// add some names
	names.Insert("Jane")
	names.Insert("Andrea")
	names.Insert("Steve")
	names.Insert("Andy")
	names.Insert("Janet")
	names.Insert("Andy")

	// Iterate over the maps and print each user
	names.Scan(func(key string) bool {
		fmt.Printf("%s\n", key)
		return true
	})
	fmt.Printf("\n")

	// Delete a couple
	names.Delete("Steve")
	names.Delete("Andy")

	// print the map again
	names.Scan(func(key string) bool {
		fmt.Printf("%s\n", key)
		return true
	})
	fmt.Printf("\n")

	// Output:
	// Andrea
	// Andy
	// Jane
	// Janet
	// Steve
	//
	// Andrea
	// Jane
	// Janet
}
```

### btree.BTreeG

```go
// Basic
Set(item)               // insert or replace an item
Get(item)               // get an existing item
Delete(item)            // delete an item
Len()                   // return the number of items in the btree

// Iteration
Scan(iter)              // scan items in ascending order
Reverse(iter)           // scan items in descending order
Ascend(key, iter)       // scan items in ascending order that are >= to key
Descend(key, iter)      // scan items in descending order that are <= to key.
Iter()                  // returns a read-only iterator for for-loops.

// Array-like operations
GetAt(index)            // returns the item at index
DeleteAt(index)         // deletes the item at index

// Bulk-loading
Load(item)              // load presorted items into tree

// Path hinting
SetHint(item, *hint)    // insert or replace an existing item
GetHint(item, *hint)    // get an existing item
DeleteHint(item, *hint) // delete an item

// Copy-on-write
Copy()                  // copy the btree
```

#### Example

```go
package main

import (
	"fmt"

	"github.com/tidwall/btree"
)

type Item struct {
	Key, Val string
}

// byKeys is a comparison function that compares item keys and returns true
// when a is less than b.
func byKeys(a, b Item) bool {
	return a.Key < b.Key
}

// byVals is a comparison function that compares item values and returns true
// when a is less than b.
func byVals(a, b Item) bool {
	if a.Val < b.Val {
		return true
	}
	if a.Val > b.Val {
		return false
	}
	// Both vals are equal so we should fall though
	// and let the key comparison take over.
	return byKeys(a, b)
}

func main() {
	// Create a tree for keys and a tree for values.
	// The "keys" tree will be sorted on the Keys field.
	// The "values" tree will be sorted on the Values field.
	keys := btree.NewBTreeG[Item](byKeys)
	vals := btree.NewBTreeG[Item](byVals)

	// Create some items.
	users := []Item{
		Item{Key: "user:1", Val: "Jane"},
		Item{Key: "user:2", Val: "Andy"},
		Item{Key: "user:3", Val: "Steve"},
		Item{Key: "user:4", Val: "Andrea"},
		Item{Key: "user:5", Val: "Janet"},
		Item{Key: "user:6", Val: "Andy"},
	}

	// Insert each user into both trees
	for _, user := range users {
		keys.Set(user)
		vals.Set(user)
	}

	// Iterate over each user in the key tree
	keys.Scan(func(item Item) bool {
		fmt.Printf("%s %s\n", item.Key, item.Val)
		return true
	})
	fmt.Printf("\n")

	// Iterate over each user in the val tree
	vals.Scan(func(item Item) bool {
		fmt.Printf("%s %s\n", item.Key, item.Val)
		return true
	})

	// Output:
	// user:1 Jane
	// user:2 Andy
	// user:3 Steve
	// user:4 Andrea
	// user:5 Janet
	// user:6 Andy
	//
	// user:4 Andrea
	// user:2 Andy
	// user:6 Andy
	// user:1 Jane
	// user:5 Janet
	// user:3 Steve
}
```

### btree.BTree

```go
// Basic
Set(item)               // insert or replace an item
Get(item)               // get an existing item
Delete(item)            // delete an item
Len()                   // return the number of items in the btree

// Iteration
Scan(iter)              // scan items in ascending order
Reverse(iter)           // scan items in descending order
Ascend(key, iter)       // scan items in ascending order that are >= to key
Descend(key, iter)      // scan items in descending order that are <= to key.
Iter()                  // returns a read-only iterator for for-loops.

// Array-like operations
GetAt(index)            // returns the item at index
DeleteAt(index)         // deletes the item at index

// Bulk-loading
Load(item)              // load presorted items into tree

// Path hinting
SetHint(item, *hint)    // insert or replace an existing item
GetHint(item, *hint)    // get an existing item
DeleteHint(item, *hint) // delete an item

// Copy-on-write
Copy()                  // copy the btree
```

#### Example

```go
package main

import (
	"fmt"

	"github.com/tidwall/btree"
)

type Item struct {
	Key, Val string
}

// byKeys is a comparison function that compares item keys and returns true
// when a is less than b.
func byKeys(a, b interface{}) bool {
	i1, i2 := a.(*Item), b.(*Item)
	return i1.Key < i2.Key
}

// byVals is a comparison function that compares item values and returns true
// when a is less than b.
func byVals(a, b interface{}) bool {
	i1, i2 := a.(*Item), b.(*Item)
	if i1.Val < i2.Val {
		return true
	}
	if i1.Val > i2.Val {
		return false
	}
	// Both vals are equal so we should fall though
	// and let the key comparison take over.
	return byKeys(a, b)
}

func main() {
	// Create a tree for keys and a tree for values.
	// The "keys" tree will be sorted on the Keys field.
	// The "values" tree will be sorted on the Values field.
	keys := btree.New(byKeys)
	vals := btree.New(byVals)

	// Create some items.
	users := []*Item{
		&Item{Key: "user:1", Val: "Jane"},
		&Item{Key: "user:2", Val: "Andy"},
		&Item{Key: "user:3", Val: "Steve"},
		&Item{Key: "user:4", Val: "Andrea"},
		&Item{Key: "user:5", Val: "Janet"},
		&Item{Key: "user:6", Val: "Andy"},
	}

	// Insert each user into both trees
	for _, user := range users {
		keys.Set(user)
		vals.Set(user)
	}

	// Iterate over each user in the key tree
	keys.Ascend(nil, func(item interface{}) bool {
		kvi := item.(*Item)
		fmt.Printf("%s %s\n", kvi.Key, kvi.Val)
		return true
	})

	fmt.Printf("\n")
	// Iterate over each user in the val tree
	vals.Ascend(nil, func(item interface{}) bool {
		kvi := item.(*Item)
		fmt.Printf("%s %s\n", kvi.Key, kvi.Val)
		return true
	})

	// Output:
	// user:1 Jane
	// user:2 Andy
	// user:3 Steve
	// user:4 Andrea
	// user:5 Janet
	// user:6 Andy
	//
	// user:4 Andrea
	// user:2 Andy
	// user:6 Andy
	// user:1 Jane
	// user:5 Janet
	// user:3 Steve
}
```

## Performance

See [tidwall/btree-benchmark](https://github.com/tidwall/btree-benchmark) for benchmark numbers.

## Contact

Josh Baker [@tidwall](http://twitter.com/tidwall)

## License

Source code is available under the MIT [License](/LICENSE).

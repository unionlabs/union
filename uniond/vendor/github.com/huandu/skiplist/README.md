# Skip List in Golang

[![Go](https://github.com/huandu/skiplist/workflows/Go/badge.svg)](https://github.com/huandu/skiplist/actions)
[![Go Doc](https://godoc.org/github.com/huandu/skiplist?status.svg)](https://pkg.go.dev/github.com/huandu/skiplist)
[![Go Report](https://goreportcard.com/badge/github.com/huandu/skiplist)](https://goreportcard.com/report/github.com/huandu/skiplist)
[![Coverage Status](https://coveralls.io/repos/github/huandu/skiplist/badge.svg?branch=master)](https://coveralls.io/github/huandu/skiplist?branch=master)

Skip list is an ordered map. See wikipedia page [skip list](http://en.wikipedia.org/wiki/Skip_list) to learn algorithm details about this data structure.

Highlights in this implementation:

- Built-in types can be used as key with predefined key types. See [Int](https://pkg.go.dev/github.com/huandu/skiplist#Int) and related constants as a sample.
- Support custom comparable function so that any type can be used as key.
- Key sort order can be changed quite easily. See [Reverse](https://pkg.go.dev/github.com/huandu/skiplist#Reverse) and [LessThanFunc](https://pkg.go.dev/github.com/huandu/skiplist#LessThanFunc).
- Rand source and max level can be changed per list. It can be useful in performance critical scenarios.

## Install

Install this package through `go get`.

```bash
    go get github.com/huandu/skiplist
```

## Basic Usage

Here is a quick sample.

```go
package main

import (
    "fmt"

    "github.com/huandu/skiplist"
)

func main() {
    // Create a skip list with int key.
    list := skiplist.New(skiplist.Int)

    // Add some values. Value can be anything.
    list.Set(12, "hello world")
    list.Set(34, 56)
    list.Set(78, 90.12)

    // Get element by index.
    elem := list.Get(34)                // Value is stored in elem.Value.
    fmt.Println(elem.Value)             // Output: 56
    next := elem.Next()                 // Get next element.
    prev := next.Prev()                 // Get previous element.
    fmt.Println(next.Value, prev.Value) // Output: 90.12    56

    // Or, directly get value just like a map
    val, ok := list.GetValue(34)
    fmt.Println(val, ok) // Output: 56  true

    // Find first elements with score greater or equal to key
    foundElem := list.Find(30)
    fmt.Println(foundElem.Key(), foundElem.Value) // Output: 34 56

    // Remove an element for key.
    list.Remove(34)
}
```

## Using `GreaterThanFunc` and `LessThanFunc`

Define your own `GreaterThanFunc` or `LessThanFunc` to use any custom type as the key in a skip list.

The signature of `GreaterThanFunc` are `LessThanFunc` are the same.
The only difference between them is that `LessThanFunc` reverses result returned by custom func
to make the list ordered by key in a reversed order.

```go
type T struct {
    Rad float64
}
list := New(GreaterThanFunc(func(k1, k2 interface{}) int {
    s1 := math.Sin(k1.(T).Rad)
    s2 := math.Sin(k2.(T).Rad)

    if s1 > s2 {
        return 1
    } else if s1 < s2 {
        return -1
    }

    return 0
}))
list.Set(T{math.Pi / 8}, "sin(π/8)")
list.Set(T{math.Pi / 2}, "sin(π/2)")
list.Set(T{math.Pi}, "sin(π)")

fmt.Println(list.Front().Value) // Output: sin(π)
fmt.Println(list.Back().Value)  // Output: sin(π/2)
```

## License

This library is licensed under MIT license. See LICENSE for details.

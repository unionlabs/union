//go:build jemalloc && !allocator
// +build jemalloc,!allocator

package main

import (
	"unsafe"

	"github.com/dgraph-io/ristretto/z"
)

func newNode(val int) *node {
	b := z.Calloc(nodeSz, "demo")
	n := (*node)(unsafe.Pointer(&b[0]))
	n.val = val
	return n
}

func freeNode(n *node) {
	buf := (*[z.MaxArrayLen]byte)(unsafe.Pointer(n))[:nodeSz:nodeSz]
	z.Free(buf)
}

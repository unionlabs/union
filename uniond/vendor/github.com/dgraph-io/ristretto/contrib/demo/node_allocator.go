//go:build jemalloc && allocator
// +build jemalloc,allocator

package main

import (
	"unsafe"

	"github.com/dgraph-io/ristretto/z"
)

// Defined in node.go.
func init() {
	alloc = z.NewAllocator(10<<20, "demo")
}

func newNode(val int) *node {
	// b := alloc.Allocate(nodeSz)
	b := alloc.AllocateAligned(nodeSz)
	n := (*node)(unsafe.Pointer(&b[0]))
	n.val = val
	alloc.Allocate(1) // Extra allocate just to demonstrate AllocateAligned is working as expected.
	return n
}

func freeNode(n *node) {
	// buf := (*[z.MaxArrayLen]byte)(unsafe.Pointer(n))[:nodeSz:nodeSz]
	// z.Free(buf)
}

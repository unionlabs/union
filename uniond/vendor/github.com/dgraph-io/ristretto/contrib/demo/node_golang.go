//go:build !jemalloc
// +build !jemalloc

package main

func newNode(val int) *node {
	return &node{val: val}
}

func freeNode(n *node) {}

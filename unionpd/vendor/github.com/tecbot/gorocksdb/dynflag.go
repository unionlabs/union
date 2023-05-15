//go:build !linux || !static
// +build !linux !static

package gorocksdb

// #cgo LDFLAGS: -lrocksdb -lstdc++ -lm -ldl
import "C"

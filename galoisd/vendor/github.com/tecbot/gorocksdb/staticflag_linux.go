// +build static

package gorocksdb

// #cgo LDFLAGS: -l:librocksdb.a -l:libstdc++.a -lm -ldl
import "C"

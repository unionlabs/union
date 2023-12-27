//go:build !grocksdb_no_link && grocksdb_clean_link

package grocksdb

// #cgo LDFLAGS: -lrocksdb -pthread -lstdc++ -ldl
import "C"

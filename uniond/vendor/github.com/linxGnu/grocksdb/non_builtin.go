//go:build !grocksdb_no_link && !grocksdb_clean_link

// The default link options, to customize it, you can try build tag `grocksdb_clean_link` for a cleaner set of flags,
// or `grocksdb_no_link` where you have full control through `CGO_LDFLAGS` environment variable.
package grocksdb

// #cgo LDFLAGS: -lrocksdb -pthread -lstdc++ -ldl -lm -lzstd -llz4 -lz -lsnappy
import "C"

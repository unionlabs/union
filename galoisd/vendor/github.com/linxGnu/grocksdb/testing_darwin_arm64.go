//go:build testing

package grocksdb

// #cgo CFLAGS: -I${SRCDIR}/dist/darwin_arm64/include
// #cgo CXXFLAGS: -I${SRCDIR}/dist/darwin_arm64/include
// #cgo LDFLAGS: -L${SRCDIR}/dist/darwin_arm64/lib -lrocksdb -pthread -lstdc++ -ldl -lm -lzstd -llz4 -lz -lsnappy
import "C"

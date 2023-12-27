//go:build external_libzstd
// +build external_libzstd

package zstd

// #cgo CFLAGS: -DUSE_EXTERNAL_ZSTD
// #cgo pkg-config: libzstd
/*
#include<zstd.h>
#if ZSTD_VERSION_NUMBER < 10400
#error "ZSTD version >= 1.4 is required"
#endif
*/
import "C"

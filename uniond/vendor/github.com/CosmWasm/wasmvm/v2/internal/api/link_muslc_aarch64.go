//go:build linux && muslc && arm64 && !sys_wasmvm

package api

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lwasmvm_muslc.aarch64
import "C"

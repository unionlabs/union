//go:build darwin && static_wasm && !sys_wasmvm

package api

// #cgo LDFLAGS: -L${SRCDIR} -lwasmvmstatic_darwin
import "C"

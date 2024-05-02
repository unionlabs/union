package api

/*
#include "bindings.h"
*/
import "C"

func LibwasmvmVersion() (string, error) {
	version_ptr, err := C.version_str()
	if err != nil {
		return "", err
	}
	// For C.GoString documentation see https://pkg.go.dev/cmd/cgo and
	// https://gist.github.com/helinwang/2c7bd2867ea5110f70e6431a7c80cd9b
	version_copy := C.GoString(version_ptr)
	return version_copy, nil
}

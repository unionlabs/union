package blst

import (
    "path/filepath"
    "runtime"
)

var SrcRoot string

func init() {
    if _, self, _, ok := runtime.Caller(0); ok {
        SrcRoot = filepath.Dir(filepath.Dir(self))
    }
}

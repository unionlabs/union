//go:build !go1.20

package sentry

import (
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestFilterCompilerGeneratedSymbols(t *testing.T) {
	tests := []struct {
		symbol              string
		expectedPackageName string
	}{
		{"type..eq.[9]debug/elf.intName", ""},
		{"type..hash.debug/elf.ProgHeader", ""},
		{"type..eq.runtime._panic", ""},
		{"type..hash.struct { runtime.gList; runtime.n int32 }", ""},
		{"go.(*struct { sync.Mutex; math/big.table [64]math/big", ""},
		{"github.com/getsentry/sentry-go.Test.func2.1.1", "github.com/getsentry/sentry-go"},
	}

	for _, tt := range tests {
		t.Run(tt.symbol, func(t *testing.T) {
			packageName := packageName(tt.symbol)
			if diff := cmp.Diff(tt.expectedPackageName, packageName); diff != "" {
				t.Errorf("Package name mismatch (-want +got):\n%s", diff)
			}
		})
	}
}

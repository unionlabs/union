//go:build !appengine
// +build !appengine

package xxhash

import (
	"os/exec"
	"sort"
	"strings"
	"testing"
)

func TestStringAllocs(t *testing.T) {
	longStr := strings.Repeat("a", 1000)
	t.Run("Sum64String", func(t *testing.T) {
		testAllocs(t, func() {
			sink = Sum64String(longStr)
		})
	})
	t.Run("Digest.WriteString", func(t *testing.T) {
		testAllocs(t, func() {
			d := New()
			d.WriteString(longStr)
			sink = d.Sum64()
		})
	})
}

// This test is inspired by the Go runtime tests in https://go.dev/cl/57410.
// It asserts that certain important functions may be inlined.
func TestInlining(t *testing.T) {
	funcs := map[string]struct{}{
		"Sum64String":           {},
		"(*Digest).WriteString": {},
	}

	cmd := exec.Command("go", "test", "-gcflags=-m", "-run", "xxxx")
	out, err := cmd.CombinedOutput()
	if err != nil {
		t.Log(string(out))
		t.Fatal(err)
	}

	for _, line := range strings.Split(string(out), "\n") {
		parts := strings.Split(line, ": can inline")
		if len(parts) < 2 {
			continue
		}
		delete(funcs, strings.TrimSpace(parts[1]))
	}

	var failed []string
	for fn := range funcs {
		failed = append(failed, fn)
	}
	sort.Strings(failed)
	for _, fn := range failed {
		t.Errorf("function %s not inlined", fn)
	}
}

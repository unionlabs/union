// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package constraints

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"runtime"
	"testing"
)

type (
	testSigned[T Signed]     struct{ f T }
	testUnsigned[T Unsigned] struct{ f T }
	testInteger[T Integer]   struct{ f T }
	testFloat[T Float]       struct{ f T }
	testComplex[T Complex]   struct{ f T }
	testOrdered[T Ordered]   struct{ f T }
)

// TestTypes passes if it compiles.
type TestTypes struct {
	_ testSigned[int]
	_ testSigned[int64]
	_ testUnsigned[uint]
	_ testUnsigned[uintptr]
	_ testInteger[int8]
	_ testInteger[uint8]
	_ testInteger[uintptr]
	_ testFloat[float32]
	_ testComplex[complex64]
	_ testOrdered[int]
	_ testOrdered[float64]
	_ testOrdered[string]
}

var prolog = []byte(`
package constrainttest

import "golang.org/x/exp/constraints"

type (
	testSigned[T constraints.Signed]     struct{ f T }
	testUnsigned[T constraints.Unsigned] struct{ f T }
	testInteger[T constraints.Integer]   struct{ f T }
	testFloat[T constraints.Float]       struct{ f T }
	testComplex[T constraints.Complex]   struct{ f T }
	testOrdered[T constraints.Ordered]   struct{ f T }
)
`)

func TestFailure(t *testing.T) {
	switch runtime.GOOS {
	case "android", "js", "ios":
		t.Skipf("can't run go tool on %s", runtime.GOOS)
	}

	var exeSuffix string
	if runtime.GOOS == "windows" {
		exeSuffix = ".exe"
	}
	gocmd := filepath.Join(runtime.GOROOT(), "bin", "go"+exeSuffix)
	if _, err := os.Stat(gocmd); err != nil {
		t.Skipf("skipping because can't stat %s: %v", gocmd, err)
	}

	tmpdir := t.TempDir()

	cwd, err := os.Getwd()
	if err != nil {
		t.Fatal(err)
	}
	// This package is golang.org/x/exp/constraints, so the root of the x/exp
	// module is the parent directory of the directory in which this test runs.
	expModDir := filepath.Dir(cwd)

	modFile := fmt.Sprintf(`module constraintest

go 1.18

replace golang.org/x/exp => %s
`, expModDir)
	if err := os.WriteFile(filepath.Join(tmpdir, "go.mod"), []byte(modFile), 0666); err != nil {
		t.Fatal(err)
	}

	// Write the prolog as its own file so that 'go mod tidy' has something to inspect.
	// This will ensure that the go.mod and go.sum files include any dependencies
	// needed by the constraints package (which should just be some version of
	// x/exp itself).
	if err := os.WriteFile(filepath.Join(tmpdir, "prolog.go"), []byte(prolog), 0666); err != nil {
		t.Fatal(err)
	}

	tidyCmd := exec.Command(gocmd, "mod", "tidy")
	tidyCmd.Dir = tmpdir
	tidyCmd.Env = append(os.Environ(), "PWD="+tmpdir)
	if out, err := tidyCmd.CombinedOutput(); err != nil {
		t.Fatalf("%v: %v\n%s", tidyCmd, err, out)
	} else {
		t.Logf("%v:\n%s", tidyCmd, out)
	}

	// Test for types that should not satisfy a constraint.
	// For each pair of constraint and type, write a Go file
	//     var V constraint[type]
	// For example,
	//     var V testSigned[uint]
	// This should not compile, as testSigned (above) uses
	// constraints.Signed, and uint does not satisfy that constraint.
	// Therefore, the build of that code should fail.
	for i, test := range []struct {
		constraint, typ string
	}{
		{"testSigned", "uint"},
		{"testUnsigned", "int"},
		{"testInteger", "float32"},
		{"testFloat", "int8"},
		{"testComplex", "float64"},
		{"testOrdered", "bool"},
	} {
		i := i
		test := test
		t.Run(fmt.Sprintf("%s %d", test.constraint, i), func(t *testing.T) {
			t.Parallel()
			name := fmt.Sprintf("go%d.go", i)
			f, err := os.Create(filepath.Join(tmpdir, name))
			if err != nil {
				t.Fatal(err)
			}
			if _, err := f.Write(prolog); err != nil {
				t.Fatal(err)
			}
			if _, err := fmt.Fprintf(f, "var V %s[%s]\n", test.constraint, test.typ); err != nil {
				t.Fatal(err)
			}
			if err := f.Close(); err != nil {
				t.Fatal(err)
			}
			cmd := exec.Command(gocmd, "build", name)
			cmd.Dir = tmpdir
			if out, err := cmd.CombinedOutput(); err == nil {
				t.Error("build succeeded, but expected to fail")
			} else if len(out) > 0 {
				t.Logf("%s", out)
				if !wantRx.Match(out) {
					t.Errorf("output does not match %q", wantRx)
				}
			} else {
				t.Error("no error output, expected something")
			}
		})
	}
}

var wantRx = regexp.MustCompile("does not (implement|satisfy)")

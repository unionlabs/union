// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package lint

import (
	"bytes"
	"fmt"
	"go/build"
	"os/exec"
	"regexp"
	"runtime"
	"strings"
	"testing"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/ghemawat/stream"
	"github.com/stretchr/testify/require"
)

const (
	cmdGo       = "go"
	golint      = "golang.org/x/lint/golint@6edffad5e6160f5949cdefc81710b2706fbcd4f6"
	staticcheck = "honnef.co/go/tools/cmd/staticcheck@2023.1"
	crlfmt      = "github.com/cockroachdb/crlfmt@44a36ec7"
)

func dirCmd(t *testing.T, dir string, name string, args ...string) stream.Filter {
	cmd := exec.Command(name, args...)
	cmd.Dir = dir
	out, err := cmd.CombinedOutput()
	switch err.(type) {
	case nil:
	case *exec.ExitError:
		// Non-zero exit is expected.
	default:
		require.NoError(t, err)
	}
	return stream.ReadLines(bytes.NewReader(out))
}

func ignoreGoMod() stream.Filter {
	return stream.GrepNot(`^go: (finding|extracting|downloading)`)
}

func TestLint(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("lint checks skipped on Windows")
	}
	if invariants.RaceEnabled {
		// We are not interested in race-testing the linters themselves.
		t.Skip("lint checks skipped on race builds")
	}

	const root = "github.com/cockroachdb/pebble"

	pkg, err := build.Import(root, "../..", 0)
	require.NoError(t, err)

	var pkgs []string
	if err := stream.ForEach(
		stream.Sequence(
			dirCmd(t, pkg.Dir, "go", "list", "./..."),
			ignoreGoMod(),
		), func(s string) {
			pkgs = append(pkgs, s)
		}); err != nil {
		require.NoError(t, err)
	}

	t.Run("TestGolint", func(t *testing.T) {
		t.Parallel()

		args := []string{"run", golint}
		args = append(args, pkgs...)

		// This is overkill right now, but provides a structure for filtering out
		// lint errors we don't care about.
		if err := stream.ForEach(
			stream.Sequence(
				dirCmd(t, pkg.Dir, cmdGo, args...),
				stream.GrepNot("go: downloading"),
			), func(s string) {
				t.Errorf("\n%s", s)
			}); err != nil {
			t.Error(err)
		}
	})

	t.Run("TestStaticcheck", func(t *testing.T) {
		t.Parallel()

		args := []string{"run", staticcheck}
		args = append(args, pkgs...)

		if err := stream.ForEach(
			stream.Sequence(
				dirCmd(t, pkg.Dir, cmdGo, args...),
				stream.GrepNot("go: downloading"),
			), func(s string) {
				t.Errorf("\n%s", s)
			}); err != nil {
			t.Error(err)
		}
	})

	t.Run("TestGoVet", func(t *testing.T) {
		t.Parallel()

		if err := stream.ForEach(
			stream.Sequence(
				dirCmd(t, pkg.Dir, "go", "vet", "-all", "./..."),
				stream.GrepNot(`^#`), // ignore comment lines
				ignoreGoMod(),
			), func(s string) {
				t.Errorf("\n%s", s)
			}); err != nil {
			t.Error(err)
		}
	})

	t.Run("TestFmtErrorf", func(t *testing.T) {
		t.Parallel()

		if err := stream.ForEach(
			dirCmd(t, pkg.Dir, "git", "grep", "fmt\\.Errorf("),
			func(s string) {
				t.Errorf("\n%s <- please use \"errors.Errorf\" instead", s)
			}); err != nil {
			t.Error(err)
		}
	})

	t.Run("TestOSIsErr", func(t *testing.T) {
		t.Parallel()

		if err := stream.ForEach(
			dirCmd(t, pkg.Dir, "git", "grep", "os\\.Is"),
			func(s string) {
				t.Errorf("\n%s <- please use the \"oserror\" equivalent instead", s)
			}); err != nil {
			t.Error(err)
		}
	})

	t.Run("TestSetFinalizer", func(t *testing.T) {
		t.Parallel()

		if err := stream.ForEach(
			stream.Sequence(
				dirCmd(t, pkg.Dir, "git", "grep", "-B1", "runtime\\.SetFinalizer("),
				lintIgnore("lint:ignore SetFinalizer"),
				stream.GrepNot(`^internal/invariants/finalizer_on.go`),
			), func(s string) {
				t.Errorf("\n%s <- please use the \"invariants.SetFinalizer\" equivalent instead", s)
			}); err != nil {
			t.Error(err)
		}
	})

	// Disallow "raw" atomics; wrappers like atomic.Int32 provide much better
	// safety and alignment guarantees.
	t.Run("TestRawAtomics", func(t *testing.T) {
		t.Parallel()
		if err := stream.ForEach(
			stream.Sequence(
				dirCmd(t, pkg.Dir, "git", "grep", `atomic\.\(Load\|Store\|Add\|Swap\|Compare\)`),
				lintIgnore("lint:ignore RawAtomics"),
			), func(s string) {
				t.Errorf("\n%s <- please use atomic wrappers (like atomic.Int32) instead", s)
			}); err != nil {
			t.Error(err)
		}
	})

	t.Run("TestForbiddenImports", func(t *testing.T) {
		t.Parallel()

		// Forbidden-import-pkg -> permitted-replacement-pkg
		forbiddenImports := map[string]string{
			"errors":     "github.com/cockroachdb/errors",
			"pkg/errors": "github.com/cockroachdb/errors",
		}

		// grepBuf creates a grep string that matches any forbidden import pkgs.
		var grepBuf bytes.Buffer
		grepBuf.WriteByte('(')
		for forbiddenPkg := range forbiddenImports {
			grepBuf.WriteByte('|')
			grepBuf.WriteString(regexp.QuoteMeta(forbiddenPkg))
		}
		grepBuf.WriteString(")$")

		filter := stream.FilterFunc(func(arg stream.Arg) error {
			for _, path := range pkgs {
				buildContext := build.Default
				buildContext.UseAllFiles = true
				importPkg, err := buildContext.Import(path, pkg.Dir, 0)
				if _, ok := err.(*build.MultiplePackageError); ok {
					buildContext.UseAllFiles = false
					importPkg, err = buildContext.Import(path, pkg.Dir, 0)
				}

				switch err.(type) {
				case nil:
					for _, s := range importPkg.Imports {
						arg.Out <- importPkg.ImportPath + ": " + s
					}
					for _, s := range importPkg.TestImports {
						arg.Out <- importPkg.ImportPath + ": " + s
					}
					for _, s := range importPkg.XTestImports {
						arg.Out <- importPkg.ImportPath + ": " + s
					}
				case *build.NoGoError:
				default:
					return errors.Wrapf(err, "error loading package %s", path)
				}
			}
			return nil
		})
		if err := stream.ForEach(stream.Sequence(
			filter,
			stream.Sort(),
			stream.Uniq(),
			stream.Grep(grepBuf.String()),
		), func(s string) {
			pkgStr := strings.Split(s, ": ")
			importedPkg := pkgStr[1]

			// Test that a disallowed package is not imported.
			if replPkg, ok := forbiddenImports[importedPkg]; ok {
				t.Errorf("\n%s <- please use %q instead of %q", s, replPkg, importedPkg)
			}
		}); err != nil {
			t.Error(err)
		}
	})

	t.Run("TestCrlfmt", func(t *testing.T) {
		t.Parallel()

		args := []string{"run", crlfmt, "-fast", "-tab", "2", "."}
		var buf bytes.Buffer
		if err := stream.ForEach(
			stream.Sequence(
				dirCmd(t, pkg.Dir, cmdGo, args...),
				stream.GrepNot("go: downloading"),
			),
			func(s string) {
				fmt.Fprintln(&buf, s)
			}); err != nil {
			t.Error(err)
		}
		errs := buf.String()
		if len(errs) > 0 {
			t.Errorf("\n%s", errs)
		}

		if t.Failed() {
			reWriteCmd := []string{crlfmt, "-w"}
			reWriteCmd = append(reWriteCmd, args...)
			t.Logf("run the following to fix your formatting:\n"+
				"\n%s\n\n"+
				"Don't forget to add amend the result to the correct commits.",
				strings.Join(reWriteCmd, " "),
			)
		}
	})
}

// lintIgnore is a stream.FilterFunc that filters out lines that are preceded by
// the given ignore directive. The function assumes the input stream receives a
// sequence of strings that are to be considered as pairs. If the first string
// in the sequence matches the ignore directive, the following string is
// dropped, else it is emitted.
//
// For example, given the sequence "foo", "bar", "baz", "bam", and an ignore
// directive "foo", the sequence "baz", "bam" would be emitted. If the directive
// was "baz", the sequence "foo", "bar" would be emitted.
func lintIgnore(ignore string) stream.FilterFunc {
	return func(arg stream.Arg) error {
		var prev string
		var i int
		for s := range arg.In {
			if i%2 == 0 {
				// Fist string in the pair is used as the filter. Store it.
				prev = s
			} else {
				// Second string is emitted only if it _does not_ match the directive.
				if !strings.Contains(prev, ignore) {
					arg.Out <- s
				}
			}
			i++
		}
		return nil
	}
}

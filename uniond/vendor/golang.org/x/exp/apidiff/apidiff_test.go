package apidiff

import (
	"bufio"
	"fmt"
	"go/types"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"sort"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
	"golang.org/x/tools/go/packages"
	"golang.org/x/tools/go/packages/packagestest"
)

func TestModuleChanges(t *testing.T) {
	packagestest.TestAll(t, testModuleChanges)
}

func testModuleChanges(t *testing.T, x packagestest.Exporter) {
	e := packagestest.Export(t, x, []packagestest.Module{
		{
			Name: "example.com/moda",
			Files: map[string]any{
				"foo/foo.go":     "package foo\n\nconst Version = 1",
				"foo/baz/baz.go": "package baz",
			},
		},
		{
			Name: "example.com/modb",
			Files: map[string]any{
				"foo/foo.go": "package foo\n\nconst Version = 2\nconst Other = 1",
				"bar/bar.go": "package bar",
			},
		},
	})
	defer e.Cleanup()

	a, err := loadModule(t, e.Config, "example.com/moda")
	if err != nil {
		t.Fatal(err)
	}
	b, err := loadModule(t, e.Config, "example.com/modb")
	if err != nil {
		t.Fatal(err)
	}
	report := ModuleChanges(a, b)
	if len(report.Changes) == 0 {
		t.Fatal("expected some changes, but got none")
	}
	wanti := []string{
		"./foo.Version: value changed from 1 to 2",
		"package example.com/moda/foo/baz: removed",
	}
	sort.Strings(wanti)

	got := report.messages(false)
	sort.Strings(got)

	if diff := cmp.Diff(wanti, got); diff != "" {
		t.Errorf("incompatibles: mismatch (-want, +got)\n%s", diff)
	}

	wantc := []string{
		"./foo.Other: added",
		"package example.com/modb/bar: added",
	}
	sort.Strings(wantc)

	got = report.messages(true)
	sort.Strings(got)

	if diff := cmp.Diff(wantc, got); diff != "" {
		t.Errorf("compatibles: mismatch (-want, +got)\n%s", diff)
	}
}

func TestChanges(t *testing.T) {
	testfiles, err := filepath.Glob(filepath.Join("testdata", "*.go"))
	if err != nil {
		t.Fatal(err)
	}
	for _, testfile := range testfiles {
		name := strings.TrimSuffix(filepath.Base(testfile), ".go")
		t.Run(name, func(t *testing.T) {
			dir := filepath.Join(t.TempDir(), "go")
			wanti, wantc := splitIntoPackages(t, testfile, dir)
			sort.Strings(wanti)
			sort.Strings(wantc)

			oldpkg, err := loadPackage(t, "apidiff/old", dir)
			if err != nil {
				t.Fatal(err)
			}
			newpkg, err := loadPackage(t, "apidiff/new", dir)
			if err != nil {
				t.Fatal(err)
			}

			report := Changes(oldpkg.Types, newpkg.Types)

			got := report.messages(false)
			if diff := cmp.Diff(wanti, got); diff != "" {
				t.Errorf("incompatibles: mismatch (-want, +got)\n%s", diff)
			}
			got = report.messages(true)
			if diff := cmp.Diff(wantc, got); diff != "" {
				t.Errorf("compatibles: mismatch (-want, +got)\n%s", diff)
			}
		})
	}
}

func splitIntoPackages(t *testing.T, file, dir string) (incompatibles, compatibles []string) {
	// Read the input file line by line.
	// Write a line into the old or new package,
	// dependent on comments.
	// Also collect expected messages.
	f, err := os.Open(file)
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()

	if err := os.MkdirAll(filepath.Join(dir, "src", "apidiff"), 0700); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dir, "src", "apidiff", "go.mod"), []byte("module apidiff\ngo 1.18\n"), 0600); err != nil {
		t.Fatal(err)
	}

	oldd := filepath.Join(dir, "src/apidiff/old")
	newd := filepath.Join(dir, "src/apidiff/new")
	if err := os.MkdirAll(oldd, 0700); err != nil {
		t.Fatal(err)
	}
	if err := os.Mkdir(newd, 0700); err != nil && !os.IsExist(err) {
		t.Fatal(err)
	}

	oldf, err := os.Create(filepath.Join(oldd, "old.go"))
	if err != nil {
		t.Fatal(err)
	}
	defer func() {
		if err := oldf.Close(); err != nil {
			t.Fatal(err)
		}
	}()

	newf, err := os.Create(filepath.Join(newd, "new.go"))
	if err != nil {
		t.Fatal(err)
	}
	defer func() {
		if err := newf.Close(); err != nil {
			t.Fatal(err)
		}
	}()

	wl := func(f *os.File, line string) {
		if _, err := fmt.Fprintln(f, line); err != nil {
			t.Fatal(err)
		}
	}
	writeBoth := func(line string) { wl(oldf, line); wl(newf, line) }
	writeln := writeBoth
	s := bufio.NewScanner(f)
	for s.Scan() {
		line := s.Text()
		tl := strings.TrimSpace(line)
		switch {
		case tl == "// old":
			writeln = func(line string) { wl(oldf, line) }
		case tl == "// new":
			writeln = func(line string) { wl(newf, line) }
		case tl == "// both":
			writeln = writeBoth
		case strings.HasPrefix(tl, "// i "):
			incompatibles = append(incompatibles, strings.TrimSpace(tl[4:]))
		case strings.HasPrefix(tl, "// c "):
			compatibles = append(compatibles, strings.TrimSpace(tl[4:]))
		default:
			writeln(line)
		}
	}
	if s.Err() != nil {
		t.Fatal(s.Err())
	}
	return
}

// Copied from cmd/apidiff/main.go.
func loadModule(t *testing.T, cfg *packages.Config, modulePath string) (*Module, error) {
	needsGoPackages(t)

	cfg.Mode = cfg.Mode | packages.LoadTypes
	loaded, err := packages.Load(cfg, fmt.Sprintf("%s/...", modulePath))
	if err != nil {
		return nil, err
	}
	if len(loaded) == 0 {
		return nil, fmt.Errorf("found no packages for module %s", modulePath)
	}
	var tpkgs []*types.Package
	for _, p := range loaded {
		if len(p.Errors) > 0 {
			// TODO: use errors.Join once Go 1.21 is released.
			return nil, p.Errors[0]
		}
		tpkgs = append(tpkgs, p.Types)
	}

	return &Module{Path: modulePath, Packages: tpkgs}, nil
}

func loadPackage(t *testing.T, importPath, goPath string) (*packages.Package, error) {
	needsGoPackages(t)

	cfg := &packages.Config{
		Mode: packages.LoadTypes,
	}
	if goPath != "" {
		cfg.Env = append(os.Environ(), "GOPATH="+goPath)
		cfg.Dir = filepath.Join(goPath, "src", filepath.FromSlash(importPath))
	}
	pkgs, err := packages.Load(cfg, importPath)
	if err != nil {
		return nil, err
	}
	if len(pkgs[0].Errors) > 0 {
		return nil, pkgs[0].Errors[0]
	}
	return pkgs[0], nil
}

func TestExportedFields(t *testing.T) {
	pkg, err := loadPackage(t, "golang.org/x/exp/apidiff/testdata/exported_fields", "")
	if err != nil {
		t.Fatal(err)
	}
	typeof := func(name string) types.Type {
		return pkg.Types.Scope().Lookup(name).Type()
	}

	s := typeof("S")
	su := s.(*types.Named).Underlying().(*types.Struct)

	ef := exportedSelectableFields(su)
	wants := []struct {
		name string
		typ  types.Type
	}{
		{"A1", typeof("A1")},
		{"D", types.Typ[types.Bool]},
		{"E", types.Typ[types.Int]},
		{"F", typeof("F")},
		{"S", types.NewPointer(s)},
	}

	if got, want := len(ef), len(wants); got != want {
		t.Errorf("got %d fields, want %d\n%+v", got, want, ef)
	}
	for _, w := range wants {
		if got := ef[w.name]; got != nil && !types.Identical(got.Type(), w.typ) {
			t.Errorf("%s: got %v, want %v", w.name, got.Type(), w.typ)
		}
	}
}

// needsGoPackages skips t if the go/packages driver (or 'go' tool) implied by
// the current process environment is not present in the path.
//
// Copied and adapted from golang.org/x/tools/internal/testenv.
func needsGoPackages(t *testing.T) {
	t.Helper()

	tool := os.Getenv("GOPACKAGESDRIVER")
	switch tool {
	case "off":
		// "off" forces go/packages to use the go command.
		tool = "go"
	case "":
		if _, err := exec.LookPath("gopackagesdriver"); err == nil {
			tool = "gopackagesdriver"
		} else {
			tool = "go"
		}
	}

	needsTool(t, tool)
}

// needsTool skips t if the named tool is not present in the path.
//
// Copied and adapted from golang.org/x/tools/internal/testenv.
func needsTool(t *testing.T, tool string) {
	_, err := exec.LookPath(tool)
	if err == nil {
		return
	}

	t.Helper()
	if allowMissingTool(tool) {
		t.Skipf("skipping because %s tool not available: %v", tool, err)
	} else {
		t.Fatalf("%s tool not available: %v", tool, err)
	}
}

func allowMissingTool(tool string) bool {
	if runtime.GOOS == "android" {
		// Android builds generally run tests on a separate machine from the build,
		// so don't expect any external tools to be available.
		return true
	}

	if tool == "go" && os.Getenv("GO_BUILDER_NAME") == "illumos-amd64-joyent" {
		// Work around a misconfigured builder (see https://golang.org/issue/33950).
		return true
	}

	// If a developer is actively working on this test, we expect them to have all
	// of its dependencies installed. However, if it's just a dependency of some
	// other module (for example, being run via 'go test all'), we should be more
	// tolerant of unusual environments.
	return !packageMainIsDevel()
}

// packageMainIsDevel reports whether the module containing package main
// is a development version (if module information is available).
//
// Builds in GOPATH mode and builds that lack module information are assumed to
// be development versions.
var packageMainIsDevel = func() bool { return true }

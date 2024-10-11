// Copyright 2017 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package pipeline

import (
	"bufio"
	"bytes"
	"encoding/json"
	"flag"
	"fmt"
	"go/build"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"reflect"
	"runtime"
	"strings"
	"testing"

	"golang.org/x/text/language"
)

var genFiles = flag.Bool("gen", false, "generate output files instead of comparing")

// setHelper is testing.T.Helper on Go 1.9+, overridden by go19_test.go.
var setHelper = func(t *testing.T) {}

func TestFullCycle(t *testing.T) {
	if runtime.GOOS == "android" {
		t.Skip("cannot load outside packages on android")
	}
	if b := os.Getenv("GO_BUILDER_NAME"); b == "plan9-arm" {
		t.Skipf("skipping: test frequently times out on %s", b)
	}
	if _, err := exec.LookPath("go"); err != nil {
		t.Skipf("skipping because 'go' command is unavailable: %v", err)
	}

	GOPATH, err := os.MkdirTemp("", "pipeline_test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(GOPATH)
	testdata := filepath.Join(GOPATH, "src", "testdata")

	// Copy the testdata contents into a new module.
	copyTestdata(t, testdata)
	initTestdataModule(t, testdata)

	// Several places hard-code the use of build.Default.
	// Adjust it to match the test's temporary GOPATH.
	defer func(prev string) { build.Default.GOPATH = prev }(build.Default.GOPATH)
	build.Default.GOPATH = GOPATH + string(filepath.ListSeparator) + build.Default.GOPATH
	if wd := reflect.ValueOf(&build.Default).Elem().FieldByName("WorkingDir"); wd.IsValid() {
		defer func(prev string) { wd.SetString(prev) }(wd.String())
		wd.SetString(testdata)
	}

	// To work around https://golang.org/issue/34860, execute the commands
	// that (transitively) use go/build in the working directory of the
	// corresponding module.
	wd, _ := os.Getwd()
	defer os.Chdir(wd)

	dirs, err := os.ReadDir(testdata)
	if err != nil {
		t.Fatal(err)
	}
	for _, f := range dirs {
		if !f.IsDir() {
			continue
		}
		t.Run(f.Name(), func(t *testing.T) {
			chk := func(t *testing.T, err error) {
				setHelper(t)
				if err != nil {
					t.Fatal(err)
				}
			}
			dir := filepath.Join(testdata, f.Name())
			pkgPath := "testdata/" + f.Name()
			config := Config{
				SourceLanguage: language.AmericanEnglish,
				Packages:       []string{pkgPath},
				Dir:            filepath.Join(dir, "locales"),
				GenFile:        "catalog_gen.go",
				GenPackage:     pkgPath,
			}

			os.Chdir(dir)

			// TODO: load config if available.
			s, err := Extract(&config)
			chk(t, err)
			chk(t, s.Import())
			chk(t, s.Merge())
			// TODO:
			//  for range s.Config.Actions {
			//  	//  TODO: do the actions.
			//  }
			chk(t, s.Export())
			chk(t, s.Generate())

			os.Chdir(wd)

			writeJSON(t, filepath.Join(dir, "extracted.gotext.json"), s.Extracted)
			checkOutput(t, dir, f.Name())
		})
	}
}

func copyTestdata(t *testing.T, dst string) {
	err := filepath.Walk("testdata", func(p string, f os.FileInfo, err error) error {
		if p == "testdata" || strings.HasSuffix(p, ".want") {
			return nil
		}

		rel := strings.TrimPrefix(p, "testdata"+string(filepath.Separator))
		if f.IsDir() {
			return os.MkdirAll(filepath.Join(dst, rel), 0755)
		}

		data, err := os.ReadFile(p)
		if err != nil {
			return err
		}
		return os.WriteFile(filepath.Join(dst, rel), data, 0644)
	})
	if err != nil {
		t.Fatal(err)
	}
}

func initTestdataModule(t *testing.T, dst string) {
	xTextDir, err := filepath.Abs("../..")
	if err != nil {
		t.Fatal(err)
	}

	goMod := fmt.Sprintf(`module testdata

replace golang.org/x/text => %s
`, xTextDir)
	if err := os.WriteFile(filepath.Join(dst, "go.mod"), []byte(goMod), 0644); err != nil {
		t.Fatal(err)
	}

	// Copy in the checksums from the parent module so that we won't
	// need to re-fetch them from the checksum database.
	data, err := os.ReadFile(filepath.Join(xTextDir, "go.sum"))
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dst, "go.sum"), data, 0644); err != nil {
		t.Fatal(err)
	}

	// We've added a replacement for the parent version of x/text,
	// but now we need to populate the correct version.
	// (We can't just replace the zero-version because x/text
	// may indirectly depend on some nonzero version of itself.)
	//
	// We use 'go get' instead of 'go mod tidy' to avoid the old-release
	// compatibility check when graph pruning is enabled, and to avoid doing
	// more work than necessary for test dependencies of imported packages
	// (we're not going to run those tests here anyway).
	//
	// We 'go get' the packages in the testdata module — not specific dependencies
	// of those packages — so that they will resolve to whatever version is
	// already required in the (replaced) x/text go.mod file.

	getCmd := exec.Command("go", "get", "-d", "./...")
	getCmd.Dir = dst
	getCmd.Env = append(os.Environ(), "PWD="+dst, "GOPROXY=off", "GOCACHE=off")
	if out, err := getCmd.CombinedOutput(); err != nil {
		t.Logf("%s", out)
		t.Fatal(err)
	}
}

func checkOutput(t *testing.T, gen string, testdataDir string) {
	err := filepath.Walk(gen, func(gotFile string, f os.FileInfo, err error) error {
		if f.IsDir() {
			return nil
		}
		rel := strings.TrimPrefix(gotFile, gen+string(filepath.Separator))

		wantFile := filepath.Join("testdata", testdataDir, rel+".want")
		if _, err := os.Stat(wantFile); os.IsNotExist(err) {
			return nil
		}

		got, err := os.ReadFile(gotFile)
		if err != nil {
			t.Errorf("failed to read %q", gotFile)
			return nil
		}
		if *genFiles {
			if err := os.WriteFile(wantFile, got, 0644); err != nil {
				t.Fatal(err)
			}
		}
		want, err := os.ReadFile(wantFile)
		if err != nil {
			t.Errorf("failed to read %q", wantFile)
		} else {
			scanGot := bufio.NewScanner(bytes.NewReader(got))
			scanWant := bufio.NewScanner(bytes.NewReader(want))
			line := 0
			clean := func(s string) string {
				if i := strings.LastIndex(s, "//"); i != -1 {
					s = s[:i]
				}
				return path.Clean(filepath.ToSlash(s))
			}
			for scanGot.Scan() && scanWant.Scan() {
				got := clean(scanGot.Text())
				want := clean(scanWant.Text())
				if got != want {
					t.Errorf("file %q differs from .want file at line %d:\n\t%s\n\t%s", gotFile, line, got, want)
					break
				}
				line++
			}
			if scanGot.Scan() || scanWant.Scan() {
				t.Errorf("file %q differs from .want file at line %d.", gotFile, line)
			}
		}
		return nil
	})
	if err != nil {
		t.Fatal(err)
	}
}

func writeJSON(t *testing.T, path string, x interface{}) {
	data, err := json.MarshalIndent(x, "", "    ")
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(path, data, 0644); err != nil {
		t.Fatal(err)
	}
}

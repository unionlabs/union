// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.22
// +build go1.22

package interp_test

import (
	"bytes"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"reflect"
	"testing"

	"golang.org/x/tools/internal/testenv"
)

func init() {
	testdataTests = append(testdataTests,
		"rangevarlifetime_go122.go",
		"forvarlifetime_go122.go",
	)
}

// TestExperimentRange tests files in testdata with GOEXPERIMENT=range set.
func TestExperimentRange(t *testing.T) {
	testenv.NeedsGoExperiment(t, "range")

	// TODO: Is cwd actually needed here?
	goroot := makeGoroot(t)
	cwd, err := os.Getwd()
	if err != nil {
		log.Fatal(err)
	}
	run(t, filepath.Join(cwd, "testdata", "rangeoverint.go"), goroot)
}

// TestRangeFunc tests range-over-func in a subprocess.
func TestRangeFunc(t *testing.T) {
	testenv.NeedsGo1Point(t, 23)

	// TODO(taking): Remove subprocess from the test and capture output another way.
	if os.Getenv("INTERPTEST_CHILD") == "1" {
		testRangeFunc(t)
		return
	}

	testenv.NeedsExec(t)
	testenv.NeedsTool(t, "go")

	cmd := exec.Command(os.Args[0], "-test.run=TestRangeFunc")
	cmd.Env = append(os.Environ(), "INTERPTEST_CHILD=1")
	out, err := cmd.CombinedOutput()
	if len(out) > 0 {
		t.Logf("out=<<%s>>", out)
	}

	// Check the output of the tests.
	const (
		RERR_DONE      = "Saw expected panic: yield function called after range loop exit"
		RERR_MISSING   = "Saw expected panic: iterator call did not preserve panic"
		RERR_EXHAUSTED = RERR_DONE // ssa does not distinguish. Same message as RERR_DONE.

		CERR_DONE      = "Saw expected panic: checked rangefunc error: loop iteration after body done"
		CERR_EXHAUSTED = "Saw expected panic: checked rangefunc error: loop iteration after iterator exit"
		CERR_MISSING   = "Saw expected panic: checked rangefunc error: loop iterator swallowed panic"

		panickyIterMsg = "Saw expected panic: Panicky iterator panicking"
	)
	expected := map[string][]string{
		// rangefunc.go
		"TestCheck":                           []string{"i = 45", CERR_DONE},
		"TestCooperativeBadOfSliceIndex":      []string{RERR_EXHAUSTED, "i = 36"},
		"TestCooperativeBadOfSliceIndexCheck": []string{CERR_EXHAUSTED, "i = 36"},
		"TestTrickyIterAll":                   []string{"i = 36", RERR_EXHAUSTED},
		"TestTrickyIterOne":                   []string{"i = 1", RERR_EXHAUSTED},
		"TestTrickyIterZero":                  []string{"i = 0", RERR_EXHAUSTED},
		"TestTrickyIterZeroCheck":             []string{"i = 0", CERR_EXHAUSTED},
		"TestTrickyIterEcho": []string{
			"first loop i=0",
			"first loop i=1",
			"first loop i=3",
			"first loop i=6",
			"i = 10",
			"second loop i=0",
			RERR_EXHAUSTED,
			"end i=0",
		},
		"TestTrickyIterEcho2": []string{
			"k=0,x=1,i=0",
			"k=0,x=2,i=1",
			"k=0,x=3,i=3",
			"k=0,x=4,i=6",
			"i = 10",
			"k=1,x=1,i=0",
			RERR_EXHAUSTED,
			"end i=1",
		},
		"TestBreak1":                []string{"[1 2 -1 1 2 -2 1 2 -3]"},
		"TestBreak2":                []string{"[1 2 -1 1 2 -2 1 2 -3]"},
		"TestContinue":              []string{"[-1 1 2 -2 1 2 -3 1 2 -4]"},
		"TestBreak3":                []string{"[100 10 2 4 200 10 2 4 20 2 4 300 10 2 4 20 2 4 30]"},
		"TestBreak1BadA":            []string{"[1 2 -1 1 2 -2 1 2 -3]", RERR_DONE},
		"TestBreak1BadB":            []string{"[1 2]", RERR_DONE},
		"TestMultiCont0":            []string{"[1000 10 2 4 2000]"},
		"TestMultiCont1":            []string{"[1000 10 2 4]", RERR_DONE},
		"TestMultiCont2":            []string{"[1000 10 2 4]", RERR_DONE},
		"TestMultiCont3":            []string{"[1000 10 2 4]", RERR_DONE},
		"TestMultiBreak0":           []string{"[1000 10 2 4]", RERR_DONE},
		"TestMultiBreak1":           []string{"[1000 10 2 4]", RERR_DONE},
		"TestMultiBreak2":           []string{"[1000 10 2 4]", RERR_DONE},
		"TestMultiBreak3":           []string{"[1000 10 2 4]", RERR_DONE},
		"TestPanickyIterator1":      []string{panickyIterMsg},
		"TestPanickyIterator1Check": []string{panickyIterMsg},
		"TestPanickyIterator2":      []string{RERR_MISSING},
		"TestPanickyIterator2Check": []string{CERR_MISSING},
		"TestPanickyIterator3":      []string{"[100 10 1 2 200 10 1 2]"},
		"TestPanickyIterator3Check": []string{"[100 10 1 2 200 10 1 2]"},
		"TestPanickyIterator4":      []string{RERR_MISSING},
		"TestPanickyIterator4Check": []string{CERR_MISSING},
		"TestVeryBad1":              []string{"[1 10]"},
		"TestVeryBad2":              []string{"[1 10]"},
		"TestVeryBadCheck":          []string{"[1 10]"},
		"TestOk":                    []string{"[1 10]"},
		"TestBreak1BadDefer":        []string{RERR_DONE, "[1 2 -1 1 2 -2 1 2 -3 -30 -20 -10]"},
		"TestReturns":               []string{"[-1 1 2 -10]", "[-1 1 2 -10]", RERR_DONE, "[-1 1 2 -10]", RERR_DONE},
		"TestGotoA":                 []string{"testGotoA1[-1 1 2 -2 1 2 -3 1 2 -4 -30 -20 -10]", "testGotoA2[-1 1 2 -2 1 2 -3 1 2 -4 -30 -20 -10]", RERR_DONE, "testGotoA3[-1 1 2 -10]", RERR_DONE},
		"TestGotoB":                 []string{"testGotoB1[-1 1 2 999 -10]", "testGotoB2[-1 1 2 -10]", RERR_DONE, "testGotoB3[-1 1 2 -10]", RERR_DONE},
		"TestPanicReturns": []string{
			"Got expected 'f return'",
			"Got expected 'g return'",
			"Got expected 'h return'",
			"Got expected 'k return'",
			"Got expected 'j return'",
			"Got expected 'm return'",
			"Got expected 'n return and n closure return'",
		},
	}
	got := make(map[string][]string)
	for _, ln := range bytes.Split(out, []byte("\n")) {
		if ind := bytes.Index(ln, []byte(" \t ")); ind >= 0 {
			n, m := string(ln[:ind]), string(ln[ind+3:])
			got[n] = append(got[n], m)
		}
	}
	for n, es := range expected {
		if gs := got[n]; !reflect.DeepEqual(es, gs) {
			t.Errorf("Output of test %s did not match expected output %v. got %v", n, es, gs)
		}
	}
	for n, gs := range got {
		if expected[n] == nil {
			t.Errorf("No expected output for test %s. got %v", n, gs)
		}
	}

	var exitcode int
	if err, ok := err.(*exec.ExitError); ok {
		exitcode = err.ExitCode()
	}
	const want = 0
	if exitcode != want {
		t.Errorf("exited %d, want %d", exitcode, want)
	}
}

func testRangeFunc(t *testing.T) {
	goroot := makeGoroot(t)
	cwd, err := os.Getwd()
	if err != nil {
		log.Fatal(err)
	}

	input := "rangefunc.go"
	run(t, filepath.Join(cwd, "testdata", input), goroot)
}

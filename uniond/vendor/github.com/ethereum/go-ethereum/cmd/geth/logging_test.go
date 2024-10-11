//go:build integrationtests

// Copyright 2023 The go-ethereum Authors
// This file is part of go-ethereum.
//
// go-ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// go-ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with go-ethereum. If not, see <http://www.gnu.org/licenses/>.

package main

import (
	"bufio"
	"bytes"
	"fmt"
	"io"
	"math/rand"
	"os"
	"os/exec"
	"strings"
	"testing"

	"github.com/docker/docker/pkg/reexec"
)

func runSelf(args ...string) ([]byte, error) {
	cmd := &exec.Cmd{
		Path: reexec.Self(),
		Args: append([]string{"geth-test"}, args...),
	}
	return cmd.CombinedOutput()
}

func split(input io.Reader) []string {
	var output []string
	scanner := bufio.NewScanner(input)
	scanner.Split(bufio.ScanLines)
	for scanner.Scan() {
		output = append(output, strings.TrimSpace(scanner.Text()))
	}
	return output
}

func censor(input string, start, end int) string {
	if len(input) < end {
		return input
	}
	return input[:start] + strings.Repeat("X", end-start) + input[end:]
}

func TestLogging(t *testing.T) {
	testConsoleLogging(t, "terminal", 6, 24)
	testConsoleLogging(t, "logfmt", 2, 26)
}

func testConsoleLogging(t *testing.T, format string, tStart, tEnd int) {
	haveB, err := runSelf("--log.format", format, "logtest")
	if err != nil {
		t.Fatal(err)
	}
	readFile, err := os.Open(fmt.Sprintf("testdata/logging/logtest-%v.txt", format))
	if err != nil {
		t.Fatal(err)
	}
	wantLines := split(readFile)
	haveLines := split(bytes.NewBuffer(haveB))
	for i, want := range wantLines {
		if i > len(haveLines)-1 {
			t.Fatalf("format %v, line %d missing, want:%v", format, i, want)
		}
		have := haveLines[i]
		for strings.Contains(have, "Unknown config environment variable") {
			// This can happen on CI runs. Drop it.
			haveLines = append(haveLines[:i], haveLines[i+1:]...)
			have = haveLines[i]
		}

		// Black out the timestamp
		have = censor(have, tStart, tEnd)
		want = censor(want, tStart, tEnd)
		if have != want {
			t.Logf(nicediff([]byte(have), []byte(want)))
			t.Fatalf("format %v, line %d\nhave %v\nwant %v", format, i, have, want)
		}
	}
	if len(haveLines) != len(wantLines) {
		t.Errorf("format %v, want %d lines, have %d", format, len(haveLines), len(wantLines))
	}
}

func TestVmodule(t *testing.T) {
	checkOutput := func(level int, want, wantNot string) {
		t.Helper()
		output, err := runSelf("--log.format", "terminal", "--verbosity=0", "--log.vmodule", fmt.Sprintf("logtestcmd_active.go=%d", level), "logtest")
		if err != nil {
			t.Fatal(err)
		}
		if len(want) > 0 && !strings.Contains(string(output), want) { // trace should be present at 5
			t.Errorf("failed to find expected string ('%s') in output", want)
		}
		if len(wantNot) > 0 && strings.Contains(string(output), wantNot) { // trace should be present at 5
			t.Errorf("string ('%s') should not be present in output", wantNot)
		}
	}
	checkOutput(5, "log at level trace", "")                   // trace should be present at 5
	checkOutput(4, "log at level debug", "log at level trace") // debug should be present at 4, but trace should be missing
	checkOutput(3, "log at level info", "log at level debug")  // info should be present at 3, but debug should be missing
	checkOutput(2, "log at level warn", "log at level info")   // warn should be present at 2, but info should be missing
	checkOutput(1, "log at level error", "log at level warn")  // error should be present at 1, but warn should be missing
}

func nicediff(have, want []byte) string {
	var i = 0
	for ; i < len(have) && i < len(want); i++ {
		if want[i] != have[i] {
			break
		}
	}
	var end = i + 40
	var start = i - 50
	if start < 0 {
		start = 0
	}
	var h, w string
	if end < len(have) {
		h = string(have[start:end])
	} else {
		h = string(have[start:])
	}
	if end < len(want) {
		w = string(want[start:end])
	} else {
		w = string(want[start:])
	}
	return fmt.Sprintf("have vs want:\n%q\n%q\n", h, w)
}

func TestFileOut(t *testing.T) {
	var (
		have, want []byte
		err        error
		path       = fmt.Sprintf("%s/test_file_out-%d", os.TempDir(), rand.Int63())
	)
	t.Cleanup(func() { os.Remove(path) })
	if want, err = runSelf(fmt.Sprintf("--log.file=%s", path), "logtest"); err != nil {
		t.Fatal(err)
	}
	if have, err = os.ReadFile(path); err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(have, want) {
		// show an intelligent diff
		t.Logf(nicediff(have, want))
		t.Errorf("file content wrong")
	}
}

func TestRotatingFileOut(t *testing.T) {
	var (
		have, want []byte
		err        error
		path       = fmt.Sprintf("%s/test_file_out-%d", os.TempDir(), rand.Int63())
	)
	t.Cleanup(func() { os.Remove(path) })
	if want, err = runSelf(fmt.Sprintf("--log.file=%s", path), "--log.rotate", "logtest"); err != nil {
		t.Fatal(err)
	}
	if have, err = os.ReadFile(path); err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(have, want) {
		// show an intelligent diff
		t.Logf(nicediff(have, want))
		t.Errorf("file content wrong")
	}
}

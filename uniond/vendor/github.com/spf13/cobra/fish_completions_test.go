// Copyright 2013-2023 The Cobra Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package cobra

import (
	"bytes"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"testing"
)

func TestCompleteNoDesCmdInFishScript(t *testing.T) {
	rootCmd := &Command{Use: "root", Args: NoArgs, Run: emptyRun}
	child := &Command{
		Use:               "child",
		ValidArgsFunction: validArgsFunc,
		Run:               emptyRun,
	}
	rootCmd.AddCommand(child)

	buf := new(bytes.Buffer)
	assertNoErr(t, rootCmd.GenFishCompletion(buf, false))
	output := buf.String()

	check(t, output, ShellCompNoDescRequestCmd)
}

func TestCompleteCmdInFishScript(t *testing.T) {
	rootCmd := &Command{Use: "root", Args: NoArgs, Run: emptyRun}
	child := &Command{
		Use:               "child",
		ValidArgsFunction: validArgsFunc,
		Run:               emptyRun,
	}
	rootCmd.AddCommand(child)

	buf := new(bytes.Buffer)
	assertNoErr(t, rootCmd.GenFishCompletion(buf, true))
	output := buf.String()

	check(t, output, ShellCompRequestCmd)
	checkOmit(t, output, ShellCompNoDescRequestCmd)
}

func TestProgWithDash(t *testing.T) {
	rootCmd := &Command{Use: "root-dash", Args: NoArgs, Run: emptyRun}
	buf := new(bytes.Buffer)
	assertNoErr(t, rootCmd.GenFishCompletion(buf, false))
	output := buf.String()

	// Functions name should have replace the '-'
	check(t, output, "__root_dash_perform_completion")
	checkOmit(t, output, "__root-dash_perform_completion")

	// The command name should not have replaced the '-'
	check(t, output, "-c root-dash")
	checkOmit(t, output, "-c root_dash")
}

func TestProgWithColon(t *testing.T) {
	rootCmd := &Command{Use: "root:colon", Args: NoArgs, Run: emptyRun}
	buf := new(bytes.Buffer)
	assertNoErr(t, rootCmd.GenFishCompletion(buf, false))
	output := buf.String()

	// Functions name should have replace the ':'
	check(t, output, "__root_colon_perform_completion")
	checkOmit(t, output, "__root:colon_perform_completion")

	// The command name should not have replaced the ':'
	check(t, output, "-c root:colon")
	checkOmit(t, output, "-c root_colon")
}

func TestFishCompletionNoActiveHelp(t *testing.T) {
	c := &Command{Use: "c", Run: emptyRun}

	buf := new(bytes.Buffer)
	assertNoErr(t, c.GenFishCompletion(buf, true))
	output := buf.String()

	// check that active help is being disabled
	activeHelpVar := activeHelpEnvVar(c.Name())
	check(t, output, fmt.Sprintf("%s=0", activeHelpVar))
}

func TestGenFishCompletionFile(t *testing.T) {
	tmpFile, err := os.CreateTemp("", "cobra-test")
	if err != nil {
		t.Fatal(err.Error())
	}

	defer os.Remove(tmpFile.Name())

	rootCmd := &Command{Use: "root", Args: NoArgs, Run: emptyRun}
	child := &Command{
		Use:               "child",
		ValidArgsFunction: validArgsFunc,
		Run:               emptyRun,
	}
	rootCmd.AddCommand(child)

	assertNoErr(t, rootCmd.GenFishCompletionFile(tmpFile.Name(), false))
}

func TestFailGenFishCompletionFile(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "cobra-test")
	if err != nil {
		t.Fatal(err.Error())
	}

	defer os.RemoveAll(tmpDir)

	f, _ := os.OpenFile(filepath.Join(tmpDir, "test"), os.O_CREATE, 0400)
	defer f.Close()

	rootCmd := &Command{Use: "root", Args: NoArgs, Run: emptyRun}
	child := &Command{
		Use:               "child",
		ValidArgsFunction: validArgsFunc,
		Run:               emptyRun,
	}
	rootCmd.AddCommand(child)

	got := rootCmd.GenFishCompletionFile(f.Name(), false)
	if !errors.Is(got, os.ErrPermission) {
		t.Errorf("got: %s, want: %s", got.Error(), os.ErrPermission.Error())
	}
}

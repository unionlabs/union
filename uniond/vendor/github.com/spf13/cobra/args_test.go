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
	"fmt"
	"strings"
	"testing"
)

func getCommand(args PositionalArgs, withValid bool) *Command {
	c := &Command{
		Use:  "c",
		Args: args,
		Run:  emptyRun,
	}
	if withValid {
		c.ValidArgs = []string{"one", "two", "three"}
	}
	return c
}

func expectSuccess(output string, err error, t *testing.T) {
	if output != "" {
		t.Errorf("Unexpected output: %v", output)
	}
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
}

func validOnlyWithInvalidArgs(err error, t *testing.T) {
	if err == nil {
		t.Fatal("Expected an error")
	}
	got := err.Error()
	expected := `invalid argument "a" for "c"`
	if got != expected {
		t.Errorf("Expected: %q, got: %q", expected, got)
	}
}

func noArgsWithArgs(err error, t *testing.T, arg string) {
	if err == nil {
		t.Fatal("Expected an error")
	}
	got := err.Error()
	expected := `unknown command "` + arg + `" for "c"`
	if got != expected {
		t.Errorf("Expected: %q, got: %q", expected, got)
	}
}

func minimumNArgsWithLessArgs(err error, t *testing.T) {
	if err == nil {
		t.Fatal("Expected an error")
	}
	got := err.Error()
	expected := "requires at least 2 arg(s), only received 1"
	if got != expected {
		t.Fatalf("Expected %q, got %q", expected, got)
	}
}

func maximumNArgsWithMoreArgs(err error, t *testing.T) {
	if err == nil {
		t.Fatal("Expected an error")
	}
	got := err.Error()
	expected := "accepts at most 2 arg(s), received 3"
	if got != expected {
		t.Fatalf("Expected %q, got %q", expected, got)
	}
}

func exactArgsWithInvalidCount(err error, t *testing.T) {
	if err == nil {
		t.Fatal("Expected an error")
	}
	got := err.Error()
	expected := "accepts 2 arg(s), received 3"
	if got != expected {
		t.Fatalf("Expected %q, got %q", expected, got)
	}
}

func rangeArgsWithInvalidCount(err error, t *testing.T) {
	if err == nil {
		t.Fatal("Expected an error")
	}
	got := err.Error()
	expected := "accepts between 2 and 4 arg(s), received 1"
	if got != expected {
		t.Fatalf("Expected %q, got %q", expected, got)
	}
}

// NoArgs

func TestNoArgs(t *testing.T) {
	c := getCommand(NoArgs, false)
	output, err := executeCommand(c)
	expectSuccess(output, err, t)
}

func TestNoArgs_WithArgs(t *testing.T) {
	c := getCommand(NoArgs, false)
	_, err := executeCommand(c, "one")
	noArgsWithArgs(err, t, "one")
}

func TestNoArgs_WithValid_WithArgs(t *testing.T) {
	c := getCommand(NoArgs, true)
	_, err := executeCommand(c, "one")
	noArgsWithArgs(err, t, "one")
}

func TestNoArgs_WithValid_WithInvalidArgs(t *testing.T) {
	c := getCommand(NoArgs, true)
	_, err := executeCommand(c, "a")
	noArgsWithArgs(err, t, "a")
}

func TestNoArgs_WithValidOnly_WithInvalidArgs(t *testing.T) {
	c := getCommand(MatchAll(OnlyValidArgs, NoArgs), true)
	_, err := executeCommand(c, "a")
	validOnlyWithInvalidArgs(err, t)
}

// OnlyValidArgs

func TestOnlyValidArgs(t *testing.T) {
	c := getCommand(OnlyValidArgs, true)
	output, err := executeCommand(c, "one", "two")
	expectSuccess(output, err, t)
}

func TestOnlyValidArgs_WithInvalidArgs(t *testing.T) {
	c := getCommand(OnlyValidArgs, true)
	_, err := executeCommand(c, "a")
	validOnlyWithInvalidArgs(err, t)
}

// ArbitraryArgs

func TestArbitraryArgs(t *testing.T) {
	c := getCommand(ArbitraryArgs, false)
	output, err := executeCommand(c, "a", "b")
	expectSuccess(output, err, t)
}

func TestArbitraryArgs_WithValid(t *testing.T) {
	c := getCommand(ArbitraryArgs, true)
	output, err := executeCommand(c, "one", "two")
	expectSuccess(output, err, t)
}

func TestArbitraryArgs_WithValid_WithInvalidArgs(t *testing.T) {
	c := getCommand(ArbitraryArgs, true)
	output, err := executeCommand(c, "a")
	expectSuccess(output, err, t)
}

func TestArbitraryArgs_WithValidOnly_WithInvalidArgs(t *testing.T) {
	c := getCommand(MatchAll(OnlyValidArgs, ArbitraryArgs), true)
	_, err := executeCommand(c, "a")
	validOnlyWithInvalidArgs(err, t)
}

// MinimumNArgs

func TestMinimumNArgs(t *testing.T) {
	c := getCommand(MinimumNArgs(2), false)
	output, err := executeCommand(c, "a", "b", "c")
	expectSuccess(output, err, t)
}

func TestMinimumNArgs_WithValid(t *testing.T) {
	c := getCommand(MinimumNArgs(2), true)
	output, err := executeCommand(c, "one", "three")
	expectSuccess(output, err, t)
}

func TestMinimumNArgs_WithValid__WithInvalidArgs(t *testing.T) {
	c := getCommand(MinimumNArgs(2), true)
	output, err := executeCommand(c, "a", "b")
	expectSuccess(output, err, t)
}

func TestMinimumNArgs_WithValidOnly_WithInvalidArgs(t *testing.T) {
	c := getCommand(MatchAll(OnlyValidArgs, MinimumNArgs(2)), true)
	_, err := executeCommand(c, "a", "b")
	validOnlyWithInvalidArgs(err, t)
}

func TestMinimumNArgs_WithLessArgs(t *testing.T) {
	c := getCommand(MinimumNArgs(2), false)
	_, err := executeCommand(c, "a")
	minimumNArgsWithLessArgs(err, t)
}

func TestMinimumNArgs_WithLessArgs_WithValid(t *testing.T) {
	c := getCommand(MinimumNArgs(2), true)
	_, err := executeCommand(c, "one")
	minimumNArgsWithLessArgs(err, t)
}

func TestMinimumNArgs_WithLessArgs_WithValid_WithInvalidArgs(t *testing.T) {
	c := getCommand(MinimumNArgs(2), true)
	_, err := executeCommand(c, "a")
	minimumNArgsWithLessArgs(err, t)
}

func TestMinimumNArgs_WithLessArgs_WithValidOnly_WithInvalidArgs(t *testing.T) {
	c := getCommand(MatchAll(OnlyValidArgs, MinimumNArgs(2)), true)
	_, err := executeCommand(c, "a")
	validOnlyWithInvalidArgs(err, t)
}

// MaximumNArgs

func TestMaximumNArgs(t *testing.T) {
	c := getCommand(MaximumNArgs(3), false)
	output, err := executeCommand(c, "a", "b")
	expectSuccess(output, err, t)
}

func TestMaximumNArgs_WithValid(t *testing.T) {
	c := getCommand(MaximumNArgs(2), true)
	output, err := executeCommand(c, "one", "three")
	expectSuccess(output, err, t)
}

func TestMaximumNArgs_WithValid_WithInvalidArgs(t *testing.T) {
	c := getCommand(MaximumNArgs(2), true)
	output, err := executeCommand(c, "a", "b")
	expectSuccess(output, err, t)
}

func TestMaximumNArgs_WithValidOnly_WithInvalidArgs(t *testing.T) {
	c := getCommand(MatchAll(OnlyValidArgs, MaximumNArgs(2)), true)
	_, err := executeCommand(c, "a", "b")
	validOnlyWithInvalidArgs(err, t)
}

func TestMaximumNArgs_WithMoreArgs(t *testing.T) {
	c := getCommand(MaximumNArgs(2), false)
	_, err := executeCommand(c, "a", "b", "c")
	maximumNArgsWithMoreArgs(err, t)
}

func TestMaximumNArgs_WithMoreArgs_WithValid(t *testing.T) {
	c := getCommand(MaximumNArgs(2), true)
	_, err := executeCommand(c, "one", "three", "two")
	maximumNArgsWithMoreArgs(err, t)
}

func TestMaximumNArgs_WithMoreArgs_WithValid_WithInvalidArgs(t *testing.T) {
	c := getCommand(MaximumNArgs(2), true)
	_, err := executeCommand(c, "a", "b", "c")
	maximumNArgsWithMoreArgs(err, t)
}

func TestMaximumNArgs_WithMoreArgs_WithValidOnly_WithInvalidArgs(t *testing.T) {
	c := getCommand(MatchAll(OnlyValidArgs, MaximumNArgs(2)), true)
	_, err := executeCommand(c, "a", "b", "c")
	validOnlyWithInvalidArgs(err, t)
}

// ExactArgs

func TestExactArgs(t *testing.T) {
	c := getCommand(ExactArgs(3), false)
	output, err := executeCommand(c, "a", "b", "c")
	expectSuccess(output, err, t)
}

func TestExactArgs_WithValid(t *testing.T) {
	c := getCommand(ExactArgs(3), true)
	output, err := executeCommand(c, "three", "one", "two")
	expectSuccess(output, err, t)
}

func TestExactArgs_WithValid_WithInvalidArgs(t *testing.T) {
	c := getCommand(ExactArgs(3), true)
	output, err := executeCommand(c, "three", "a", "two")
	expectSuccess(output, err, t)
}

func TestExactArgs_WithValidOnly_WithInvalidArgs(t *testing.T) {
	c := getCommand(MatchAll(OnlyValidArgs, ExactArgs(3)), true)
	_, err := executeCommand(c, "three", "a", "two")
	validOnlyWithInvalidArgs(err, t)
}

func TestExactArgs_WithInvalidCount(t *testing.T) {
	c := getCommand(ExactArgs(2), false)
	_, err := executeCommand(c, "a", "b", "c")
	exactArgsWithInvalidCount(err, t)
}

func TestExactArgs_WithInvalidCount_WithValid(t *testing.T) {
	c := getCommand(ExactArgs(2), true)
	_, err := executeCommand(c, "three", "one", "two")
	exactArgsWithInvalidCount(err, t)
}

func TestExactArgs_WithInvalidCount_WithValid_WithInvalidArgs(t *testing.T) {
	c := getCommand(ExactArgs(2), true)
	_, err := executeCommand(c, "three", "a", "two")
	exactArgsWithInvalidCount(err, t)
}

func TestExactArgs_WithInvalidCount_WithValidOnly_WithInvalidArgs(t *testing.T) {
	c := getCommand(MatchAll(OnlyValidArgs, ExactArgs(2)), true)
	_, err := executeCommand(c, "three", "a", "two")
	validOnlyWithInvalidArgs(err, t)
}

// RangeArgs

func TestRangeArgs(t *testing.T) {
	c := getCommand(RangeArgs(2, 4), false)
	output, err := executeCommand(c, "a", "b", "c")
	expectSuccess(output, err, t)
}

func TestRangeArgs_WithValid(t *testing.T) {
	c := getCommand(RangeArgs(2, 4), true)
	output, err := executeCommand(c, "three", "one", "two")
	expectSuccess(output, err, t)
}

func TestRangeArgs_WithValid_WithInvalidArgs(t *testing.T) {
	c := getCommand(RangeArgs(2, 4), true)
	output, err := executeCommand(c, "three", "a", "two")
	expectSuccess(output, err, t)
}

func TestRangeArgs_WithValidOnly_WithInvalidArgs(t *testing.T) {
	c := getCommand(MatchAll(OnlyValidArgs, RangeArgs(2, 4)), true)
	_, err := executeCommand(c, "three", "a", "two")
	validOnlyWithInvalidArgs(err, t)
}

func TestRangeArgs_WithInvalidCount(t *testing.T) {
	c := getCommand(RangeArgs(2, 4), false)
	_, err := executeCommand(c, "a")
	rangeArgsWithInvalidCount(err, t)
}

func TestRangeArgs_WithInvalidCount_WithValid(t *testing.T) {
	c := getCommand(RangeArgs(2, 4), true)
	_, err := executeCommand(c, "two")
	rangeArgsWithInvalidCount(err, t)
}

func TestRangeArgs_WithInvalidCount_WithValid_WithInvalidArgs(t *testing.T) {
	c := getCommand(RangeArgs(2, 4), true)
	_, err := executeCommand(c, "a")
	rangeArgsWithInvalidCount(err, t)
}

func TestRangeArgs_WithInvalidCount_WithValidOnly_WithInvalidArgs(t *testing.T) {
	c := getCommand(MatchAll(OnlyValidArgs, RangeArgs(2, 4)), true)
	_, err := executeCommand(c, "a")
	validOnlyWithInvalidArgs(err, t)
}

// Takes(No)Args

func TestRootTakesNoArgs(t *testing.T) {
	rootCmd := &Command{Use: "root", Run: emptyRun}
	childCmd := &Command{Use: "child", Run: emptyRun}
	rootCmd.AddCommand(childCmd)

	_, err := executeCommand(rootCmd, "illegal", "args")
	if err == nil {
		t.Fatal("Expected an error")
	}

	got := err.Error()
	expected := `unknown command "illegal" for "root"`
	if !strings.Contains(got, expected) {
		t.Errorf("expected %q, got %q", expected, got)
	}
}

func TestRootTakesArgs(t *testing.T) {
	rootCmd := &Command{Use: "root", Args: ArbitraryArgs, Run: emptyRun}
	childCmd := &Command{Use: "child", Run: emptyRun}
	rootCmd.AddCommand(childCmd)

	_, err := executeCommand(rootCmd, "legal", "args")
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}
}

func TestChildTakesNoArgs(t *testing.T) {
	rootCmd := &Command{Use: "root", Run: emptyRun}
	childCmd := &Command{Use: "child", Args: NoArgs, Run: emptyRun}
	rootCmd.AddCommand(childCmd)

	_, err := executeCommand(rootCmd, "child", "illegal", "args")
	if err == nil {
		t.Fatal("Expected an error")
	}

	got := err.Error()
	expected := `unknown command "illegal" for "root child"`
	if !strings.Contains(got, expected) {
		t.Errorf("expected %q, got %q", expected, got)
	}
}

func TestChildTakesArgs(t *testing.T) {
	rootCmd := &Command{Use: "root", Run: emptyRun}
	childCmd := &Command{Use: "child", Args: ArbitraryArgs, Run: emptyRun}
	rootCmd.AddCommand(childCmd)

	_, err := executeCommand(rootCmd, "child", "legal", "args")
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
}

func TestMatchAll(t *testing.T) {
	// Somewhat contrived example check that ensures there are exactly 3
	// arguments, and each argument is exactly 2 bytes long.
	pargs := MatchAll(
		ExactArgs(3),
		func(cmd *Command, args []string) error {
			for _, arg := range args {
				if len([]byte(arg)) != 2 {
					return fmt.Errorf("expected to be exactly 2 bytes long")
				}
			}
			return nil
		},
	)

	testCases := map[string]struct {
		args []string
		fail bool
	}{
		"happy path": {
			[]string{"aa", "bb", "cc"},
			false,
		},
		"incorrect number of args": {
			[]string{"aa", "bb", "cc", "dd"},
			true,
		},
		"incorrect number of bytes in one arg": {
			[]string{"aa", "bb", "abc"},
			true,
		},
	}

	rootCmd := &Command{Use: "root", Args: pargs, Run: emptyRun}

	for name, tc := range testCases {
		t.Run(name, func(t *testing.T) {
			_, err := executeCommand(rootCmd, tc.args...)
			if err != nil && !tc.fail {
				t.Errorf("unexpected: %v\n", err)
			}
			if err == nil && tc.fail {
				t.Errorf("expected error")
			}
		})
	}
}

// DEPRECATED

func TestExactValidArgs(t *testing.T) {
	c := getCommand(ExactValidArgs(3), true)
	output, err := executeCommand(c, "three", "one", "two")
	expectSuccess(output, err, t)
}

func TestExactValidArgs_WithInvalidCount(t *testing.T) {
	c := getCommand(ExactValidArgs(2), false)
	_, err := executeCommand(c, "three", "one", "two")
	exactArgsWithInvalidCount(err, t)
}

func TestExactValidArgs_WithInvalidCount_WithInvalidArgs(t *testing.T) {
	c := getCommand(ExactValidArgs(2), true)
	_, err := executeCommand(c, "three", "a", "two")
	exactArgsWithInvalidCount(err, t)
}

func TestExactValidArgs_WithInvalidArgs(t *testing.T) {
	c := getCommand(ExactValidArgs(2), true)
	_, err := executeCommand(c, "three", "a")
	validOnlyWithInvalidArgs(err, t)
}

// This test make sure we keep backwards-compatibility with respect
// to the legacyArgs() function.
// It makes sure the root command accepts arguments if it does not have
// sub-commands.
func TestLegacyArgsRootAcceptsArgs(t *testing.T) {
	rootCmd := &Command{Use: "root", Args: nil, Run: emptyRun}

	_, err := executeCommand(rootCmd, "somearg")
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
}

// This test make sure we keep backwards-compatibility with respect
// to the legacyArgs() function.
// It makes sure a sub-command accepts arguments and further sub-commands
func TestLegacyArgsSubcmdAcceptsArgs(t *testing.T) {
	rootCmd := &Command{Use: "root", Args: nil, Run: emptyRun}
	childCmd := &Command{Use: "child", Args: nil, Run: emptyRun}
	grandchildCmd := &Command{Use: "grandchild", Args: nil, Run: emptyRun}
	rootCmd.AddCommand(childCmd)
	childCmd.AddCommand(grandchildCmd)

	_, err := executeCommand(rootCmd, "child", "somearg")
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
}

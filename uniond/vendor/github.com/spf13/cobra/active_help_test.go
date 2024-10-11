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
	"os"
	"strings"
	"testing"
)

const (
	activeHelpMessage  = "This is an activeHelp message"
	activeHelpMessage2 = "This is the rest of the activeHelp message"
)

func TestActiveHelpAlone(t *testing.T) {
	rootCmd := &Command{
		Use: "root",
		Run: emptyRun,
	}

	activeHelpFunc := func(cmd *Command, args []string, toComplete string) ([]string, ShellCompDirective) {
		comps := AppendActiveHelp(nil, activeHelpMessage)
		return comps, ShellCompDirectiveDefault
	}

	// Test that activeHelp can be added to a root command
	rootCmd.ValidArgsFunction = activeHelpFunc

	output, err := executeCommand(rootCmd, ShellCompNoDescRequestCmd, "")
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}

	expected := strings.Join([]string{
		fmt.Sprintf("%s%s", activeHelpMarker, activeHelpMessage),
		":0",
		"Completion ended with directive: ShellCompDirectiveDefault", ""}, "\n")

	if output != expected {
		t.Errorf("expected: %q, got: %q", expected, output)
	}

	rootCmd.ValidArgsFunction = nil

	// Test that activeHelp can be added to a child command
	childCmd := &Command{
		Use:   "thechild",
		Short: "The child command",
		Run:   emptyRun,
	}
	rootCmd.AddCommand(childCmd)

	childCmd.ValidArgsFunction = activeHelpFunc

	output, err = executeCommand(rootCmd, ShellCompNoDescRequestCmd, "thechild", "")
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}

	expected = strings.Join([]string{
		fmt.Sprintf("%s%s", activeHelpMarker, activeHelpMessage),
		":0",
		"Completion ended with directive: ShellCompDirectiveDefault", ""}, "\n")

	if output != expected {
		t.Errorf("expected: %q, got: %q", expected, output)
	}
}

func TestActiveHelpWithComps(t *testing.T) {
	rootCmd := &Command{
		Use: "root",
		Run: emptyRun,
	}

	childCmd := &Command{
		Use:   "thechild",
		Short: "The child command",
		Run:   emptyRun,
	}
	rootCmd.AddCommand(childCmd)

	// Test that activeHelp can be added following other completions
	childCmd.ValidArgsFunction = func(cmd *Command, args []string, toComplete string) ([]string, ShellCompDirective) {
		comps := []string{"first", "second"}
		comps = AppendActiveHelp(comps, activeHelpMessage)
		return comps, ShellCompDirectiveDefault
	}

	output, err := executeCommand(rootCmd, ShellCompNoDescRequestCmd, "thechild", "")
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}

	expected := strings.Join([]string{
		"first",
		"second",
		fmt.Sprintf("%s%s", activeHelpMarker, activeHelpMessage),
		":0",
		"Completion ended with directive: ShellCompDirectiveDefault", ""}, "\n")

	if output != expected {
		t.Errorf("expected: %q, got: %q", expected, output)
	}

	// Test that activeHelp can be added preceding other completions
	childCmd.ValidArgsFunction = func(cmd *Command, args []string, toComplete string) ([]string, ShellCompDirective) {
		var comps []string
		comps = AppendActiveHelp(comps, activeHelpMessage)
		comps = append(comps, []string{"first", "second"}...)
		return comps, ShellCompDirectiveDefault
	}

	output, err = executeCommand(rootCmd, ShellCompNoDescRequestCmd, "thechild", "")
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}

	expected = strings.Join([]string{
		fmt.Sprintf("%s%s", activeHelpMarker, activeHelpMessage),
		"first",
		"second",
		":0",
		"Completion ended with directive: ShellCompDirectiveDefault", ""}, "\n")

	if output != expected {
		t.Errorf("expected: %q, got: %q", expected, output)
	}

	// Test that activeHelp can be added interleaved with other completions
	childCmd.ValidArgsFunction = func(cmd *Command, args []string, toComplete string) ([]string, ShellCompDirective) {
		comps := []string{"first"}
		comps = AppendActiveHelp(comps, activeHelpMessage)
		comps = append(comps, "second")
		return comps, ShellCompDirectiveDefault
	}

	output, err = executeCommand(rootCmd, ShellCompNoDescRequestCmd, "thechild", "")
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}

	expected = strings.Join([]string{
		"first",
		fmt.Sprintf("%s%s", activeHelpMarker, activeHelpMessage),
		"second",
		":0",
		"Completion ended with directive: ShellCompDirectiveDefault", ""}, "\n")

	if output != expected {
		t.Errorf("expected: %q, got: %q", expected, output)
	}
}

func TestMultiActiveHelp(t *testing.T) {
	rootCmd := &Command{
		Use: "root",
		Run: emptyRun,
	}

	childCmd := &Command{
		Use:   "thechild",
		Short: "The child command",
		Run:   emptyRun,
	}
	rootCmd.AddCommand(childCmd)

	// Test that multiple activeHelp message can be added
	childCmd.ValidArgsFunction = func(cmd *Command, args []string, toComplete string) ([]string, ShellCompDirective) {
		comps := AppendActiveHelp(nil, activeHelpMessage)
		comps = AppendActiveHelp(comps, activeHelpMessage2)
		return comps, ShellCompDirectiveNoFileComp
	}

	output, err := executeCommand(rootCmd, ShellCompNoDescRequestCmd, "thechild", "")
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}

	expected := strings.Join([]string{
		fmt.Sprintf("%s%s", activeHelpMarker, activeHelpMessage),
		fmt.Sprintf("%s%s", activeHelpMarker, activeHelpMessage2),
		":4",
		"Completion ended with directive: ShellCompDirectiveNoFileComp", ""}, "\n")

	if output != expected {
		t.Errorf("expected: %q, got: %q", expected, output)
	}

	// Test that multiple activeHelp messages can be used along with completions
	childCmd.ValidArgsFunction = func(cmd *Command, args []string, toComplete string) ([]string, ShellCompDirective) {
		comps := []string{"first"}
		comps = AppendActiveHelp(comps, activeHelpMessage)
		comps = append(comps, "second")
		comps = AppendActiveHelp(comps, activeHelpMessage2)
		return comps, ShellCompDirectiveNoFileComp
	}

	output, err = executeCommand(rootCmd, ShellCompNoDescRequestCmd, "thechild", "")
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}

	expected = strings.Join([]string{
		"first",
		fmt.Sprintf("%s%s", activeHelpMarker, activeHelpMessage),
		"second",
		fmt.Sprintf("%s%s", activeHelpMarker, activeHelpMessage2),
		":4",
		"Completion ended with directive: ShellCompDirectiveNoFileComp", ""}, "\n")

	if output != expected {
		t.Errorf("expected: %q, got: %q", expected, output)
	}
}

func TestActiveHelpForFlag(t *testing.T) {
	rootCmd := &Command{
		Use: "root",
		Run: emptyRun,
	}
	flagname := "flag"
	rootCmd.Flags().String(flagname, "", "A flag")

	// Test that multiple activeHelp message can be added
	_ = rootCmd.RegisterFlagCompletionFunc(flagname, func(cmd *Command, args []string, toComplete string) ([]string, ShellCompDirective) {
		comps := []string{"first"}
		comps = AppendActiveHelp(comps, activeHelpMessage)
		comps = append(comps, "second")
		comps = AppendActiveHelp(comps, activeHelpMessage2)
		return comps, ShellCompDirectiveNoFileComp
	})

	output, err := executeCommand(rootCmd, ShellCompNoDescRequestCmd, "--flag", "")
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}

	expected := strings.Join([]string{
		"first",
		fmt.Sprintf("%s%s", activeHelpMarker, activeHelpMessage),
		"second",
		fmt.Sprintf("%s%s", activeHelpMarker, activeHelpMessage2),
		":4",
		"Completion ended with directive: ShellCompDirectiveNoFileComp", ""}, "\n")

	if output != expected {
		t.Errorf("expected: %q, got: %q", expected, output)
	}
}

func TestConfigActiveHelp(t *testing.T) {
	rootCmd := &Command{
		Use: "root",
		Run: emptyRun,
	}

	childCmd := &Command{
		Use:   "thechild",
		Short: "The child command",
		Run:   emptyRun,
	}
	rootCmd.AddCommand(childCmd)

	activeHelpCfg := "someconfig,anotherconfig"
	// Set the variable that the user would be setting
	os.Setenv(activeHelpEnvVar(rootCmd.Name()), activeHelpCfg)

	childCmd.ValidArgsFunction = func(cmd *Command, args []string, toComplete string) ([]string, ShellCompDirective) {
		receivedActiveHelpCfg := GetActiveHelpConfig(cmd)
		if receivedActiveHelpCfg != activeHelpCfg {
			t.Errorf("expected activeHelpConfig: %q, but got: %q", activeHelpCfg, receivedActiveHelpCfg)
		}
		return nil, ShellCompDirectiveDefault
	}

	_, err := executeCommand(rootCmd, ShellCompNoDescRequestCmd, "thechild", "")
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}

	// Test active help config for a flag
	activeHelpCfg = "a config for a flag"
	// Set the variable that the completions scripts will be setting
	os.Setenv(activeHelpEnvVar(rootCmd.Name()), activeHelpCfg)

	flagname := "flag"
	childCmd.Flags().String(flagname, "", "A flag")

	// Test that multiple activeHelp message can be added
	_ = childCmd.RegisterFlagCompletionFunc(flagname, func(cmd *Command, args []string, toComplete string) ([]string, ShellCompDirective) {
		receivedActiveHelpCfg := GetActiveHelpConfig(cmd)
		if receivedActiveHelpCfg != activeHelpCfg {
			t.Errorf("expected activeHelpConfig: %q, but got: %q", activeHelpCfg, receivedActiveHelpCfg)
		}
		return nil, ShellCompDirectiveDefault
	})

	_, err = executeCommand(rootCmd, ShellCompNoDescRequestCmd, "thechild", "--flag", "")
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}
}

func TestDisableActiveHelp(t *testing.T) {
	rootCmd := &Command{
		Use: "root",
		Run: emptyRun,
	}

	childCmd := &Command{
		Use:   "thechild",
		Short: "The child command",
		Run:   emptyRun,
	}
	rootCmd.AddCommand(childCmd)

	// Test the disabling of activeHelp using the specific program
	// environment variable that the completions scripts will be setting.
	// Make sure the disabling value is "0" by hard-coding it in the tests;
	// this is for backwards-compatibility as programs will be using this value.
	os.Setenv(activeHelpEnvVar(rootCmd.Name()), "0")

	childCmd.ValidArgsFunction = func(cmd *Command, args []string, toComplete string) ([]string, ShellCompDirective) {
		comps := []string{"first"}
		comps = AppendActiveHelp(comps, activeHelpMessage)
		return comps, ShellCompDirectiveDefault
	}

	output, err := executeCommand(rootCmd, ShellCompNoDescRequestCmd, "thechild", "")
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}
	os.Unsetenv(activeHelpEnvVar(rootCmd.Name()))

	// Make sure there is no ActiveHelp in the output
	expected := strings.Join([]string{
		"first",
		":0",
		"Completion ended with directive: ShellCompDirectiveDefault", ""}, "\n")

	if output != expected {
		t.Errorf("expected: %q, got: %q", expected, output)
	}

	// Now test the global disabling of ActiveHelp
	os.Setenv(activeHelpGlobalEnvVar, "0")
	// Set the specific variable, to make sure it is ignored when the global env
	// var is set properly
	os.Setenv(activeHelpEnvVar(rootCmd.Name()), "1")

	output, err = executeCommand(rootCmd, ShellCompNoDescRequestCmd, "thechild", "")
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}

	// Make sure there is no ActiveHelp in the output
	expected = strings.Join([]string{
		"first",
		":0",
		"Completion ended with directive: ShellCompDirectiveDefault", ""}, "\n")

	if output != expected {
		t.Errorf("expected: %q, got: %q", expected, output)
	}

	// Make sure that if the global env variable is set to anything else than
	// the disable value it is ignored
	os.Setenv(activeHelpGlobalEnvVar, "on")
	// Set the specific variable, to make sure it is used (while ignoring the global env var)
	activeHelpCfg := "1"
	os.Setenv(activeHelpEnvVar(rootCmd.Name()), activeHelpCfg)

	childCmd.ValidArgsFunction = func(cmd *Command, args []string, toComplete string) ([]string, ShellCompDirective) {
		receivedActiveHelpCfg := GetActiveHelpConfig(cmd)
		if receivedActiveHelpCfg != activeHelpCfg {
			t.Errorf("expected activeHelpConfig: %q, but got: %q", activeHelpCfg, receivedActiveHelpCfg)
		}
		return nil, ShellCompDirectiveDefault
	}

	_, err = executeCommand(rootCmd, ShellCompNoDescRequestCmd, "thechild", "")
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}
}

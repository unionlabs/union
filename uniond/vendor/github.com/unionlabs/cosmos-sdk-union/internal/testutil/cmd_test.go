package testutil_test

import (
	"fmt"
	"testing"

	"github.com/spf13/cobra"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/internal/testutil"
)

// TestSetArgsWithOriginalMethod is used to illustrate cobra.Command.SetArgs won't reset args as expected
func TestSetArgsWithOriginalMethod(t *testing.T) {
	getCMD := func() *cobra.Command {
		cmd := &cobra.Command{
			Use: "testcmd",
			RunE: func(cmd *cobra.Command, args []string) error {
				a, _ := cmd.Flags().GetBool("a")
				b, _ := cmd.Flags().GetBool("b")
				c, _ := cmd.Flags().GetBool("c")
				switch {
				case a && b, a && c, b && c:
					return fmt.Errorf("a,b,c only one could be true")
				}
				return nil
			},
		}
		f := cmd.Flags()
		f.BoolP("a", "a", false, "a,b,c only one could be true")
		f.BoolP("b", "b", false, "a,b,c only one could be true")
		f.Bool("c", false, "a,b,c only one could be true")
		return cmd
	}

	cmd := getCMD()

	cmd.SetArgs([]string{
		"testcmd",
		"--a=true",
	})
	require.NoError(t, cmd.Execute())

	// This call to cmd.SetArgs is expected to set only the 'b' flag. However, due to the bug, the 'a' flag remains set from the previous call to cmd.SetArgs, leading to an error.
	cmd.SetArgs([]string{
		"testcmd",
		"--b=true",
	})
	require.True(t, cmd.Flags().Changed("a"))
	require.Error(t, cmd.Execute())

	// This call to cmd.SetArgs is expected to set only the 'c' flag. However, the 'a' and 'b' flags remain set from the previous calls, causing an unexpected error.
	cmd.SetArgs([]string{
		"testcmd",
		"--c=true",
	})
	require.Error(t, cmd.Execute())

	// To work around the bug, we must explicitly reset the 'a' and 'b' flags to false, even though we only want to set the 'c' flag to true.
	cmd.SetArgs([]string{
		"testcmd",
		"--a=false",
		"--b=false",
		"--c=true",
	})
	require.NoError(t, cmd.Execute())
}

func TestSetArgsWithWrappedMethod(t *testing.T) {
	var (
		mockFlagWithCommaD = testutil.MockFlagsWithComma{Ary: []string{"g;m", "g;n"}}
		mockFlagWithCommaE testutil.MockFlagsWithComma
	)
	var (
		mockFlagWithSemicolonF = testutil.MockFlagsWithSemicolon{Ary: []string{"g,m", "g,n"}}
		mockFlagWithSemicolonG testutil.MockFlagsWithSemicolon
	)
	getCMD := func() *cobra.Command {
		cmd := &cobra.Command{
			Use: "testcmd",
			RunE: func(cmd *cobra.Command, args []string) error {
				return nil
			},
		}
		f := cmd.Flags()
		f.BoolP("a", "a", false, "check built-in pflag.Value")
		f.IntSlice("b", []int{1, 2}, "check built-in pflag.SliceValue with default value")
		f.IntSliceP("c", "c", nil, "check built pflag.SliceValue with nil default value")
		f.Var(&mockFlagWithCommaD, "d", "check custom implementation of pflag.SliceValue with splitting by comma and default value")
		f.VarP(&mockFlagWithCommaE, "e", "e", "check custom implementation of pflag.SliceValue with splitting by comma and nil default value")
		f.Var(&mockFlagWithSemicolonF, "f", "check custom implementation of pflag.SliceValue with splitting by semicolon and default value")
		f.VarP(&mockFlagWithSemicolonG, "g", "g", "check custom implementation of pflag.SliceValue with splitting by semicolon and nil default value")
		return cmd
	}

	cmd := getCMD()

	checkFlagsValue := func(cmd *cobra.Command, notDefaultFlags map[string]string) bool {
		require.NoError(t, cmd.Execute())
		for _, k := range []string{"a", "b", "c", "d", "e", "f", "g"} {
			curVal := cmd.Flag(k).Value
			curDefVal := cmd.Flag(k).DefValue
			if v, ok := notDefaultFlags[k]; ok {
				require.NotEqual(t, curVal.String(), curDefVal, fmt.Sprintf("flag: %s, cmp_to: %v", k, curVal))
				require.Equal(t, curVal.String(), v, fmt.Sprintf("flag: %s, cmp_to: %v", k, curVal))
			} else {
				require.Equal(t, curVal.String(), curDefVal, fmt.Sprintf("flag: %s, cmp_to: %v", k, curVal))
			}
		}
		return true
	}

	testCases := []struct {
		name  string
		steps []struct {
			args                  []string
			expectNotDefaultFlags map[string]string
		}
	}{
		{
			name: "no args",
			steps: []struct {
				args                  []string
				expectNotDefaultFlags map[string]string
			}{
				{
					args:                  nil,
					expectNotDefaultFlags: nil,
				},
			},
		},
		{
			name: "built-in implementation of pflag.Value",
			steps: []struct {
				args                  []string
				expectNotDefaultFlags map[string]string
			}{
				{
					args:                  []string{"--a=true"},
					expectNotDefaultFlags: map[string]string{"a": "true"},
				},
			},
		},
		{
			name: "built-in implementation of pflag.SliceValue",
			steps: []struct {
				args                  []string
				expectNotDefaultFlags map[string]string
			}{
				{
					args:                  []string{"--b=3,4"},
					expectNotDefaultFlags: map[string]string{"b": "[3,4]"},
				},
				{
					args:                  []string{"--c=3,4"},
					expectNotDefaultFlags: map[string]string{"c": "[3,4]"},
				},
			},
		},
		{
			name: "custom implementation of pflag.SliceValue with comma",
			steps: []struct {
				args                  []string
				expectNotDefaultFlags map[string]string
			}{
				{
					args:                  []string{"--d=g;n,g;m"},
					expectNotDefaultFlags: map[string]string{"d": "g;n,g;m"},
				},
				{
					args:                  []string{"--e=g;n,g;m"},
					expectNotDefaultFlags: map[string]string{"e": "g;n,g;m"},
				},
			},
		},
		{
			// custom implementation of pflag.SliceValue with splitting by semicolon is not compatible with testutil.SetArgs.
			// So `f` is changed to "g;m;g;n" (split to ["g", "m;g", "n"], and then join with ";"), not default value "g,m;g,n"
			name: "custom implementation of pflag.SliceValue with semicolon",
			steps: []struct {
				args                  []string
				expectNotDefaultFlags map[string]string
			}{
				{
					args:                  []string{"--f=g,n;g,m"},
					expectNotDefaultFlags: map[string]string{"f": "g,n;g,m"},
				},
				{
					args:                  []string{"--g=g,n;g,m"},
					expectNotDefaultFlags: map[string]string{"f": "g;m;g;n", "g": "g,n;g,m"},
				},
			},
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			for _, step := range testCase.steps {
				testutil.ResetArgs(t, cmd)
				args := append([]string{"testcmd"}, step.args...)
				cmd.SetArgs(args)
				checkFlagsValue(cmd, step.expectNotDefaultFlags)
			}
		})
	}
}

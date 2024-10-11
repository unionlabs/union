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
	"strings"
	"testing"
)

func TestValidateFlagGroups(t *testing.T) {
	getCmd := func() *Command {
		c := &Command{
			Use: "testcmd",
			Run: func(cmd *Command, args []string) {
			}}
		// Define lots of flags to utilize for testing.
		for _, v := range []string{"a", "b", "c", "d"} {
			c.Flags().String(v, "", "")
		}
		for _, v := range []string{"e", "f", "g"} {
			c.PersistentFlags().String(v, "", "")
		}
		subC := &Command{
			Use: "subcmd",
			Run: func(cmd *Command, args []string) {
			}}
		subC.Flags().String("subonly", "", "")
		c.AddCommand(subC)
		return c
	}

	// Each test case uses a unique command from the function above.
	testcases := []struct {
		desc                        string
		flagGroupsRequired          []string
		flagGroupsOneRequired       []string
		flagGroupsExclusive         []string
		subCmdFlagGroupsRequired    []string
		subCmdFlagGroupsOneRequired []string
		subCmdFlagGroupsExclusive   []string
		args                        []string
		expectErr                   string
	}{
		{
			desc: "No flags no problem",
		}, {
			desc:                "No flags no problem even with conflicting groups",
			flagGroupsRequired:  []string{"a b"},
			flagGroupsExclusive: []string{"a b"},
		}, {
			desc:               "Required flag group not satisfied",
			flagGroupsRequired: []string{"a b c"},
			args:               []string{"--a=foo"},
			expectErr:          "if any flags in the group [a b c] are set they must all be set; missing [b c]",
		}, {
			desc:                  "One-required flag group not satisfied",
			flagGroupsOneRequired: []string{"a b"},
			args:                  []string{"--c=foo"},
			expectErr:             "at least one of the flags in the group [a b] is required",
		}, {
			desc:                "Exclusive flag group not satisfied",
			flagGroupsExclusive: []string{"a b c"},
			args:                []string{"--a=foo", "--b=foo"},
			expectErr:           "if any flags in the group [a b c] are set none of the others can be; [a b] were all set",
		}, {
			desc:               "Multiple required flag group not satisfied returns first error",
			flagGroupsRequired: []string{"a b c", "a d"},
			args:               []string{"--c=foo", "--d=foo"},
			expectErr:          `if any flags in the group [a b c] are set they must all be set; missing [a b]`,
		}, {
			desc:                  "Multiple one-required flag group not satisfied returns first error",
			flagGroupsOneRequired: []string{"a b", "d e"},
			args:                  []string{"--c=foo", "--f=foo"},
			expectErr:             `at least one of the flags in the group [a b] is required`,
		}, {
			desc:                "Multiple exclusive flag group not satisfied returns first error",
			flagGroupsExclusive: []string{"a b c", "a d"},
			args:                []string{"--a=foo", "--c=foo", "--d=foo"},
			expectErr:           `if any flags in the group [a b c] are set none of the others can be; [a c] were all set`,
		}, {
			desc:               "Validation of required groups occurs on groups in sorted order",
			flagGroupsRequired: []string{"a d", "a b", "a c"},
			args:               []string{"--a=foo"},
			expectErr:          `if any flags in the group [a b] are set they must all be set; missing [b]`,
		}, {
			desc:                  "Validation of one-required groups occurs on groups in sorted order",
			flagGroupsOneRequired: []string{"d e", "a b", "f g"},
			args:                  []string{"--c=foo"},
			expectErr:             `at least one of the flags in the group [a b] is required`,
		}, {
			desc:                "Validation of exclusive groups occurs on groups in sorted order",
			flagGroupsExclusive: []string{"a d", "a b", "a c"},
			args:                []string{"--a=foo", "--b=foo", "--c=foo"},
			expectErr:           `if any flags in the group [a b] are set none of the others can be; [a b] were all set`,
		}, {
			desc:                "Persistent flags utilize required and exclusive groups and can fail required groups",
			flagGroupsRequired:  []string{"a e", "e f"},
			flagGroupsExclusive: []string{"f g"},
			args:                []string{"--a=foo", "--f=foo", "--g=foo"},
			expectErr:           `if any flags in the group [a e] are set they must all be set; missing [e]`,
		}, {
			desc:                  "Persistent flags utilize one-required and exclusive groups and can fail one-required groups",
			flagGroupsOneRequired: []string{"a b", "e f"},
			flagGroupsExclusive:   []string{"e f"},
			args:                  []string{"--e=foo"},
			expectErr:             `at least one of the flags in the group [a b] is required`,
		}, {
			desc:                "Persistent flags utilize required and exclusive groups and can fail mutually exclusive groups",
			flagGroupsRequired:  []string{"a e", "e f"},
			flagGroupsExclusive: []string{"f g"},
			args:                []string{"--a=foo", "--e=foo", "--f=foo", "--g=foo"},
			expectErr:           `if any flags in the group [f g] are set none of the others can be; [f g] were all set`,
		}, {
			desc:                "Persistent flags utilize required and exclusive groups and can pass",
			flagGroupsRequired:  []string{"a e", "e f"},
			flagGroupsExclusive: []string{"f g"},
			args:                []string{"--a=foo", "--e=foo", "--f=foo"},
		}, {
			desc:                  "Persistent flags utilize one-required and exclusive groups and can pass",
			flagGroupsOneRequired: []string{"a e", "e f"},
			flagGroupsExclusive:   []string{"f g"},
			args:                  []string{"--a=foo", "--e=foo", "--f=foo"},
		}, {
			desc:                     "Subcmds can use required groups using inherited flags",
			subCmdFlagGroupsRequired: []string{"e subonly"},
			args:                     []string{"subcmd", "--e=foo", "--subonly=foo"},
		}, {
			desc:                        "Subcmds can use one-required groups using inherited flags",
			subCmdFlagGroupsOneRequired: []string{"e subonly"},
			args:                        []string{"subcmd", "--e=foo", "--subonly=foo"},
		}, {
			desc:                        "Subcmds can use one-required groups using inherited flags and fail one-required groups",
			subCmdFlagGroupsOneRequired: []string{"e subonly"},
			args:                        []string{"subcmd"},
			expectErr:                   "at least one of the flags in the group [e subonly] is required",
		}, {
			desc:                      "Subcmds can use exclusive groups using inherited flags",
			subCmdFlagGroupsExclusive: []string{"e subonly"},
			args:                      []string{"subcmd", "--e=foo", "--subonly=foo"},
			expectErr:                 "if any flags in the group [e subonly] are set none of the others can be; [e subonly] were all set",
		}, {
			desc:                      "Subcmds can use exclusive groups using inherited flags and pass",
			subCmdFlagGroupsExclusive: []string{"e subonly"},
			args:                      []string{"subcmd", "--e=foo"},
		}, {
			desc:                     "Flag groups not applied if not found on invoked command",
			subCmdFlagGroupsRequired: []string{"e subonly"},
			args:                     []string{"--e=foo"},
		},
	}
	for _, tc := range testcases {
		t.Run(tc.desc, func(t *testing.T) {
			c := getCmd()
			sub := c.Commands()[0]
			for _, flagGroup := range tc.flagGroupsRequired {
				c.MarkFlagsRequiredTogether(strings.Split(flagGroup, " ")...)
			}
			for _, flagGroup := range tc.flagGroupsOneRequired {
				c.MarkFlagsOneRequired(strings.Split(flagGroup, " ")...)
			}
			for _, flagGroup := range tc.flagGroupsExclusive {
				c.MarkFlagsMutuallyExclusive(strings.Split(flagGroup, " ")...)
			}
			for _, flagGroup := range tc.subCmdFlagGroupsRequired {
				sub.MarkFlagsRequiredTogether(strings.Split(flagGroup, " ")...)
			}
			for _, flagGroup := range tc.subCmdFlagGroupsOneRequired {
				sub.MarkFlagsOneRequired(strings.Split(flagGroup, " ")...)
			}
			for _, flagGroup := range tc.subCmdFlagGroupsExclusive {
				sub.MarkFlagsMutuallyExclusive(strings.Split(flagGroup, " ")...)
			}
			c.SetArgs(tc.args)
			err := c.Execute()
			switch {
			case err == nil && len(tc.expectErr) > 0:
				t.Errorf("Expected error %q but got nil", tc.expectErr)
			case err != nil && err.Error() != tc.expectErr:
				t.Errorf("Expected error %q but got %q", tc.expectErr, err)
			}
		})
	}
}

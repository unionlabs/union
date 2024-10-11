// Copyright 2015 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package buildutil_test

import (
	"bytes"
	"flag"
	"go/build"
	"os/exec"
	"reflect"
	"strings"
	"testing"

	"golang.org/x/tools/go/buildutil"
	"golang.org/x/tools/internal/testenv"
)

func TestTags(t *testing.T) {

	type tagTestCase struct {
		tags    string
		want    []string
		wantErr bool
	}

	for name, tc := range map[string]tagTestCase{
		// Normal valid cases
		"empty": {
			tags: "",
			want: []string{},
		},
		"commas": {
			tags: "tag1,tag_2,🐹,tag/3,tag-4",
			want: []string{"tag1", "tag_2", "🐹", "tag/3", "tag-4"},
		},
		"delimiters are spaces": {
			tags: "a b\tc\rd\ne",
			want: []string{"a", "b", "c", "d", "e"},
		},
		"old quote and space form": {
			tags: "'a' 'b' 'c'",
			want: []string{"a", "b", "c"},
		},

		// Normal error cases
		"unterminated": {
			tags:    `"missing closing quote`,
			want:    []string{},
			wantErr: true,
		},
		"unterminated single": {
			tags:    `'missing closing quote`,
			want:    []string{},
			wantErr: true,
		},

		// Maybe surprising difference for unterminated quotes, no spaces
		"unterminated no spaces": {
			tags: `"missing_closing_quote`,
			want: []string{"\"missing_closing_quote"},
		},
		"unterminated no spaces single": {
			tags:    `'missing_closing_quote`,
			want:    []string{},
			wantErr: true,
		},

		// Permitted but not recommended
		"delimiters contiguous spaces": {
			tags: "a \t\r\n, b \t\r\nc,d\te\tf",
			want: []string{"a", ",", "b", "c,d", "e", "f"},
		},
		"quotes and spaces": {
			tags: ` 'one'"two"	'three "four"'`,
			want: []string{"one", "two", "three \"four\""},
		},
		"quotes single no spaces": {
			tags: `'t1','t2',"t3"`,
			want: []string{"t1", ",'t2',\"t3\""},
		},
		"quotes double no spaces": {
			tags: `"t1","t2","t3"`,
			want: []string{`"t1"`, `"t2"`, `"t3"`},
		},
	} {
		t.Run(name, func(t *testing.T) {
			f := flag.NewFlagSet("TestTags", flag.ContinueOnError)
			var ctxt build.Context
			f.Var((*buildutil.TagsFlag)(&ctxt.BuildTags), "tags", buildutil.TagsFlagDoc)

			// Normal case valid parsed tags
			f.Parse([]string{"-tags", tc.tags, "rest"})

			// BuildTags
			if !reflect.DeepEqual(ctxt.BuildTags, tc.want) {
				t.Errorf("Case = %s, BuildTags = %q, want %q", name, ctxt.BuildTags, tc.want)
			}

			// Args()
			if want := []string{"rest"}; !reflect.DeepEqual(f.Args(), want) {
				t.Errorf("Case = %s, f.Args() = %q, want %q", name, f.Args(), want)
			}

			// Regression check against base go tooling
			cmd := testenv.Command(t, "go", "list", "-f", "{{context.BuildTags}}", "-tags", tc.tags, ".")
			var out bytes.Buffer
			cmd.Stdout = &out
			if err := cmd.Run(); err != nil {
				if ee, ok := err.(*exec.ExitError); ok && len(ee.Stderr) > 0 {
					t.Logf("stderr:\n%s", ee.Stderr)
				}
				if !tc.wantErr {
					t.Errorf("%v: %v", cmd, err)
				}
			} else if tc.wantErr {
				t.Errorf("Expected failure for %v", cmd)
			} else {
				wantDescription := strings.Join(tc.want, " ")
				output := strings.Trim(strings.TrimSuffix(out.String(), "\n"), "[]")
				if output != wantDescription {
					t.Errorf("Output = %s, want %s", output, wantDescription)
				}
			}
		})
	}
}

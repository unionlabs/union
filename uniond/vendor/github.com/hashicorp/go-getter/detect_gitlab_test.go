package getter

import (
	"testing"
)

func TestGitLabDetector(t *testing.T) {
	cases := []struct {
		Input  string
		Output string
	}{
		// HTTP
		{"gitlab.com/hashicorp/foo", "git::https://gitlab.com/hashicorp/foo.git"},
		{"gitlab.com/hashicorp/foo.git", "git::https://gitlab.com/hashicorp/foo.git"},
		{
			"gitlab.com/hashicorp/foo/bar",
			"git::https://gitlab.com/hashicorp/foo.git//bar",
		},
		{
			"gitlab.com/hashicorp/foo?foo=bar",
			"git::https://gitlab.com/hashicorp/foo.git?foo=bar",
		},
		{
			"gitlab.com/hashicorp/foo.git?foo=bar",
			"git::https://gitlab.com/hashicorp/foo.git?foo=bar",
		},
	}

	pwd := "/pwd"
	f := new(GitLabDetector)
	for i, tc := range cases {
		output, ok, err := f.Detect(tc.Input, pwd)
		if err != nil {
			t.Fatalf("err: %s", err)
		}
		if !ok {
			t.Fatal("not ok")
		}

		if output != tc.Output {
			t.Fatalf("%d: bad: %#v", i, output)
		}
	}
}

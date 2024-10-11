// Copyright (C) 2022 Michael J. Fromberger. All Rights Reserved.

package tomledit_test

import (
	"io/fs"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/creachadair/tomledit"
)

type testCase struct {
	name  string
	input string
}

func mustLoadTests(t *testing.T, dir string) []*testCase {
	t.Helper()

	var cases []*testCase
	if err := filepath.Walk(dir, func(path string, fi fs.FileInfo, err error) error {
		if err != nil {
			return err
		} else if filepath.Ext(fi.Name()) != ".toml" {
			return nil // skip
		}
		data, err := os.ReadFile(path)
		if err != nil {
			return err
		}
		stem := filepath.Base(filepath.Dir(path))
		name := strings.TrimSuffix(filepath.Base(path), ".toml")
		cases = append(cases, &testCase{
			name:  stem + "/" + name,
			input: string(data),
		})
		return nil
	}); err != nil {
		t.Fatalf("Loading tests failed: %v", err)
	}
	return cases
}

func TestCompliance(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipped compliance tests because -test.short is set")
	}
	t.Run("Valid", func(t *testing.T) {
		cases := mustLoadTests(t, "testdata/valid")
		for _, test := range cases {
			t.Run(test.name, func(t *testing.T) {
				r := strings.NewReader(test.input)
				if _, err := tomledit.Parse(r); err != nil {
					t.Errorf("Parse failed: %v", err)
				}
			})
		}
	})

	t.Run("Invalid", func(t *testing.T) {
		cases := mustLoadTests(t, "testdata/invalid")
		for _, test := range cases {
			t.Run(test.name, func(t *testing.T) {
				r := strings.NewReader(test.input)
				doc, err := tomledit.Parse(r)
				if err == nil {
					t.Errorf("Parse succeeded with %v", doc)
				} else {
					t.Logf("Parse correctly failed: %v", err)
				}
			})
		}
	})
}

package gen

import (
	"bytes"
	"io/ioutil"
	"path/filepath"
	"testing"

	"github.com/mmcloughlin/addchain/acc/parse"
	"github.com/mmcloughlin/addchain/acc/pass"
	"github.com/mmcloughlin/addchain/internal/test"
)

func TestBuiltinTemplatesGolden(t *testing.T) {
	d := LoadTestData(t)
	for _, name := range BuiltinTemplateNames() {
		name := name // scopelint
		t.Run(name, func(t *testing.T) {
			// Load the template.
			tmpl, err := BuiltinTemplate(name)
			if err != nil {
				t.Fatal(err)
			}

			// Generate output.
			var buf bytes.Buffer
			if err := Generate(&buf, tmpl, d); err != nil {
				t.Fatal(err)
			}
			got := buf.Bytes()

			// Compare to golden case.
			filename := test.GoldenName(filepath.Join("builtin", name))

			if test.Golden() {
				if err := ioutil.WriteFile(filename, got, 0644); err != nil {
					t.Fatalf("write golden file: %v", err)
				}
			}

			expect, err := ioutil.ReadFile(filename)
			if err != nil {
				t.Fatalf("read golden file: %v", err)
			}

			if !bytes.Equal(got, expect) {
				t.Fatal("output does not match golden file")
			}
		})
	}
}

func LoadTestData(t *testing.T) *Data {
	t.Helper()

	// Prepare data for a fixed test input.
	s, err := parse.File("testdata/input.acc")
	if err != nil {
		t.Fatal(err)
	}

	cfg := Config{
		Allocator: pass.Allocator{
			Input:  "x",
			Output: "z",
			Format: "t%d",
		},
	}

	d, err := PrepareData(cfg, s)
	if err != nil {
		t.Fatal(err)
	}

	return d
}

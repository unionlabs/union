package ensemble

import (
	"encoding/json"
	"io/ioutil"
	"reflect"
	"testing"

	"github.com/mmcloughlin/addchain/alg/exec"
	"github.com/mmcloughlin/addchain/internal/results"
	"github.com/mmcloughlin/addchain/internal/test"
)

// TestResults confirms that chain lengths from every ensemble algorithm remain
// unchanged. This is intended to provide confidence when making risky changes
// or refactors. The results package has its own similar test, but this is
// focussed on verifying that the best recorded result is still the same.
//
// The test uses the "golden test" technique, where we dump
// known-good results into golden files and later verify that we get the same
// result.
func TestResults(t *testing.T) {
	t.Parallel()

	as := Ensemble()
	for _, c := range results.Results {
		c := c // scopelint
		t.Run(c.Slug, func(t *testing.T) {
			t.Parallel()

			// Execute.
			ex := exec.NewParallel()
			rs := ex.Execute(c.Target(), as)

			// Summarize results in map from algorithm name to program length.
			got := map[string]int{}
			for _, r := range rs {
				if r.Err != nil {
					t.Fatalf("error with %s: %v", r.Algorithm, r.Err)
				}
				got[r.Algorithm.String()] = len(r.Program)
			}

			// If golden, write out results data file.
			filename := test.GoldenName(c.Slug)

			if test.Golden() {
				t.Logf("writing golden file %s", filename)
				b, err := json.MarshalIndent(got, "", "\t")
				if err != nil {
					t.Fatal(err)
				}
				if err := ioutil.WriteFile(filename, b, 0644); err != nil {
					t.Fatalf("write golden file: %v", err)
				}
			}

			// Load golden file.
			b, err := ioutil.ReadFile(filename)
			if err != nil {
				t.Fatalf("read golden file: %v", err)
			}

			var expect map[string]int
			if err := json.Unmarshal(b, &expect); err != nil {
				t.Fatal(err)
			}

			// Verify equal.
			if !reflect.DeepEqual(expect, got) {
				t.Fatal("results do not match golden file")
			}
		})
	}
}

func BenchmarkResults(b *testing.B) {
	as := Ensemble()
	for _, c := range results.Results {
		c := c // scopelint
		n := c.Target()
		b.Run(c.Slug, func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				for _, a := range as {
					if _, err := a.FindChain(n); err != nil {
						b.Fatal(err)
					}
				}
			}
		})
	}
}

package results

import (
	"flag"
	"log"
	"os"
	"sort"
	"testing"

	"github.com/mmcloughlin/addchain/acc"
	"github.com/mmcloughlin/addchain/alg/ensemble"
	"github.com/mmcloughlin/addchain/alg/exec"
	"github.com/mmcloughlin/addchain/internal/test"
)

// verbose flag for customizing log output from algorithm executor.
var verbose = flag.Bool("verbose", false, "enable verbose logging")

func TestResults(t *testing.T) {
	t.Parallel()

	as := ensemble.Ensemble()
	for _, c := range Results {
		c := c // scopelint
		t.Run(c.Slug, func(t *testing.T) {
			t.Parallel()

			// Tests with a best known result are prioritized. Only run all tests in
			// stress test mode.
			if c.BestKnown == 0 {
				test.RequireStress(t)
			} else {
				test.RequireLong(t)
			}

			n := c.Target()
			t.Logf("n-%d=%x", c.D, n)

			// Execute.
			ex := exec.NewParallel()
			if *verbose {
				ex.SetLogger(log.New(os.Stderr, "", log.Ltime|log.Lmicroseconds))
			}
			rs := ex.Execute(n, as)

			// Check for errors.
			for _, r := range rs {
				if r.Err != nil {
					t.Fatalf("error with %s: %v", r.Algorithm, r.Err)
				}
			}

			// Find the best.
			best := []exec.Result{rs[0]}
			for _, r := range rs[1:] {
				if len(r.Program) == len(best[0].Program) {
					best = append(best, r)
				} else if len(r.Program) < len(best[0].Program) {
					best = []exec.Result{r}
				}
			}

			sort.Slice(best, func(i, j int) bool {
				ai := best[i].Program.Adds()
				aj := best[j].Program.Adds()
				ni := best[i].Algorithm.String()
				nj := best[j].Algorithm.String()
				return ai < aj || (ai == aj && ni < nj)
			})

			// Report.
			for _, b := range best {
				doubles, adds := b.Program.Count()
				t.Logf("  alg: %s", b.Algorithm)
				t.Logf("total: %d\tadds: %d\tdoubles: %d", adds+doubles, adds, doubles)
			}
			b := best[0]
			length := len(b.Program)

			if c.BestKnown > 0 {
				t.Logf("known: %d", c.BestKnown)
				t.Logf("delta: %+d", length-c.BestKnown)
			}

			// Ensure the recorded best length and algorithm name are correct.
			if c.Length != length {
				t.Errorf("incorrect best value %d; expect %d", length, c.Length)
			}
			if c.AlgorithmName != b.Algorithm.String() {
				t.Errorf("incorrect algorithm name %q; expect %q", c.AlgorithmName, b.Algorithm)
			}

			// Compare to golden file.
			golden, err := acc.LoadFile(test.GoldenName(c.Slug))
			if err != nil {
				t.Logf("failed to load golden file: %s", err)
			}

			save := test.Golden()
			switch {
			case golden == nil:
				t.Errorf("missing golden file")
				save = true
			case len(golden.Program) < length:
				t.Errorf("regression from golden: %d to %d", len(golden.Program), length)
			case len(golden.Program) > len(b.Program):
				t.Logf("improvement: %d to %d", len(golden.Program), length)
				save = true
			}

			if save {
				t.Log("saving golden file")

				r, err := acc.Decompile(b.Program)
				if err != nil {
					t.Fatal(err)
				}

				if err := acc.Save(test.GoldenName(c.Slug), r); err != nil {
					t.Fatal(err)
				}
			}
		})
	}
}

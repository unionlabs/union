package generic_test

// This is package generic_test in order to get around an import cycle: this
// package imports teststat to do its testing, but package teststat imports
// generic to use its Histogram in the Quantiles helper function.

import (
	"go/ast"
	"go/importer"
	"go/parser"
	"go/token"
	"go/types"
	"io/ioutil"
	"math"
	"math/rand"
	"sync"
	"testing"

	"github.com/go-kit/kit/metrics/generic"
	"github.com/go-kit/kit/metrics/teststat"
)

func TestCounter(t *testing.T) {
	name := "my_counter"
	counter := generic.NewCounter(name).With("label", "counter").(*generic.Counter)
	if want, have := name, counter.Name; want != have {
		t.Errorf("Name: want %q, have %q", want, have)
	}
	value := counter.Value
	if err := teststat.TestCounter(counter, value); err != nil {
		t.Fatal(err)
	}
}

func TestValueReset(t *testing.T) {
	counter := generic.NewCounter("test_value_reset")
	counter.Add(123)
	counter.Add(456)
	counter.Add(789)
	if want, have := float64(123+456+789), counter.ValueReset(); want != have {
		t.Errorf("want %f, have %f", want, have)
	}
	if want, have := float64(0), counter.Value(); want != have {
		t.Errorf("want %f, have %f", want, have)
	}
}

func TestGauge(t *testing.T) {
	name := "my_gauge"
	gauge := generic.NewGauge(name).With("label", "gauge").(*generic.Gauge)
	if want, have := name, gauge.Name; want != have {
		t.Errorf("Name: want %q, have %q", want, have)
	}
	value := func() []float64 { return []float64{gauge.Value()} }
	if err := teststat.TestGauge(gauge, value); err != nil {
		t.Fatal(err)
	}
}

func TestHistogram(t *testing.T) {
	name := "my_histogram"
	histogram := generic.NewHistogram(name, 50).With("label", "histogram").(*generic.Histogram)
	if want, have := name, histogram.Name; want != have {
		t.Errorf("Name: want %q, have %q", want, have)
	}
	quantiles := func() (float64, float64, float64, float64) {
		return histogram.Quantile(0.50), histogram.Quantile(0.90), histogram.Quantile(0.95), histogram.Quantile(0.99)
	}
	if err := teststat.TestHistogram(histogram, quantiles, 0.01); err != nil {
		t.Fatal(err)
	}
}

func TestIssue424(t *testing.T) {
	var (
		histogram   = generic.NewHistogram("dont_panic", 50)
		concurrency = 100
		operations  = 1000
		wg          sync.WaitGroup
	)

	wg.Add(concurrency)
	for i := 0; i < concurrency; i++ {
		go func() {
			defer wg.Done()
			for j := 0; j < operations; j++ {
				histogram.Observe(float64(j))
				histogram.Observe(histogram.Quantile(0.5))
			}
		}()
	}
	wg.Wait()
}

func TestSimpleHistogram(t *testing.T) {
	histogram := generic.NewSimpleHistogram().With("label", "simple_histogram").(*generic.SimpleHistogram)
	var (
		sum   int
		count = 1234 // not too big
	)
	for i := 0; i < count; i++ {
		value := rand.Intn(1000)
		sum += value
		histogram.Observe(float64(value))
	}

	var (
		want      = float64(sum) / float64(count)
		have      = histogram.ApproximateMovingAverage()
		tolerance = 0.001 // real real slim
	)
	if math.Abs(want-have)/want > tolerance {
		t.Errorf("want %f, have %f", want, have)
	}
}

// Naive atomic alignment test.
// The problem is related to the use of `atomic.*` and not directly to a structure.
// But currently works for Counter and Gauge.
// To have a more solid test, this test should be removed and the other tests should be run on a 32-bit arch.
func TestAtomicAlignment(t *testing.T) {
	content, err := ioutil.ReadFile("./generic.go")
	if err != nil {
		t.Fatal(err)
	}

	fset := token.NewFileSet()

	file, err := parser.ParseFile(fset, "generic.go", content, parser.ParseComments)
	if err != nil {
		t.Fatal(err)
	}

	conf := types.Config{Importer: importer.ForCompiler(fset, "source", nil)}

	pkg, err := conf.Check(".", fset, []*ast.File{file}, nil)
	if err != nil {
		t.Fatal(err)
	}

	// uses ARM as reference for 32-bit arch
	sizes := types.SizesFor("gc", "arm")

	names := []string{"Counter", "Gauge"}

	for _, name := range names {
		t.Run(name, func(t *testing.T) {
			checkAtomicAlignment(t, sizes, pkg.Scope().Lookup(name), pkg)
		})
	}
}

func checkAtomicAlignment(t *testing.T, sizes types.Sizes, obj types.Object, pkg *types.Package) {
	t.Helper()

	st := obj.Type().Underlying().(*types.Struct)

	posToCheck := make(map[int]types.Type)

	var vars []*types.Var
	for i := 0; i < st.NumFields(); i++ {
		field := st.Field(i)

		if v, ok := field.Type().(*types.Basic); ok {
			switch v.Kind() {
			case types.Uint64, types.Float64, types.Int64:
				posToCheck[i] = v
			}
		}

		vars = append(vars, types.NewVar(field.Pos(), pkg, field.Name(), field.Type()))
	}

	offsets := sizes.Offsetsof(vars)
	for i, offset := range offsets {
		if _, ok := posToCheck[i]; !ok {
			continue
		}

		if offset%8 != 0 {
			t.Errorf("misalignment detected in %s for the type %s, offset %d", obj.Name(), posToCheck[i], offset)
		}
	}
}

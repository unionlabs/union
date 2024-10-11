// Copyright 2016 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package apd

import (
	"bufio"
	"bytes"
	"flag"
	"fmt"
	"io"
	"io/ioutil"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"testing"
	"time"
)

const testDir = "testdata"

var (
	flagFailFast   = flag.Bool("fast", false, "stop work after first error; disables parallel testing")
	flagIgnore     = flag.Bool("ignore", false, "print ignore lines on errors")
	flagNoParallel = flag.Bool("noparallel", false, "disables parallel testing")
	flagTime       = flag.Duration("time", 0, "interval at which to print long-running functions; 0 disables")
)

type TestCase struct {
	Precision                int
	MaxExponent, MinExponent int
	Rounding                 string
	Extended, Clamp          bool

	ID         string
	Operation  string
	Operands   []string
	Result     string
	Conditions []string
}

func (tc TestCase) HasNull() bool {
	if tc.Result == "#" {
		return true
	}
	for _, o := range tc.Operands {
		if o == "#" {
			return true
		}
	}
	return false
}

func (tc TestCase) SkipPrecision() bool {
	switch tc.Operation {
	case "tosci", "toeng", "apply":
		return false
	default:
		return true
	}
}

func ParseDecTest(r io.Reader) ([]TestCase, error) {
	scanner := bufio.NewScanner(r)
	tc := TestCase{
		Extended: true,
	}
	var err error
	var res []TestCase

	for scanner.Scan() {
		text := scanner.Text()
		// TODO(mjibson): support these test cases
		if strings.Contains(text, "#") {
			continue
		}
		line := strings.Fields(strings.ToLower(text))
		for i, t := range line {
			if strings.HasPrefix(t, "--") {
				line = line[:i]
				break
			}
		}
		if len(line) == 0 {
			continue
		}
		if strings.HasSuffix(line[0], ":") {
			if len(line) != 2 {
				return nil, fmt.Errorf("expected 2 tokens, got %q", text)
			}
			switch directive := line[0]; directive[:len(directive)-1] {
			case "precision":
				tc.Precision, err = strconv.Atoi(line[1])
				if err != nil {
					return nil, err
				}
			case "maxexponent":
				tc.MaxExponent, err = strconv.Atoi(line[1])
				if err != nil {
					return nil, err
				}
			case "minexponent":
				tc.MinExponent, err = strconv.Atoi(line[1])
				if err != nil {
					return nil, err
				}
			case "rounding":
				tc.Rounding = line[1]
			case "version":
				// ignore
			case "extended":
				tc.Extended = line[1] == "1"
			case "clamp":
				tc.Clamp = line[1] == "1"
			default:
				return nil, fmt.Errorf("unsupported directive: %s", directive)
			}
		} else {
			if len(line) < 5 {
				return nil, fmt.Errorf("short test case line: %q", text)
			}
			tc.ID = line[0]
			tc.Operation = line[1]
			tc.Operands = nil
			var ops []string
			line = line[2:]
			for i, o := range line {
				if o == "->" {
					tc.Operands = ops
					line = line[i+1:]
					break
				}
				o = cleanNumber(o)
				ops = append(ops, o)
			}
			if tc.Operands == nil || len(line) < 1 {
				return nil, fmt.Errorf("bad test case line: %q", text)
			}
			tc.Result = strings.ToUpper(cleanNumber(line[0]))
			tc.Conditions = line[1:]
			res = append(res, tc)
		}
	}
	if err := scanner.Err(); err != nil {
		return nil, err
	}
	return res, nil
}

func cleanNumber(s string) string {
	if len(s) > 1 && s[0] == '\'' && s[len(s)-1] == '\'' {
		s = s[1 : len(s)-1]
		s = strings.Replace(s, `''`, `'`, -1)
	} else if len(s) > 1 && s[0] == '"' && s[len(s)-1] == '"' {
		s = s[1 : len(s)-1]
		s = strings.Replace(s, `""`, `"`, -1)
	}
	return s
}

func TestParseDecTest(t *testing.T) {
	files, err := ioutil.ReadDir(testDir)
	if err != nil {
		t.Fatal(err)
	}
	for _, fi := range files {
		t.Run(fi.Name(), func(t *testing.T) {
			f, err := os.Open(filepath.Join(testDir, fi.Name()))
			if err != nil {
				t.Fatal(err)
			}
			defer f.Close()
			_, err = ParseDecTest(f)
			if err != nil {
				t.Fatal(err)
			}
		})
	}
}

var GDAfiles = []string{
	"abs",
	"add",
	"base",
	"compare",
	"comparetotal",
	"divide",
	"divideint",
	"exp",
	"ln",
	"log10",
	"minus",
	"multiply",
	"plus",
	"power",
	"powersqrt",
	"quantize",
	"randoms",
	"reduce",
	"remainder",
	"rounding",
	"squareroot",
	"subtract",
	"tointegral",
	"tointegralx",

	// non-GDA tests
	"cuberoot-apd",
}

func TestGDA(t *testing.T) {
	var buf bytes.Buffer
	fmt.Fprintf(&buf, "%10s%8s%8s%8s%8s%8s%8s\n", "name", "total", "success", "fail", "ignore", "skip", "missing")
	for _, fname := range GDAfiles {
		succeed := t.Run(fname, func(t *testing.T) {
			path, tcs := readGDA(t, fname)
			gdaTest(t, path, tcs)
		})
		if !succeed && *flagFailFast {
			break
		}
	}
}

func (tc TestCase) Run(c *Context, done chan error, d, x, y *Decimal) (res Condition, err error) {
	switch tc.Operation {
	case "abs":
		res, err = c.Abs(d, x)
	case "add":
		res, err = c.Add(d, x, y)
	case "compare":
		res, err = c.Cmp(d, x, y)
	case "cuberoot":
		res, err = c.Cbrt(d, x)
	case "divide":
		res, err = c.Quo(d, x, y)
	case "divideint":
		res, err = c.QuoInteger(d, x, y)
	case "exp":
		res, err = c.Exp(d, x)
	case "ln":
		res, err = c.Ln(d, x)
	case "log10":
		res, err = c.Log10(d, x)
	case "minus":
		res, err = c.Neg(d, x)
	case "multiply":
		res, err = c.Mul(d, x, y)
	case "plus":
		res, err = c.Add(d, x, decimalZero)
	case "power":
		res, err = c.Pow(d, x, y)
	case "quantize":
		res, err = c.Quantize(d, x, y.Exponent)
	case "reduce":
		_, res, err = c.Reduce(d, x)
	case "remainder":
		res, err = c.Rem(d, x, y)
	case "squareroot":
		res, err = c.Sqrt(d, x)
	case "subtract":
		res, err = c.Sub(d, x, y)
	case "tointegral":
		res, err = c.RoundToIntegralValue(d, x)
	case "tointegralx":
		res, err = c.RoundToIntegralExact(d, x)

	// Below used only in benchmarks. Tests call it themselves.
	case "comparetotal":
		x.CmpTotal(y)
	case "tosci":
		_ = x.String()

	default:
		done <- fmt.Errorf("unknown operation: %s", tc.Operation)
	}
	return
}

// BenchmarkGDA benchmarks a GDA test. It should not be used without specifying
// a sub-benchmark to run. For example:
// go test -run XX -bench GDA/squareroot
func BenchmarkGDA(b *testing.B) {
	for _, fname := range GDAfiles {
		b.Run(fname, func(b *testing.B) {
			b.StopTimer()
			_, tcs := readGDA(b, fname)
			res := new(Decimal)
			for i := 0; i < b.N; i++ {
			Loop:
				for _, tc := range tcs {
					if GDAignore[tc.ID] || tc.Result == "?" || tc.HasNull() {
						continue
					}
					switch tc.Operation {
					case "apply", "toeng":
						continue
					}
					operands := make([]*Decimal, 2)
					for i, o := range tc.Operands {
						d, _, err := NewFromString(o)
						if err != nil {
							continue Loop
						}
						operands[i] = d
					}
					c := tc.Context(b)
					b.StartTimer()
					// Ignore errors here because the full tests catch them.
					_, _ = tc.Run(c, nil, res, operands[0], operands[1])
					b.StopTimer()
				}
			}
		})
	}
}

func readGDA(t testing.TB, name string) (string, []TestCase) {
	path := filepath.Join(testDir, name+".decTest")
	f, err := os.Open(path)
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()
	tcs, err := ParseDecTest(f)
	if err != nil {
		t.Fatal(err)
	}
	return path, tcs
}

func (tc TestCase) Context(t testing.TB) *Context {
	_, ok := Roundings[tc.Rounding]
	if !ok {
		t.Fatalf("unsupported rounding mode %s", tc.Rounding)
	}
	c := &Context{
		Precision:   uint32(tc.Precision),
		MaxExponent: int32(tc.MaxExponent),
		MinExponent: int32(tc.MinExponent),
		Rounding:    tc.Rounding,
		Traps:       0,
	}
	return c
}

func gdaTest(t *testing.T, path string, tcs []TestCase) {
	for _, tc := range tcs {
		tc := tc
		succeed := t.Run(tc.ID, func(t *testing.T) {
			if *flagTime > 0 {
				timeDone := make(chan struct{}, 1)
				go func() {
					start := time.Now()
					for {
						select {
						case <-timeDone:
							return
						case <-time.After(*flagTime):
							fmt.Println(tc.ID, "running for", time.Since(start))
						}
					}
				}()
				defer func() { timeDone <- struct{}{} }()
			}
			defer func() {
				if t.Failed() {
					if *flagIgnore {
						tc.PrintIgnore()
					}
				}
			}()
			if GDAignore[tc.ID] {
				t.Skip("ignored")
			}
			if tc.HasNull() {
				t.Skip("has null")
			}
			switch tc.Operation {
			case "toeng", "apply":
				t.Skip("unsupported")
			}
			if !*flagNoParallel && !*flagFailFast {
				t.Parallel()
			}
			// helpful acme address link
			t.Logf("%s:/^%s ", path, tc.ID)
			t.Logf("%s %s = %s (%s)", tc.Operation, strings.Join(tc.Operands, " "), tc.Result, strings.Join(tc.Conditions, " "))
			t.Logf("prec: %d, round: %s, Emax: %d, Emin: %d", tc.Precision, tc.Rounding, tc.MaxExponent, tc.MinExponent)
			_, ok := Roundings[tc.Rounding]
			if !ok {
				t.Fatalf("unsupported rounding mode %s", tc.Rounding)
			}
			operands := make([]*Decimal, 2)
			c := tc.Context(t)
			var res, opres Condition
			opctx := c
			if tc.SkipPrecision() {
				opctx = opctx.WithPrecision(1000)
				opctx.MaxExponent = MaxExponent
				opctx.MinExponent = MinExponent
			}
			for i, o := range tc.Operands {
				d, ores, err := opctx.NewFromString(o)
				expectError := tc.Result == "NAN" && strings.Join(tc.Conditions, "") == "conversion_syntax"
				if err != nil {
					if expectError {
						// Successfully detected bad syntax.
						return
					}
					switch tc.Operation {
					case "tosci":
						// Skip cases with exponents larger than we will parse.
						if strings.Contains(err.Error(), "value out of range") {
							return
						}
					}
					testExponentError(t, err)
					if tc.Result == "?" {
						return
					}
					t.Logf("%v, %v, %v", tc.Result, tc.Conditions, tc.Operation)
					t.Fatalf("operand %d: %s: %+v", i, o, err)
				} else if expectError {
					t.Fatalf("expected error, got %s", d)
				}
				operands[i] = d
				opres |= ores
			}
			switch tc.Operation {
			case "power":
				tmp := new(Decimal).Abs(operands[1])
				// We don't handle power near the max exp limit.
				if tmp.Cmp(New(MaxExponent, 0)) >= 0 {
					t.Skip("x ** large y")
				}
				if tmp.Cmp(New(int64(c.MaxExponent), 0)) >= 0 {
					t.Skip("x ** large y")
				}
			case "quantize":
				if operands[1].Form != Finite {
					t.Skip("quantize requires finite second operand")
				}
			}
			var s string
			// Fill d with bogus data to make sure all fields are correctly set.
			d := &Decimal{
				Form:     -2,
				Negative: true,
				Exponent: -6437897,
			}
			// Use d1 and d2 to verify that the result can be the same as the first and
			// second operand.
			var d1, d2 *Decimal
			d.Coeff.SetInt64(9221)
			start := time.Now()
			defer func() {
				t.Logf("duration: %s", time.Since(start))
			}()

			done := make(chan error, 1)
			var err error
			go func() {
				switch tc.Operation {
				case "tosci":
					s = operands[0].String()
					// non-extended tests don't retain exponents for 0
					if !tc.Extended && operands[0].IsZero() {
						s = "0"
					}
					// Clear d's bogus data.
					d.Set(operands[0])
					// Set d1 to prevent the result-equals-operand check failing.
					d1 = d
				case "comparetotal":
					var c int
					c = operands[0].CmpTotal(operands[1])
					d.SetInt64(int64(c))
				default:
					var wg sync.WaitGroup
					wg.Add(2)
					// Check that the result is correct even if it is either argument. Use some
					// go routines since we are running tc.Run three times.
					go func() {
						d1 = new(Decimal).Set(operands[0])
						tc.Run(c, done, d1, d1, operands[1])
						wg.Done()
					}()
					go func() {
						if operands[1] != nil {
							d2 = new(Decimal).Set(operands[1])
							tc.Run(c, done, d2, operands[0], d2)
						}
						wg.Done()
					}()
					res, err = tc.Run(c, done, d, operands[0], operands[1])
					wg.Wait()
				}
				done <- nil
			}()
			select {
			case err := <-done:
				if err != nil {
					t.Fatal(err)
				}
			case <-time.After(time.Second * 5):
				t.Fatalf("timeout")
			}
			if d.Coeff.Sign() < 0 {
				t.Fatalf("negative coeff: %s", d.Coeff.String())
			}
			// Make sure the bogus Form above got cleared.
			if d.Form < 0 {
				t.Fatalf("unexpected form: %#v", d)
			}
			// Verify the operands didn't change.
			for i, o := range tc.Operands {
				v := newDecimal(t, opctx, o)
				if v.CmpTotal(operands[i]) != 0 {
					t.Fatalf("operand %d changed from %s to %s", i, o, operands[i])
				}
			}
			// Verify the result-equals-operand worked correctly.
			if d1 != nil && d.CmpTotal(d1) != 0 {
				t.Errorf("first operand as result mismatch: got %s, expected %s", d1, d)
			}
			if d2 != nil && d.CmpTotal(d2) != 0 {
				t.Errorf("second operand as result mismatch: got %s, expected %s", d2, d)
			}
			if !GDAignoreFlags[tc.ID] {
				var rcond Condition
				for _, cond := range tc.Conditions {
					switch cond {
					case "underflow":
						rcond |= Underflow
					case "inexact":
						rcond |= Inexact
					case "overflow":
						rcond |= Overflow
					case "subnormal":
						rcond |= Subnormal
					case "division_undefined":
						rcond |= DivisionUndefined
					case "division_by_zero":
						rcond |= DivisionByZero
					case "division_impossible":
						rcond |= DivisionImpossible
					case "invalid_operation":
						rcond |= InvalidOperation
					case "rounded":
						rcond |= Rounded
					case "clamped":
						rcond |= Clamped

					case "invalid_context":
						// ignore

					default:
						t.Fatalf("unknown condition: %s", cond)
					}
				}

				switch tc.Operation {
				case "tosci":
					// We only care about the operand flags for the string conversion operations.
					res |= opres
				}

				t.Logf("want flags (%d): %s", rcond, rcond)
				t.Logf("have flags (%d): %s", res, res)

				// TODO(mjibson): after upscaling, operations need to remove the 0s added
				// after the operation is done. Since this isn't happening, things are being
				// rounded when they shouldn't because the coefficient has so many trailing 0s.
				// Manually remove Rounded flag from context until the TODO is fixed.
				res &= ^Rounded
				rcond &= ^Rounded

				switch tc.Operation {
				case "log10":
					// TODO(mjibson): Under certain conditions these are exact, but we don't
					// correctly mark them. Ignore these flags for now.
					// squareroot sometimes marks things exact when GDA says they should be
					// inexact.
					rcond &= ^Inexact
					res &= ^Inexact
				}

				// Don't worry about these flags; they are handled by GoError.
				res &= ^SystemOverflow
				res &= ^SystemUnderflow

				if (res.Overflow() || res.Underflow()) && (strings.HasPrefix(tc.ID, "rpow") ||
					strings.HasPrefix(tc.ID, "powr")) {
					t.Skip("overflow")
				}

				// Ignore Clamped on error.
				if tc.Result == "?" {
					rcond &= ^Clamped
					res &= ^Clamped
				}

				if rcond != res {
					if tc.Operation == "power" && (res.Overflow() || res.Underflow()) {
						t.Skip("power overflow")
					}
					t.Logf("got: %s (%#v)", d, d)
					t.Logf("error: %+v", err)
					t.Errorf("expected flags %q (%d); got flags %q (%d)", rcond, rcond, res, res)
				}
			}

			if tc.Result == "?" {
				if err != nil {
					return
				}
				t.Fatalf("expected error, got %s", d)
			}
			if err != nil {
				testExponentError(t, err)
				if tc.Operation == "power" && (res.Overflow() || res.Underflow()) {
					t.Skip("power overflow")
				}
				t.Fatalf("%+v", err)
			}
			switch tc.Operation {
			case "tosci", "toeng":
				if strings.HasPrefix(tc.Result, "-NAN") {
					tc.Result = "-NaN"
				}
				if strings.HasPrefix(tc.Result, "-SNAN") {
					tc.Result = "-sNaN"
				}
				if strings.HasPrefix(tc.Result, "NAN") {
					tc.Result = "NaN"
				}
				if strings.HasPrefix(tc.Result, "SNAN") {
					tc.Result = "sNaN"
				}
				if !strings.EqualFold(s, tc.Result) {
					t.Fatalf("expected %s, got %s", tc.Result, s)
				}
				return
			}
			r := newDecimal(t, testCtx, tc.Result)
			var equal bool
			if d.Form == Finite {
				// Don't worry about trailing zeros being inequal in CmpTotal.
				equal = d.Cmp(r) == 0 && d.Negative == r.Negative
			} else {
				equal = d.CmpTotal(r) == 0
			}
			if !equal {
				t.Logf("want: %s", tc.Result)
				t.Logf("got: %s (%#v)", d, d)
				// Some operations allow 1ulp of error in tests.
				switch tc.Operation {
				case "exp", "ln", "log10", "power":
					nc := c.WithPrecision(0)
					nc.Sub(d, d, r)
					if d.Coeff.Cmp(bigOne) == 0 {
						t.Logf("pass: within 1ulp: %s, %s", d, r)
						return
					}
				}
				t.Fatalf("unexpected result")
			} else {
				t.Logf("got: %s (%#v)", d, d)
			}
		})
		if !succeed {
			if *flagFailFast {
				break
			}
		}
	}
}

func (tc TestCase) PrintIgnore() {
	fmt.Printf("	\"%s\": true,\n", tc.ID)
}

var GDAignore = map[string]bool{
	// GDA says decNumber should skip these
	"powx4302": true,
	"powx4303": true,

	// Invalid context
	"expx901":  true,
	"expx902":  true,
	"expx903":  true,
	"expx905":  true,
	"lnx901":   true,
	"lnx902":   true,
	"lnx903":   true,
	"lnx905":   true,
	"logx901":  true,
	"logx902":  true,
	"logx903":  true,
	"logx905":  true,
	"powx4001": true,
	"powx4002": true,
	"powx4003": true,
	"powx4005": true,

	// NaN payloads with weird digits
	"basx725": true,
	"basx745": true,

	// NaN payloads
	"cotx970": true,
	"cotx973": true,
	"cotx974": true,
	"cotx977": true,
	"cotx980": true,
	"cotx983": true,
	"cotx984": true,
	"cotx987": true,
	"cotx994": true,

	// too large exponents, supposed to fail anyway
	"quax525": true,
	"quax531": true,
	"quax805": true,
	"quax806": true,
	"quax807": true,
	"quax808": true,
	"quax809": true,
	"quax810": true,
	"quax811": true,
	"quax812": true,
	"quax813": true,
	"quax814": true,
	"quax815": true,
	"quax816": true,
	"quax817": true,
	"quax818": true,
	"quax819": true,
	"quax820": true,
	"quax821": true,
	"quax822": true,
	"quax861": true,
	"quax862": true,
	"quax866": true,

	// TODO(mjibson): fix tests below

	// log10(x) with large exponents, overflows
	"logx0020": true,
	"logx1146": true,
	"logx1147": true,
	"logx1156": true,
	"logx1157": true,
	"logx1176": true,
	"logx1177": true,

	// log10(x) where x is 1.0 +/- some tiny epsilon. Our results are close but
	// differ in the last places.
	"logx1304": true,
	"logx1309": true,
	"logx1310": true,

	// overflows to infinity
	"powx4125": true,
	"powx4145": true,

	// exceeds system overflow
	"expx291": true,
	"expx292": true,
	"expx293": true,
	"expx294": true,
	"expx295": true,
	"expx296": true,

	// inexact zeros
	"addx1633":  true,
	"addx1634":  true,
	"addx1638":  true,
	"addx61633": true,
	"addx61634": true,
	"addx61638": true,

	// should be -0E-398, got -1E-398
	"addx1613":  true,
	"addx1614":  true,
	"addx1618":  true,
	"addx61613": true,
	"addx61614": true,
	"addx61618": true,

	// extreme input range, but should work
	"lnx0902":  true,
	"sqtx8137": true,
	"sqtx8139": true,
	"sqtx8145": true,
	"sqtx8147": true,
	"sqtx8149": true,
	"sqtx8150": true,
	"sqtx8151": true,
	"sqtx8158": true,
	"sqtx8165": true,
	"sqtx8168": true,
	"sqtx8174": true,
	"sqtx8175": true,
	"sqtx8179": true,
	"sqtx8182": true,
	"sqtx8185": true,
	"sqtx8186": true,
	"sqtx8195": true,
	"sqtx8196": true,
	"sqtx8197": true,
	"sqtx8199": true,
	"sqtx8204": true,
	"sqtx8212": true,
	"sqtx8213": true,
	"sqtx8214": true,
	"sqtx8218": true,
	"sqtx8219": true,
	"sqtx8323": true,
	"sqtx8324": true,
	"sqtx8331": true,
	"sqtx8626": true,
	"sqtx8627": true,
	"sqtx8628": true,
	"sqtx8631": true,
	"sqtx8632": true,
	"sqtx8633": true,
	"sqtx8634": true,
	"sqtx8636": true,
	"sqtx8637": true,
	"sqtx8639": true,
	"sqtx8640": true,
	"sqtx8641": true,
	"sqtx8644": true,
	"sqtx8645": true,
	"sqtx8646": true,
	"sqtx8647": true,
	"sqtx8648": true,
	"sqtx8649": true,
	"sqtx8650": true,
	"sqtx8651": true,
	"sqtx8652": true,
	"sqtx8654": true,

	// tricky cases of underflow subnormals
	"sqtx8700": true,
	"sqtx8701": true,
	"sqtx8702": true,
	"sqtx8703": true,
	"sqtx8704": true,
	"sqtx8705": true,
	"sqtx8706": true,
	"sqtx8707": true,
	"sqtx8708": true,
	"sqtx8709": true,
	"sqtx8710": true,
	"sqtx8711": true,
	"sqtx8712": true,
	"sqtx8713": true,
	"sqtx8714": true,
	"sqtx8715": true,
	"sqtx8716": true,
	"sqtx8717": true,
	"sqtx8718": true,
	"sqtx8719": true,
	"sqtx8720": true,
	"sqtx8721": true,
	"sqtx8722": true,
	"sqtx8723": true,
	"sqtx8724": true,
	"sqtx8725": true,
	"sqtx8726": true,
	"sqtx8727": true,
	"sqtx8728": true,
	"sqtx8729": true,
}

var GDAignoreFlags = map[string]bool{
	// unflagged clamped
	"sqtx9024": true,
	"sqtx9025": true,
	"sqtx9026": true,
	"sqtx9027": true,
	"sqtx9038": true,
	"sqtx9039": true,
	"sqtx9040": true,
	"sqtx9045": true,

	// missing underflow, subnormal
	"expx048": true,
	"expx756": true,
	"expx757": true,
}

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

package apd_test

import (
	"fmt"

	"github.com/cockroachdb/apd/v2"
)

// ExampleOverflow demonstrates how to detect or error on overflow.
func ExampleContext_overflow() {
	// Create a context that will overflow at 1e3.
	c := apd.Context{
		MaxExponent: 2,
		Traps:       apd.Overflow,
	}
	one := apd.New(1, 0)
	d := apd.New(997, 0)
	for {
		res, err := c.Add(d, d, one)
		fmt.Printf("d: %8s, overflow: %5v, err: %v\n", d, res.Overflow(), err)
		if err != nil {
			return
		}
	}
	// Output: d:      998, overflow: false, err: <nil>
	// d:      999, overflow: false, err: <nil>
	// d: Infinity, overflow:  true, err: overflow
}

// ExampleInexact demonstrates how to detect inexact operations.
func ExampleContext_inexact() {
	d := apd.New(27, 0)
	three := apd.New(3, 0)
	c := apd.BaseContext.WithPrecision(5)
	for {
		res, err := c.Quo(d, d, three)
		fmt.Printf("d: %7s, inexact: %5v, err: %v\n", d, res.Inexact(), err)
		if err != nil {
			return
		}
		if res.Inexact() {
			return
		}
	}
	// Output: d:       9, inexact: false, err: <nil>
	// d:       3, inexact: false, err: <nil>
	// d:       1, inexact: false, err: <nil>
	// d: 0.33333, inexact:  true, err: <nil>
}

func ExampleContext_Quantize() {
	input, _, _ := apd.NewFromString("123.45")
	output := new(apd.Decimal)
	c := apd.BaseContext.WithPrecision(10)
	for i := int32(-3); i <= 3; i++ {
		res, _ := c.Quantize(output, input, i)
		fmt.Printf("%2v: %s", i, output)
		if res != 0 {
			fmt.Printf(" (%s)", res)
		}
		fmt.Println()
	}
	// Output: -3: 123.450
	// -2: 123.45
	// -1: 123.5 (inexact, rounded)
	//  0: 123 (inexact, rounded)
	//  1: 1.2E+2 (inexact, rounded)
	//  2: 1E+2 (inexact, rounded)
	//  3: 0E+3 (inexact, rounded)
}

func ExampleErrDecimal() {
	c := apd.BaseContext.WithPrecision(5)
	ed := apd.MakeErrDecimal(c)
	d := apd.New(10, 0)
	fmt.Printf("%s, err: %v\n", d, ed.Err())
	ed.Add(d, d, apd.New(2, 1)) // add 20
	fmt.Printf("%s, err: %v\n", d, ed.Err())
	ed.Quo(d, d, apd.New(0, 0)) // divide by zero
	fmt.Printf("%s, err: %v\n", d, ed.Err())
	ed.Sub(d, d, apd.New(1, 0)) // attempt to subtract 1
	// The subtraction doesn't occur and doesn't change the error.
	fmt.Printf("%s, err: %v\n", d, ed.Err())
	// Output: 10, err: <nil>
	// 30, err: <nil>
	// Infinity, err: division by zero
	// Infinity, err: division by zero
}

// ExampleRoundToIntegralExact demonstrates how to use RoundToIntegralExact to
// check if a number is an integer or not. Note the variations between integer
// (which allows zeros after the decimal point) and strict (which does not). See
// the documentation on Inexact and Rounded.
func ExampleContext_RoundToIntegralExact() {
	inputs := []string{
		"123.4",
		"123.0",
		"123",
		"12E1",
		"120E-1",
		"120E-2",
	}
	for _, input := range inputs {
		d, _, _ := apd.NewFromString(input)
		res, _ := apd.BaseContext.RoundToIntegralExact(d, d)
		integer := !res.Inexact()
		strict := !res.Rounded()
		fmt.Printf("input: % 6s, output: %3s, integer: %5t, strict: %5t, res:", input, d, integer, strict)
		if res != 0 {
			fmt.Printf(" %s", res)
		}
		fmt.Println()
	}
	// Output: input:  123.4, output: 123, integer: false, strict: false, res: inexact, rounded
	// input:  123.0, output: 123, integer:  true, strict: false, res: rounded
	// input:    123, output: 123, integer:  true, strict:  true, res:
	// input:   12E1, output: 120, integer:  true, strict:  true, res:
	// input: 120E-1, output:  12, integer:  true, strict: false, res: rounded
	// input: 120E-2, output:   1, integer: false, strict: false, res: inexact, rounded
}

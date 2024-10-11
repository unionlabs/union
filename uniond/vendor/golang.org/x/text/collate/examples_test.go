// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package collate_test

import (
	"fmt"

	"golang.org/x/text/collate"
	"golang.org/x/text/language"
)

func ExampleNew() {
	letters := []string{"ä", "å", "ö", "o", "a"}

	ec := collate.New(language.English)
	ec.SortStrings(letters)
	fmt.Printf("English Sorting: %v\n", letters)

	sc := collate.New(language.Swedish)
	sc.SortStrings(letters)
	fmt.Printf("Swedish Sorting: %v\n", letters)

	numbers := []string{"0", "11", "01", "2", "3", "23"}

	ec.SortStrings(numbers)
	fmt.Printf("Alphabetic Sorting: %v\n", numbers)

	nc := collate.New(language.English, collate.Numeric)
	nc.SortStrings(numbers)
	fmt.Printf("Numeric Sorting: %v\n", numbers)
	// Output:
	// English Sorting: [a å ä o ö]
	// Swedish Sorting: [a o å ä ö]
	// Alphabetic Sorting: [0 01 11 2 23 3]
	// Numeric Sorting: [0 01 2 3 11 23]
}

func ExampleCollator_SortStrings() {
	c := collate.New(language.English)
	words := []string{"meow", "woof", "bark", "moo"}
	c.SortStrings(words)
	fmt.Println(words)
	// Output:
	// [bark meow moo woof]
}

func ExampleCollator_CompareString() {
	c := collate.New(language.English)
	r := c.CompareString("meow", "woof")
	fmt.Println(r)

	r = c.CompareString("woof", "meow")
	fmt.Println(r)

	r = c.CompareString("meow", "meow")
	fmt.Println(r)
	// Output:
	// -1
	// 1
	// 0
}

func ExampleCollator_Compare() {
	c := collate.New(language.English)
	r := c.Compare([]byte("meow"), []byte("woof"))
	fmt.Println(r)

	r = c.Compare([]byte("woof"), []byte("meow"))
	fmt.Println(r)

	r = c.Compare([]byte("meow"), []byte("meow"))
	fmt.Println(r)
	// Output:
	// -1
	// 1
	// 0
}

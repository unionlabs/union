// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package collate_test

import (
	"fmt"

	"golang.org/x/text/collate"
	"golang.org/x/text/language"
)

type book struct {
	title string
}

type bookcase struct {
	books []book
}

func (bc bookcase) Len() int {
	return len(bc.books)
}

func (bc bookcase) Swap(i, j int) {
	temp := bc.books[i]
	bc.books[i] = bc.books[j]
	bc.books[j] = temp
}

func (bc bookcase) Bytes(i int) []byte {
	// returns the bytes of text at index i
	return []byte(bc.books[i].title)
}

func ExampleCollator_Sort() {
	bc := bookcase{
		books: []book{
			{title: "If Cats Disappeared from the World"},
			{title: "The Guest Cat"},
			{title: "Catwings"},
		},
	}

	cc := collate.New(language.English)
	cc.Sort(bc)

	for _, b := range bc.books {
		fmt.Println(b.title)
	}
	// Output:
	// Catwings
	// If Cats Disappeared from the World
	// The Guest Cat
}

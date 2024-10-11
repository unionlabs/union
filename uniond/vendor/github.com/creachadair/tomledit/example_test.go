// Copyright (C) 2022 Michael J. Fromberger. All Rights Reserved.

package tomledit_test

import (
	"fmt"
	"log"
	"os"
	"strings"

	"github.com/creachadair/tomledit"
	"github.com/creachadair/tomledit/parser"
)

func ExampleParse() {
	doc, err := tomledit.Parse(strings.NewReader(`# Example config

verbose=true

# A commented section
[commented]
  x = 3    # line comment

  # a commented mapping
  y = true
`))
	if err != nil {
		log.Fatalf("Parse: %v", err)
	}

	// Scan through the parsed document printing out all the keys defined in it,
	// in their order of occurrence.
	doc.Scan(func(key parser.Key, _ *tomledit.Entry) bool {
		fmt.Println(key)
		return true
	})
	// Output:
	//
	// verbose
	// commented
	// commented.x
	// commented.y
}

func ExampleDocument_First() {
	doc, err := tomledit.Parse(strings.NewReader(`[foo]
bar . baz = "frob"
# end
`))
	if err != nil {
		log.Fatalf("Parse: %v", err)
	}

	// Print the first item with the given key. The default string output for an
	// item is for human consumption and is not valid TOML.
	fmt.Println(doc.First("foo", "bar", "baz"))
	// Output:
	// [foo] :: bar.baz = "frob"
}

func ExampleFormatter() {
	doc, err := tomledit.Parse(strings.NewReader(`# A
b='c'
[q."r"]
# D
e.f=true
 g=false
# h
i={j=1,k=2} # L
`))
	if err != nil {
		log.Fatalf("Parse: %v", err)
	}

	if err := tomledit.Format(os.Stdout, doc); err != nil {
		log.Fatalf("Format: %v", err)
	}
	// Output:
	//
	// # A
	// b = 'c'
	//
	// [q.r]
	//
	// # D
	// e.f = true
	// g = false
	//
	// # h
	// i = {j = 1, k = 2}  # L
	//
}

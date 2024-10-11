// Copyright 2017 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package idna_test

import (
	"fmt"

	"golang.org/x/text/internal/export/idna"
)

func ExampleProfile() {
	// Raw Punycode has no restrictions and does no mappings.
	fmt.Println(idna.ToASCII(""))
	fmt.Println(idna.ToASCII("*.GÖPHER.com"))
	fmt.Println(idna.Punycode.ToASCII("*.GÖPHER.com"))

	// Rewrite IDN for lookup.
	fmt.Println(idna.Lookup.ToASCII(""))
	fmt.Println(idna.Lookup.ToASCII("www.GÖPHER.com"))

	// Convert an IDN to ASCII for registration purposes.
	// This reports an error if the input was illformed.
	fmt.Println(idna.Registration.ToASCII("www.GÖPHER.com"))
	fmt.Println(idna.Registration.ToASCII("www.göpher.com"))

	// Output:
	//  <nil>
	// *.xn--GPHER-1oa.com <nil>
	// *.xn--GPHER-1oa.com <nil>
	//  <nil>
	// www.xn--gpher-jua.com <nil>
	// www.xn--GPHER-1oa.com idna: disallowed rune U+0047
	// www.xn--gpher-jua.com <nil>
}

func ExampleNew() {
	var p *idna.Profile

	// Raw Punycode has no restrictions and does no mappings.
	p = idna.New()
	fmt.Println(p.ToASCII("*.faß.com"))

	// Do mappings. Note that star is not allowed in a DNS lookup.
	p = idna.New(
		idna.MapForLookup(),
		idna.Transitional(true)) // Map ß -> ss
	fmt.Println(p.ToASCII("*.faß.com"))

	// Lookup for registration. Also does not allow '*'.
	p = idna.New(idna.ValidateForRegistration())
	fmt.Println(p.ToUnicode("*.faß.com"))

	// Set up a profile maps for lookup, but allows wild cards.
	p = idna.New(
		idna.MapForLookup(),
		idna.Transitional(true),      // Map ß -> ss
		idna.StrictDomainName(false)) // Set more permissive ASCII rules.
	fmt.Println(p.ToASCII("*.faß.com"))

	// Output:
	// *.xn--fa-hia.com <nil>
	// *.fass.com idna: disallowed rune U+002A
	// *.faß.com idna: disallowed rune U+002A
	// *.fass.com <nil>
}

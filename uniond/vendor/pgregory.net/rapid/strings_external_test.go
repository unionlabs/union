// Copyright 2019 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid_test

import (
	"strconv"
	"testing"
	"unicode"
	"unicode/utf8"

	. "pgregory.net/rapid"
)

func TestStringExamples(t *testing.T) {
	g := StringN(10, -1, -1)

	for i := 0; i < 100; i++ {
		s := g.Example()
		t.Log(len(s), s)
	}
}

func TestRegexpExamples(t *testing.T) {
	g := StringMatching("^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$")

	for i := 0; i < 100; i++ {
		s := g.Example()
		t.Log(len(s), s)
	}
}

func TestStringOfRunesIsUTF8(t *testing.T) {
	t.Parallel()

	gens := []*Generator[string]{
		String(),
		StringN(2, 10, -1),
		StringOf(Rune()),
		StringOfN(Rune(), 2, 10, -1),
		StringOf(RuneFrom(nil, unicode.Cyrillic)),
		StringOf(RuneFrom([]rune{'a', 'b', 'c'})),
	}

	for _, g := range gens {
		t.Run(g.String(), MakeCheck(func(t *T) {
			s := g.Draw(t, "s")
			if !utf8.ValidString(s) {
				t.Fatalf("invalid UTF-8 string: %q", s)
			}
		}))
	}
}

func TestStringRuneCountLimits(t *testing.T) {
	t.Parallel()

	genFuncs := []func(i, j int) *Generator[string]{
		func(i, j int) *Generator[string] { return StringN(i, j, -1) },
		func(i, j int) *Generator[string] { return StringOfN(Rune(), i, j, -1) },
	}

	for i, gf := range genFuncs {
		t.Run(strconv.Itoa(i), MakeCheck(func(t *T) {
			minRunes := IntRange(0, 256).Draw(t, "minRunes")
			maxRunes := IntMin(minRunes).Draw(t, "maxRunes")

			s := gf(minRunes, maxRunes).Draw(t, "s")
			n := utf8.RuneCountInString(s)
			if n < minRunes {
				t.Fatalf("got string with %v runes with lower limit %v", n, minRunes)
			}
			if n > maxRunes {
				t.Fatalf("got string with %v runes with upper limit %v", n, maxRunes)
			}
		}))
	}
}

func TestStringNMaxLen(t *testing.T) {
	t.Parallel()

	genFuncs := []func(int) *Generator[string]{
		func(i int) *Generator[string] { return StringN(-1, -1, i) },
		func(i int) *Generator[string] { return StringOfN(Rune(), -1, -1, i) },
	}

	for i, gf := range genFuncs {
		t.Run(strconv.Itoa(i), MakeCheck(func(t *T) {
			maxLen := IntMin(0).Draw(t, "maxLen")
			s := gf(maxLen).Draw(t, "s")
			if len(s) > maxLen {
				t.Fatalf("got string of length %v with maxLen %v", len(s), maxLen)
			}
		}))
	}
}

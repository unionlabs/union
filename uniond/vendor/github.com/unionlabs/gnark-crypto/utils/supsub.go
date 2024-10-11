package utils

import (
	"bytes"
	"fmt"
)

// The following is taken from https://github.com/lynn9388/supsub
// developed by lynn9388@gmail.com under Apache 2.0.
// It provides a mapping from normal unicode text to superscript
// and subscript.

var superscripts = map[rune]rune{
	// Superscripts - Superscripts and Subscripts
	// https://www.unicode.org/charts/PDF/U2070.pdf
	'\u0030': '\u2070',
	'\u0069': '\u2071',
	'\u0034': '\u2074',
	'\u0035': '\u2075',
	'\u0036': '\u2076',
	'\u0037': '\u2077',
	'\u0038': '\u2078',
	'\u0039': '\u2079',
	'\u002b': '\u207a',
	'\u2212': '\u207b',
	'\u003d': '\u207c',
	'\u0028': '\u207d',
	'\u0029': '\u207e',
	'\u006e': '\u207f',

	// Latin superscript modifier letters - Spacing Modifier Letters
	// https://www.unicode.org/charts/PDF/U02B0.pdf
	'\u0068': '\u02b0',
	'\u0266': '\u02b1',
	'\u006a': '\u02b2',
	'\u0072': '\u02b3',
	'\u0279': '\u02b4',
	'\u027b': '\u02b5',
	'\u0281': '\u02b6',
	'\u0077': '\u02b7',
	'\u0079': '\u02b8',

	// Additions based on 1989 IPA - Spacing Modifier Letters
	// https://www.unicode.org/charts/PDF/U02B0.pdf
	'\u0263': '\u02e0',
	'\u006c': '\u02e1',
	'\u0073': '\u02e2',
	'\u0078': '\u02e3',
	'\u0295': '\u02e4',

	// Latin superscript modifier letters - Phonetic Extensions
	// https://www.unicode.org/charts/PDF/U1D00.pdf
	'\u0041': '\u1d2c',
	'\u00c6': '\u1d2d',
	'\u0042': '\u1d2e',
	'\u0044': '\u1d30',
	'\u0045': '\u1d31',
	'\u018e': '\u1d32',
	'\u0047': '\u1d33',
	'\u0048': '\u1d34',
	'\u0049': '\u1d35',
	'\u004a': '\u1d36',
	'\u004b': '\u1d37',
	'\u004c': '\u1d38',
	'\u004d': '\u1d39',
	'\u004e': '\u1d3a',
	'\u004f': '\u1d3c',
	'\u0222': '\u1d3d',
	'\u0050': '\u1d3e',
	'\u0052': '\u1d3f',
	'\u0054': '\u1d40',
	'\u0055': '\u1d41',
	'\u0057': '\u1d42',
	'\u0061': '\u1d43', // '\u0061'(a) '\u1d43'(ᵃ) '\u00aa'(ª)
	'\u0250': '\u1d44',
	'\u0251': '\u1d45',
	'\u1d02': '\u1d46',
	'\u0062': '\u1d47',
	'\u0064': '\u1d48',
	'\u0065': '\u1d49',
	'\u0259': '\u1d4a',
	'\u025b': '\u1d4b',
	'\u1d08': '\u1d4c',
	'\u0067': '\u1d4d',
	'\u006b': '\u1d4f',
	'\u006d': '\u1d50',
	'\u014b': '\u1d51',
	'\u006f': '\u1d52', // '\u006f'(o) '\u1d52'(ᵒ) '\u00ba'(º)
	'\u0254': '\u1d53',
	'\u1d16': '\u1d54',
	'\u1d17': '\u1d55',
	'\u0070': '\u1d56',
	'\u0074': '\u1d57',
	'\u0075': '\u1d58',
	'\u1d1d': '\u1d59',
	'\u026f': '\u1d5a',
	'\u0076': '\u1d5b',
	'\u1d25': '\u1d5c',

	// Greek superscript modifier letters - Phonetic Extensions
	// https://www.unicode.org/charts/PDF/U1D00.pdf
	'\u03b2': '\u1d5d',
	'\u03b3': '\u1d5e',
	'\u03b4': '\u1d5f',
	'\u03c6': '\u1d60',
	'\u03c7': '\u1d61',

	// Caucasian linguistics - Phonetic Extensions
	// https://www.unicode.org/charts/PDF/U1D00.pdf
	'\u043d': '\u1d78',

	// Modifier letters - Phonetic Extensions Supplement
	// https://www.unicode.org/charts/PDF/U1D80.pdf
	'\u0252': '\u1d9b',
	'\u0063': '\u1d9c',
	'\u0255': '\u1d9d',
	'\u00f0': '\u1d9e',
	'\u025c': '\u1d9f',
	'\u0066': '\u1da0',
	'\u025f': '\u1da1',
	'\u0261': '\u1da2',
	'\u0265': '\u1da3',
	'\u0268': '\u1da4',
	'\u0269': '\u1da5',
	'\u026a': '\u1da6',
	'\u1d7b': '\u1da7',
	'\u029d': '\u1da8',
	'\u026d': '\u1da9',
	'\u1d85': '\u1daa',
	'\u029f': '\u1dab',
	'\u0271': '\u1dac',
	'\u0270': '\u1dad',
	'\u0272': '\u1dae',
	'\u0273': '\u1daf',
	'\u0274': '\u1db0',
	'\u0275': '\u1db1',
	'\u0278': '\u1db2',
	'\u0282': '\u1db3',
	'\u0283': '\u1db4',
	'\u01ab': '\u1db5',
	'\u0289': '\u1db6',
	'\u028a': '\u1db7',
	'\u1d1c': '\u1db8',
	'\u028b': '\u1db9',
	'\u028c': '\u1dba',
	'\u007a': '\u1dbb',
	'\u0290': '\u1dbc',
	'\u0291': '\u1dbd',
	'\u0292': '\u1dbe',
	'\u03b8': '\u1dbf',

	// Latin-1 punctuation and symbols - C1 Controls and Latin-1 Supplement
	// https://www.unicode.org/charts/PDF/U0080.pdf
	//'\u0061': '\u00aa', // '\u0061'(a) '\u1d43'(ᵃ) '\u00aa'(ª)
	'\u0032': '\u00b2',
	'\u0033': '\u00b3',
	'\u0031': '\u00b9',
	//'\u006f': '\u00ba', // '\u006f'(o) '\u1d52'(ᵒ) '\u00ba'(º)
}
var subscripts = map[rune]rune{
	// Subscripts - Superscripts and Subscripts
	// https://www.unicode.org/charts/PDF/U2070.pdf
	'\u0030': '\u2080',
	'\u0031': '\u2081',
	'\u0032': '\u2082',
	'\u0033': '\u2083',
	'\u0034': '\u2084',
	'\u0035': '\u2085',
	'\u0036': '\u2086',
	'\u0037': '\u2087',
	'\u0038': '\u2088',
	'\u0039': '\u2089',
	'\u002b': '\u208a',
	'\u2212': '\u208b',
	'\u003d': '\u208c',
	'\u0028': '\u208d',
	'\u0029': '\u208e',
	'\u0061': '\u2090',
	'\u0065': '\u2091',
	'\u006f': '\u2092',
	'\u0078': '\u2093',
	'\u0259': '\u2094',

	// Subscripts for UPA - Superscripts and Subscripts
	// https://www.unicode.org/charts/PDF/U2070.pdf
	'\u0068': '\u2095',
	'\u006b': '\u2096',
	'\u006c': '\u2097',
	'\u006d': '\u2098',
	'\u006e': '\u2099',
	'\u0070': '\u209a',
	'\u0073': '\u209b',
	'\u0074': '\u209c',

	// Latin subscript modifier letters - Phonetic Extensions
	// https://www.unicode.org/charts/PDF/U1D00.pdf
	'\u0069': '\u1d62',
	'\u0072': '\u1d63',
	'\u0075': '\u1d64',
	'\u0076': '\u1d65',

	// Greek subscript modifier letters - Phonetic Extensions
	// https://www.unicode.org/charts/PDF/U1D00.pdf
	'\u03b2': '\u1d66',
	'\u03b3': '\u1d67',
	'\u03c1': '\u1d68',
	'\u03c6': '\u1d69',
	'\u03c7': '\u1d6a',
}

// sup converts a rune to superscript. It returns the superscript or the
// original rune and a error if there is no corresponding superscript.
func sup(r rune) (rune, error) {
	s, ok := superscripts[r]
	if !ok {
		return r, fmt.Errorf("no corresponding superscript: %c", r)
	}
	return s, nil
}

// ToSuperscript converts a string to superscript to the utmost. It will use original
// rune if there has no corresponding superscript for a letter.
func ToSuperscript(s string) string {
	buf := bytes.NewBuffer(make([]byte, 0, len(s)*3))
	for _, r := range s {
		sup, _ := sup(r)
		buf.WriteRune(sup)
	}
	return buf.String()
}

// sub converts a rune to subscript. It returns the subscript or the original
// rune and a error if there is no corresponding subscript.
func sub(r rune) (rune, error) {
	s, ok := subscripts[r]
	if !ok {
		return r, fmt.Errorf("no corresponding subscript: %c", r)
	}
	return s, nil
}

// ToSubscript converts a string to subscript to the utmost. It will use original
// rune if there has no corresponding subscript for a letter.
func ToSubscript(s string) string {
	buf := bytes.NewBuffer(make([]byte, 0, len(s)*3))
	for _, r := range s {
		sub, _ := sub(r)
		buf.WriteRune(sub)
	}
	return buf.String()
}

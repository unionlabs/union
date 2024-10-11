// Copyright 2015 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package currency

import (
	"testing"

	"golang.org/x/text/language"
	"golang.org/x/text/message"
)

var (
	de    = language.German
	en    = language.English
	fr    = language.French
	de_CH = language.MustParse("de-CH")
	en_US = language.AmericanEnglish
	en_GB = language.BritishEnglish
	en_AU = language.MustParse("en-AU")
	und   = language.Und
)

func TestFormatting(t *testing.T) {
	testCases := []struct {
		tag    language.Tag
		value  interface{}
		format Formatter
		want   string
	}{
		0: {en, USD.Amount(0.1), nil, "USD 0.10"},
		1: {en, XPT.Amount(1.0), Symbol, "XPT 1.00"},

		2: {en, USD.Amount(2.0), ISO, "USD 2.00"},
		3: {und, USD.Amount(3.0), Symbol, "US$ 3.00"},
		4: {en, USD.Amount(4.0), Symbol, "$ 4.00"},

		5: {en, USD.Amount(5.20), NarrowSymbol, "$ 5.20"},
		6: {en, AUD.Amount(6.20), Symbol, "A$ 6.20"},

		7: {en_AU, AUD.Amount(7.20), Symbol, "$ 7.20"},
		8: {en_GB, USD.Amount(8.20), Symbol, "US$ 8.20"},

		9:  {en, 9.0, Symbol.Default(EUR), "€ 9.00"},
		10: {en, 10.123, Symbol.Default(KRW), "₩ 10"},
		11: {fr, 11.52, Symbol.Default(TWD), "TWD 11,52"},
		12: {en, 12.123, Symbol.Default(czk), "CZK 12.12"},
		13: {en, 13.123, Symbol.Default(czk).Kind(Cash), "CZK 13"},
		14: {en, 14.12345, ISO.Default(MustParseISO("CLF")), "CLF 14.1235"},
		15: {en, USD.Amount(15.00), ISO.Default(TWD), "USD 15.00"},
		16: {en, KRW.Amount(16.00), ISO.Kind(Cash), "KRW 16"},

		17: {en, USD, nil, "USD"},
		18: {en, USD, ISO, "USD"},
		19: {en, USD, Symbol, "$"},
		20: {en_GB, USD, Symbol, "US$"},
		21: {en_AU, USD, NarrowSymbol, "$"},

		// https://en.wikipedia.org/wiki/Decimal_separator
		22: {de, EUR.Amount(1234567.89), nil, "EUR 1.234.567,89"},
		23: {fr, EUR.Amount(1234567.89), nil, "EUR 1\u00a0234\u00a0567,89"},
		24: {en_AU, EUR.Amount(1234567.89), nil, "EUR 1,234,567.89"},
		25: {de_CH, EUR.Amount(1234567.89), nil, "EUR 1’234’567.89"},

		// https://en.wikipedia.org/wiki/Cash_rounding
		26: {de, NOK.Amount(2.49), ISO.Kind(Cash), "NOK 2"},
		27: {de, NOK.Amount(2.50), ISO.Kind(Cash), "NOK 3"},
		28: {de, DKK.Amount(0.24), ISO.Kind(Cash), "DKK 0,00"},
		29: {de, DKK.Amount(0.25), ISO.Kind(Cash), "DKK 0,50"},

		// integers
		30: {de, EUR.Amount(1234567), nil, "EUR 1.234.567,00"},
		31: {en, CNY.Amount(0), NarrowSymbol, "¥ 0.00"},
		32: {en, CNY.Amount(0), Symbol, "CN¥ 0.00"},
	}
	for i, tc := range testCases {
		p := message.NewPrinter(tc.tag)
		v := tc.value
		if tc.format != nil {
			v = tc.format(v)
		}
		if got := p.Sprint(v); got != tc.want {
			t.Errorf("%d: got %q; want %q", i, got, tc.want)
		}
	}
}

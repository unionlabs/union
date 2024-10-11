package runewidth

import (
	"testing"
	"unicode/utf8"
)

var benchSink int

//
// RuneWidth
//

func benchRuneWidth(b *testing.B, eastAsianWidth bool, start, stop rune, want int) int {
	b.Helper()
	n := 0
	b.Run("regular", func(b *testing.B) {
		got := -1
		c := NewCondition()
		c.EastAsianWidth = eastAsianWidth
		b.ReportAllocs()
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			got = n
			for r := start; r < stop; r++ {
				n += c.RuneWidth(r)
			}
			got = n - got
		}
		if want != 0 && got != want { // some extra checks
			b.Errorf("got %d, want %d\n", got, want)
		}
	})
	b.Run("lut", func(b *testing.B) {
		got := -1
		n = 0
		c := NewCondition()
		c.EastAsianWidth = eastAsianWidth
		c.CreateLUT()
		b.ReportAllocs()
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			got = n
			for r := start; r < stop; r++ {
				n += c.RuneWidth(r)
			}
			got = n - got
		}
		if want != 0 && got != want { // some extra checks
			b.Errorf("got %d, want %d\n", got, want)
		}
	})
	return n
}
func BenchmarkRuneWidthAll(b *testing.B) {
	benchSink = benchRuneWidth(b, false, 0, utf8.MaxRune+1, 1293942)
}
func BenchmarkRuneWidth768(b *testing.B) {
	benchSink = benchRuneWidth(b, false, 0, 0x300, 702)
}
func BenchmarkRuneWidthAllEastAsian(b *testing.B) {
	benchSink = benchRuneWidth(b, true, 0, utf8.MaxRune+1, 1432568)
}
func BenchmarkRuneWidth768EastAsian(b *testing.B) {
	benchSink = benchRuneWidth(b, true, 0, 0x300, 794)
}

//
// String1Width - strings which consist of a single rune
//

func benchString1Width(b *testing.B, eastAsianWidth bool, start, stop rune, want int) int {
	b.Helper()
	n := 0
	b.Run("regular", func(b *testing.B) {
		got := -1
		c := NewCondition()
		c.EastAsianWidth = eastAsianWidth
		b.ResetTimer()
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			got = n
			for r := start; r < stop; r++ {
				s := string(r)
				n += c.StringWidth(s)
			}
			got = n - got
		}
		if want != 0 && got != want { // some extra checks
			b.Errorf("got %d, want %d\n", got, want)
		}
	})
	b.Run("lut", func(b *testing.B) {
		got := -1
		n = 0
		c := NewCondition()
		c.EastAsianWidth = eastAsianWidth
		c.CreateLUT()
		b.ResetTimer()
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			got = n
			for r := start; r < stop; r++ {
				s := string(r)
				n += c.StringWidth(s)
			}
			got = n - got
		}
		if want != 0 && got != want { // some extra checks
			b.Errorf("got %d, want %d\n", got, want)
		}
	})
	return n
}
func BenchmarkString1WidthAll(b *testing.B) {
	benchSink = benchString1Width(b, false, 0, utf8.MaxRune+1, 1295990)
}
func BenchmarkString1Width768(b *testing.B) {
	benchSink = benchString1Width(b, false, 0, 0x300, 702)
}
func BenchmarkString1WidthAllEastAsian(b *testing.B) {
	benchSink = benchString1Width(b, true, 0, utf8.MaxRune+1, 1436664)
}
func BenchmarkString1Width768EastAsian(b *testing.B) {
	benchSink = benchString1Width(b, true, 0, 0x300, 794)
}

//
// tables
//
func benchTable(b *testing.B, tbl table) int {
	n := 0
	for i := 0; i < b.N; i++ {
		for r := rune(0); r <= utf8.MaxRune; r++ {
			if inTable(r, tbl) {
				n++
			}
		}
	}
	return n
}

func BenchmarkTablePrivate(b *testing.B) {
	benchSink = benchTable(b, private)
}
func BenchmarkTableNonprint(b *testing.B) {
	benchSink = benchTable(b, nonprint)
}
func BenchmarkTableCombining(b *testing.B) {
	benchSink = benchTable(b, combining)
}
func BenchmarkTableDoublewidth(b *testing.B) {
	benchSink = benchTable(b, doublewidth)
}
func BenchmarkTableAmbiguous(b *testing.B) {
	benchSink = benchTable(b, ambiguous)
}
func BenchmarkTableEmoji(b *testing.B) {
	benchSink = benchTable(b, emoji)
}
func BenchmarkTableNarrow(b *testing.B) {
	benchSink = benchTable(b, narrow)
}
func BenchmarkTableNeutral(b *testing.B) {
	benchSink = benchTable(b, neutral)
}

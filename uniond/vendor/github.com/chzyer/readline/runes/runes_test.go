package runes

import (
	"reflect"
	"testing"
)

type twidth struct {
	r      []rune
	length int
}

func TestRuneWidth(t *testing.T) {
	runes := []twidth{
		{[]rune("☭"), 1},
		{[]rune("a"), 1},
		{[]rune("你"), 2},
		{ColorFilter([]rune("☭\033[13;1m你")), 3},
	}
	for _, r := range runes {
		if w := WidthAll(r.r); w != r.length {
			t.Fatal("result not expect", r.r, r.length, w)
		}
	}
}

type tagg struct {
	r      [][]rune
	e      [][]rune
	length int
}

func TestAggRunes(t *testing.T) {
	runes := []tagg{
		{
			[][]rune{[]rune("ab"), []rune("a"), []rune("abc")},
			[][]rune{[]rune("b"), []rune(""), []rune("bc")},
			1,
		},
		{
			[][]rune{[]rune("addb"), []rune("ajkajsdf"), []rune("aasdfkc")},
			[][]rune{[]rune("ddb"), []rune("jkajsdf"), []rune("asdfkc")},
			1,
		},
		{
			[][]rune{[]rune("ddb"), []rune("ajksdf"), []rune("aasdfkc")},
			[][]rune{[]rune("ddb"), []rune("ajksdf"), []rune("aasdfkc")},
			0,
		},
		{
			[][]rune{[]rune("ddb"), []rune("ddajksdf"), []rune("ddaasdfkc")},
			[][]rune{[]rune("b"), []rune("ajksdf"), []rune("aasdfkc")},
			2,
		},
	}
	for _, r := range runes {
		same, off := Aggregate(r.r)
		if off != r.length {
			t.Fatal("result not expect", off)
		}
		if len(same) != off {
			t.Fatal("result not expect", same)
		}
		if !reflect.DeepEqual(r.r, r.e) {
			t.Fatal("result not expect")
		}
	}
}

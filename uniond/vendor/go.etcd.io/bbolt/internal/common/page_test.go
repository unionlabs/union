package common

import (
	"reflect"
	"sort"
	"testing"
	"testing/quick"
)

// Ensure that the page type can be returned in human readable format.
func TestPage_typ(t *testing.T) {
	if typ := (&Page{flags: BranchPageFlag}).Typ(); typ != "branch" {
		t.Fatalf("exp=branch; got=%v", typ)
	}
	if typ := (&Page{flags: LeafPageFlag}).Typ(); typ != "leaf" {
		t.Fatalf("exp=leaf; got=%v", typ)
	}
	if typ := (&Page{flags: MetaPageFlag}).Typ(); typ != "meta" {
		t.Fatalf("exp=meta; got=%v", typ)
	}
	if typ := (&Page{flags: FreelistPageFlag}).Typ(); typ != "freelist" {
		t.Fatalf("exp=freelist; got=%v", typ)
	}
	if typ := (&Page{flags: 20000}).Typ(); typ != "unknown<4e20>" {
		t.Fatalf("exp=unknown<4e20>; got=%v", typ)
	}
}

// Ensure that the hexdump debugging function doesn't blow up.
func TestPage_dump(t *testing.T) {
	(&Page{id: 256}).hexdump(16)
}

func TestPgids_merge(t *testing.T) {
	a := Pgids{4, 5, 6, 10, 11, 12, 13, 27}
	b := Pgids{1, 3, 8, 9, 25, 30}
	c := a.Merge(b)
	if !reflect.DeepEqual(c, Pgids{1, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13, 25, 27, 30}) {
		t.Errorf("mismatch: %v", c)
	}

	a = Pgids{4, 5, 6, 10, 11, 12, 13, 27, 35, 36}
	b = Pgids{8, 9, 25, 30}
	c = a.Merge(b)
	if !reflect.DeepEqual(c, Pgids{4, 5, 6, 8, 9, 10, 11, 12, 13, 25, 27, 30, 35, 36}) {
		t.Errorf("mismatch: %v", c)
	}
}

func TestPgids_merge_quick(t *testing.T) {
	if err := quick.Check(func(a, b Pgids) bool {
		// Sort incoming lists.
		sort.Sort(a)
		sort.Sort(b)

		// Merge the two lists together.
		got := a.Merge(b)

		// The expected value should be the two lists combined and sorted.
		exp := append(a, b...)
		sort.Sort(exp)

		if !reflect.DeepEqual(exp, got) {
			t.Errorf("\nexp=%+v\ngot=%+v\n", exp, got)
			return false
		}

		return true
	}, nil); err != nil {
		t.Fatal(err)
	}
}

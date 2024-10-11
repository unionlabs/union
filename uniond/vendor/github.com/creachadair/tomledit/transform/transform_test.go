// Copyright (C) 2022 Michael J. Fromberger. All Rights Reserved.

package transform_test

import (
	"context"
	"errors"
	"strings"
	"testing"

	"github.com/creachadair/tomledit"
	"github.com/creachadair/tomledit/parser"
	"github.com/creachadair/tomledit/transform"
)

func TestTransform(t *testing.T) {
	doc, err := tomledit.Parse(strings.NewReader(`
# Welcome

[empty]

# Topic of much discussion.
[alpha_bravo]
charlie_delta = 'echo'
golf = 0
whisky = { tango = false }

# some crud at the end

[[x]]
a = 1

[[x]]
a = 2

[stale]
great_balls_of = "fire"

[quite.late]
white.rabbit=true
`))
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	p := transform.Plan{
		{
			Desc: "Convert snake_case to kebab-case",
			T:    transform.SnakeToKebab(),
		},
		{
			Desc: "Ensure absent key is present",
			T: transform.EnsureKey(
				parser.Key{"alpha-bravo"},
				&parser.KeyValue{
					Name:  parser.Key{"new", "item"},
					Value: parser.MustValue("true").WithComment("A new value"),
				},
			),
		},
		{
			Desc: "Ensure present key is not replaced",
			T: transform.EnsureKey(
				parser.Key{"alpha-bravo"},
				&parser.KeyValue{
					Name:  parser.Key{"golf"},
					Value: parser.MustValue(`"xyzzy"`),
				},
			),
		},
		{
			Desc: "Rename section",
			T: transform.Rename(
				parser.Key{"alpha-bravo"},
				parser.Key{"charlie", "fox trot"},
			),
		},
		{
			Desc: "Rename inline key",
			T: transform.Rename(
				parser.Key{"charlie", "fox trot", "whisky", "tango"},
				parser.Key{"epsilon"},
			),
		},
		{
			Desc: "Move item to a new location",
			T: transform.MoveKey(
				parser.Key{"stale", "great-balls-of"},
				parser.Key{"empty"},
				parser.Key{"horking-great-balls-of"},
			),
		},
		{
			Desc: "Rename now-non-empty section",
			T: transform.Rename(
				parser.Key{"empty"},
				parser.Key{"non-empty"},
			),
		},
		{
			Desc: "Remove stale section",
			T:    transform.Remove(parser.Key{"stale"}),
		},
		{
			Desc: "Sort sections by name",
			T: transform.Func(func(_ context.Context, doc *tomledit.Document) error {
				transform.SortSectionsByName(doc.Sections)
				return nil
			}),
		},
		{
			Desc: "Sort key-value pairs by name",
			T: transform.Func(func(_ context.Context, doc *tomledit.Document) error {
				tab := transform.FindTable(doc, "charlie", "fox trot")
				if tab == nil {
					return errors.New("target table not found")
				}
				transform.SortKeyValuesByName(tab.Items)
				return nil
			}),
		},
	}
	t.Logf("Applying transformation plan with %d steps", len(p))
	if err := p.Apply(context.Background(), doc); err != nil {
		t.Fatalf("Plan failed: %v", err)
	}

	// Check that the transformations did what they were supposed to.
	t.Run("CheckKeyCase", func(t *testing.T) {
		doc.Scan(func(full parser.Key, e *tomledit.Entry) bool {
			got := full.String()
			if strings.Contains(got, "_") {
				t.Errorf("Key %q contains underscores (%v)", got, e)
			}
			return true
		})
	})
	t.Run("CheckAdded", func(t *testing.T) {
		want := parser.Key{"charlie", "fox trot", "new", "item"}
		if doc.First(want...) == nil {
			t.Errorf("Key %q not found", want)
		}
	})
	t.Run("CheckUnchanged", func(t *testing.T) {
		key := parser.Key{"charlie", "fox trot", "golf"}
		const want = `0`
		if got := doc.First(key...); got == nil {
			t.Fatalf("Key %#q not found", key)
		} else if v := got.Value.X.String(); v != want {
			t.Errorf("Key %#q value: got %q, want %q", key, v, want)
		}
	})
	t.Run("CheckMoved", func(t *testing.T) {
		old := parser.Key{"stale", "great-balls-of"}
		if e := doc.First(old...); e != nil {
			t.Errorf("Unexpectedly found: %v", e)
		}
		key := parser.Key{"non-empty", "horking-great-balls-of"}
		const want = `"fire"`
		if e := doc.First(key...); e == nil {
			t.Fatalf("Key %#q not found", key)
		} else if v := e.Value.X.String(); v != want {
			t.Errorf("Key %#q value: got %#q, want %#q", key, v, want)
		}
	})
	t.Run("CheckSectionOrder", func(t *testing.T) {
		for i := 0; i < len(doc.Sections)-1; i++ {
			this := doc.Sections[i].Name.String()
			next := doc.Sections[i+1].Name.String()
			if this > next {
				t.Errorf("Order violation at %d: %#q > %#q", i, this, next)
			}
		}
	})
	t.Run("CheckKeyValueOrder", func(t *testing.T) {
		key := parser.Key{"charlie", "fox trot"}
		e := doc.First(key...)
		if e == nil {
			t.Fatalf("Key %q not found", key)
		} else if !e.IsSection() {
			t.Fatalf("Value for %q is not a section: %v", key, e)
		}

		var got []string
		e.Scan(func(key parser.Key, _ *tomledit.Entry) bool {
			got = append(got, key.String())
			return true
		})
		for i := 0; i < len(got)-1; i++ {
			if got[i] > got[i+1] {
				t.Errorf("Order violation at %d: %#q > %#q", i, got[i], got[i+1])
			}
		}
	})
}

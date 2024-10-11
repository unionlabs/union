package metavars

import (
	"bytes"
	"reflect"
	"testing"
)

func TestFileAccessors(t *testing.T) {
	f := &File{
		Package: "test",
	}

	// Get non-existent property.
	if _, ok := f.Get("name"); ok {
		t.Fatal("returned ok for non-existent property")
	}

	// Add it.
	p := Property{Name: "name", Value: "value"}
	if err := f.Add(p); err != nil {
		t.Fatal(err)
	}

	// Cannot Add the same property again.
	if err := f.Add(p); err == nil {
		t.Fatal("cannot add duplicate properties")
	}

	// Get should work now.
	if v, ok := f.Get("name"); v != "value" || !ok {
		t.Fatal("unexpected property value")
	}

	// Set it to something else.
	if err := f.Set("name", "new"); err != nil {
		t.Fatal(err)
	}

	// Get the new value.
	if v, ok := f.Get("name"); v != "new" || !ok {
		t.Fatal("did not see value update")
	}

	// Cannot set unknown property.
	if err := f.Set("other", "value"); err == nil {
		t.Fatal("cannot set unknown property")
	}
}

func TestRoundtrip(t *testing.T) {
	cases := []*File{
		{
			Package: "standard",
			Properties: []Property{
				{Name: "A", Doc: "A is the first variable.", Value: "one"},
				{Name: "B", Doc: "B is the second variable.", Value: "two"},
				{Name: "C", Doc: "C is the third variable.", Value: "three"},
			},
		},
		{
			Package: "single",
			Properties: []Property{
				{Name: "A", Doc: "A is the first variable.", Value: "one"},
			},
		},
		{
			Package: "empty",
		},
		{
			Package: "nodoc",
			Properties: []Property{
				{Name: "A", Value: "one"},
				{Name: "B", Value: "two"},
			},
		},
		{
			Package: "mixeddoc",
			Properties: []Property{
				{Name: "A", Doc: "A is the first variable.", Value: "one"},
				{Name: "B", Value: "two"},
				{Name: "C", Doc: "C is the third variable.", Value: "three"},
			},
		},
		{
			Package: "quoting",
			Properties: []Property{
				{Name: "A", Value: `this "would" need 'to' be quoted correctly`},
			},
		},
	}
	for _, f := range cases {
		f := f // scopelint
		t.Run(f.Package, func(t *testing.T) {
			buf := bytes.NewBuffer(nil)
			if err := Write(buf, f); err != nil {
				t.Fatal(err)
			}

			got, err := Read(buf)
			if err != nil {
				t.Fatal(err)
			}

			if !reflect.DeepEqual(got, f) {
				t.Logf("got    = %#v", got)
				t.Logf("expect = %#v", f)
				t.Fatal("roundtrip fail")
			}
		})
	}
}

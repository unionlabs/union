package gateway

import (
	"testing"
)

func TestMarshalNonProtoSliceField(t *testing.T) {
	type testData struct {
		List []string
	}

	data := testData{}
	j := &JSONPb{}

	t.Run("disable EmitDefaults", func(t *testing.T) {
		j.EmitDefaults = false

		r, err := j.Marshal(data.List)
		if err != nil {
			t.Errorf("Marshal failed with %v", err)
		}
		if want := "null"; want != string(r) {
			t.Errorf("want (%v), got (%v)", want, string(r))
		}
	})

	t.Run("enable EmitDefaults", func(t *testing.T) {
		j.EmitDefaults = true

		r, err := j.Marshal(data.List)
		if err != nil {
			t.Errorf("Marshal failed with %v", err)
		}
		if want := "[]"; want != string(r) {
			t.Errorf("want (%v), got (%v)", want, string(r))
		}
	})
}

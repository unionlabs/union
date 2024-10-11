package genesis

import (
	"testing"
)

func TestTarget(t *testing.T) {
	target := &RawJSONTarget{}

	w, err := target.Target()("foo")
	if err != nil {
		t.Errorf("Error creating target: %s", err)
	}
	_, err = w.Write([]byte("1"))
	if err != nil {
		t.Errorf("Error writing to target: %s", err)
	}
	if err := w.Close(); err != nil {
		t.Errorf("Error closing target: %s", err)
	}

	w, err = target.Target()("bar")
	if err != nil {
		t.Errorf("Error creating target: %s", err)
	}
	_, err = w.Write([]byte(`"abc"`))
	if err != nil {
		t.Errorf("Error writing to target: %s", err)
	}
	if err := w.Close(); err != nil {
		t.Errorf("Error closing target: %s", err)
	}

	bz, err := target.JSON()
	if err != nil {
		t.Errorf("Error getting JSON: %s", err)
	}

	// test that it's correct by reading back with a source
	source, err := SourceFromRawJSON(bz)
	if err != nil {
		t.Errorf("Error creating source from JSON: %s", err)
	}

	expectJSON(t, source, "foo", "1")
	expectJSON(t, source, "bar", `"abc"`)
}

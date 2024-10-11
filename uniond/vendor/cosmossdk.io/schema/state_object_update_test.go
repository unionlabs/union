package schema

import "testing"

func TestMapValueUpdates_Iterate(t *testing.T) {
	updates := MapValueUpdates(map[string]interface{}{
		"a": "abc",
		"b": 123,
	})

	got := map[string]interface{}{}
	err := updates.Iterate(func(fieldname string, value interface{}) bool {
		got[fieldname] = value
		return true
	})
	if err != nil {
		t.Errorf("unexpected error: %v", err)
	}

	if len(got) != 2 {
		t.Errorf("expected 2 updates, got: %v", got)
	}

	if got["a"] != "abc" {
		t.Errorf("expected a=abc, got: %v", got)
	}

	if got["b"] != 123 {
		t.Errorf("expected b=123, got: %v", got)
	}

	got = map[string]interface{}{}
	err = updates.Iterate(func(fieldname string, value interface{}) bool {
		if len(got) == 1 {
			return false
		}
		got[fieldname] = value
		return true
	})
	if err != nil {
		t.Errorf("unexpected error: %v", err)
	}

	if len(got) != 1 {
		t.Errorf("expected 1 updates, got: %v", got)
	}

	// should have gotten the first field in order
	if got["a"] != "abc" {
		t.Errorf("expected a=abc, got: %v", got)
	}
}

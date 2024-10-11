package journald

import "testing"

func TestStringifyEntries(t *testing.T) {
	input := map[string]interface{}{
		"foo":     "bar",
		"baz":     123,
		"foo-foo": "x",
		"-bar":    "1",
	}

	output := stringifyEntries(input)
	if output["FOO"] != "bar" {
		t.Fatalf("%v", output)
		t.Fatalf("expected value 'bar'. Got %q", output["FOO"])
	}
	if output["BAZ"] != "123" {
		t.Fatalf("expected value '123'. Got %q", output["BAZ"])
	}
	if output["FOO_FOO"] != "x" {
		t.Fatalf("expected value 'x'. Got %q", output["FOO_FOO"])
	}
	if output["BAR"] != "1" {
		t.Fatalf("expected value 'x'. Got %q", output["BAR"])
	}
}

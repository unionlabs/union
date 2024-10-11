package gen

import "testing"

func TestFunctionSignature(t *testing.T) {
	f := &Function{
		Name:        "sig",
		Description: "Test for function signature",
		Func:        func(x, y int) int { return x + y },
	}
	expect := "func(int, int) int"
	if f.Signature() != expect {
		t.FailNow()
	}
}

func TestFunctionsLint(t *testing.T) {
	for _, f := range Functions {
		if f.Name == "" {
			t.Fatalf("nameless function")
		}
		if f.Description == "" {
			t.Errorf("%s: missing description", f.Name)
		}
	}
}

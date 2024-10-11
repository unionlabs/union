package assert_test

import (
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"runtime"
	"strings"
	"testing"

	"gotest.tools/v3/assert"
	"gotest.tools/v3/internal/source"
)

func TestEqual_WithGoldenUpdate(t *testing.T) {
	t.Run("assert failed with -update=false", func(t *testing.T) {
		ft := &fakeTestingT{}
		actual := `not this value`
		assert.Equal(ft, actual, expectedOne)
		assert.Assert(t, ft.failNowed)
	})

	t.Run("var is updated when -update=true", func(t *testing.T) {
		patchUpdate(t)
		t.Cleanup(func() {
			resetVariable(t, "expectedOne", "")
		})

		actual := `this is the
actual value
that we are testing
`
		assert.Equal(t, actual, expectedOne)

		raw, err := os.ReadFile(fileName(t))
		assert.NilError(t, err)

		expected := "var expectedOne = `this is the\nactual value\nthat we are testing\n`"
		assert.Assert(t, strings.Contains(string(raw), expected), "actual=%v", string(raw))
	})

	t.Run("const is updated when -update=true", func(t *testing.T) {
		patchUpdate(t)
		t.Cleanup(func() {
			resetVariable(t, "expectedTwo", "")
		})

		actual := `this is the new
expected value
`
		assert.Equal(t, actual, expectedTwo)

		raw, err := os.ReadFile(fileName(t))
		assert.NilError(t, err)

		expected := "const expectedTwo = `this is the new\nexpected value\n`"
		assert.Assert(t, strings.Contains(string(raw), expected), "actual=%v", string(raw))
	})

	t.Run("var inside function is updated when -update=true", func(t *testing.T) {
		patchUpdate(t)
		t.Cleanup(func() {
			resetVariable(t, "expectedInsideFunc", "")
		})

		actual := `this is the new
expected value
for var inside function
`
		expectedInsideFunc := ``

		assert.Equal(t, actual, expectedInsideFunc)

		raw, err := os.ReadFile(fileName(t))
		assert.NilError(t, err)

		expected := "expectedInsideFunc := `this is the new\nexpected value\nfor var inside function\n`"
		assert.Assert(t, strings.Contains(string(raw), expected), "actual=%v", string(raw))
	})

	t.Run("const inside function is updated when -update=true", func(t *testing.T) {
		patchUpdate(t)
		t.Cleanup(func() {
			resetVariable(t, "expectedConstInsideFunc", "")
		})

		actual := `this is the new
expected value
for const inside function
`
		const expectedConstInsideFunc = ``

		assert.Equal(t, actual, expectedConstInsideFunc)

		raw, err := os.ReadFile(fileName(t))
		assert.NilError(t, err)

		expected := "const expectedConstInsideFunc = `this is the new\nexpected value\nfor const inside function\n`"
		assert.Assert(t, strings.Contains(string(raw), expected), "actual=%v", string(raw))
	})
}

// expectedOne is updated by running the tests with -update
var expectedOne = ``

// expectedTwo is updated by running the tests with -update
const expectedTwo = ``

func patchUpdate(t *testing.T) {
	source.Update = true
	t.Cleanup(func() {
		source.Update = false
	})
}

func fileName(t *testing.T) string {
	t.Helper()
	_, filename, _, ok := runtime.Caller(1)
	assert.Assert(t, ok, "failed to get call stack")
	return filename
}

func resetVariable(t *testing.T, varName string, value string) {
	t.Helper()
	_, filename, _, ok := runtime.Caller(1)
	assert.Assert(t, ok, "failed to get call stack")

	fileset := token.NewFileSet()
	astFile, err := parser.ParseFile(fileset, filename, nil, parser.AllErrors|parser.ParseComments)
	assert.NilError(t, err)

	var ident *ast.Ident
	ast.Inspect(astFile, func(n ast.Node) bool {
		switch v := n.(type) {
		case *ast.AssignStmt:
			if len(v.Lhs) == 1 {
				if id, ok := v.Lhs[0].(*ast.Ident); ok {
					if id.Name == varName {
						ident = id
						return false
					}
				}
			}

		case *ast.ValueSpec:
			for _, id := range v.Names {
				if id.Name == varName {
					ident = id
					return false
				}
			}
		}

		return true
	})
	assert.Assert(t, ident != nil, "failed to get ident for %s", varName)

	err = source.UpdateVariable(filename, fileset, astFile, ident, value)
	assert.NilError(t, err, "failed to reset file")
}

type fakeTestingT struct {
	failNowed bool
	failed    bool
	msgs      []string
}

func (f *fakeTestingT) FailNow() {
	f.failNowed = true
}

func (f *fakeTestingT) Fail() {
	f.failed = true
}

func (f *fakeTestingT) Log(args ...interface{}) {
	f.msgs = append(f.msgs, args[0].(string))
}

func (f *fakeTestingT) Helper() {}
